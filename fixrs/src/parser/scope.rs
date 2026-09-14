use syn::{File, Item, UseTree};

use crate::hash::{HashMap, HashSet};

/// 记录当前文件顶层的作用域信息（已定义的符号与已存在的 use 语句）
#[derive(Debug, Default, Clone)]
pub struct FileScope {
  /// 顶层在作用域内的所有名称（函数名、类型名、mod名、use 导入的名等）
  pub in_scope_idents: HashSet<String>,
  /// 顶层所有显式 use 导入的完整路径（如 "std::io::Error", "std::io" 等）
  pub existing_use_paths: HashSet<String>,
  /// 显式通过 as 重命名的 use 映射（如 "async_lock::Mutex" -> "AsyncLockMutex"）
  pub renamed_uses: HashMap<String, String>,
}

/// 收集指定 items 中定义的所有标识符与现有 use 导入路径
pub fn collect_items_scope(items: &[Item]) -> FileScope {
  let mut scope = FileScope::default();
  let mut buf = String::with_capacity(64);

  for item in items {
    let ident = match item {
      Item::Fn(f) => Some(&f.sig.ident),
      Item::Struct(s) => Some(&s.ident),
      Item::Enum(e) => Some(&e.ident),
      Item::Const(c) => Some(&c.ident),
      Item::Static(s) => Some(&s.ident),
      Item::Trait(t) => Some(&t.ident),
      Item::TraitAlias(ta) => Some(&ta.ident),
      Item::Type(t) => Some(&t.ident),
      Item::Union(u) => Some(&u.ident),
      Item::Mod(m) => Some(&m.ident),
      Item::Macro(m) => m.ident.as_ref(),
      Item::ExternCrate(ec) => Some(ec.rename.as_ref().map_or(&ec.ident, |(_, r)| r)),
      Item::Use(u) => {
        buf.clear();
        walk_use_tree(&u.tree, &mut buf, &mut scope);
        None
      }
      _ => None,
    };
    if let Some(ident) = ident {
      scope.in_scope_idents.insert(ident.to_string());
    }
  }

  scope
}

/// 收集文件顶层定义的所有标识符与现有 use 导入路径
#[inline]
pub fn collect_file_scope(file: &File) -> FileScope {
  collect_items_scope(&file.items)
}

/// 衍生子模块作用域（继承父级直接定义的符号，并收集当前模块的 items 与 use 语句）
pub fn child_scope(parent: &FileScope, items: &[Item]) -> FileScope {
  let mut scope = collect_items_scope(items);
  scope
    .in_scope_idents
    .extend(parent.in_scope_idents.iter().cloned());
  for (k, v) in &parent.renamed_uses {
    if !scope.renamed_uses.contains_key(k) {
      scope.renamed_uses.insert(k.clone(), v.clone());
    }
  }
  scope
}

fn walk_use_tree(tree: &syn::UseTree, prefix: &mut String, scope: &mut FileScope) {
  let prev_len = prefix.len();
  match tree {
    UseTree::Path(p) => {
      if !prefix.is_empty() {
        prefix.push_str("::");
      }
      let ident_str = p.ident.to_string();
      prefix.push_str(&ident_str);
      walk_use_tree(&p.tree, prefix, scope);
      prefix.truncate(prev_len);
    }
    UseTree::Name(n) => {
      if n.ident == "self" {
        if !prefix.is_empty() {
          if let Some(tail) = prefix.rsplit("::").next() {
            scope.in_scope_idents.insert(tail.to_string());
          }
          scope.existing_use_paths.insert(prefix.clone());
        }
      } else {
        let ident = n.ident.to_string();
        let full_path = if prefix.is_empty() {
          ident.clone()
        } else {
          let mut s = String::with_capacity(prefix.len() + 2 + ident.len());
          s.push_str(prefix);
          s.push_str("::");
          s.push_str(&ident);
          s
        };
        scope.in_scope_idents.insert(ident);
        scope.existing_use_paths.insert(full_path);
      }
    }
    UseTree::Rename(r) => {
      let rename_str = r.rename.to_string();
      if rename_str != "_" {
        scope.in_scope_idents.insert(rename_str.clone());
      }
      let full_path = if r.ident == "self" {
        prefix.clone()
      } else if prefix.is_empty() {
        r.ident.to_string()
      } else {
        let r_ident = r.ident.to_string();
        let mut s = String::with_capacity(prefix.len() + 2 + r_ident.len());
        s.push_str(prefix);
        s.push_str("::");
        s.push_str(&r_ident);
        s
      };
      if r.ident != "_" {
        if r.rename == r.ident {
          scope.existing_use_paths.insert(full_path);
        } else if rename_str != "_" {
          scope
            .existing_use_paths
            .insert(format!("{full_path} as {rename_str}"));
          scope.renamed_uses.insert(full_path, rename_str);
        }
      }
    }
    UseTree::Group(g) => {
      for item in &g.items {
        walk_use_tree(item, prefix, scope);
      }
    }
    UseTree::Glob(_) => {
      if !prefix.is_empty() {
        scope.existing_use_paths.insert(format!("{prefix}::*"));
      }
    }
  }
}
