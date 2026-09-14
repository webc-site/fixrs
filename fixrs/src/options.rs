use std::{
  fs,
  hash::Hasher,
  io::{IsTerminal, stdout},
  path::{Path, PathBuf},
};

use clap::Parser;
use toml::{Table, Value};

use crate::{
  cache::find_upward_files,
  hash::{HashSet, new_hasher},
  i18n,
};

const CLIPPY_CONFIG_FILES: &[&str] = &["clippy.toml", ".clippy.toml"];
const CLIPPY_ALLOWED_CRATES_KEYS: &[&str] = &[
  "absolute-paths-allowed-crates",
  "absolute_paths_allowed_crates",
];
const CLIPPY_MAX_SEGMENTS_KEYS: &[&str] =
  &["absolute-paths-max-segments", "absolute_paths_max_segments"];

#[derive(Parser, Debug, Clone)]
#[command(
  name = "fixrs",
  about = "Fast CLI tool to replace Rust absolute qualified paths with use statements (fixes clippy::absolute_paths)"
)]
pub struct Options {
  /// 待检查或处理的文件或目录路径（默认自动向上检索包含 Cargo.toml 的工程根目录）
  #[arg(value_name = "PATH")]
  pub path: Option<PathBuf>,

  /// 试运行模式，仅检查预览，不实际修改文件（默认直接原地修改并格式化）
  #[arg(short = 'n', long = "dry-run")]
  pub dry_run: bool,

  /// 原地修改文件并自动运行 rustfmt（默认开启）
  #[arg(short = 'w', long = "write", default_value_t = true, action = clap::ArgAction::SetTrue)]
  pub write: bool,

  /// 检查模式：发现需要简化的绝对路径时以非零状态码退出
  #[arg(short = 'c', long = "check")]
  pub check: bool,

  /// 最大允许的路径段数，超过则触发简化（默认 2，与 clippy::absolute_paths 一致）
  #[arg(short = 'm', long = "max-segments", default_value = "2")]
  pub max_segments: usize,

  /// 简化后保留的路径段数（默认 1，即只保留最后一级名称；如设为 2 则保留 module::item）
  #[arg(short = 'k', long = "keep-segments", default_value = "1")]
  pub keep_segments: usize,

  /// 允许保持绝对路径的 crate 白名单（如 std, core, crate 等）
  #[arg(short = 'a', long = "allow-crate")]
  pub allow_crates: Vec<String>,

  /// 显式指定的额外已知 crate 列表（用于补充或自定义依赖）
  #[arg(long = "crate")]
  pub extra_crates: Vec<String>,

  /// 安静模式，不输出修改明细
  #[arg(short = 'q', long = "quiet")]
  pub quiet: bool,

  /// 显示模式，强制输出修改明细（用于覆盖非交互环境下的默认安静模式）
  #[arg(short = 's', long = "show")]
  pub show: bool,

  /// 详细输出处理日志
  #[arg(long = "verbose")]
  pub verbose: bool,

  /// 禁用增量缓存，强制全量重新检查
  #[arg(long = "no-cache")]
  pub no_cache: bool,
}

impl Options {
  /// 将当前语言的国际化帮助文本动态注入到 clap::Command 中
  pub fn apply_i18n(mut cmd: clap::Command) -> clap::Command {
    let m = i18n::msg();
    cmd = cmd.about(m.about);
    cmd = cmd
      .mut_arg("path", |a| a.help(m.path))
      .mut_arg("dry_run", |a| a.help(m.dry_run))
      .mut_arg("write", |a| a.help(m.write))
      .mut_arg("check", |a| a.help(m.check))
      .mut_arg("max_segments", |a| a.help(m.max_segments))
      .mut_arg("keep_segments", |a| a.help(m.keep_segments))
      .mut_arg("allow_crates", |a| a.help(m.allow_crate))
      .mut_arg("extra_crates", |a| a.help(m.extra_crate))
      .mut_arg("quiet", |a| a.help(m.quiet))
      .mut_arg("show", |a| a.help(m.show))
      .mut_arg("verbose", |a| a.help(m.verbose))
      .mut_arg("no_cache", |a| a.help(m.no_cache));
    cmd
  }
}

impl Default for Options {
  fn default() -> Self {
    Self {
      path: None,
      dry_run: false,
      write: true,
      check: false,
      max_segments: 2,
      keep_segments: 1,
      allow_crates: Vec::new(),
      extra_crates: Vec::new(),
      quiet: false,
      show: false,
      verbose: false,
      no_cache: false,
    }
  }
}

