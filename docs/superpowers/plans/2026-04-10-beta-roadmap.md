# Wardex 0.2.0 Beta Roadmap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship Wardex from 0.2.0-alpha6 to 0.2.0-beta1 across two intermediate alphas — fixing correctness issues, adding dynamic completions, and establishing CI.

**Architecture:** alpha7 fixes safety/correctness (unwrap panics, shell quoting, experimental labels, test coverage, clippy). alpha8 adds dynamic shell completions and hardens context resolution. beta1 adds CI and cuts the release branch.

**Tech Stack:** Rust, clap 4.5 + clap_complete 4.6, assert_cmd + predicates (tests), GitHub Actions (CI)

---

## File Map

### alpha7 — Files to modify

| File | Changes |
|------|---------|
| `src/engine/ctf/challenge.rs` | Replace 5 `.unwrap()` calls with proper error handling |
| `src/engine/ctf/shelve.rs` | Replace 3 `.unwrap()` calls with proper error handling |
| `src/engine/ctf/archive.rs` | Replace 1 `.unwrap()` call with proper error handling |
| `src/engine/stats.rs` | Replace `Arc::try_unwrap().unwrap()` chain, fix unused import |
| `src/main.rs` | Fix shell quoting in 3 `cd` output sites, add `[experimental]` to 11 command doc comments |
| `tests/cli_integration.rs` | Add ~7 new integration tests |

### alpha8 — Files to create/modify

| File | Changes |
|------|---------|
| `src/engine/ctf/completions.rs` | New: dynamic completion helpers (list events, categories, challenges) |
| `src/engine/ctf/mod.rs` | Add `pub mod completions;` |
| `src/main.rs` | Wire dynamic completions into clap args, or add `__complete` hidden subcommand |
| `tests/cli_integration.rs` | Add context resolution edge-case tests, output contract tests |

### beta1 — Files to create/modify

| File | Changes |
|------|---------|
| `.github/workflows/ci.yml` | New: CI pipeline |
| `Cargo.toml` | Version bump to `0.2.0-beta1` |
| `CHANGELOG.md` | alpha7, alpha8, beta1 entries |
| `docs/ctf-lifecycle.md` | Status: Stable |
| `docs/plan/roadmap.md` | Phase 1, 2 marked complete |

---

## Task 1: Fix unwrap() panics in challenge.rs

**Files:**
- Modify: `src/engine/ctf/challenge.rs:24,89,247-248,260-261,269,354`

- [ ] **Step 1: Fix `file_name().unwrap()` at line 24**

In `add_challenge()`, replace:
```rust
            let cat_name = current_dir.file_name().unwrap().to_string_lossy();
```
with:
```rust
            let cat_name = current_dir
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine category: current directory has no name"))?
                .to_string_lossy();
```

- [ ] **Step 2: Fix `file_name().unwrap()` at line 89**

In `solve_challenge()`, replace:
```rust
                    let cat_name = cwd.file_name().unwrap().to_string_lossy().to_string();
```
with:
```rust
                    let cat_name = cwd
                        .file_name()
                        .ok_or_else(|| anyhow::anyhow!("Cannot determine category: current directory has no name"))?
                        .to_string_lossy()
                        .to_string();
```

- [ ] **Step 3: Fix `file_name().unwrap()` at lines 247–248**

In `solve_challenge()` archive block, replace:
```rust
                    let category_name = category_dir.file_name().unwrap().to_string_lossy();
                    let event_name = event_dir.file_name().unwrap().to_string_lossy();
```
with:
```rust
                    let category_name = category_dir
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    let event_name = event_dir
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
```

- [ ] **Step 4: Fix `parent().unwrap()` at lines 260–261, 269**

Replace:
```rust
                    if !target_dir.parent().unwrap().exists() {
                        fs::create_dir_all(target_dir.parent().unwrap())?;
                    }
```
with:
```rust
                    if let Some(parent) = target_dir.parent() {
                        if !parent.exists() {
                            fs::create_dir_all(parent)?;
                        }
                    }
```

And replace:
```rust
                        fs_extra::dir::copy(&current_dir, target_dir.parent().unwrap(), &options)
```
with:
```rust
                        fs_extra::dir::copy(
                            &current_dir,
                            target_dir.parent().context("Archive target path has no parent directory")?,
                            &options,
                        )
```

