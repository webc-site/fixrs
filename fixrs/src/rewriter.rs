use std::{cmp::Ordering, collections::hash_map::Entry};

use crate::{
  hash::{HashMap, HashSet},
  options::Options,
  parser::{FileScope, ModuleContext, QualifiedPath},
};

#[derive(Debug, Clone)]
pub struct RewriteResult {
  pub content: String,
  pub replacements: Vec<(String, String)>,
}

#[derive(Debug)]
enum Action {
  Insert {
    pos: usize,
    text: String,
  },
  Replace {
    start: usize,
    end: usize,
    original: String,
    replacement: String,
  },
}

impl Action {
  #[inline]
  fn pos(&self) -> usize {
    match self {
      Action::Insert { pos, .. } => *pos,
      Action::Replace { start, .. } => *start,
    }
  }
}

/// 构建每行的起始字节偏移索引（基于 memchr SIMD 极速向量化扫描，预分配容量）
pub fn compute_line_offsets(source: &str) -> Vec<usize> {
  let mut offsets = Vec::with_capacity(source.len() / 40 + 1);
  offsets.push(0);
  for idx in memchr::memchr_iter(b'\n', source.as_bytes()) {
    offsets.push(idx + 1);
  }
  offsets
}

/// 对单个文件的源代码执行绝对路径替换与 use 注入（单次流式拼接，绝无内存重复搬移）
pub fn rewrite_source(
  source: &str,
  paths: &[QualifiedPath],
  insert_pos: usize,
  scope: &FileScope,
  options: &Options,
) -> Option<RewriteResult> {
  let line_offsets = compute_line_offsets(source);
  rewrite_source_with_offsets(source, &line_offsets, paths, insert_pos, scope, options)
}

/// 接收外部已计算好的行偏移切片，彻底消除跨模块重复行扫描（兼容旧单模块接口）
pub fn rewrite_source_with_offsets(
  source: &str,
  line_offsets: &[usize],
  paths: &[QualifiedPath],
  insert_pos: usize,
  scope: &FileScope,
  options: &Options,
) -> Option<RewriteResult> {
  let root_mod = ModuleContext {
    insert_pos,
    indent: String::new(),
    has_existing_use: !scope.existing_use_paths.is_empty(),
    scope: scope.clone(),
    paths: paths.to_vec(),
  };
  rewrite_modules_with_offsets(source, line_offsets, &[root_mod], options)
}

/// 将 snake_case / kebab-case 原地追加转换为规范的 PascalCase（零冗余分配）
#[inline]
pub fn to_pascal_case_in(s: &str, out: &mut String) {
  let mut capitalize_next = true;
  for c in s.chars() {
    if c == '_' || c == '-' {
      capitalize_next = true;
    } else if capitalize_next {
      if c.is_ascii() {
        out.push(c.to_ascii_uppercase());
      } else {
        out.extend(c.to_uppercase());
      }
      capitalize_next = false;
    } else {
      out.push(c);
    }
  }
}

/// 将 snake_case / kebab-case 转换为规范的 PascalCase（零冗余分配）
pub fn to_pascal_case(s: &str) -> String {
  let mut res = String::with_capacity(s.len());
  to_pascal_case_in(s, &mut res);
  res
}

#[inline]
fn append_screaming_snake(s: &str, out: &mut String) {
  let mut prev_was_colon = false;
  for b in s.bytes() {
    if b == b':' {
      if prev_was_colon {
        out.push('_');
        prev_was_colon = false;
      } else {
        prev_was_colon = true;
      }
    } else {
      prev_was_colon = false;
      if b == b'-' {
        out.push('_');
      } else {
        out.push(b.to_ascii_uppercase() as char);
      }
    }
  }
}

#[inline]
fn append_snake(s: &str, out: &mut String) {
  let mut prev_was_colon = false;
  for b in s.bytes() {
    if b == b':' {
      if prev_was_colon {
        out.push('_');
        prev_was_colon = false;
      } else {
        prev_was_colon = true;
      }
    } else {
      prev_was_colon = false;
      if b == b'-' {
        out.push('_');
      } else {
        out.push(b.to_ascii_lowercase() as char);
      }
    }
  }
}

