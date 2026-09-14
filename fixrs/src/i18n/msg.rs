/// 国际化词条结构体（编译期常量聚合模式，零虚表、零堆分配、编译期强制完备性检查）
#[derive(Clone, Copy, Debug)]
pub struct I18n {
  // CLI 描述与帮助
  pub about: &'static str,
  pub path: &'static str,
  pub dry_run: &'static str,
  pub write: &'static str,
  pub check: &'static str,
  pub max_segments: &'static str,
  pub keep_segments: &'static str,
  pub allow_crate: &'static str,
  pub extra_crate: &'static str,
  pub quiet: &'static str,
  pub show: &'static str,
  pub verbose: &'static str,
  pub no_cache: &'static str,

  // 运行时提示与报告
  pub found_exceeding_limit: fn(usize) -> String,
  pub can_be_simplified: fn(usize) -> String,
  pub failed_init_runtime: &'static str,
  pub error_processing_file: fn(&str, &str) -> String,
}