- [ ] **Step 5: Fix `file_name().unwrap()` at line 354**

In `challenge_status()`, replace:
```rust
    let event_name = event_root.file_name().unwrap().to_string_lossy();
```
with:
```rust
    let event_name = event_root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
```

- [ ] **Step 6: Verify no remaining unwraps**

Run:
```bash
grep -n '\.unwrap()' src/engine/ctf/challenge.rs
```
Expected: zero lines (or only safe constant unwraps).

- [ ] **Step 7: Run tests**

Run: `cargo test`
Expected: all tests pass.

- [ ] **Step 8: Commit**

```bash
git add src/engine/ctf/challenge.rs
git commit -m "fix: Replace unwrap() panics with proper error handling in challenge.rs"
```

---

## Task 2: Fix unwrap() panics in shelve.rs and archive.rs

**Files:**
- Modify: `src/engine/ctf/shelve.rs:256,263,280`
- Modify: `src/engine/ctf/archive.rs:54`

- [ ] **Step 1: Fix shelve.rs `file_name().unwrap()` at lines 256 and 263**

In `archive_challenge()`, replace:
```rust
            let event_meta = CtfMeta::load(event_dir)?.unwrap_or_else(|| {
                CtfMeta::new(
                    &event_dir.file_name().unwrap().to_string_lossy(),
                    None,
                    None,
                    None,
                )
            });

            let event_name = event_dir.file_name().unwrap().to_string_lossy();
```
with:
```rust
            let event_dir_name = event_dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let event_meta = CtfMeta::load(event_dir)?.unwrap_or_else(|| {
                CtfMeta::new(&event_dir_name, None, None, None)
            });

            let event_name = &event_dir_name;
```

Note: also update the `ctf_archive_path` call below to use `event_name` (already a `&str`/`&String`, no `.as_ref()` change needed since we changed the type).

- [ ] **Step 2: Fix shelve.rs `parent().unwrap()` at line 280**

Replace:
```rust
                fs_extra::dir::copy(challenge_dir, target_dir.parent().unwrap(), &options)
                    .context("Failed to archive (cross-device move)")?;
```
with:
```rust
                fs_extra::dir::copy(
                    challenge_dir,
                    target_dir.parent().context("Archive target path has no parent directory")?,
                    &options,
                )
                .context("Failed to archive (cross-device move)")?;
```

- [ ] **Step 3: Fix archive.rs `file_name().unwrap()` at line 54**

In `archive_event()`, replace:
```rust
    let target_dir = archive_year_dir.join(event_dir.file_name().unwrap());
```
with:
```rust
    let dir_name = event_dir
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Event directory has no name: {:?}", event_dir))?;
    let target_dir = archive_year_dir.join(dir_name);
```

- [ ] **Step 4: Verify no remaining unwraps in both files**

