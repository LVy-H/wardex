# Evaluation — 0.3.0-alpha3

Checkpoint on state vs. [`long-term-strategy.md`](long-term-strategy.md) and the
remaining items in [`task-list.md`](task-list.md). Written 2026-04-23 at commit
`75aacae`.

## Shipped against the 0.3.x strategy

All five original 0.3.x themes from [`long-term-strategy.md`](long-term-strategy.md) line 15
are now shipped:

| Theme | Commits | Shipped in |
|---|---|---|
| Writeup generation improvements (metadata-driven, flag redaction) | `0ac5876` | 0.3.0-alpha1 |
| Config validation (`wardex config validate`) | `c0f143e` | 0.3.0-alpha1 |
| CTF status enrichment (solver, notes, summary, JSON output) | `209a41e` | 0.3.0-alpha1 |
| Multi-event context handling (`ctf use -`, `ctf recent`) | `d5caf87` | 0.3.0-alpha1 |
| Challenge-path dynamic completion | `acbaf67` | 0.3.0-alpha1 |

Plugin/extension system (deferred to 0.4+) is the only remaining 0.3.x item, and
the strategy explicitly marks it as "too early" with "document hook points instead" —
not blocking.

## Work done in this alpha3 cycle (not originally in strategy)

| Area | Commit |
|---|---|
| Fix `~<TAB>` leaking `/etc/passwd` user list into completions | `880ca31` |
| Migrate Nix build to `crane` (incremental, dep-cached) | `06e6955` |
| Home Manager `zsh.initExtra` → `initContent` deprecation | (alpha2) |
| Rewrite CTF completers to be instruction-based (no silent fallbacks) | (alpha2) |
| `assert_cmd::Command::cargo_bin` deprecation migration | `1ffcaa1` |
| Clippy `unnecessary_sort_by` / rust-overlay 1.95 sync | `75aacae` |

These were driven by field feedback and maintenance debt surfacing during the
alpha2 rollout. They were not in the original 0.3.x plan but fit the
"stabilize before expanding" principle from [`README.md`](README.md) line 30.

## Test surface (tests/cli_integration.rs)

Per-CTF-command test count:

| Command | Tests | Risk if broken |
|---|---|---|
| `shelve` | 6 | high — active-use command |
| `add` | 5 | high |
| `init` | 5 | high |
| `path` | 4 | high — eval-wrapped in shell |
| `import` | 2 | medium |
| `status` | 2 | low |
| `writeup` | 2 | medium |
| `use` | 2 | medium |
| `solve` (alias) | 2 | low |
| `list` | 2 | low |
| `info` | 2 | low |
| **`finish`** | **1** | **high — destructive, competition end-of-day** |
| **`archive`** | **1** | **high — moves full event dir** |
| `schedule` | 1 | low |
| `check` | 1 | low |
| `recent` | 1 | low |

The **`finish` and `archive`** commands are both destructive event-lifecycle
operations with only one test each. These are the riskiest commands in the
binary (they move or compress entire directory trees) and are invoked at the
end of a competition when the user has least tolerance for bugs.

## CI health

CI is green as of `75aacae`. The prior 4 consecutive red runs (alpha2 through
`cargo_bin_cmd` migration) were caused by:

1. `cargo fmt --check` under stable rustfmt 1.8 wanting wrapped asserts that
   pinned devshell 1.92 rustfmt let pass.
2. `cargo clippy -- -D warnings` under stable 1.95 firing
   `unnecessary_sort_by` that 1.92 didn't have.

Root cause: **devshell toolchain drift vs CI toolchain**. Fixed in `75aacae` by
bumping `rust-overlay` in `flake.lock`. As long as `flake.lock` stays current
with nixpkgs-unstable, the devshell will track what CI's
`dtolnay/rust-toolchain@stable` pulls.

Long-term fix: pin the CI toolchain (`dtolnay/rust-toolchain@1.95.0` or use the
flake's devshell under `DeterminateSystems/nix-installer-action`) so both sides
move on one lever instead of two.

## Remaining roadmap items

From [`task-list.md`](task-list.md), with current ranking:

### High value, not done

- **T017 — Expand CTF integration coverage**. The test-surface analysis above
  puts this first. `finish` and `archive` specifically need:
  - successful archive move with metadata preservation
  - cross-device archive fallback (copy+delete)
  - `finish` with `--no-archive`
  - `finish --dry-run` semantics
  - `finish` on an event with unsolved challenges
  - `archive` collision (target dir already exists)
- **T012 — Make context resolution predictable**. Resolution precedence
  (`local cwd > global state > latest event`) is mentioned in
  [`shell-output-contracts.md`](../shell-output-contracts.md) but not enforced
  through a single resolver in code. Each command computes it locally, which is
  why each new bug in this area (alpha2 `~` handling, silent-fallback in
  challenge completer) has been a fresh find. One `ContextResolver` type with
  an explicit precedence chain and a set of unit tests would collapse the
  surface area.

### Lower value, defer

- **T015 — Better challenge templates**: user demand unclear; `.challenge.json`
  already covers the metadata half.
- **T016 — Notes/writeup convention**: writeup is metadata-driven now; the
  remaining ambiguity is cosmetic.
- **T018 — Completion verification**: alpha3 already added 8 unit tests for the
  completion helpers. Dynamic completion has adequate coverage for its size.

## Recommended next phase: T017 hardening

The gap analysis points at the same target twice — through test coverage
(`finish`/`archive` have 1 test each) and through risk model (those two
commands are the most destructive in the binary).

Concretely for the next alpha:

1. Add ≥5 tests covering `finish` (dry-run, `--no-archive`, grace-period
   behaviour, event with unsolved challenges, happy path vs sad path).
2. Add ≥4 tests covering `archive` (rename vs copy+delete, target-exists,
   metadata-preservation, cross-event collision).
3. Add ≥3 tests covering `schedule`/`check`/`recent` where existing single
   tests are smoke-only.
4. If coverage reveals actual defects, fix them in follow-up commits within
   the same alpha.

After that: fold T012 into an alpha5, since that's a refactor with semantic
risk and deserves a clear gate rather than being bundled with test work.

## Exit criteria for next alpha (0.3.0-alpha4)

- `finish` and `archive` each have ≥5 and ≥4 tests respectively.
- No CTF command has only 1 integration test.
- Any defects uncovered during test-writing have paired fix commits.
- CI remains green end-to-end.
- `cargo clippy -- -D warnings` passes under the current stable toolchain.
