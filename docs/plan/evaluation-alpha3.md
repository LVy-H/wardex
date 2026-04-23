# Evaluation — 0.3.0-alpha3

Checkpoint on state vs. [`long-term-strategy.md`](long-term-strategy.md) and the
remaining items in [`task-list.md`](task-list.md). Written 2026-04-23 at commit
`75aacae`.

## Shipped against the 0.3.x strategy

All five original 0.3.x themes from [`long-term-strategy.md`](long-term-strategy.md) line 15
are **implemented in code**. The 0.3.x line is still in alpha (0.3.0-alpha3).
Per the versioning policy in [`../../CHANGELOG.md`](../../CHANGELOG.md), the
stable 0.3.0 release still requires: alpha4 (test-depth additions) → beta1
(feature freeze) → field-soak → 0.3.0 cut.

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

## Recommended stabilization path: alpha4 → beta1 → 0.3.0

The gap analysis points at test coverage — `finish`/`archive` had 1 test
each at the start of alpha3; `schedule`/`check`/`recent` still do. These
are the destructive lifecycle commands where regression coverage matters
most. Stabilization-appropriate work: pure test additions and hygiene.

### alpha4: test-depth + toolchain pin

1. **T017 phase 2** — Add coverage to reach ≥3 tests per CTF command:
   - `finish`: end-time metadata write, no-git-repo path, unsolved-events path
   - `schedule`: update an existing schedule, both date orders
   - `check`: expired event, soon-to-expire, empty CTF root
   - `recent`: multi-event cycling, 5-entry cap enforcement
   - `import`: cross-device fallback (rename fails → copy+delete)
2. **T021** — Pin devshell and CI rust toolchains to the same version.
   Prevent recurrence of the 4-run red-CI cycle from alpha3.
3. **T013** — README pass adding Bash/Zsh wrapper examples for
   `ctf path --cd` / `ctf add --cd` (code is already escape-hardened).

### beta1 cut

When:

- Every CTF command has ≥3 integration tests.
- CI goes ≥10 consecutive pushes green without toolchain-drift breakage.
- Docs (`docs/ctf-lifecycle.md`, `docs/shell-output-contracts.md`) match
  current code.
- No new features accepted into this branch — bug fixes only.

### 0.3.0 release cut

When beta1 has soaked for one field-use cycle (a real CTF event or
equivalent) with zero regression reports.

### Deferred to 0.4.x (post-0.3.0)

- **T012 `ContextResolver` refactor**: whole-binary refactor with semantic
  risk. Wrong shape for a beta cycle; opens 0.4.x instead.

## Exit criteria for alpha4

- Every CTF command has ≥3 integration tests; no command left at 1 test.
- CI remains green end-to-end.
- `cargo clippy -- -D warnings` passes under the pinned toolchain.
- Any defects uncovered during test-writing have paired fix commits in
  the same alpha.