Run:
```bash
grep -n '\.unwrap()' src/engine/ctf/shelve.rs src/engine/ctf/archive.rs
```
Expected: zero lines outside of safe patterns (`.unwrap_or`, `unwrap_or_else`, or dialoguer's `.unwrap_or(false)` on line 136 which is intentional).

- [ ] **Step 5: Run tests**

Run: `cargo test`
Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/engine/ctf/shelve.rs src/engine/ctf/archive.rs
git commit -m "fix: Replace unwrap() panics with proper error handling in shelve.rs and archive.rs"
```

---

## Task 3: Fix unwrap() chain in stats.rs and clean clippy warning

**Files:**
- Modify: `src/engine/stats.rs:89-91`

- [ ] **Step 1: Replace `Arc::try_unwrap().unwrap()` chain**

The Arc/Mutex are used in a single-threaded walker loop, so they always have refcount=1 at this point. But the unwrap is still a panic risk. Replace lines 89-91:

```rust
    let (total_size, total_files, file_types) =
        Arc::try_unwrap(stats_mutex).unwrap().into_inner().unwrap();
    let total_repos = Arc::try_unwrap(repos_count).unwrap().into_inner().unwrap();
```

with:

```rust
    let (total_size, total_files, file_types) = Arc::try_unwrap(stats_mutex)
        .map_err(|_| anyhow::anyhow!("Stats collection still in use"))?
        .into_inner()
        .map_err(|e| anyhow::anyhow!("Stats mutex poisoned: {}", e))?;
    let total_repos = Arc::try_unwrap(repos_count)
        .map_err(|_| anyhow::anyhow!("Repo counter still in use"))?
        .into_inner()
        .map_err(|e| anyhow::anyhow!("Repo counter mutex poisoned: {}", e))?;
```

- [ ] **Step 2: Remove unused import if present**

Check if there's an unused `tempfile::TempDir` import in a test module at the bottom of the file. If `stats.rs` has no test module, this warning may be in a different stats-related test. Verify:

```bash
grep -n 'tempfile' src/engine/stats.rs
```

If found, remove the unused import line.

- [ ] **Step 3: Run tests and clippy**

Run: `cargo test && cargo clippy --all-targets 2>&1 | grep -c 'warning'`
Expected: tests pass. Warning count should decrease.

- [ ] **Step 4: Commit**

```bash
git add src/engine/stats.rs
git commit -m "fix: Replace unwrap() panics in stats.rs with proper error propagation"
```

---

## Task 4: Fix shell quoting bug in cd output

**Files:**
- Modify: `src/main.rs:417,437,476`
- Test: `tests/cli_integration.rs`

- [ ] **Step 1: Write failing test for single-quote in path**

Add to `tests/cli_integration.rs`:

```rust
#[test]
#[serial_test::serial]
fn test_ctf_add_cd_escapes_single_quotes() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd()
        .args(&["ctf", "init", "QuoteTest"])
        .assert()
        .success();

    // Challenge name with a single quote
    env.cmd()
        .args(&["ctf", "add", "web/bob's-chall", "--cd"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bob'\\''s-chall"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_ctf_add_cd_escapes_single_quotes -- --nocapture`
Expected: FAIL — output contains unescaped single quote.

- [ ] **Step 3: Add shell_escape helper function in main.rs**

Add this helper function before `fn main()` (around line 297):

```rust
/// Escape a path for safe use in `cd '<path>'` shell evaluation.
/// Replaces `'` with `'\''` (end quote, escaped quote, start quote).
fn shell_quote_cd(path: &std::path::Path) -> String {
    let s = path.display().to_string();
    format!("cd '{}'", s.replace('\'', "'\\''"))
}
```

- [ ] **Step 4: Apply shell_quote_cd at line 417 (ctf add --cd)**

Replace:
```rust
                if *cd {
                    println!("cd '{}'", challenge_dir.display());
                }
```
with:
```rust
                if *cd {
                    println!("{}", shell_quote_cd(&challenge_dir));
                }
```

- [ ] **Step 5: Apply shell_quote_cd at line 437 (ctf path --cd)**

Replace:
```rust
                if *cd {
                    println!("cd '{}'", path.display());
                }
```
with:
```rust
                if *cd {
                    println!("{}", shell_quote_cd(&path));
                }
```

- [ ] **Step 6: Apply shell_quote_cd at line 476 (ctf work alias)**

Replace:
```rust
                println!("cd '{}'", challenge_dir.display());
```
with:
```rust
                println!("{}", shell_quote_cd(&challenge_dir));
```

- [ ] **Step 7: Run test to verify it passes**

Run: `cargo test test_ctf_add_cd_escapes_single_quotes -- --nocapture`
Expected: PASS.

- [ ] **Step 8: Run all tests**

Run: `cargo test`
Expected: all tests pass (including existing cd tests).

- [ ] **Step 9: Commit**

```bash
git add src/main.rs tests/cli_integration.rs
git commit -m "fix: Escape single quotes in cd output to prevent shell injection"
```

---

## Task 5: Mark non-CTF commands as experimental

**Files:**
- Modify: `src/main.rs:211-267` (Commands enum doc comments)
- Test: `tests/cli_integration.rs`

- [ ] **Step 1: Write test that verifies experimental labels**

Add to `tests/cli_integration.rs`:

```rust
#[test]
fn test_experimental_labels_in_help() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let output = cmd.args(&["--help"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Non-CTF commands should be marked experimental
    let experimental_commands = ["clean", "watch", "audit", "search", "find", "grep", "stats", "undo"];
    for cmd_name in experimental_commands {
        assert!(
            stdout.contains(&format!("[experimental]"))
                || stdout.lines().any(|l| l.contains(cmd_name) && l.contains("experimental")),
            "Command '{}' should be marked as experimental in help",
            cmd_name
        );
    }

    // CTF, config, completions should NOT be experimental
    for cmd_name in ["ctf", "config", "completions"] {
        let is_experimental = stdout.lines().any(|l| l.contains(cmd_name) && l.contains("experimental"));
        assert!(!is_experimental, "Command '{}' should NOT be marked experimental", cmd_name);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_experimental_labels_in_help`
Expected: FAIL — no experimental labels present.

- [ ] **Step 3: Update Commands enum doc comments**

In `src/main.rs`, update the doc comments on the `Commands` enum variants:

```rust
#[derive(Subcommand)]
enum Commands {
    /// [experimental] Initialize a new project
    Init {
        #[arg(short, long, help = "Project type (rust, python, node)")]
        type_: String,
        #[arg(short, long, help = "Project name")]
        name: String,
    },
    /// [experimental] Sort items from Inbox into Projects/Resources
    Clean {
        #[arg(long, help = "Simulate moves without executing")]
        dry_run: bool,
    },
    /// Manage CTF events
    Ctf {
        #[command(subcommand)]
        command: CtfCommands,
    },
    /// [experimental] Audit workspace health (files, empty folders)
    Audit,
    /// [experimental] Undo last movement operation
    Undo {
        #[arg(short, long, default_value_t = 1)]
        count: usize,
    },
    /// [experimental] Watch Inbox and auto-sort
    Watch,
    /// [experimental] Show git status dashboard
    Status,
    /// [experimental] Search for flags recursively
    Search {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(short, long)]
        pattern: Option<String>,
    },
    /// [experimental] Fuzzy find projects
    Find { name: String },
    /// [experimental] Grep content in Projects/Resources
    Grep { pattern: String },
    /// [experimental] Show workspace analytics
    Stats,
    /// [experimental] Launch interactive TUI dashboard
    Dashboard,
    /// [experimental] Quick file/project info
    Info { path: Option<PathBuf> },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Generate shell completion scripts
    Completions {
        #[arg(help = "Shell to generate completions for (bash, zsh)")]
        shell: Shell,
    },
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_experimental_labels_in_help`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/main.rs tests/cli_integration.rs
git commit -m "feat: Mark non-CTF commands as [experimental] in help text"
```

---

## Task 6: Expand CTF test coverage

**Files:**
- Modify: `tests/cli_integration.rs`

These tests cover untested core CTF commands. Each follows the existing `TestEnv` pattern.

- [ ] **Step 1: Add test for `ctf use`**

```rust
#[test]
#[serial_test::serial]
fn test_ctf_use_switches_context() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    // Create two events
    env.cmd().args(&["ctf", "init", "EventA"]).assert().success();
    env.cmd().args(&["ctf", "init", "EventB"]).assert().success();

    // Info should show EventB (last init auto-activates)
    env.cmd()
        .args(&["ctf", "info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("EventB"));

    // Switch to EventA
    env.cmd().args(&["ctf", "use", "EventA"]).assert().success();

    // Info should now show EventA
    env.cmd()
        .args(&["ctf", "info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("EventA"));
}
```

- [ ] **Step 2: Add test for `ctf info`**

```rust
#[test]
#[serial_test::serial]
fn test_ctf_info_shows_context() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd().args(&["ctf", "init", "InfoTestCTF"]).assert().success();

    env.cmd()
        .args(&["ctf", "info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("InfoTestCTF"));
}
```

- [ ] **Step 3: Add test for `ctf writeup`**

```rust
#[test]
#[serial_test::serial]
fn test_ctf_writeup_generates_output() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd().args(&["ctf", "init", "WriteupCTF"]).assert().success();

    // Add a challenge and write notes
    env.cmd().args(&["ctf", "add", "web/writeup-test"]).assert().success();

    let ctf_root = env.path().join("1_Projects/CTFs");
    let event_dir = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("WriteupCTF"))
        .unwrap()
        .path();

    // Write notes for the challenge
    let notes_path = event_dir.join("web/writeup-test/notes.md");
    fs::write(&notes_path, "# Solution\nUsed SQL injection on login form.").unwrap();

    // Generate writeup
    env.cmd()
        .args(&["ctf", "writeup"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated writeup"));

    // Verify Writeup.md was created
    assert!(event_dir.join("Writeup.md").exists());
    let content = fs::read_to_string(event_dir.join("Writeup.md")).unwrap();
    assert!(content.contains("writeup-test"));
    assert!(content.contains("SQL injection"));
}
```

- [ ] **Step 4: Add test for `ctf archive`**

```rust
#[test]
#[serial_test::serial]
fn test_ctf_archive_moves_event() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd().args(&["ctf", "init", "ArchiveTestCTF"]).assert().success();

    env.cmd()
        .args(&["ctf", "archive", "ArchiveTestCTF"])
        .assert()
        .success()
        .stdout(predicate::str::contains("archived"));

    // Verify event moved to archives
    let archives = env.path().join("4_Archives/CTFs");
    let archive_entries: Vec<_> = fs::read_dir(&archives)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(!archive_entries.is_empty(), "Event should be in archives");
}
```

- [ ] **Step 5: Add test for `ctf solve` (legacy alias)**

```rust
#[test]
#[serial_test::serial]
fn test_ctf_solve_legacy_writes_flag() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd().args(&["ctf", "init", "SolveLegacyCTF"]).assert().success();
    env.cmd().args(&["ctf", "add", "misc/solve-test"]).assert().success();

    let ctf_root = env.path().join("1_Projects/CTFs");
    let event_dir = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("SolveLegacyCTF"))
        .unwrap()
        .path();

    let challenge_dir = event_dir.join("misc/solve-test");

    // Init git for the commit
    let _ = std::process::Command::new("git").arg("init").current_dir(&event_dir).output();
    let _ = std::process::Command::new("git").args(&["config", "user.name", "Test"]).current_dir(&event_dir).output();
    let _ = std::process::Command::new("git").args(&["config", "user.email", "t@t.com"]).current_dir(&event_dir).output();
    let _ = std::process::Command::new("git").args(&["commit", "--allow-empty", "-m", "init"]).current_dir(&event_dir).output();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&challenge_dir);
    cmd.env("WX_PATHS_WORKSPACE", env.path());
    cmd.env("XDG_CONFIG_HOME", env.path());
    cmd.env("XDG_DATA_HOME", env.path());
    cmd.env("HOME", env.path());
    cmd.args(&["ctf", "solve", "flag{legacy_test}", "--no-archive"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Saved flag"));

    // Verify flag.txt was written (legacy solve writes flag.txt)
    assert!(challenge_dir.join("flag.txt").exists());
    let flag_content = fs::read_to_string(challenge_dir.join("flag.txt")).unwrap();
    assert_eq!(flag_content, "flag{legacy_test}");
}
```

- [ ] **Step 6: Run all tests**

Run: `cargo test`
Expected: all tests pass, including the 5+ new tests.

- [ ] **Step 7: Commit**

```bash
git add tests/cli_integration.rs
git commit -m "test: Add integration tests for ctf use, info, writeup, archive, and solve"
```

---

## Task 7: Clippy cleanup and alpha7 version bump

**Files:**
- Modify: `Cargo.toml`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Run clippy and fix remaining warnings**

Run: `cargo clippy --all-targets 2>&1 | grep 'warning\[' | head -20`

Fix any remaining warnings in `src/` code (not test framework deprecation warnings which are external). Common fixes:
- Needless borrows: `&foo` → `foo` where ownership isn't needed
- Unused imports: remove them

- [ ] **Step 2: Run full test suite**

Run: `cargo test`
Expected: all tests pass.

- [ ] **Step 3: Bump version to 0.2.0-alpha7**

In `Cargo.toml`, change:
```toml
version = "0.2.0-alpha6"
```
to:
```toml
version = "0.2.0-alpha7"
```

- [ ] **Step 4: Add CHANGELOG entry for alpha7**

Add to `CHANGELOG.md` after the alpha6 entry:

```markdown
## [0.2.0-alpha7] - 2026-04-10

### Summary

Safety and correctness release: fixes panic risks, shell quoting, marks non-CTF
commands as experimental, and expands test coverage.

### Fixed

| Fix | Description |
|-----|-------------|
| Panic on non-UTF-8 paths | `.unwrap()` calls in challenge.rs, shelve.rs, archive.rs replaced with proper error handling |
| Shell quoting in `--cd` output | Paths with single quotes no longer break `eval $(wardex ctf ...)` |
| Panic in stats.rs | `Arc::try_unwrap` chain replaced with error propagation |

### Changed

| Change | Before | After |
|--------|--------|-------|
| Non-CTF commands | No label | `[experimental]` prefix in help text |

### Added

| Feature | Description |
|---------|-------------|
| New tests | Integration tests for `ctf use`, `ctf info`, `ctf writeup`, `ctf archive`, `ctf solve` |
| Shell quoting test | Tests that paths with special characters produce valid shell output |

### Migration Guide

#### From 0.2.0-alpha6

**No breaking changes.** Non-CTF commands now show `[experimental]` in help text but behavior is unchanged.
```

- [ ] **Step 5: Commit and tag**

```bash
git add -A
git commit -m "chore: Bump version to 0.2.0-alpha7"
git tag v0.2.0-alpha7
```

---

## Task 8: Spike dynamic completion approach

**Files:**
- Read: `clap_complete` docs and source for custom value completion

This is a research task — determine the right mechanism before implementing.

- [ ] **Step 1: Check clap_complete custom completion support**

Run:
```bash
cargo doc --open -p clap_complete 2>/dev/null || cargo doc -p clap_complete
```

Check if `clap_complete` 4.6 supports:
- `CompleteEnv` for runtime completions
- Custom `ValueHint` or completion callbacks
- Hidden subcommand approach (e.g., `wardex __complete <context>`)

- [ ] **Step 2: Evaluate approaches**

Choose between:

**A) `clap_complete`'s built-in runtime completions** (if supported in 4.6):
- Pros: Integrated with clap, less code
- Cons: May not be stable or flexible enough

**B) Hidden `__complete` subcommand + shell function wrapper:**
- Pros: Full control, works with any shell
- Cons: More code, custom shell functions needed

**C) Custom shell functions that call wardex subcommands:**
- Pros: Simple, predictable
- Cons: Separate install step for dynamic completions

- [ ] **Step 3: Document decision**

Write a short note in the commit message about which approach was chosen and why.

- [ ] **Step 4: Commit research notes (if any files created)**

```bash
git commit --allow-empty -m "research: Determine dynamic completion approach for alpha8"
```

---

## Task 9: Implement dynamic completion helpers

**Files:**
- Create: `src/engine/ctf/completions.rs`
- Modify: `src/engine/ctf/mod.rs`

- [ ] **Step 1: Create completions.rs with list functions**

Create `src/engine/ctf/completions.rs`:

```rust
//! Dynamic completion helpers — list events, categories, and challenges
//! for shell tab-completion.

use crate::config::Config;
use std::path::PathBuf;

/// List all event directory names from the CTF root.
pub fn list_events(config: &Config) -> Vec<String> {
    let ctf_root = config.ctf_root();
    if !ctf_root.exists() {
        return Vec::new();
    }

    let mut events = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&ctf_root) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    events.push(name.to_string());
                }
            }
        }
    }
    events.sort();
    events
}