/// 根据导入路径与符号特征生成重命名候选别名（遵循 RFC 430 命名规范）
fn generate_alias_candidates(import: &str) -> Vec<String> {
  let import = import.strip_prefix("::").unwrap_or(import);
  let Some((qualifiers_str, item)) = import.rsplit_once("::") else {
    return Vec::new();
  };

  let crate_name = qualifiers_str.split("::").next().unwrap_or(qualifiers_str);

  // 严格禁止对本地 crate::, super::, self:: 路径生成别名（避免将 crate::Error 误改为 CrateError）
  if crate_name == "crate" || crate_name == "super" || crate_name == "self" {
    return Vec::new();
  }

  let has_multiple_qualifiers = qualifiers_str.contains("::");
  let mut candidates = Vec::with_capacity(2);

  let is_upper = item.as_bytes().first().is_some_and(u8::is_ascii_uppercase);
  let is_all_upper = is_upper
    && item
      .chars()
      .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');

  if is_all_upper {
    // 常量 / 静态变量风格（SCREAMING_SNAKE_CASE）
    let mut cand1 = String::with_capacity(crate_name.len() + 1 + item.len());
    append_screaming_snake(crate_name, &mut cand1);
    cand1.push('_');
    cand1.push_str(item);
    candidates.push(cand1);

    if has_multiple_qualifiers {
      let mut cand2 = String::with_capacity(qualifiers_str.len() + 1 + item.len());
      append_screaming_snake(qualifiers_str, &mut cand2);
      cand2.push('_');
      cand2.push_str(item);
      if !candidates.contains(&cand2) {
        candidates.push(cand2);
      }
    }
  } else if is_upper {
    // 类型 / Trait / 枚举风格（PascalCase）
    let crate_pascal = to_pascal_case(crate_name);
    let mut cand1 = String::with_capacity(crate_pascal.len() + item.len());
    cand1.push_str(&crate_pascal);
    cand1.push_str(item);
    candidates.push(cand1);

    if has_multiple_qualifiers {
      let mut full = String::with_capacity(qualifiers_str.len() + item.len());
      for q in qualifiers_str.split("::") {
        to_pascal_case_in(q, &mut full);
      }
      full.push_str(item);
      if !candidates.contains(&full) {
        candidates.push(full);
      }
    }
  } else {
    // 函数 / 方法风格（snake_case）
    let mut cand1 = String::with_capacity(crate_name.len() + 1 + item.len());
    append_snake(crate_name, &mut cand1);
    cand1.push('_');
    cand1.push_str(item);
    candidates.push(cand1);

    if has_multiple_qualifiers {
      let mut cand2 = String::with_capacity(qualifiers_str.len() + 1 + item.len());
      append_snake(qualifiers_str, &mut cand2);
      cand2.push('_');
      cand2.push_str(item);
      if !candidates.contains(&cand2) {
        candidates.push(cand2);
      }
    }
  }

  candidates
}

/// 从候选别名中选取首个在当前模块作用域内未占用的有效别名
fn pick_available_alias(
  import: &str,
  scope: &FileScope,
  new_in_scope: &HashSet<String>,
) -> Option<String> {
  let candidates = generate_alias_candidates(import);
  candidates
    .into_iter()
    .find(|cand| !scope.in_scope_idents.contains(cand) && !new_in_scope.contains(cand))
}

