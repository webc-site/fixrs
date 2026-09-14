[English](#en) | [中文](#zh)

---

<a name="en"></a>

# fixrs

[![crates.io](https://img.shields.io/crates/v/fixrs.svg)](https://crates.io/crates/fixrs)
[![docs.rs](https://docs.rs/fixrs/badge.svg)](https://docs.rs/fixrs)
[![github](https://img.shields.io/badge/github-webc--site/fixrs-blue.svg?logo=github)](https://github.com/webc-site/fixrs)

Blazing-fast Rust absolute long path refactoring and `use` injection tool, purpose-built to automatically resolve `clippy::absolute_paths` warnings. **Can be used both as a standalone CLI / Cargo plugin, and embedded directly as a Rust library crate**.

---

- [Quick Verification & Demo](#quick-verification-demo)
  - [1. Before & After Demo](#1-before-after-demo)
  - [2. One-line CLI Verification](#2-one-line-cli-verification)
- [Why fixrs? The Pain Points Solved](#why-fixrs-the-pain-points-solved)
- [Automated Git Hooks for Team Collaboration](#automated-git-hooks-for-team-collaboration)
  - [1. Check Git Hooks into Version Control](#1-check-git-hooks-into-version-control)
  - [2. One-Line Team Activation](#2-one-line-team-activation)
- [Can Be Used as a Library (Library Usage)](#can-be-used-as-a-library-library-usage)
  - [1. In-Memory Source Rewriting (`fix_str`)](#1-in-memory-source-rewriting-fix_str)
  - [2. Inspect Rewriting Results & Replacement Details (`fix_source`)](#2-inspect-rewriting-results-replacement-details-fix_source)
  - [3. Process Files or Projects (`fix_file` / `fix_project`)](#3-process-files-or-projects-fix_file-fix_project)
- [CLI Options & Usage](#cli-options-usage)
  - [Common Commands](#common-commands)
  - [Complete Options Reference](#complete-options-reference)
- [Architecture & Production Verification](#architecture-production-verification)

## Quick Verification & Demo

### 1. Before & After Demo

Before refactoring:

```rust
// Original code: littered with redundant, qualified absolute paths
fn handle_data() -> Result<(), std::io::Error> {
    let now = std::time::Instant::now();
    let data = wbase::time::now_secs();
    let code = std::io::ErrorKind::NotFound;
    Ok(())
}
```

After running `fixrs` (automatically injects `use` statements at the top, simplifies call-sites, and runs `rustfmt`):

```rust
use std::io::{Error, ErrorKind};
use std::time::Instant;
use wbase::time::now_secs;

fn handle_data() -> Result<(), Error> {
    let now = Instant::now();
    let data = now_secs();
    let code = ErrorKind::NotFound;
    Ok(())
}
```

### 2. One-line CLI Verification

```bash
# Install
cargo install fixrs

# Run in any Rust project root (in-place rewrite + auto rustfmt)
fixrs
# Or as a cargo plugin
cargo fixrs

# Dry-run mode (preview without modifying files)
fixrs -n
```

---

## Why fixrs? The Pain Points Solved

Clippy's [`clippy::absolute_paths`](https://rust-lang.github.io/rust-clippy/master/index.html#absolute_paths) restriction lint flags long qualified paths in favor of clean, readable imports. However, **`cargo clippy --fix` explicitly does not support auto-fixing this lint** (see official implementation [`clippy_lints/src/absolute_paths.rs`](https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/absolute_paths.rs), which only issues warnings and lacks MachineApplicable automated suggestions).

Here is why:

1. **Overlapping Replacement Conflicts**: Fixing qualified paths requires modifying multiple syntax locations across AST spans: inserting `use` declarations at the file header and shortening paths inside functions. When multiple replacements occur in the same file, `cargo fix` aborts due to overlapping replacements.
2. **Scope Shadowing & Name Collisions**: If `std::io::Error` is blindly shortened to `Error` with `use std::io::Error;`, but the file already declares an `enum Error` or a local variable `let Error = ...`, fatal compilation errors occur.
3. **Trait Method Ambiguity Disaster (E0034)**: If `std::fmt::Debug` or `std::fmt::Display` is imported directly into scope, calling `.fmt(f)` on objects implementing multiple format traits fails with compiler error E0034.
4. **Breaking Conditional Compilation and `#![no_implicit_prelude]`**: Extracting a path from a `#[cfg(feature = "...")]` block into an unconditional top-level `use` breaks `no_std` or cross-platform builds. Replacing paths inside `macro_rules!` can corrupt macro expansion.
5. **Significant Token & Context Savings in AI-Assisted Coding**:
   - Modern AI code generation tools (e.g., Cursor, GitHub Copilot, Claude, ChatGPT) frequently output long, fully-qualified paths (such as `std::collections::HashMap::new()`, `tokio::sync::mpsc::channel()`, `std::sync::atomic::AtomicBool::new()`) to avoid missing imports;
   - These repetitive paths consume valuable LLM context window limits and inflate inference token costs;
   - By automatically shortening paths to compact identifiers and injecting single-line top-level `use` imports, code density improves dramatically, reducing token usage across multi-turn chats, automated refactoring, and AI code reviews while speeding up model inference.
6. **Tedious & Error-Prone Manual Work**: In large codebases, manually finding and editing hundreds of qualified paths while checking for conflicts is painful, tedious, and prone to regressions.

**`fixrs` addresses every single one of these challenges**:
- Deterministic AST scope analysis: automatically detects identifier collisions with top-level items or local parameters, smartly **elevating to module-level imports** (e.g., retaining `io::Error` with `use std::io;`);
- Special protection for `(core|std)::fmt`: preserves the `fmt::` prefix to prevent Trait method ambiguity;
- Strict isolation for `#[cfg]` blocks, `macro_rules!` definitions, and `#![no_implicit_prelude]` environments;
- Automatic in-place single-file pipelined `rustfmt` formatting.

---

## Automated Git Hooks for Team Collaboration

In collaborative team environments, the default `.git/hooks/` directory is local-only and not tracked by Git, making manual hook setup error-prone for new contributors.

The recommended best practice is to use Git's native **`core.hooksPath` tracked directory mechanism** (zero third-party dependencies, version-controlled, synced automatically across all team members):

### 1. Check Git Hooks into Version Control

Create a version-controlled `.githooks` directory at the project root and add the `pre-commit` hook:

```bash
mkdir -p .githooks
cat << 'EOF' > .githooks/pre-commit
#!/bin/sh
set -e

# Automatically simplify qualified paths and format before commit
echo "[git-hook] Running fixrs to optimize imports..."
fixrs

# Stage the cleaned and formatted files
git add -u
EOF

chmod +x .githooks/pre-commit
git add .githooks/pre-commit
```

### 2. One-Line Team Activation

When team members clone the repository, they simply point Git's hook path to the tracked directory:

```bash
git config core.hooksPath .githooks
```

You can also embed this one-liner into your project's setup script, Makefile, or Justfile. Future updates to the hook are tracked in Git and propagate to everyone seamlessly on `git pull`.

---

## Can Be Used as a Library (Library Usage)

`fixrs` is not just a command-line utility. Its entire engine is cleanly encapsulated as a high-cohesion, lightweight Rust library that can be integrated into code generators, macros, language servers, or automated refactoring pipelines.

Add `fixrs` to your `Cargo.toml`:

```toml
[dependencies]
fixrs = "0.1"
```

### 1. In-Memory Source Rewriting (`fix_str`)

```rust
use fixrs::{Options, fix_str};

fn main() -> fixrs::Result<()> {
    let source = r#"
fn run() -> Result<(), std::io::Error> {
    let now = std::time::Instant::now();
    Ok(())
}
"#;

    let options = Options::new().max_segments(2);
    if let Some(fixed_code) = fix_str(source, &options)? {
        println!("{fixed_code}");
    }
    Ok(())
}
```

### 2. Inspect Rewriting Results & Replacement Details (`fix_source`)

```rust
use fixrs::{Options, fix_source};

let code = "fn test() { let _ = std::time::Instant::now(); }";
let options = Options::new().max_segments(2);

if let Some(result) = fix_source(code, &options)? {
    println!("Rewritten source:\n{}", result.content);
    for (original, replacement) in result.replacements {
        println!("Replacement: {original} -> {replacement}");
    }
}
```

### 3. Process Files or Projects (`fix_file` / `fix_project`)

```rust
use std::path::Path;
use fixrs::{Options, fix_file, fix_project};

// Process a single file (automatically discovers project root Cargo.toml)
let file_result = fix_file(Path::new("src/main.rs"), &Options::default())?;

// Batch process an entire project directory
let summary = fix_project(&Options::new().path("."))?;
println!("Completed: changed {} files with {} replacements", summary.changed_files, summary.total_replacements);
```

---

## CLI Options & Usage

### Common Commands

```bash
# 1. Default: scan current Cargo workspace, rewrite in place, and format
fixrs

# 2. Specify target path: process a directory or file
fixrs src/
fixrs src/main.rs

# 3. Dry-run mode: preview changes without writing to disk
fixrs -n

# 4. CI check mode: exit with code 1 if long paths exist
fixrs -c

# 5. Quiet mode: suppress detailed replacement output
fixrs -q
```

### Complete Options Reference

| Option | Default | Description |
| :--- | :--- | :--- |
| `[PATH]` | Nearest `Cargo.toml` root | Target directory or `.rs` file to process |
| `-n, --dry-run` | `false` | Dry-run preview mode; do not modify files |
| `-c, --check` | `false` | CI check mode; exit code 1 if unallowed paths exist |
| `-s, --show` | `false` | Force printing file paths and replacement details |
| `-q, --quiet` | `false` | Quiet mode; suppress detailed outputs |
| `-m, --max-segments <N>` | `2` | Maximum allowed path segments (matching Clippy default) |
| `-k, --keep-segments <N>` | `1` | Number of segments to keep in code (default: last 1 segment) |
| `-a, --allow-crate <NAME>` | None | Crates allowed to keep qualified paths (e.g. `-a std -a core`) |
| `--crate <NAME>` | None | Extra external crate names to recognize |
| `--no-cache` | `false` | Disable incremental cache and force a full re-scan |
| `--verbose` | `false` | Enable verbose logging output |

---

## Architecture & Production Verification

- **Pure Syntax Parsing**: Powered by `syn` AST traversal. No `rustc` compiler invocation, no heavy metadata loading—single files are parsed in milliseconds.
- **Thread-Per-Core Asynchronous Concurrency**: Built on the `compio` runtime, dispatching work per CPU core for maximum throughput across massive codebases in hundreds of milliseconds.
- **Smart Incremental Caching**: Uses `museair` nanosecond hashing and project metadata caching. Unmodified files skip instantaneously; cache invalidates automatically when options change.
- **Dependency Awareness**: Automatically parses `Cargo.toml` and workspace dependencies/dev-dependencies to distinguish local modules from external crates.
- **Smart Directory Walker**: Built on `ignore`, honoring `.gitignore` rules and skipping `target/`, `node_modules/`, and hidden folders.
- **Verified on 30+ Industrial Open-Source Crates**: Validated in real-world codebases including `tokio`, `serde`, `regex`, `tower`, `clap`, `rustix`, `mio`, `socket2`, `rand`, `tempfile`, `thiserror`, `tar`, `backtrace`, and more—passing 100% of official test suites after refactoring!

---

<a name="zh"></a>

# fixrs

[![crates.io](https://img.shields.io/crates/v/fixrs.svg)](https://crates.io/crates/fixrs)
[![docs.rs](https://docs.rs/fixrs/badge.svg)](https://docs.rs/fixrs)
[![github](https://img.shields.io/badge/github-webc--site/fixrs-blue.svg?logo=github)](https://github.com/webc-site/fixrs)

极速 Rust 绝对长路径自动重构与 `use` 导入注入工具，专为全自动修复 `clippy::absolute_paths` 警告而生。**既可作为独立命令行工具或 Cargo 插件，也可直接作为 Rust 库嵌入程序中使用**。

---

- [快速验证与使用演示](#快速验证与使用演示)
  - [1. 效果演示（重构前后对比）](#1-效果演示重构前后对比)
  - [2. 命令行一键验证](#2-命令行一键验证)
- [为什么需要 fixrs？解决的痛点](#为什么需要-fixrs解决的痛点)
- [多人协作自动化配置 Git 提交钩子](#多人协作自动化配置-git-提交钩子)
  - [1. 将提交钩子纳入版本库管理](#1-将提交钩子纳入版本库管理)
  - [2. 团队成员一行命令激活](#2-团队成员一行命令激活)
- [作为 Rust 库使用](#作为-rust-库使用)
  - [1. 内存中极速简化源码字符串（`fix_str`）](#1-内存中极速简化源码字符串fix_str)
  - [2. 获取改写结果与详细替换清单（`fix_source`）](#2-获取改写结果与详细替换清单fix_source)
  - [3. 处理单文件或整个项目工程（`fix_file` / `fix_project`）](#3-处理单文件或整个项目工程fix_file-fix_project)
- [命令行参数与用法](#命令行参数与用法)
  - [常用命令](#常用命令)
  - [完整参数清单](#完整参数清单)
- [核心特性与设计保障](#核心特性与设计保障)

## 快速验证与使用演示

### 1. 效果演示（重构前后对比）

重构前代码：

```rust
// 原始代码：充斥冗长的跨模块、跨 crate 绝对路径
fn handle_data() -> Result<(), std::io::Error> {
    let now = std::time::Instant::now();
    let data = wbase::time::now_secs();
    let code = std::io::ErrorKind::NotFound;
    Ok(())
}
```

运行 `fixrs` 重构后（全自动在顶层注入 `use` 并简化，保留排版与注释）：

```rust
use std::io::{Error, ErrorKind};
use std::time::Instant;
use wbase::time::now_secs;

fn handle_data() -> Result<(), Error> {
    let now = Instant::now();
    let data = now_secs();
    let code = ErrorKind::NotFound;
    Ok(())
}
```

### 2. 命令行一键验证

```bash
# 安装
cargo install fixrs

# 进入任意 Rust 项目目录，一键扫描、就地重构并自动调用 rustfmt 格式化
fixrs
# 或作为 cargo 子命令运行
cargo fixrs

# 试运行预览（仅在终端输出待替换明细，不改写文件）
fixrs -n
```

---

## 为什么需要 fixrs？解决的痛点

Clippy 的 [`clippy::absolute_paths`](https://rust-lang.github.io/rust-clippy/master/index.html#absolute_paths) 检查规则会严格限制代码中的绝对长路径，倡导提升代码可读性。然而，**Rust 官方的 `cargo clippy --fix` 明确不支持该规则的自动修复**（见官方实现 [`clippy_lints/src/absolute_paths.rs`](https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/absolute_paths.rs)，仅发出警告而未提供任何可机读自动修复）。

痛点原因如下：

1. **跨语法树节点重叠补丁冲突**：修复长路径必须跨越语法树在文件头部插入 `use` 声明，并在函数或结构体内部替换标识符。同一文件中多处长路径替换极易导致补丁重叠冲突，官方工具直接放弃修复。
2. **名称遮蔽与作用域冲突**：若直接将 `std::io::Error` 简化为 `Error` 并引入 `use std::io::Error;`，一旦当前文件已有自定义 `enum Error` 或函数局部变量 `let Error = ...`，就会引发编译报错。
3. **特征方法多义性冲突**：若将 `std::fmt::Debug` 裸导入，任何同时实现了 `Debug` 和 `Display` 的对象在调用 `.fmt(f)` 时，将因方法多义性而导致编译失败。
4. **条件编译与宏环境破坏**：在带有 `#[cfg(feature = "...")]` 的代码块中若将路径提取为顶层无条件 `use`，会导致跨平台构建或 `no_std` 环境被破坏；在宏定义 `macro_rules!` 中盲目替换更会导致宏展开失效。
5. **大幅节约大模型辅助编程的上下文与令牌消耗**：
   - 现有人工智能编程助手在生成 Rust 代码时，为了避免缺失导入，极易产生大量长绝对路径（例如 `std::sync::atomic::AtomicBool::new()`、`tokio::sync::mpsc::channel()`、`std::collections::HashMap::new()` 等）；
   - 这些冗长的重复路径不仅让代码臃肿，还会**剧烈侵占有限的大模型上下文窗口，成倍浪费推理令牌与提问成本**；
   - 自动规约为精炼短路径并注入单条顶层导入后，代码结构高度紧凑，在多轮对话与代码审查中能显著降低令牌开销，并大幅加快模型的生成速度。
6. **人工手动修改极其枯燥**：工程中手工逐个修改成百上千处长路径、梳理顶层引用并规避各种符号冲突，不仅效率低下且极易引入疏漏。

**`fixrs` 彻底解决了上述全部痛点**：
- 采用确定性语法树作用域解析与冲突检测，遇同名符号时**智能提升保留上一级命名空间**（如保留 `io::Error` 并引入 `use std::io;`）；
- 针对 `(core|std)::fmt` 统一强制保留 `fmt::` 前缀并仅导入模块，从源头杜绝特征方法多义性；
- 严格隔离 `#[cfg]` 条件编译块与 `macro_rules!` 宏定义；
- 自动识别 `#![no_implicit_prelude]` 并实施安全保护；
- 原地修改后自动调用单文件管道 `rustfmt` 完成优雅排版。

---

## 多人协作自动化配置 Git 提交钩子

在多人协作团队中，默认的 `.git/hooks/` 属于本地私有目录，不会被 Git 版本库跟踪，新成员拉取代码后常常遗漏配置。

推荐采用 Git 原生支持的 **`core.hooksPath` 版本受控目录方案**（零第三方工具依赖，一人配置，全员自动同步）：

### 1. 将提交钩子纳入版本库管理

在项目根目录下创建受版本控制的 `.githooks` 目录，并添加 `pre-commit` 脚本：

```bash
mkdir -p .githooks
cat << 'EOF' > .githooks/pre-commit
#!/bin/sh
set -e

# 提交前全自动运行 fixrs 简化长绝对路径并格式化
echo "[git-hook] 正在运行 fixrs 优化路径..."
fixrs

# 将被自动修复的文件重新加入暂存区
git add -u
EOF

chmod +x .githooks/pre-commit
git add .githooks/pre-commit
```

### 2. 团队成员一行命令激活

团队新成员克隆仓库后，只需在项目根目录运行一行配置，将仓库钩子路径指向受控目录：

```bash
git config core.hooksPath .githooks
```

亦可将此命令加入项目的初始化脚本、构建脚本（如 Makefile、Justfile）或安装文档中。后续提交钩子的任何规则迭代与更新都会随 `git pull` 自动同步给全体成员，在每次 `git commit` 时全自动执行长路径清理与规范排版。

---

## 作为 Rust 库使用

`fixrs` 核心解析与改写引擎完全封装为高内聚、轻量级的 Rust 原生库，可轻松集成至代码生成器、过程宏测试、静态分析平台或工程重构工具链中。

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
fixrs = "0.1"
```

### 1. 内存中极速简化源码字符串（`fix_str`）

```rust
use fixrs::{Options, fix_str};

fn main() -> fixrs::Result<()> {
    let source = r#"
fn run() -> Result<(), std::io::Error> {
    let now = std::time::Instant::now();
    Ok(())
}
"#;

    let options = Options::new().max_segments(2);
    if let Some(fixed_code) = fix_str(source, &options)? {
        println!("{fixed_code}");
    }
    Ok(())
}
```

### 2. 获取改写结果与详细替换清单（`fix_source`）

```rust
use fixrs::{Options, fix_source};

let code = "fn test() { let _ = std::time::Instant::now(); }";
let options = Options::new().max_segments(2);

if let Some(result) = fix_source(code, &options)? {
    println!("改写后的源码:\n{}", result.content);
    for (original, replacement) in result.replacements {
        println!("替换项: {original} -> {replacement}");
    }
}
```

### 3. 处理单文件或整个项目工程（`fix_file` / `fix_project`）

```rust
use std::path::Path;
use fixrs::{Options, fix_file, fix_project};

// 处理单个源文件（自动识别最近的 Cargo.toml 解析依赖关系）
let file_result = fix_file(Path::new("src/main.rs"), &Options::default())?;

// 批量扫描并就地处理整个项目
let summary = fix_project(&Options::new().path("."))?;
println!("处理完成：成功改写 {} 个文件，共 {} 处替换", summary.changed_files, summary.total_replacements);
```

---

## 命令行参数与用法

### 常用命令

```bash
# 1. 默认执行：递归扫描当前 Cargo 项目所有源文件，就地修改并格式化
fixrs

# 2. 指定路径：处理特定目录或单文件
fixrs src/
fixrs src/main.rs

# 3. 试运行模式：仅在终端输出待替换明细，不改写文件
fixrs -n

# 4. CI 检查模式：发现未简化长路径时返回退出码 1
fixrs -c

# 5. 静默模式：不输出任何细节
fixrs -q
```

### 完整参数清单

| 参数 | 默认值 | 说明 |
| :--- | :--- | :--- |
| `[PATH]` | 自动向上检索的 `Cargo.toml` 根目录 | 待处理的目标目录或单个 `.rs` 文件 |
| `-n, --dry-run` | `false` | 试运行预览模式，不实际写回文件 |
| `-c, --check` | `false` | 检查模式，存在待简化路径时退出码为 1 |
| `-s, --show` | `false` | 强制在终端打印每个文件的详细替换明细 |
| `-q, --quiet` | `false` | 静默模式，压制明细输出 |
| `-m, --max-segments <N>` | `2` | 允许的最大绝对路径段数（与 Clippy 默认一致） |
| `-k, --keep-segments <N>` | `1` | 替换后代码中保留的段数（默认保留末尾 1 段） |
| `-a, --allow-crate <NAME>` | 无 | 允许保留绝对长路径的 crate 白名单（如 `-a std -a core`） |
| `--crate <NAME>` | 无 | 额外指定的已知外部 crate 名称 |
| `--no-cache` | `false` | 禁用增量缓存，强制重新解析全部文件 |
| `--verbose` | `false` | 输出详细处理日志 |

---

## 核心特性与设计保障

- **纯语法解析，零编译器依赖**：基于 `syn` 抽象语法树遍历，无需调用 `rustc` 或加载庞大元数据，单文件毫秒级完成。
- **异步高并发架构**：结合 `compio` 运行时调度，一个 CPU 核心绑定一个独立工作线程，超大工程数百毫秒极速完成。
- **智能增量缓存**：基于 `museair` 纳秒级哈希与项目元数据缓存，跳过未修改文件；规则变更时缓存自动失效。
- **依赖树感知**：自动解析 `Cargo.toml` 及工作区直接依赖与开发依赖，避免将项目内部模块误判为外部 crate。
- **智能文件遍历**：集成 `ignore` 库，严格遵循 `.gitignore`，自动跳过 `target/`、`node_modules/`、隐藏目录及构建输出。
- **工业级实测验证**：已连续在 **30+ 知名主流底层开源库**（包括 `tokio`、`serde`、`regex`、`tower`、`clap`、`rustix`、`mio`、`socket2`、`rand`、`tempfile`、`thiserror`、`tar`、`backtrace` 等）的真实代码库中实测验证，所有修改通过全部官方单元测试与回归测试！