/// List category directories within an event (or the active event).
pub fn list_categories(config: &Config, event_root: Option<&PathBuf>) -> Vec<String> {
    let root = match event_root {
        Some(r) => r.clone(),
        None => match super::get_active_event_root() {
            Ok(r) => r,
            Err(_) => return config.ctf.default_categories.clone(),
        },
    };

    if !root.exists() {
        return Vec::new();
    }

    let mut categories = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&root) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.starts_with('.') {
                    categories.push(name);
                }
            }
        }
    }
    categories.sort();
    categories
}

/// List challenge paths as `category/challenge` within an event.
pub fn list_challenges(config: &Config, event_root: Option<&PathBuf>) -> Vec<String> {
    let root = match event_root {
        Some(r) => r.clone(),
        None => match super::get_active_event_root() {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        },
    };

    if !root.exists() {
        return Vec::new();
    }

    let mut challenges = Vec::new();
    if let Ok(cats) = std::fs::read_dir(&root) {
        for cat in cats.flatten() {
            if cat.path().is_dir() {
                let cat_name = cat.file_name().to_string_lossy().to_string();
                if cat_name.starts_with('.') {
                    continue;
                }
                if let Ok(chals) = std::fs::read_dir(cat.path()) {
                    for chal in chals.flatten() {
                        if chal.path().is_dir() {
                            let chal_name = chal.file_name().to_string_lossy().to_string();
                            challenges.push(format!("{}/{}", cat_name, chal_name));
                        }
                    }
                }
            }
        }
    }
    challenges.sort();
    challenges
}
```

- [ ] **Step 2: Register module in mod.rs**

Add to `src/engine/ctf/mod.rs`:

```rust
pub mod completions;
```

- [ ] **Step 3: Run cargo check**

Run: `cargo check`
Expected: compiles with no errors.

- [ ] **Step 4: Commit**

```bash
git add src/engine/ctf/completions.rs src/engine/ctf/mod.rs
git commit -m "feat: Add dynamic completion helpers for events, categories, and challenges"
```

---

## Task 10: Wire dynamic completions into CLI

**Files:**
- Modify: `src/main.rs`

The wiring approach depends on Task 8's spike result. This task shows the **hidden subcommand approach** as the most portable option. Adjust if the spike chose differently.

- [ ] **Step 1: Add hidden `__complete` subcommand to Commands enum**

Add to `Commands` enum in `main.rs`:

```rust
    /// Hidden: list completions for dynamic shell completion
    #[command(name = "__complete", hide = true)]
    Complete {
        #[arg(help = "What to complete: events, categories, challenges")]
        kind: String,
    },
