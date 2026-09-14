use crate::i18n::msg::I18n;

pub const ZH: I18n = I18n {
  about: "自动精简 Rust 源码中冗长的绝对路径为顶层 use，修复 clippy::absolute_paths",
  path: "待检查或处理的文件或目录路径（默认自动向上检索包含 Cargo.toml 的工程根目录）",
  dry_run: "试运行预览模式，仅检查输出，不实际修改文件（默认直接原地修改并格式化）",
  write: "原地修改文件并自动运行 rustfmt（默认开启）",
  check: "检查模式：发现需要简化的绝对路径时以非零状态码退出（用于 CI）",
  max_segments: "最大允许的路径段数，超过则触发简化（默认 2，与 clippy::absolute_paths 一致）",
  keep_segments: "简化后保留的路径段数（默认 1，即只保留最后一级；设为 2 则保留 module::item）",
  allow_crate: "允许保持绝对路径的 crate 白名单（如 std, core 等）",
  extra_crate: "显式指定的额外已知 crate 列表（用于补充或自定义依赖）",
  quiet: "安静模式，不输出修改明细（非交互终端默认开启）",
  show: "显示模式，强制输出修改明细（用于覆盖非交互环境下的默认安静模式）",
  verbose: "详细输出处理日志",
  no_cache: "禁用增量缓存，强制全量重新检查",

  found_exceeding_limit: |count| format!("发现 {count} 个包含超长绝对路径的文件"),
  can_be_simplified: |count| {
    format!("\n共 {count} 个文件可被简化，运行不带 `--dry-run` 的命令以应用修改")
  },
  failed_init_runtime: "初始化 compio 运行时失败",
  error_processing_file: |file, err| format!("[warn] 处理文件 {file} 时出错: {err}，已跳过"),
};
