# fixrs

[![crates.io](https://img.shields.io/crates/v/fixrs.svg)](https://crates.io/crates/fixrs)
[![docs.rs](https://docs.rs/fixrs/badge.svg)](https://docs.rs/fixrs)
[![github](https://img.shields.io/badge/github-webc--site/fixrs-blue.svg?logo=github)](https://github.com/webc-site/fixrs)

Blazing-fast Rust absolute long path refactoring and `use` injection tool, purpose-built to automatically resolve `clippy::absolute_paths` warnings. **Can be used both as a standalone CLI / Cargo plugin, and embedded directly as a Rust library crate**.

---

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