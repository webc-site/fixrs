use syn::{Item, Path, spanned::Spanned};

use super::{path::QualifiedPath, scope::FileScope};
use crate::{hash::HashSet, options::Options};

/// 单个模块（顶层文件或内嵌 mod）的作用域与替换上下文
#[derive(Debug, Clone)]
pub struct ModuleContext {
  pub insert_pos: usize,
  pub indent: String,
  pub has_existing_use: bool,
  pub scope: FileScope,
  pub paths: Vec<QualifiedPath>,
}

/// AST 绝对路径收集器
pub struct PathCollector<'a> {
  pub max_segments: usize,
  pub keep_segments: usize,
  pub allow_crates: &'a [String],
  pub known_crates: &'a HashSet<String>,
  pub source: &'a str,
  pub line_offsets: &'a [usize],
  pub use_depth: usize,
  pub attr_depth: usize,
  pub cfg_depth: usize,
  pub local_idents: Vec<String>,
  pub module_stack: Vec<ModuleContext>,
  pub modules: Vec<ModuleContext>,
}

impl<'a> PathCollector<'a> {
  pub fn new(
    options: &'a Options,
    known_crates: &'a HashSet<String>,
    root_module: ModuleContext,
    source: &'a str,
    line_offsets: &'a [usize],
  ) -> Self {
    Self {
      max_segments: options.max_segments,
      keep_segments: options.keep_segments,
      allow_crates: &options.allow_crates,
      known_crates,
      source,
      line_offsets,
      use_depth: 0,
      attr_depth: 0,
      cfg_depth: 0,
      local_idents: Vec::new(),
      module_stack: vec![root_module],
      modules: Vec::new(),
    }
  }

  /// 完成收集并返回所有模块上下文（包括顶层模块与所有内嵌模块）
  pub fn finish(mut self) -> Vec<ModuleContext> {
    while let Some(m) = self.module_stack.pop() {
      self.modules.push(m);
    }
    self.modules
  }

