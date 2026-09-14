use std::{
  env::current_dir,
  fs,
  io::Write,
  path::{Path, PathBuf},
  process::{Command, Stdio, exit},
  thread::{self, available_parallelism},
};

use compio::runtime::Runtime;
use syn::{AttrStyle, Item, spanned::Spanned};

use crate::{
  cache::{self, FileMeta, find_project_root, get_cache_file_path, load_cache, save_cache},
  error::Result,
  hash::HashSet,
  i18n, manifest,
  options::Options,
  rewriter::{RewriteResult, compute_line_offsets},
  walker::scan_rs_files,
};

/// 文件批处理统计摘要
#[derive(Debug, Default)]
pub struct ProcessSummary {
  pub changed_files: usize,
  pub total_files: usize,
  pub total_replacements: usize,
}

/// 内存流式管道格式化（通过 stdin 管道直接在内存中执行 rustfmt，避免二次磁盘 I/O，并继承工程目录配置）
pub fn format_source(content: &[u8], dir: Option<&Path>) -> Option<Vec<u8>> {
  let mut cmd = Command::new("rustfmt");
  cmd
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::null());
  if let Some(d) = dir {
    cmd.current_dir(d);
  }
  let mut child = cmd.spawn().ok()?;
  if let Some(mut stdin) = child.stdin.take() {
    let _ = stdin.write_all(content);
  }
  let output = child.wait_with_output().ok()?;
  if output.status.success() && !output.stdout.is_empty() && output.stdout != content {
    Some(output.stdout)
  } else {
    None
  }
}

/// 格式化修改后的源文件（兼容旧文件路径接口）
pub fn format_file(file: &Path) {
  if let Ok(content) = fs::read(file)
    && let Some(formatted) = format_source(&content, file.parent())
  {
    let _ = fs::write(file, formatted);
  }
}

/// 处理单个 Rust 源文件（传入已预加载的已知 crate 集合，避免重复解析 Cargo.toml）
pub fn process_file_with_crates(
  path: &Path,
  options: &Options,
  known_crates: &HashSet<String>,
) -> Result<Option<RewriteResult>> {
  let content = fs::read_to_string(path)?;
  let syntax = match syn::parse_file(&content) {
    Ok(s) => s,
    Err(e) => {
      // 语法解析错误输出提示，跳过，不影响正常程序流程
      eprintln!(
        "[warn] syntax parse error in {}: {e}, skipping",
        path.display()
      );
      return Ok(None);
    }
  };

  // 显式声明 #![no_implicit_prelude] 的模块或测试文件完全禁用标准库 prelude，跳过以保护其特殊环境
  if syntax
    .attrs
    .iter()
    .any(|a| a.path().is_ident("no_implicit_prelude"))
  {
    return Ok(None);
  }

  Ok(crate::fix_syntax_with_crates(
    &syntax,
    &content,
    options,
    known_crates,
  ))
}

/// 计算文件顶层 Item::Use 块后的绝对插入字节位置，避免误入局部函数体内
pub fn find_top_level_insert_pos(syntax: &syn::File, source: &str) -> usize {
  let line_offsets = compute_line_offsets(source);
  find_top_level_insert_pos_with_offsets(syntax, source, &line_offsets)
}

/// 接收外部行偏移切片的高性能版本，保护 inner attributes 并避免重复扫描行
pub fn find_top_level_insert_pos_with_offsets(
  syntax: &syn::File,
  source: &str,
  line_offsets: &[usize],
) -> usize {
  let max_inner_attr_end_line = syntax
    .attrs
    .iter()
    .filter(|attr| matches!(attr.style, AttrStyle::Inner(_)))
    .map(|attr| attr.span().end().line)
    .max()
    .unwrap_or(0);

  let last_top_use_end_line = syntax
    .items
    .iter()
    .map_while(|item| match item {
      Item::Use(u) => Some(u.span().end().line),
      _ => None,
    })
    .last();

  if let Some(line) = last_top_use_end_line {
    return line_offsets.get(line).copied().unwrap_or(source.len());
  }

  let inner_attr_pos = (max_inner_attr_end_line > 0)
    .then_some(max_inner_attr_end_line)
    .and_then(|line| line_offsets.get(line).copied());

  let first_item_start_line = syntax
    .items
    .iter()
    .find(|item| !matches!(item, Item::Use(_)))
    .map(|item| item.span().start().line);

  if let Some(line) = first_item_start_line
    && let Some(&pos) = line.checked_sub(1).and_then(|idx| line_offsets.get(idx))
  {
    return inner_attr_pos.map_or(pos, |attr_pos| pos.max(attr_pos));
  }

  inner_attr_pos.unwrap_or(0)
}

type FileCheckResult = (PathBuf, String, Option<FileMeta>, Option<RewriteResult>);

