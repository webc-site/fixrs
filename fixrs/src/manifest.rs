use std::{fs, path::Path, str};

use memchr::{memchr, memmem};
use toml::{Table, Value};

use crate::{cache::find_upward_file, hash::HashSet};

pub const BUILTIN_CRATES: &[&str] = &["std", "core", "alloc", "proc_macro", "test"];
const DEP_KEYS: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

/// 收集标准库内置 crate 以及项目 Cargo.toml 中声明的所有依赖（包括直接依赖、开发依赖、构建依赖与 workspace 依赖）
pub fn load_project_crates(project_root: &Path, extra_crates: &[String]) -> HashSet<String> {
  let mut crates = HashSet::with_capacity_and_hasher(64 + extra_crates.len(), Default::default());

  // 1. 标准库内置顶层 crate
  for &builtin in BUILTIN_CRATES {
    crates.insert(builtin.to_string());
  }

  for extra in extra_crates {
    insert_crate_name(&mut crates, extra);
  }

  // 2. 解析当前工程的 Cargo.toml
  let cargo_path = project_root.join("Cargo.toml");
  if let Ok(content) = fs::read_to_string(&cargo_path)
    && let Ok(table) = content.parse::<Table>()
  {
    extract_crates_from_cargo_table(&table, &mut crates);
    extract_workspace_members_crates(project_root, &table, &mut crates);
  }

  // 3. 向上遍历检索 workspace Cargo.toml（如果是子 package）
  let mut ws_root_opt = None;
  let mut current_dir = project_root.parent();
  while let Some(parent) = current_dir {
    if let Some(ws_cargo) = find_upward_file(parent, "Cargo.toml")
      && let Ok(content) = fs::read_to_string(&ws_cargo)
      && let Ok(table) = content.parse::<Table>()
    {
      extract_crates_from_cargo_table(&table, &mut crates);
      let ws_root = ws_cargo.parent().map(Path::to_path_buf);
      if let Some(ref root) = ws_root {
        extract_workspace_members_crates(root, &table, &mut crates);
      }
      let is_workspace = table.contains_key("workspace");
      ws_root_opt = ws_root;
      if is_workspace {
        break;
      }
      current_dir = ws_root_opt.as_deref().and_then(Path::parent);
      continue;
    }
    break;
  }

  // 4. 解析 Cargo.lock（就近向上检索）
  if let Some(lock_path) = find_upward_file(project_root, "Cargo.lock") {
    load_crates_from_lockfile(&lock_path, &mut crates);
  } else if let Some(ws_root) = ws_root_opt {
    let ws_lock = ws_root.join("Cargo.lock");
    load_crates_from_lockfile(&ws_lock, &mut crates);
  }

  crates
}

fn extract_workspace_members_crates(ws_root: &Path, table: &Table, crates: &mut HashSet<String>) {
  let Some(Value::Table(ws)) = table.get("workspace") else {
    return;
  };
  let Some(Value::Array(members)) = ws.get("members") else {
    return;
  };

  let excluded: Vec<&str> = ws
    .get("exclude")
    .and_then(|e| e.as_array())
    .map(|arr| {
      arr
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.trim_end_matches('/'))
        .collect()
    })
    .unwrap_or_default();

  for m in members {
    let Value::String(s) = m else {
      continue;
    };
    let clean = s.trim_end_matches('/');
    if excluded.contains(&clean) {
      continue;
    }
    if clean.contains('*') || clean.contains('?') {
      let prefix = clean.trim_end_matches('*').trim_end_matches('/');
      let parent_dir = ws_root.join(prefix);
      if let Ok(entries) = fs::read_dir(&parent_dir) {
        for entry in entries.flatten() {
          if let Ok(file_type) = entry.file_type()
            && file_type.is_dir()
          {
            let p = entry.path();
            let member_cargo = p.join("Cargo.toml");
            if let Ok(content) = fs::read_to_string(&member_cargo)
              && let Ok(member_table) = content.parse::<Table>()
            {
              extract_crates_from_cargo_table(&member_table, crates);
            }
          }
        }
      }
    } else {
      let member_cargo = ws_root.join(clean).join("Cargo.toml");
      if let Ok(content) = fs::read_to_string(&member_cargo)
        && let Ok(member_table) = content.parse::<Table>()
      {
        extract_crates_from_cargo_table(&member_table, crates);
      }
    }
  }
}

