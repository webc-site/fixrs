use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

const IGNORE_DIRS: &[&str] = &["node_modules", "target", "dist", "build"];

/// 快速扫描指定路径下的所有有效 Rust 源文件
/// 自动过滤 .gitignore、隐藏文件（以 . 开头）、node_modules、target 目录
pub fn scan_rs_files(root: &Path) -> Vec<PathBuf> {
  if root.is_file() {
    if root.extension().is_some_and(|ext| ext == "rs") {
      return vec![root.to_path_buf()];
    }
    return Vec::new();
  }

  let mut builder = WalkBuilder::new(root);
  builder
    .hidden(true) // 忽略隐藏文件及目录（以 . 开头）
    .git_ignore(true) // 自动遵循 .gitignore
    .git_global(true)
    .git_exclude(true)
    .filter_entry(|entry| {
      if entry.file_type().is_some_and(|ft| ft.is_dir()) {
        let name = entry.file_name();
        !IGNORE_DIRS.iter().any(|&d| name == d)
      } else {
        true
      }
    });

  let mut files = Vec::new();
  for result in builder.build() {
    let Ok(entry) = result else { continue };
    // entry.file_type() 优先基于 readdir 元数据，避免额外的 stat 系统调用
    if entry.file_type().is_some_and(|ft| ft.is_file())
      && entry.path().extension().is_some_and(|ext| ext == "rs")
    {
      files.push(entry.into_path());
    }
  }

  files
}
