# Wardex 0.2.0 Beta Roadmap

**Date:** 2026-04-10
**Status:** Draft
**Scope:** alpha7 → alpha8 → beta1

## Goal

Ship Wardex 0.2.0-beta1 with a reliable, fully tested CTF-first CLI and shell-native completions. Beta contract: feature-complete, breaking changes only for critical bugs.

## Release Sequence

| Release | Theme | Gate |
|---------|-------|------|
| **alpha7** | Correctness & safety | All unwrap panics fixed, shell quoting safe, non-CTF marked experimental, CTF test coverage expanded |
| **alpha8** | Shell-native completions | Dynamic completions for events/categories/challenges, context resolution hardened, shell output contract tests |
| **beta1** | Release cut | CI gate passes, final doc pass, `release/0.2` branch cut |

## Out of Scope

- Non-CTF commands: remain functional but labeled `[experimental]`, active support deferred to 0.5.x
- Challenge templates: dropped permanently — CTFs too diverse for opinionated scaffolding
- Writeup improvements: deferred to 0.3.x
- Config load-time validation: deferred to 0.3.x
- TUI improvements: deferred to 0.5.x
- 0.4.x: unassigned buffer release for post-beta field feedback

## Version Roadmap

| Version | Theme |
|---------|-------|
| **0.2.x** | Beta — CTF + shell reliability |
| **0.3.x** | CTF polish — writeup, config validation, minor UX |
| **0.4.x** | Buffer — fixes and workflow gaps from real competition use |
| **0.5.x** | Non-CTF commands active support |

---

## alpha7: Correctness & Safety

### A7-1: Fix unwrap() panics

Replace bare `.unwrap()` calls with proper error handling using `.context()` or `bail!()`.

| File | Lines | Pattern | Fix |
|------|-------|---------|-----|
| `src/engine/ctf/challenge.rs` | 24, 89 | `file_name().unwrap()` on CWD | `.ok_or_else(\|\| anyhow!("..."))` with message about invalid directory name |
| `src/engine/ctf/challenge.rs` | 260–261, 269 | `parent().unwrap()` on archive target path | `.context("archive target path has no parent")?` |
| `src/engine/ctf/shelve.rs` | 256, 263 | `file_name().unwrap()` on event dir | `.ok_or_else(\|\| anyhow!("..."))` |
| `src/engine/ctf/shelve.rs` | 280 | `parent().unwrap()` on target dir | `.context("...")?` |
| `src/engine/ctf/archive.rs` | 54 | `file_name().unwrap()` on event dir | `.ok_or_else(\|\| anyhow!("..."))` |
| `src/engine/stats.rs` | 90–91 | `Arc::try_unwrap().unwrap().into_inner().unwrap()` | Chain with `.map_err()` or use `Arc::try_unwrap().ok()` with fallback |

**Proof artifact:** `cargo test` passes. `grep -rn '\.unwrap()' src/engine/ctf/ src/engine/stats.rs` returns zero hits outside of test code and safe constant unwraps (e.g., `NaiveTime::from_hms_opt(0,0,0).unwrap()`).

### A7-2: Fix shell quoting bug

`main.rs:417,437,476` currently emit:
```rust
println!("cd '{}'", path.display());
```

A path containing a single quote (e.g., `/tmp/bob's_ctf/web/chall`) produces broken shell: `cd '/tmp/bob's_ctf/web/chall'`.

**Fix:** POSIX-safe single-quote escaping — replace `'` with `'\''` inside the path:

```rust
fn shell_escape(path: &std::path::Path) -> String {
    let s = path.display().to_string();
    format!("cd '{}'", s.replace("'", "'\\''"))
}
```

Apply to all three call sites in main.rs (lines 417, 437, 476).

**Proof artifact:** New integration test that creates a challenge with a single-quote in the path and verifies the `--cd` output is valid shell.

### A7-3: Mark non-CTF commands as experimental

Add `[experimental]` to the `about` attribute of these Commands variants in `main.rs`:

- `Init` (project scaffolding)
- `Clean`
- `Watch`
- `Status` (git dashboard — the top-level one, not `ctf status`)
- `Audit`
- `Search`
- `Find`
- `Grep`
- `Stats`
- `Undo`
- `Dashboard`

Example:
```rust
/// [experimental] Sort items from Inbox into Projects/Resources
Clean { ... }
```

No behavior changes. Help text only.

**Proof artifact:** `wardex --help` output shows `[experimental]` on all listed commands. Existing tests still pass.

### A7-4: Expand CTF test coverage

Add integration tests for untested core CTF commands. Each test uses the existing `TestEnv` helper pattern from `tests/cli_integration.rs`.

| Command | Test cases |
|---------|------------|
| `ctf use` | Set active event, verify subsequent commands use it |
| `ctf solve` | Legacy alias works, writes flag to metadata, respects `--no-commit` and `--no-archive` |
| `ctf info` | Shows current context (event name, path) |
| `ctf writeup` | Generates writeup from challenge notes |
| `ctf archive` | Moves event to archives directory |