fn load_crates_from_lockfile(lock_path: &Path, crates: &mut HashSet<String>) {
  if let Ok(bytes) = fs::read(lock_path) {
    let pattern = b"name = \"";
    for pos in memmem::find_iter(&bytes, pattern) {
      if pos == 0 || bytes.get(pos - 1) == Some(&b'\n') {
        let start = pos + pattern.len();
        if let Some(quote_idx) = memchr(b'"', &bytes[start..]) {
          let name_bytes = &bytes[start..start + quote_idx];
          if let Ok(name) = str::from_utf8(name_bytes) {
            insert_crate_name(crates, name);
          }
        }
      }
    }
  }
}

#[inline]
fn insert_crate_name(crates: &mut HashSet<String>, name: &str) {
  if name.contains('-') {
    let mut buf = [0u8; 128];
    if let Some(buf_slice) = buf.get_mut(..name.len()) {
      buf_slice.copy_from_slice(name.as_bytes());
      for b in buf_slice.iter_mut() {
        if *b == b'-' {
          *b = b'_';
        }
      }
      // SAFETY: name 经有效性保证原为合法 UTF-8，仅替换 ASCII '-' 为 '_'
      let s = unsafe { str::from_utf8_unchecked(buf_slice) };
      if !crates.contains(s) {
        crates.insert(s.to_string());
      }
      return;
    }
    let replaced = name.replace('-', "_");
    if !crates.contains(&replaced) {
      crates.insert(replaced);
    }
  } else if !crates.contains(name) {
    crates.insert(name.to_string());
  }
}

fn extract_crates_from_cargo_table(table: &Table, crates: &mut HashSet<String>) {
  // 当前 package 的自身名称（在测试用例/测试文件中作为外部 crate 引用）
  if let Some(Value::Table(pkg)) = table.get("package")
    && let Some(Value::String(name)) = pkg.get("name")
  {
    insert_crate_name(crates, name);
  }

  // 若存在 [lib] name，其作为库名称可能与 package name 不同
  if let Some(Value::Table(lib)) = table.get("lib")
    && let Some(Value::String(name)) = lib.get("name")
  {
    insert_crate_name(crates, name);
  }

  // [dependencies], [dev-dependencies], [build-dependencies]
  for dep_key in DEP_KEYS {
    if let Some(Value::Table(deps)) = table.get(*dep_key) {
      extract_from_dep_table(deps, crates);
    }
  }

  // [workspace.dependencies] 与 [workspace.members]
  if let Some(Value::Table(ws)) = table.get("workspace") {
    if let Some(Value::Table(deps)) = ws.get("dependencies") {
      extract_from_dep_table(deps, crates);
    }
    if let Some(Value::Array(members)) = ws.get("members") {
      for m in members {
        if let Value::String(s) = m {
          let clean = s.trim_end_matches('/');
          if !clean.contains('*') && !clean.contains('?') {
            let name = Path::new(clean)
              .file_name()
              .and_then(|n| n.to_str())
              .unwrap_or(clean);
            insert_crate_name(crates, name);
          }
        }
      }
    }
  }

  // [target.'...'.dependencies]
  if let Some(Value::Table(targets)) = table.get("target") {
    for (_, target_val) in targets {
      if let Value::Table(target_table) = target_val {
        for dep_key in DEP_KEYS {
          if let Some(Value::Table(deps)) = target_table.get(*dep_key) {
            extract_from_dep_table(deps, crates);
          }
        }
      }
    }
  }
}

fn extract_from_dep_table(deps: &Table, crates: &mut HashSet<String>) {
  for (name, val) in deps {
    insert_crate_name(crates, name);
    if let Value::Table(item) = val
      && let Some(Value::String(pkg)) = item.get("package")
    {
      insert_crate_name(crates, pkg);
    }
  }
}
