use std::{fs, path::Path, str};

use toml::{Table, Value};

use crate::{cache::find_upward_file, hash::HashSet};

pub const BUILTIN_CRATES: &[&str] = &["std", "core", "alloc", "proc_macro", "test"];
const DEP_KEYS: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

/// 收集标准库内置 crate 以及项目 Cargo.toml 中声明的所有依赖（包括直接依赖、开发依赖、构建依赖与 workspace 依赖）
pub fn load_project_crates(project_root: &Path, extra_crates: &[String]) -> HashSet<String> {
  let mut crates = HashSet::with_capacity_and_hasher(32 + extra_crates.len(), Default::default());

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
  }

  // 3. 向上遍历检索 workspace Cargo.toml（如果是子 package）
  if let Some(parent) = project_root.parent()
    && let Some(ws_cargo) = find_upward_file(parent, "Cargo.toml")
    && let Ok(content) = fs::read_to_string(&ws_cargo)
    && let Ok(table) = content.parse::<Table>()
  {
    extract_crates_from_cargo_table(&table, &mut crates);
  }

  crates
}

#[inline]
fn insert_crate_name(crates: &mut HashSet<String>, name: &str) {
  if name.contains('-') {
    if name.len() <= 64 {
      let mut buf = [0u8; 64];
      let bytes = name.as_bytes();
      let len = bytes.len();
      let buf_slice = &mut buf[..len];
      buf_slice.copy_from_slice(bytes);
      for b in buf_slice.iter_mut() {
        if *b == b'-' {
          *b = b'_';
        }
      }
      if let Ok(s) = str::from_utf8(buf_slice) {
        if !crates.contains(s) {
          crates.insert(s.to_string());
        }
        return;
      }
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