**Proof artifact:** `cargo test` passes with all new tests. Test count increases by at least 5.

### A7-5: Clippy cleanup

- Fix unused import `tempfile::TempDir` in `src/engine/stats.rs:125`
- Address needless-borrow warnings in test code
- Target: `cargo clippy --all-targets 2>&1 | grep warning | wc -l` is under 5 (framework deprecation warnings may remain)

**Proof artifact:** `cargo clippy --all-targets` output is clean or near-clean.

---

## alpha8: Shell-Native Completions

### A8-1: Dynamic completion for event names

Implement custom completer for `ctf use`, `ctf path`, `ctf schedule`, `ctf finish`, `ctf archive` that reads event directories from the CTF root and suggests matching names.

Approach: use `clap_complete`'s `CompleteEnv` runtime completion, or custom shell functions that call a wardex subcommand to list candidates. The exact mechanism should be spiked early in alpha8 — RFC 0002 intentionally deferred this decision. Either way, the completer scans `{ctf_root}/` for directories containing `.ctf_meta.json`.

**Behavior:**
- Lists event names from CTF root directory
- Works without an active event context
- Gracefully returns empty if CTF root doesn't exist or config is missing

### A8-2: Dynamic completion for categories

Implement custom completer for the category portion of `ctf add <cat/name>` that reads category directories from the active event.

**Behavior:**
- Lists subdirectories of the active event root
- Falls back to default categories from config if no active event
- Triggers on the first path component before `/`

### A8-3: Dynamic completion for challenge paths

Implement custom completer for `ctf path <event> <challenge>` and `ctf shelve` (when run from outside a challenge dir) that suggests `category/challenge` paths within the active or specified event.

**Behavior:**
- Lists `{category}/{challenge}` paths from event directory
- Respects context resolution (active event or specified event)
- Returns empty gracefully if context can't be resolved

### A8-4: Context resolution hardening

Add regression tests for edge cases in context resolution:

| Scenario | Expected behavior |
|----------|-------------------|
| CWD inside challenge dir | Local context wins |
| CWD outside any event, global state set | Global state used |
| CWD outside any event, no global state | Fall back to latest event |
| Global state points to deleted event | Clear error, not silent wrong-target |
| Ambiguous fuzzy match | Error listing candidates, not silent pick |
| Event name with special chars (spaces, quotes) | Resolves correctly |

**Proof artifact:** Tests covering all rows above.

### A8-5: Shell output contract tests

Add integration tests that verify the stable output contracts documented in `docs/shell-output-contracts.md`:

- `ctf path --cd` output matches `cd '<escaped-path>'`
- `ctf add --cd` output matches `cd '<escaped-path>'`
- `ctf path` (no --cd) outputs a bare path with no decoration
- Machine-parseable outputs contain no ANSI escape codes

**Proof artifact:** Tests that parse command output and assert format.

---

## beta1: Release Cut

### B1-1: CI gate

Add GitHub Actions workflow (`.github/workflows/ci.yml`):

```yaml
jobs:
  check:
    - cargo fmt --check
    - cargo clippy --all-targets --all-features -- -D warnings
    - cargo test
```

### B1-2: Final documentation pass

- Update CHANGELOG.md with alpha7 and alpha8 entries
- Update README.md if any commands changed
- Review docs/ctf-lifecycle.md status (should be "Stable" at beta)
- Update docs/plan/roadmap.md Phase 1 and Phase 2 as complete

### B1-3: Version bump and branch cut

- Bump `Cargo.toml` version to `0.2.0-beta1`
- Create `release/0.2` branch from main
- Tag `v0.2.0-beta1`
- Main continues with 0.3.0-dev work

---

## Dependency Graph

```
A7-1 (fix panics) ─────────────────────┐
A7-2 (shell quoting) ──────────────────┤
A7-3 (experimental labels) ────────────┤── alpha7 release
A7-4 (CTF test coverage) ──────────────┤
A7-5 (clippy cleanup) ─────────────────┘
                                        │
                                        ▼
A8-1 (event completion) ───────────────┐
A8-2 (category completion) ────────────┤
A8-3 (challenge completion) ───────────┤── alpha8 release
A8-4 (context hardening) ──────────────┤
A8-5 (output contract tests) ──────────┘
                                        │
                                        ▼
B1-1 (CI gate) ────────────────────────┐
B1-2 (doc pass) ───────────────────────┤── beta1 release
B1-3 (version bump + branch cut) ──────┘
```

Within each alpha, tasks are independent and can be parallelized. The alpha releases are sequential gates.

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Dynamic completions complex with clap_complete | Medium | Could delay alpha8 | Start with event-name completion only, defer challenge-path if needed |
| Shell escaping edge cases beyond single quotes | Low | Medium | Test with space, backslash, dollar, backtick, newline in paths |
| Stats.rs Arc unwrap harder to fix cleanly | Low | Low | Acceptable to clone data out of Arc instead of unwrapping |
| CI setup blocks on GitHub Actions config | Low | Low | Can gate on local `cargo test` if Actions setup is slow |
