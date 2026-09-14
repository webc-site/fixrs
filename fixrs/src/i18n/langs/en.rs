use crate::i18n::msg::I18n;

pub const EN: I18n = I18n {
  about: "CLI to replace Rust qualified paths with use statements, fixing clippy::absolute_paths",
  path: "Path to file or directory to process (defaults to project root by searching for Cargo.toml upwards)",
  dry_run: "Preview changes without modifying files (defaults to in-place write and formatting)",
  write: "Modify files in-place and run rustfmt (default)",
  check: "Check mode: exit with non-zero status if qualified paths exceed limit (useful for CI)",
  max_segments: "Maximum segments allowed before simplification (default: 2, matching clippy::absolute_paths)",
  keep_segments: "Number of trailing segments to keep (default: 1; use 2 to keep module::item)",
  allow_crate: "Whitelist of crates allowed to retain qualified paths (e.g. std, core)",
  extra_crate: "Explicitly specified extra known crates (used to supplement dependencies)",
  quiet: "Quiet mode, suppress modification details (default in non-terminal/CI environments)",
  show: "Show details, force outputting modification details (overrides default quiet in non-terminal)",
  verbose: "Verbose processing logs",
  no_cache: "Disable incremental cache, force full recheck",

  found_exceeding_limit: |count| {
    format!("Found {count} file(s) with qualified paths exceeding limit")
  },
  can_be_simplified: |count| {
    format!("\n{count} file(s) can be simplified, run without `--dry-run` to apply changes")
  },
  failed_init_runtime: "Failed to initialize compio runtime",
  error_processing_file: |file, err| format!("[warn] Error processing {file}: {err}, skipping"),
};
