use std::{
  env::temp_dir,
  fs::{create_dir_all, read_dir, read_to_string, remove_dir_all, write},
  path::Path,
  process::id,
};

use aok::{OK, Void};
use fixrs::{Options, process_file, run};
use log::info;
use serde::Deserialize;

#[ctor::ctor(unsafe)]
fn _log_init() {
  log_init::init();
}

fn default_max_segments() -> usize {
  2
}

fn default_keep_segments() -> usize {
  1
}

#[derive(Deserialize, Debug)]
struct TestCase {
  pub name: String,
  #[serde(default)]
  #[allow(dead_code)]
  pub description: String,
  #[serde(default)]
  pub options: TestOptions,
  pub input: String,
  pub expected: Option<String>,
}

#[derive(Deserialize, Default, Debug)]
struct TestOptions {
  #[serde(default = "default_max_segments")]
  pub max_segments: usize,
  #[serde(default = "default_keep_segments")]
  pub keep_segments: usize,
  #[serde(default)]
  pub allow_crates: Vec<String>,
  #[serde(default)]
  pub extra_crates: Vec<String>,
}

#[test]
fn test_yaml_cases() -> Void {
  let case_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("tests")
    .join("case");

  assert!(case_dir.is_dir(), "tests/case 目录应当存在");

  let mut entries: Vec<_> = read_dir(&case_dir)?
    .filter_map(Result::ok)
    .filter(|e| {
      let p = e.path();
      p.extension().and_then(|s| s.to_str()) == Some("yml")
        || p.extension().and_then(|s| s.to_str()) == Some("yaml")
    })
    .collect();

  entries.sort_by_key(|e| e.path());
  assert!(!entries.is_empty(), "应当至少包含一个测试用例");

  let temp_test_dir = temp_dir().join(format!("fixrs_yaml_runner_{}", id()));
  create_dir_all(&temp_test_dir)?;

  for (idx, entry) in entries.iter().enumerate() {
    let case_path = entry.path();
    let yaml_content = read_to_string(&case_path)?;
    let case: TestCase = serde_yaml::from_str(&yaml_content)?;

    info!(
      "Running YAML case [{}/{}]: {}",
      idx + 1,
      entries.len(),
      case.name
    );

    let test_file = temp_test_dir.join(format!("case_{}_{}.rs", idx, case.name));
    write(&test_file, &case.input)?;

    let opts = Options {
      max_segments: case.options.max_segments,
      keep_segments: case.options.keep_segments,
      allow_crates: case.options.allow_crates,
      extra_crates: if case.options.extra_crates.is_empty() {
        vec!["wbase".to_string()]
      } else {
        case.options.extra_crates
      },
      ..Default::default()
    };

    let result = process_file(&test_file, &opts)?;

    match (&case.expected, result) {
      (None, None) => {
        // 预期不修改，实际未修改，验证通过
      }
      (Some(expected), Some(actual)) => {
        let expected_trimmed = expected.trim();
        let actual_trimmed = actual.trim();
        assert_eq!(
          actual_trimmed, expected_trimmed,
          "Case `{}` failed: 实际输出与期望不符\n--- Actual ---\n{}\n--- Expected ---\n{}",
          case.name, actual, expected
        );
      }
      (None, Some(actual)) => {
        panic!(
          "Case `{}` failed: 预期不改写，但产生了改写:\n{}",
          case.name, actual
        );
      }
      (Some(expected), None) => {
        panic!(
          "Case `{}` failed: 预期发生改写，但 process_file 返回 None\nExpected:\n{}",
          case.name, expected
        );
      }
    }
  }

  let _ = remove_dir_all(&temp_test_dir);
  OK
}

#[test]
fn test_clippy_toml_auto_detection() -> Void {
  let temp_dir = temp_dir().join(format!("fixrs_clippy_toml_{}", id()));
  create_dir_all(&temp_dir)?;

  let clippy_toml = r#"
absolute-paths-allowed-crates = ["mycrate"]
"#;
  write(temp_dir.join("clippy.toml"), clippy_toml)?;

  let mut options = Options {
    path: Some(temp_dir.clone()),
    ..Default::default()
  };
  options.load_clippy_toml();

  assert!(
    options.allowed_crates_set().contains("mycrate"),
    "应当自动解析 clippy.toml 中的 allowed-crates"
  );

  let _ = remove_dir_all(&temp_dir);
  OK
}

#[test]
fn test_incremental_cache() -> Void {
  let temp_dir = temp_dir().join(format!("fixrs_cache_test_{}", id()));
  create_dir_all(&temp_dir)?;
  let test_file = temp_dir.join("clean_file.rs");
  write(&test_file, "pub fn clean_code() { let x = 1; }\n")?;

  let options = Options {
    path: Some(temp_dir.clone()),
    write: true,
    ..Default::default()
  };

  let res1 = run(&options)?;
  assert_eq!(res1.changed_files, 0);

  let res2 = run(&options)?;
  assert_eq!(res2.changed_files, 0);

  let _ = remove_dir_all(&temp_dir);
  OK
}

#[test]
fn test_i18n() -> Void {
  use clap::Args;
  use fixrs::msg;

  let cmd = Options::apply_i18n(Options::augment_args(clap::Command::new("test")));
  assert_eq!(
    cmd.get_about().map(|s| s.to_string()),
    Some(msg().about.to_string())
  );

  OK
}

#[test]
fn test_library_api() -> Void {
  use fixrs::{fix_source, fix_str};

  let input = "fn test() { let _ = std::time::Instant::now(); }";
  let options = Options::new().max_segments(2);

  let res = fix_str(input, &options)?;
  assert!(res.is_some(), "应当成功改写代码");
  let fixed = res.unwrap();
  assert!(fixed.contains("use std::time::Instant;"));
  assert!(fixed.contains("Instant::now()"));

  let detailed = fix_source(input, &options)?;
  assert!(detailed.is_some());
  let d = detailed.unwrap();
  assert_eq!(d.replacements.len(), 1);
  assert_eq!(d.replacements[0].0, "std::time::Instant::now");
  assert_eq!(d.replacements[0].1, "Instant::now");

  OK
}

#[test]
fn test_cast_as_in_mod() -> Void {
  use fixrs::fix_str;

  let input = r#"
pub mod time_stamp {
    pub const TICKS_PER_SECOND: u64 = wbase::convert::TICKS_PER_SECOND as u64;
}
"#;
  let mut options = Options::default();
  options.extra_crates.push("wbase".to_string());
  let res = fix_str(input, &options)?;
  assert!(res.is_some(), "应当修复模块内部带有 as 的绝对长路径");
  let fixed = res.unwrap();
  assert!(
    fixed.contains("use wbase::convert;"),
    "应在 time_stamp 模块内导入上一级模块以避免与本地同名常量冲突"
  );
  assert!(
    fixed.contains("convert::TICKS_PER_SECOND as u64"),
    "应改写为 convert::TICKS_PER_SECOND as u64"
  );

  OK
}

#[test]
fn test_qself_as_trait() -> Void {
  use fixrs::fix_str;

  let input = r#"
pub fn test() {
    let _ = <MyStruct as wbase::convert::MyTrait>::foo();
}
"#;
  let mut options = Options::default();
  options.extra_crates.push("wbase".to_string());
  let res = fix_str(input, &options)?;
  assert!(res.is_some(), "应当修复 QSelf 中的 Trait 长路径");
  let fixed = res.unwrap();
  assert!(fixed.contains("use wbase::convert::MyTrait;"));
  assert!(fixed.contains("<MyStruct as MyTrait>::foo()"));

  OK
}