/// 对多个模块单元（顶层文件及内嵌 mod）执行绝对路径替换与 use 注入（单次流式拼接）
pub fn rewrite_modules_with_offsets(
  source: &str,
  line_offsets: &[usize],
  modules: &[ModuleContext],
  _options: &Options,
) -> Option<RewriteResult> {
  if modules.is_empty() {
    return None;
  }

  let mut actions = Vec::new();

  for module in modules {
    if module.paths.is_empty() {
      continue;
    }

    let paths = &module.paths;
    let scope = &module.scope;
    let mut needed_imports = HashSet::default();
    let mut new_in_scope = HashSet::default();
    let mut aliased_imports: HashMap<String, String> = HashMap::default();

    let is_in_scope = |ident: &str, new_in_scope: &HashSet<String>| {
      scope.in_scope_idents.contains(ident) || new_in_scope.contains(ident)
    };

    // 预检导入冲突：若同一模块出现多个不同来源的同名核心项（如 core::fmt::Error 与 std::io::Error）
    let conflicting_tails: HashSet<&str> = if paths.len() > 1 {
      let mut tail_to_import: HashMap<&str, &str> = HashMap::default();
      let mut conflicts = HashSet::default();
      for qp in paths {
        let tail = qp.core_ident();
        let import = qp.import.as_str();
        match tail_to_import.entry(tail) {
          Entry::Vacant(e) => {
            e.insert(import);
          }
          Entry::Occupied(e) => {
            if *e.get() != import {
              conflicts.insert(tail);
            }
          }
        }
      }
      conflicts
    } else {
      HashSet::default()
    };

    for qp in paths {
      let span_start = qp.span.start();
      if span_start.line == 0 || span_start.line > line_offsets.len() {
        continue;
      }

      let line_start = line_offsets[span_start.line - 1];
      let line_end = if span_start.line < line_offsets.len() {
        line_offsets[span_start.line]
      } else {
        source.len()
      };

      let line_text = &source[line_start..line_end];
      let target = &qp.original;
      let trimmed_target = target.trim_start_matches(':');

      // 计算 span 列对应的字符/字节起始偏移（ASCII 源码走常数时间切片快速路径）
      let start_search = if line_text.is_ascii() {
        span_start.column.min(line_text.len())
      } else {
        line_text
          .char_indices()
          .nth(span_start.column)
          .map(|(idx, _)| idx)
          .unwrap_or(line_text.len())
      };

      let match_info = if line_text[start_search..].starts_with(target) {
        Some((start_search, target.len()))
      } else if line_text[start_search..].starts_with(trimmed_target) {
        Some((start_search, trimmed_target.len()))
      } else if let Some(pos) = line_text[start_search..].find(target) {
        Some((start_search + pos, target.len()))
      } else if let Some(pos) = line_text[start_search..].find(trimmed_target) {
        Some((start_search + pos, trimmed_target.len()))
      } else if let Some(pos) = line_text.find(target) {
        Some((pos, target.len()))
      } else {
        line_text
          .find(trimmed_target)
          .map(|pos| (pos, trimmed_target.len()))
      };

      let Some((col_start, matched_len)) = match_info else {
        continue;
      };

      let start_byte = line_start + col_start;
      let end_byte = start_byte + matched_len;

      // 严谨校验切片范围
      if end_byte <= source.len() {
        let mut replacement = qp.replacement.clone();
        let mut import = qp.import.clone();
        let import_len = import.split("::").count();
        let import_tail = qp.import_tail();
        let repl_first = replacement.split("::").next().unwrap_or_default();

        // 若当前路径已在该模块（或本轮改写）中建立重命名映射，直接复用该别名
        let existing_alias = aliased_imports
          .get(&import)
          .cloned()
          .or_else(|| scope.renamed_uses.get(&import).cloned());

        if let Some(alias) = existing_alias {
          let repl = match replacement.strip_prefix(import_tail) {
            Some("") | None => alias.clone(),
            Some(rest) => format!("{alias}{rest}"),
          };
          if repl != qp.original && repl != trimmed_target {
            let alias_import = format!("{import} as {alias}");
            if !scope.existing_use_paths.contains(&alias_import)
              && !scope.existing_use_paths.contains("super::*")
              && !needed_imports.contains(&alias_import)
            {
              needed_imports.insert(alias_import);
              new_in_scope.insert(alias.clone());
            }
            actions.push(Action::Replace {
              start: start_byte,
              end: end_byte,
              original: qp.original.clone(),
              replacement: repl,
            });
          }
          continue;
        }

        // 当 replacement 的前缀符号发生冲突，或者当前仅为单段标识符且其核心类型发生跨库冲突时触发 fallback
        let mut needs_fallback = conflicting_tails.contains(repl_first);
        if !needs_fallback
          && !replacement.contains("::")
          && conflicting_tails.contains(qp.core_ident())
        {
          needs_fallback = true;
        }
        if !needs_fallback
          && !scope.existing_use_paths.contains(&import)
          && !needed_imports.contains(&import)
          && is_in_scope(repl_first, &new_in_scope)
        {
          needs_fallback = true;
        }

        if needs_fallback {
          let mut resolved = false;

          // 1. 若 import 至少有 3 段时（如 std::io::Error），尝试上一级子模块 fallback（如 use std::io; io::Error）
          if import_len >= 3
            && let parent_idx = import_len - 2
            && let Some(parent_mod) = qp.segments.get(parent_idx)
            && parent_mod != "crate"
            && let Some(prefix) = qp.segments.get(..import_len - 1)
          {
            let candidate_tail = parent_mod.as_str();
            let candidate_import = prefix.join("::");
            if candidate_import != "crate" {
              let is_already_imported = scope.existing_use_paths.contains(&candidate_import)
                || needed_imports.contains(&candidate_import);
              if (is_already_imported || !is_in_scope(candidate_tail, &new_in_scope))
                && let Some(suffix) = qp.segments.get(parent_idx..)
              {
                replacement = suffix.join("::");
                import = candidate_import;
                if !is_already_imported {
                  needed_imports.insert(import.clone());
                  new_in_scope.insert(candidate_tail.to_string());
                }
                resolved = true;
              }
            }
          }

          // 2. 若子模块 fallback 无法使用或子模块名称亦冲突（如 import_len == 2 或 candidate_tail 冲突）：
          // 采用智能别名重命名算法（如 async_lock::Mutex -> AsyncLockMutex）
          if !resolved && let Some(alias) = pick_available_alias(&import, scope, &new_in_scope) {
            let alias_import = format!("{import} as {alias}");
            let is_already_imported = scope.renamed_uses.get(&import).is_some_and(|a| a == &alias)
              || needed_imports.contains(&alias_import);

            replacement = match replacement.strip_prefix(import_tail) {
              Some("") | None => alias.clone(),
              Some(rest) => format!("{alias}{rest}"),
            };

            if !is_already_imported {
              needed_imports.insert(alias_import);
              new_in_scope.insert(alias.clone());
            }
            aliased_imports.insert(import.clone(), alias);
            resolved = true;
          }

          if !resolved {
            // 重命名之后还是冲突，再放弃改写避免破坏代码
            continue;
          }
        } else if !scope.existing_use_paths.contains(&import) && !needed_imports.contains(&import) {
          if import == "crate" {
            continue;
          }
          needed_imports.insert(import.clone());
          new_in_scope.insert(import_tail.to_string());
        }

        // 核心防护：若替换后文本与原始文本完全一致，未发生任何简化，跳过以避免重复提示和空修改
        if replacement == qp.original || replacement == trimmed_target {
          continue;
        }

        actions.push(Action::Replace {
          start: start_byte,
          end: end_byte,
          original: qp.original.clone(),
          replacement,
        });
      }
    }

    let mut to_insert: Vec<String> = needed_imports
      .into_iter()
      .filter(|imp| {
        imp != "crate" && imp != "self" && imp != "super" && !scope.existing_use_paths.contains(imp)
      })
      .collect();

    if !to_insert.is_empty() {
      to_insert.sort_unstable();
      let indent_len = module.indent.len();
      let total_len: usize = to_insert
        .iter()
        .map(|s| indent_len + s.len() + 6)
        .sum::<usize>()
        + 2;
      let mut use_block = String::with_capacity(total_len);
      for imp in &to_insert {
        use_block.push_str(&module.indent);
        use_block.push_str("use ");
        use_block.push_str(imp);
        use_block.push_str(";\n");
      }
      if !module.has_existing_use
        && module.insert_pos < source.len()
        && !source[module.insert_pos..].starts_with('\n')
      {
        use_block.push('\n');
      }
      actions.push(Action::Insert {
        pos: module.insert_pos,
        text: use_block,
      });
    }
  }

  if actions.is_empty() {
    return None;
  }

  // 排序：按起始字节位置升序；相同位置 Insert 优先于 Replace
  actions.sort_unstable_by(|a, b| {
    a.pos().cmp(&b.pos()).then_with(|| match (a, b) {
      (Action::Insert { .. }, Action::Replace { .. }) => Ordering::Less,
      (Action::Replace { .. }, Action::Insert { .. }) => Ordering::Greater,
      (Action::Replace { end: end_a, .. }, Action::Replace { end: end_b, .. }) => end_b.cmp(end_a),
      _ => Ordering::Equal,
    })
  });

  // 原地去重叠（针对 Replace）
  let mut last_end = 0;
  actions.retain(|act| match act {
    Action::Insert { .. } => true,
    Action::Replace { start, end, .. } => {
      if *start >= last_end {
        last_end = *end;
        true
      } else {
        false
      }
    }
  });

  // 单次线性流动重组拼接，预计算增量容量，杜绝运行时扩容重分配
  let extra_cap: usize = actions
    .iter()
    .map(|act| match act {
      Action::Insert { text, .. } => text.len(),
      Action::Replace {
        replacement,
        original,
        ..
      } => replacement.len().saturating_sub(original.len()),
    })
    .sum();
  let mut current = String::with_capacity(source.len() + extra_cap + 32);
  let mut cur_idx = 0;
  let mut replacements = Vec::with_capacity(actions.len());

  for act in actions {
    match act {
      Action::Insert { pos, text } => {
        if pos > cur_idx {
          current.push_str(&source[cur_idx..pos]);
          cur_idx = pos;
        }
        current.push_str(&text);
      }
      Action::Replace {
        start,
        end,
        original,
        replacement,
      } => {
        if start > cur_idx {
          current.push_str(&source[cur_idx..start]);
        }
        current.push_str(&replacement);
        cur_idx = end;
        replacements.push((original, replacement));
      }
    }
  }

  current.push_str(&source[cur_idx..]);

  if current == source {
    return None;
  }

  Some(RewriteResult {
    content: current,
    replacements,
  })
}