```

- [ ] **Step 2: Add handler in main()**

Add a match arm in the main command dispatch:

```rust
        Commands::Complete { kind } => {
            let config = Config::load(&find_config(&cli.config)?)?;
            match kind.as_str() {
                "events" => {
                    for event in ctf::completions::list_events(&config) {
                        println!("{}", event);
                    }
                }
                "categories" => {
                    for cat in ctf::completions::list_categories(&config, None) {
                        println!("{}", cat);
                    }
                }
                "challenges" => {
                    for chal in ctf::completions::list_challenges(&config, None) {
                        println!("{}", chal);
                    }
                }
                _ => {}
            }
        }
```

- [ ] **Step 3: Update completion script generation**

The static `wardex completions bash/zsh` won't auto-include dynamic completions. Users need a shell function wrapper. Document this in help or output a comment at the top of the generated completion script.

Add a note after completion generation in main.rs (near line 310):

```rust
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            clap_complete::generate(*shell, &mut cmd, "wardex", &mut std::io::stdout());
            // Print dynamic completion hint as a comment
            eprintln!("# For dynamic event/category completion, see: wardex --help or docs/rfcs/0002-shell-completions.md");
        }
```

- [ ] **Step 4: Run cargo check**

Run: `cargo check`
Expected: compiles with no errors.

- [ ] **Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: Add __complete hidden subcommand for dynamic shell completions"
```

