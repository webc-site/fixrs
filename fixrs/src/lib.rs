use std::path::Path;

use hash::HashSet;
use parser::PathCollector;
use syn::visit::Visit;

mod cache;
mod error;
mod hash;
mod i18n;
mod manifest;
mod options;
mod parser;
mod rewriter;
mod runner;
mod walker;

pub use cache::find_project_root;
pub use error::{Error, Result};
pub use i18n::{I18n, Lang, current_lang, msg, msg_for_lang, set_lang};
pub use manifest::load_project_crates;
pub use options::Options;
pub use parser::{FileScope, ModuleContext, QualifiedPath, collect_file_scope};
pub use rewriter::{
  RewriteResult, rewrite_modules_with_offsets, rewrite_source, rewrite_source_with_offsets,
};
pub use runner::{
  ProcessSummary, find_top_level_insert_pos, find_top_level_insert_pos_with_offsets, finish_run,
  format_file, process_file_with_crates, run,
};
pub use walker::scan_rs_files;

/// 直接对 Rust 源码字符串执行绝对路径简化，返回改写结果（源码与替换明细）
pub fn fix_source(source: &str, options: &Options) -> Result<Option<RewriteResult>> {
  let syntax = syn::parse_file(source)?;

  // 显式声明 #![no_implicit_prelude] 的环境完全禁用标准库 prelude，跳过以保护其特殊环境
  if syntax
    .attrs
    .iter()
    .any(|a| a.path().is_ident("no_implicit_prelude"))
  {
    return Ok(None);
  }

  let mut known_crates = HashSet::with_capacity_and_hasher(
    manifest::BUILTIN_CRATES.len() + options.extra_crates.len(),
    Default::default(),
  );
  known_crates.extend(manifest::BUILTIN_CRATES.iter().map(|&b| b.to_string()));
  known_crates.extend(options.extra_crates.iter().cloned());

  Ok(fix_syntax_with_crates(
    &syntax,
    source,
    options,
    &known_crates,
  ))
}

/// 核心改写流水线：处理已解析的 AST 语法树与已知 crate 集合
pub fn fix_syntax_with_crates(
  syntax: &syn::File,
  source: &str,
  options: &Options,
  known_crates: &HashSet<String>,
) -> Option<RewriteResult> {
  let scope = collect_file_scope(syntax);
  let line_offsets = rewriter::compute_line_offsets(source);
  let insert_pos = find_top_level_insert_pos_with_offsets(syntax, source, &line_offsets);
  let has_existing_use = !scope.existing_use_paths.is_empty();
  let root_module = ModuleContext {
    insert_pos,
    indent: String::new(),
    has_existing_use,
    scope,
    paths: Vec::new(),
  };

  let mut collector = PathCollector::new(options, known_crates, root_module, source, &line_offsets);
  collector.visit_file(syntax);

  let modules = collector.finish();
  let has_paths = modules.iter().any(|m| !m.paths.is_empty());
  if !has_paths {
    return None;
  }

  rewriter::rewrite_modules_with_offsets(source, &line_offsets, &modules, options)
}

/// 直接对 Rust 源码字符串执行绝对路径简化，直接返回改写后的新源码字符串
#[inline]
pub fn fix_str(source: &str, options: &Options) -> Result<Option<String>> {
  Ok(fix_source(source, options)?.map(|r| r.content))
}

/// 处理单个 Rust 源文件（根据项目 Cargo.toml 自动解析依赖 crate，返回包含替换明细的结果）
pub fn fix_file(path: &Path, options: &Options) -> Result<Option<RewriteResult>> {
  let project_root = find_project_root(path);
  let known_crates = manifest::load_project_crates(&project_root, &options.extra_crates);
  process_file_with_crates(path, options, &known_crates)
}

/// 批量处理工程或指定路径的快捷入口
#[inline]
pub fn fix_project(options: &Options) -> Result<ProcessSummary> {
  run(options)
}

/// 处理单个 Rust 源文件（根据项目 Cargo.toml 自动解析依赖 crate）
pub fn process_file(path: &Path, options: &Options) -> Result<Option<String>> {
  let res = fix_file(path, options)?;
  Ok(res.map(|r| r.content))
}