/// 批量处理目录或指定路径（基于 compio 生态，一个 CPU 一个线程，Thread-per-core 异步并发）
pub fn run(options: &Options) -> Result<ProcessSummary> {
  let mut opts = options.clone();

  // 默认从当前目录一层一层往上找 Cargo.toml 作为根目录，除非显式指定路径
  let target_path = opts.path.clone().unwrap_or_else(|| {
    let cwd = current_dir().unwrap_or_else(|_| PathBuf::from("."));
    find_project_root(&cwd)
  });

  let project_root = find_project_root(&target_path);
  opts.load_clippy_toml_for(&project_root);
  let cache_path = get_cache_file_path(&project_root);
  let expected_hash = opts.config_hash();

  let mut cache = if !opts.no_cache
    && let Some(ref cp) = cache_path
  {
    load_cache(cp, expected_hash)
  } else {
    cache::ProjectCache {
      config_hash: expected_hash,
      clean_files: Default::default(),
    }
  };

  let mut files = scan_rs_files(&target_path);
  if files.is_empty() {
    return Ok(ProcessSummary::default());
  }
  files.sort_unstable();

  let known_crates = manifest::load_project_crates(&project_root, &opts.extra_crates);

  // 确定工作线程数：一个 CPU 一个线程
  let num_threads = available_parallelism().map(|n| n.get()).unwrap_or(1);
  let chunk_size = files.len().div_ceil(num_threads);

  let mut all_results = Vec::new();

  thread::scope(|s| {
    let mut handles = Vec::with_capacity(num_threads);

    for chunk in files.chunks(chunk_size) {
      let opts = &opts;
      let root = &project_root;
      let clean_cache = &cache.clean_files;
      let crates = &known_crates;

      let handle = s.spawn(move || -> Vec<FileCheckResult> {
        let rt = match Runtime::new() {
          Ok(rt) => rt,
          Err(e) => {
            eprintln!("[warn] {}: {e}", i18n::msg().failed_init_runtime);
            return Vec::new();
          }
        };

        rt.block_on(async move {
          let mut thread_results = Vec::with_capacity(chunk.len());

          for file in chunk {
            let rel_path_str = file
              .strip_prefix(root.as_path())
              .unwrap_or(file.as_path())
              .to_str();

            let meta = FileMeta::from_path(file);

            // 增量缓存短路：元数据完全匹配且上次已验证为 clean，直接跳过（零堆分配检查）
            if !opts.no_cache
              && let Some(ref m) = meta
              && let Some(p_str) = rel_path_str
              && clean_cache.get(p_str) == Some(m)
            {
              continue;
            }

            let rel_path = rel_path_str.map_or_else(
              || {
                file
                  .strip_prefix(root.as_path())
                  .unwrap_or(file.as_path())
                  .to_string_lossy()
                  .into_owned()
              },
              ToString::to_string,
            );

            match process_file_with_crates(file, opts, crates) {
              Ok(Some(res)) => {
                thread_results.push((file.clone(), rel_path, meta, Some(res)));
              }
              Ok(None) => {
                thread_results.push((file.clone(), rel_path, meta, None));
              }
              Err(e) => {
                let f_str = file.display().to_string();
                let e_str = e.to_string();
                eprintln!("{}", (i18n::msg().error_processing_file)(&f_str, &e_str));
              }
            }
          }

          thread_results
        })
      });

      handles.push(handle);
    }

    // 汇聚所有 worker 线程的处理结果
    for h in handles {
      if let Ok(res) = h.join() {
        all_results.extend(res);
      }
    }
  });

  let mut changed_count = 0;
  let mut total_replacements = 0;

  for (file, rel_path, meta, rewrite_res) in all_results {
    if let Some(res) = rewrite_res {
      changed_count += 1;
      total_replacements += res.replacements.len();

      if opts.should_show_details() {
        println!("{rel_path}");
        for (orig, repl) in &res.replacements {
          println!("  {orig} -> {repl}");
        }
      }

      if opts.should_write() {
        let formatted = format_source(res.content.as_bytes(), Some(&project_root));
        let to_write = formatted.as_deref().unwrap_or(res.content.as_bytes());
        fs::write(&file, to_write)?;
        if !opts.no_cache
          && let Some(updated_meta) = FileMeta::from_path(&file)
        {
          cache.clean_files.insert(rel_path, updated_meta);
        }
      }
    } else if !opts.no_cache
      && let Some(m) = meta
    {
      cache.clean_files.insert(rel_path, m);
    }
  }

  if ((!opts.no_cache && opts.should_write()) || changed_count == 0)
    && let Some(ref cp) = cache_path
  {
    let _ = save_cache(cp, &cache);
  }

  Ok(ProcessSummary {
    changed_files: changed_count,
    total_files: files.len(),
    total_replacements,
  })
}

/// 统一 CLI 输出与退出状态码逻辑
pub fn finish_run(summary: &ProcessSummary, options: &Options) {
  let m = i18n::msg();
  if options.check && summary.changed_files > 0 {
    if options.should_show_details() {
      eprintln!("{}", (m.found_exceeding_limit)(summary.changed_files));
    }
    exit(1);
  }
  if options.should_show_details() && !options.should_write() && summary.changed_files > 0 {
    println!("{}", (m.can_be_simplified)(summary.changed_files));
  }
}