---

## Task 11: Add context resolution edge-case tests

**Files:**
- Modify: `tests/cli_integration.rs`

- [ ] **Step 1: Test global state with deleted event**

```rust
#[test]
#[serial_test::serial]
fn test_ctf_info_after_event_deleted() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd().args(&["ctf", "init", "DeletedEvent"]).assert().success();

    // Find and delete the event directory
    let ctf_root = env.path().join("1_Projects/CTFs");
    let event_dir = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("DeletedEvent"))
        .unwrap()
        .path();
    fs::remove_dir_all(&event_dir).unwrap();

    // Info should fail gracefully, not panic
    env.cmd()
        .args(&["ctf", "info"])
        .assert()
        .failure();
}
```

- [ ] **Step 2: Test path with spaces in event name**

```rust
#[test]
#[serial_test::serial]
fn test_ctf_path_with_spaces() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd().args(&["ctf", "init", "My Event 2026"]).assert().success();

    env.cmd()
        .args(&["ctf", "path", "My Event"])
        .assert()
        .success()
        .stdout(predicate::str::contains("My Event 2026"));

    // --cd should produce valid shell with spaces
    env.cmd()
        .args(&["ctf", "path", "My Event", "--cd"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("cd '"));
}
```

- [ ] **Step 3: Test shell output contracts**