impl Options {
  /// 创建默认选项实例
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  /// 设置目标路径
  #[inline]
  pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
    self.path = Some(path.into());
    self
  }

  /// 设置试运行模式（不写回磁盘）
  #[inline]
  pub fn dry_run(mut self, dry_run: bool) -> Self {
    self.dry_run = dry_run;
    self
  }

  /// 设置检查模式（有未改写路径时退出）
  #[inline]
  pub fn check(mut self, check: bool) -> Self {
    self.check = check;
    self
  }

  /// 设置最大允许的绝对路径段数（默认 2）
  #[inline]
  pub fn max_segments(mut self, max: usize) -> Self {
    self.max_segments = max;
    self
  }

  /// 设置简化后保留的路径段数（默认 1）
  #[inline]
  pub fn keep_segments(mut self, keep: usize) -> Self {
    self.keep_segments = keep;
    self
  }

  /// 添加允许保持绝对路径的 crate 白名单
  pub fn allow_crate(mut self, name: impl Into<String>) -> Self {
    self.allow_crates.push(name.into());
    self
  }

  /// 批量添加允许保持绝对路径的 crate 白名单
  pub fn allow_crates<I, S>(mut self, names: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    self.allow_crates.extend(names.into_iter().map(Into::into));
    self
  }

  /// 添加额外的已知外部 crate
  pub fn extra_crate(mut self, name: impl Into<String>) -> Self {
    self.extra_crates.push(name.into());
    self
  }

  /// 设置静音模式
  #[inline]
  pub fn quiet(mut self, quiet: bool) -> Self {
    self.quiet = quiet;
    self
  }

  /// 设置是否禁用增量缓存
  #[inline]
  pub fn no_cache(mut self, no_cache: bool) -> Self {
    self.no_cache = no_cache;
    self
  }

  /// 是否执行实际写入操作（默认即为写入模式，除非指定 --dry-run）
  pub fn should_write(&self) -> bool {
    !self.dry_run
  }

  /// 判定是否在终端输出修改明细（非交互终端默认安静以节约 token，支持 -s 强制显示或 -q 强制静音）
  pub fn should_show_details(&self) -> bool {
    if self.quiet {
      return false;
    }
    if self.show || self.verbose {
      return true;
    }
    stdout().is_terminal()
  }

  pub fn allowed_crates_set(&self) -> HashSet<String> {
    self.allow_crates.iter().cloned().collect()
  }

  /// 计算当前规则配置的哈希值，规则变动时自动全量失效旧缓存（流式写入，零多余堆分配）
  pub fn config_hash(&self) -> u64 {
    let mut hasher = new_hasher();
    hasher.write(&self.max_segments.to_le_bytes());
    hasher.write(&self.keep_segments.to_le_bytes());
    let mut sorted_crates: Vec<&str> = self.allow_crates.iter().map(String::as_str).collect();
    sorted_crates.sort_unstable();
    for c in sorted_crates {
      hasher.write(c.as_bytes());
      hasher.write(&[0]);
    }
    let mut sorted_extra: Vec<&str> = self.extra_crates.iter().map(String::as_str).collect();
    sorted_extra.sort_unstable();
    for c in sorted_extra {
      hasher.write(c.as_bytes());
      hasher.write(&[1]);
    }
    hasher.finish()
  }

  /// 自动向上查找并合并 clippy.toml 或 .clippy.toml 中的配置
  pub fn load_clippy_toml(&mut self) {
    let target = self.path.clone().unwrap_or_else(|| PathBuf::from("."));
    self.load_clippy_toml_for(&target);
  }

  /// 从指定根目录向上查找并合并 clippy.toml
  pub fn load_clippy_toml_for(&mut self, root: &Path) {
    let config_path = find_upward_files(root, CLIPPY_CONFIG_FILES);

    let Some(path) = config_path else {
      return;
    };

    let Ok(content) = fs::read_to_string(&path) else {
      return;
    };

    let Ok(table) = toml::from_str::<Table>(&content) else {
      return;
    };

    // 读取 absolute-paths-allowed-crates
    let allowed_val = CLIPPY_ALLOWED_CRATES_KEYS
      .iter()
      .find_map(|key| table.get(*key));

    if let Some(Value::Array(arr)) = allowed_val {
      for val in arr {
        if let Value::String(s) = val
          && !self.allow_crates.contains(s)
        {
          self.allow_crates.push(s.clone());
        }
      }
    }

    // 读取 absolute-paths-max-segments
    let max_seg_val = CLIPPY_MAX_SEGMENTS_KEYS
      .iter()
      .find_map(|key| table.get(*key));

    if let Some(Value::Integer(n)) = max_seg_val
      && *n > 0
    {
      self.max_segments = *n as usize;
    }
  }
}