  /// 检查并提取合格的长路径
  pub(crate) fn inspect_path(&mut self, path: &Path) {
    let segments = &path.segments;
    let total_segments = segments.len();
    if total_segments <= self.max_segments {
      return;
    }

    let Some(first_seg) = segments.first() else {
      return;
    };
    let first_ident = &first_seg.ident;
    // 忽略 self 与 super 相对路径段
    if first_ident == "self" || first_ident == "super" {
      return;
    }

    let is_crate = first_ident == "crate";

    // 检查是否在白名单中（直接比对，零额外堆分配）
    if self.allow_crates.iter().any(|c| first_ident == c.as_str()) {
      return;
    }

    let first_str = first_ident.to_string();

    // 若非全局 :: 绝对路径，检查是否属于已知 extern crate（已知集合非空时生效）
    // 采用 O(1) 哈希查询，彻底消除原本 O(N) 的线性扫描开销
    if path.leading_colon.is_none()
      && !is_crate
      && !self.known_crates.is_empty()
      && !self.known_crates.contains(&first_str)
    {
      return;
    }

    let mut seg_strings = Vec::with_capacity(total_segments);
    seg_strings.push(first_str);
    for seg in segments.iter().skip(1) {
      seg_strings.push(seg.ident.to_string());
    }
    let last_ident = seg_strings.last().map_or("", String::as_str);

    let Some(current_mod) = self.module_stack.last_mut() else {
      return;
    };

    let mut matched_existing_prefix = None;
    if self.cfg_depth > 0 {
      // 在条件编译保护块内部，仅当路径或其前缀已经在已有 use 列表中时，才允许简化（避免新增未保护的顶层 use 导致 unused_imports）
      // 采用零冗余分配的单字符串增长匹配
      let mut prefix_buf = String::with_capacity(64);
      for (i, seg) in seg_strings.iter().enumerate() {
        if i > 0 {
          prefix_buf.push_str("::");
        }
        prefix_buf.push_str(seg);
        if current_mod.scope.existing_use_paths.contains(&prefix_buf) {
          matched_existing_prefix = Some(i + 1);
        }
      }
      if matched_existing_prefix.is_none() {
        return;
      }
    }

    // 启发式判断：
    // 1. 若倒数第二项为大写（类型），最后一项为方法或枚举变体，保留末尾两段（如 Error::new）
    // 2. 若末尾项为 Result 且非标准库 result::Result，保留末尾两段（如 fmt::Result, io::Result）
    // 3. 若末尾项与当前作用域符号冲突（且非同一条 use 路径），提升保留两段（如 io::Error）
    let mut import_segments_count = total_segments;
    let mut keep_count = self.keep_segments.max(1);

    if let Some(k) = matched_existing_prefix {
      import_segments_count = k;
      keep_count = total_segments.saturating_sub(k - 1).max(1);
    } else {
      if total_segments >= 3 {
        let prev_is_upper = seg_strings
          .get(total_segments - 2)
          .and_then(|s| s.as_bytes().first())
          .is_some_and(u8::is_ascii_uppercase);

        if prev_is_upper {
          import_segments_count = total_segments - 1;
          keep_count = 2;
        }
      }

      // 保护 Prelude 中的核心类型（尤其是 Result）：
      // 若末尾是 "Result"，且导入不是标准库泛型 Result（如 core::result::Result 或 std::result::Result），
      // 强制保留末尾两段（如 fmt::Result, io::Result），避免覆盖全局 Prelude 的 Result<T, E>
      if last_ident == "Result"
        && total_segments >= 2
        && seg_strings
          .get(import_segments_count.saturating_sub(2))
          .map(String::as_str)
          != Some("result")
      {
        import_segments_count = total_segments - 1;
        keep_count = 2;
      }

      // 保护 fmt 模块类型与 Trait（避免 Display/Debug 与标准库冲突导致 .fmt() 多义性调用错误，以及 Formatter/Result 冲突）：
      // 凡是 (core|std)::fmt 下的路径，统一保留 fmt 命名空间（如 fmt::Display, fmt::Debug::fmt, fmt::Formatter, fmt::Result），头部仅导入 fmt
      if total_segments >= 3 {
        let is_std_or_core_fmt = seg_strings
          .first()
          .is_some_and(|s| s == "std" || s == "core")
          && seg_strings.get(1).is_some_and(|s| s == "fmt");

        if is_std_or_core_fmt {
          import_segments_count = 2;
          keep_count = total_segments - 1;
        } else if seg_strings
          .get(total_segments - 2)
          .is_some_and(|s| s == "fmt")
        {
          import_segments_count = total_segments - 1;
          keep_count = 2;
        }
      }
    }

    // 生成原始路径匹配字符串（预估容量一次性分配）
    let cap = seg_strings.iter().map(|s| s.len()).sum::<usize>()
      + (total_segments - 1) * 2
      + if path.leading_colon.is_some() { 2 } else { 0 };
    let mut original = String::with_capacity(cap);
    if path.leading_colon.is_some() {
      original.push_str("::");
    }
    if let Some((first, rest)) = seg_strings.split_first() {
      original.push_str(first);
      for s in rest {
        original.push_str("::");
        original.push_str(s);
      }
    }

    let full_import = original.strip_prefix("::").unwrap_or(&original);

    let Some(current_mod) = self.module_stack.last_mut() else {
      return;
    };

    // 作用域冲突保护：若末尾项已被当前作用域占用或与当前函数参数/局部变量同名，提升保留两段
    let is_shadowed = current_mod.scope.in_scope_idents.contains(last_ident)
      || self.local_idents.iter().rev().any(|ident| ident.as_str() == last_ident);

    if keep_count == 1
      && !current_mod.scope.existing_use_paths.contains(full_import)
      && !current_mod.scope.renamed_uses.contains_key(full_import)
      && is_shadowed
    {
      if total_segments >= 3 {
        import_segments_count = total_segments - 1;
        keep_count = 2;
      } else if seg_strings.first().is_some_and(|s| s == "crate") {
        return;
      }
    }

    let import = if import_segments_count == total_segments {
      full_import.to_string()
    } else {
      let prefix_len = seg_strings[..import_segments_count]
        .iter()
        .map(|s| s.len())
        .sum::<usize>()
        + (import_segments_count - 1) * 2;
      full_import[..prefix_len].to_string()
    };

    // 生成替换文本：纯标识符拼接，绝不追加泛型参数（避免源码中已有泛型出现重复）
    let replacement = if keep_count == 1 {
      last_ident.to_string()
    } else {
      let keep_start = total_segments.saturating_sub(keep_count);
      let suffix_len = seg_strings[keep_start..]
        .iter()
        .map(|s| s.len())
        .sum::<usize>()
        + (total_segments - keep_start - 1) * 2;
      full_import[full_import.len() - suffix_len..].to_string()
    };

    current_mod.paths.push(QualifiedPath {
      original,
      replacement,
      import,
      segments: seg_strings,
      span: path.span(),
    });
  }
}

fn line_indent<'a>(source: &'a str, line_offsets: &[usize], line: usize) -> Option<&'a str> {
  let line_start = *line_offsets.get(line.checked_sub(1)?)?;
  let bytes = source.as_bytes().get(line_start..)?;
  let len = bytes
    .iter()
    .take_while(|&&b| b == b' ' || b == b'\t')
    .count();
  source.get(line_start..line_start + len)
}

/// 计算内嵌模块（ItemMod）内部的 use 注入字节位置、行首缩进与是否存在现有 use
pub fn find_mod_insert_pos_and_indent(
  item_mod: &syn::ItemMod,
  items: &[syn::Item],
  source: &str,
  line_offsets: &[usize],
) -> (usize, String, bool) {
  // 1. 推导模块内缩进
  let indent = if let Some(first) = items.first() {
    line_indent(source, line_offsets, first.span().start().line)
      .unwrap_or("  ")
      .to_string()
  } else {
    let mod_indent = line_indent(source, line_offsets, item_mod.span().start().line).unwrap_or("");
    format!("{mod_indent}  ")
  };

  // 2. 推导插入点
  let last_use_end_line = items
    .iter()
    .map_while(|item| match item {
      Item::Use(u) => Some(u.span().end().line),
      _ => None,
    })
    .last();

  if let Some(end_line) = last_use_end_line {
    let pos = line_offsets.get(end_line).copied().unwrap_or(source.len());
    return (pos, indent, true);
  }

  if let Some(first) = items.first() {
    let pos = first
      .span()
      .start()
      .line
      .checked_sub(1)
      .and_then(|idx| line_offsets.get(idx).copied())
      .unwrap_or(0);
    return (pos, indent, false);
  }

  let mod_line = item_mod.span().start().line;
  let pos = line_offsets.get(mod_line).copied().unwrap_or(source.len());
  (pos, indent, false)
}