```rust
#[test]
#[serial_test::serial]
fn test_ctf_path_bare_output_no_decoration() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd().args(&["ctf", "init", "BarePathTest"]).assert().success();

    let output = env.cmd()
        .args(&["ctf", "path", "BarePathTest"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Bare path: no "cd", no quotes, no ANSI codes, just the path + newline
    assert!(!stdout.contains("cd "), "Bare path should not contain cd");
    assert!(!stdout.contains('\x1b'), "Bare path should not contain ANSI escape codes");
    assert!(stdout.trim().ends_with("BarePathTest") || stdout.contains("BarePathTest"));
}
```

- [ ] **Step 4: Run all tests**

Run: `cargo test`
Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add tests/cli_integration.rs
git commit -m "test: Add context resolution edge-case and shell output contract tests"
```

---

## Task 12: alpha8 version bump

**Files:**
- Modify: `Cargo.toml`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Bump version**

In `Cargo.toml`, change:
```toml
version = "0.2.0-alpha7"
```
to:
```toml
version = "0.2.0-alpha8"
```

- [ ] **Step 2: Add CHANGELOG entry**

Add to `CHANGELOG.md`:

```markdown
## [0.2.0-alpha8] - 2026-04-XX

### Summary

Ships **dynamic shell completions** for event names, categories, and challenge paths.
Hardens context resolution with edge-case tests and shell output contract verification.

### Added

| Feature | Description | Status |
|---------|-------------|--------|
| Dynamic completions | `wardex __complete events\|categories\|challenges` hidden subcommand | Experimental |
| Completion helpers | `src/engine/ctf/completions.rs` — list events, categories, challenges | Stable |
| Context edge-case tests | Deleted event, spaces in names, bare output format | - |
| Shell output contract tests | Verify `--cd` output format and bare path output | - |

### Migration Guide

#### From 0.2.0-alpha7

**No breaking changes.** Dynamic completions are opt-in via shell configuration.
```

- [ ] **Step 3: Run tests and commit**

```bash
cargo test
git add -A
git commit -m "chore: Bump version to 0.2.0-alpha8"
git tag v0.2.0-alpha8
```

---

## Task 13: Add CI workflow and cut beta

**Files:**
- Create: `.github/workflows/ci.yml`
- Modify: `Cargo.toml`
- Modify: `CHANGELOG.md`
- Modify: `docs/ctf-lifecycle.md`
- Modify: `docs/plan/roadmap.md`

- [ ] **Step 1: Create CI workflow**

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main, "release/**"]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo test
```

- [ ] **Step 2: Bump version to beta1**

In `Cargo.toml`, change:
```toml
version = "0.2.0-alpha8"
```
to:
```toml
version = "0.2.0-beta1"
```

- [ ] **Step 3: Update docs status markers**

In `docs/ctf-lifecycle.md`, change:
```
Status: Implemented (alpha4–alpha6)
```
to:
```
Status: Stable
```

In `docs/plan/roadmap.md`, update Phase 1 and Phase 2 headers to include `(Complete)`.

- [ ] **Step 4: Add beta1 CHANGELOG entry**

```markdown
## [0.2.0-beta1] - 2026-04-XX

### Summary

First beta release. The CTF workflow and shell integration are feature-complete.
Breaking changes from this point only for critical bugs.

### Changed

| Change | Description |
|--------|-------------|
| Version policy | Beta: feature-complete, breaking changes only for critical bugs |
| CI | GitHub Actions workflow: fmt, clippy, test on every push and PR |
| CTF lifecycle status | Promoted from "Implemented" to "Stable" |

### Migration Guide

#### From 0.2.0-alpha8

**No breaking changes.** This release formalizes the beta stability contract.
```

- [ ] **Step 5: Run all tests one final time**

Run: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`
Expected: all pass with zero warnings.

- [ ] **Step 6: Commit, tag, and create release branch**

```bash
git add -A
git commit -m "release: Wardex 0.2.0-beta1"
git tag v0.2.0-beta1
git branch release/0.2
```

---

## Dependency Graph

```
Task 1 (challenge.rs panics)  ──┐
Task 2 (shelve/archive panics) ─┤
Task 3 (stats.rs panics)  ──────┤── independent, can parallelize
Task 4 (shell quoting)  ────────┤
Task 5 (experimental labels)  ──┤
Task 6 (CTF test coverage)  ────┘
                                 │
Task 7 (clippy + alpha7 bump)  ──┤── depends on all above
                                 │
Task 8 (completion spike)  ──────┤── can start after alpha7
Task 9 (completion helpers)  ────┤── depends on Task 8
Task 10 (wire completions)  ─────┤── depends on Task 9
Task 11 (edge-case tests)  ──────┤── independent of 8-10
                                 │
Task 12 (alpha8 bump)  ──────────┤── depends on 8-11
                                 │
Task 13 (CI + beta1 cut)  ───────┘── depends on Task 12
```
