# Task List

This file turns the current CTF-shell strategy into an execution-ready backlog.

The recommendation is to build Wardex in this order:

1. Lock the CTF command contract around the shelve system.
2. Implement the shelve system and command alias mappings.
3. Implement shell completion and navigation support.
4. Harden shell-safe output and context behavior.
5. Improve challenge templates and writeup flow.
6. Add release-quality verification around the flagship path.

## Immediate Next Jobs

These are the best next tasks to start with.

### T001: Define The Canonical CTF Lifecycle (DONE)

Goal: remove ambiguity from the current command surface.

Status: Completed in `docs/ctf-lifecycle.md`.

Decisions made:

- `shelve` is the primary challenge completion verb (replaces `solve` and `done`)
- `add --cd` replaces `work` for shell navigation
- interactive-first design: navigable prompts by default, flags to skip
- challenge metadata in `.challenge.json` (flag, status, notes, not `flag.txt`)
- file triage with configurable blacklist (delete) and whitelist (keep)
- core six commands: `init`, `use`, `add`, `import`, `shelve`, `finish`
- `solve`, `done`, `work` become hidden aliases

### T002: Write RFC 0002 For Shell Completion Architecture (DONE)

Goal: choose the completion approach before implementation diverges.

Status: Completed. RFC 0002 accepted in `docs/rfcs/0002-shell-completions.md`. Decided on `clap_complete` for static completions via `wardex completions <shell>`, dynamic completions deferred.

### T003: Stabilize Shell-Oriented Output Contracts (DONE)

Goal: make shell-facing commands safe to wrap and evaluate.

Status: Completed in `docs/shell-output-contracts.md`. Documented stable vs unstable output for shell-facing commands.

## Milestone 1: Shelve System Implementation

### T004: Implement Challenge Metadata Schema (DONE)

Goal: replace `flag.txt` with structured per-challenge metadata.

Status: Completed in 0.2.0-alpha4. `ChallengeMetadata` struct with `.challenge.json`, schema versioning, and `flag.txt` migration implemented.

### T005: Implement `ctf shelve` Interactive Flow (DONE)

Goal: build the signature shelve command with interactive-first design.

Status: Completed in 0.2.0-alpha5. Full interactive flow with status, flag, file triage, note, and archive prompts. All flags (`--no-clean`, `--note`, `--move`/`--no-move`, `--auto`, `--no-commit`) implemented.

### T006: Add `--cd` Flag To `ctf add` (DONE)

Goal: make the primary creation verb also serve shell navigation.

Status: Completed in 0.2.0-alpha4. `ctf add --cd` prints `cd '<path>'` for eval. `work` is a hidden alias.

### T007: Implement File Triage System (DONE)

Goal: smart file cleanup during shelve with configurable patterns.

Status: Completed across 0.2.0-alpha5 (hardcoded patterns) and 0.2.0-alpha6 (configurable `ctf.shelve.blacklist`/`whitelist` in config.yaml).

## Milestone 2: Shell Completion

### T008: Write RFC 0002 For Shell Completion Architecture (DONE)

Same as T002. See above.

### T009: Add A Completion Entry Point (DONE)

Goal: give users a standard way to install completions.

Status: Completed in 0.2.0-alpha6. `wardex completions <bash|zsh>` generates completion scripts. Install docs in CHANGELOG and RFC 0002.

### T010: Implement Static Completion For Commands And Flags (DONE)

Goal: complete the command tree and common options.

Status: Completed in 0.2.0-alpha6. Uses `clap_complete` for subcommand and flag completion. Bash and Zsh supported.

### T011: Implement Dynamic Completion For Events And Categories (DONE)

Goal: reduce typing during real event work.

Status: Completed in 0.2.0-alpha8 (`c0fea17`). `event_completer` and
`category_completer` live in `src/engine/ctf/completions.rs`. Rewritten to
instruction-based (no silent fallback to guessed events) in 0.3.0-alpha2.

Deliverables shipped:

- ~~event-name completion for `ctf use`, `ctf path`, and related commands~~
- ~~category completion for `ctf add`, `ctf import` where applicable~~
- ~~sensible behavior when no active event exists~~ (returns empty per
  instruction-based refactor)

## Milestone 3: Context And Navigation Hardening

### T012: Make Context Resolution Predictable (DEFERRED to 0.4.x)

Goal: ensure shell wrappers and completions can rely on Wardex behavior.

Status: partially addressed alpha-by-alpha as bugs surface (alpha2 tilde
handling, alpha2 completer silent-fallback removal, alpha3 `~<TAB>` fix).
Each fix is correct but local — the broader refactor into a single
`ContextResolver` module is still outstanding.

**Deferred to 0.4.x**: this refactor touches every command's resolution
code, too large a blast-radius for the 0.3.x beta cycle. Closing it out
becomes the opening workstream of 0.4.x after 0.3.0 releases.

Deliverables:

- single `ContextResolver` type owning `local CWD > global state > explicit arg
  > latest event` precedence — one implementation, one unit-test suite.
- replace per-command resolvers in `engine/ctf/event.rs::find_event_root` /
  `get_active_event_root` / `resolve.rs` with `ContextResolver` calls.
- improve ambiguity errors (explicit arg vs active context mismatch).
- document the resolver in `docs/shell-output-contracts.md` and
  `docs/ctf-lifecycle.md`.

Depends on:

- T001 (DONE)

### T013: Harden `ctf path --cd` And `ctf add --cd`

Goal: make navigation commands dependable enough for daily shell use.

Deliverables:

- verify eval-safe output for both commands
- align behavior with the command contract
- add wrapper examples for Bash and Zsh

Depends on:

- T003
- T006
- T012

### T014: Dynamic Challenge-Path Completion (DONE)

Goal: complete real challenge targets, not just command names.

Status: Completed in 0.3.0-alpha1 (`acbaf67`) via `challenge_completer` in
`src/engine/ctf/completions.rs`. Rewritten to instruction-based (returns empty
when no active event instead of guessing "latest") in 0.3.0-alpha2.

Deliverables shipped:

- ~~challenge-path completion based on current or active event context~~
- ~~support category/name flows cleanly~~
- ~~handle missing context gracefully~~ (returns empty, nudges user toward
  `wardex ctf use <event>`)

Note: the "handle missing context gracefully" deliverable interacts with
T012 — the current implementation is correct but duplicates the resolution
logic. T012 will DRY this.

## Milestone 4: CTF Workflow Polish

### T015: Improve Challenge Templates

Goal: make new challenge folders useful immediately.

Deliverables:

- better category-specific scaffold defaults
- `.challenge.json` created with useful defaults on `add` and `import`
- tighter connection between templates and writeup generation

Depends on:

- T004
- T006

### T016: Improve Notes And Writeup Flow

Goal: make solve artifacts more consistent.

Deliverables:

- define a default notes convention
- improve `writeup` to read from `.challenge.json` metadata
- ensure shelve-time data feeds writeup assembly cleanly

Depends on:

- T005
- T015

## Milestone 5: Tests And Release Readiness

### T017: Expand CTF Integration Coverage (PHASE 1 DONE — PHASE 2 OPEN, target 0.3.0-alpha4)

Goal: protect the flagship workflow with regression tests.

Phase 1 (alpha3, `26efc36`):
- ~~`archive` fails cleanly on unknown event~~
- ~~`archive` matches partial event names~~
- ~~`finish --no-archive` preserves event location~~

Phase 2 (open):
- `finish` writes `end_time` to `.ctf_meta.json` on success.
- `finish` without a git repo: skips git clean, still archives.
- `finish` on an event with unsolved challenges: no implicit block.
- `schedule` beyond smoke: update existing schedule, dates in both orders.
- `check` beyond smoke: expired event, soon-to-expire event, no events.
- `recent` beyond smoke: multi-event cycling, 5-entry cap enforcement.
- `import` cross-device fallback path (rename fails → copy+delete).
- Context-resolution edge cases (covered by T012 unit tests).

Exit criterion: every CTF command has ≥3 integration tests; no single-test
commands remain.

Depends on:

- T003 (DONE)
- T005 (DONE)
- T012 (deferred — its seam is a 0.4.x concern)
- T013

### T018: Add Completion Verification

Goal: ensure shell integration remains shippable.

Deliverables:

- verification for generated or maintained completion artifacts
- docs for how contributors validate completion behavior

Depends on:

- T009
- T010
- T011
- T014

### T019: Add CI For The Flagship Path (DONE — stable as of alpha3)

Goal: make the CTF-shell surface safe to evolve.

Status: `.github/workflows/ci.yml` exists and runs `fmt --check`,
`clippy -- -D warnings`, and `test` on every push/PR to `main`. Green as of
`75aacae` after the alpha3 fmt/clippy fixes.

Deliverables shipped:

- ~~`cargo fmt --check`~~
- ~~`cargo clippy --all-targets --all-features -- -D warnings`~~
- ~~`cargo test`~~

Follow-up (folded into 0.4.x operational-hygiene workstream, not a new task):

- pin the CI rust toolchain or keep `flake.lock` `rust-overlay` current to
  prevent devshell drift.

## Milestone 6: 0.3.x Stabilization to 0.3.0 (Active)

Added at the 0.3.0-alpha3 retrospective. See
[`evaluation-alpha3.md`](evaluation-alpha3.md) for the rationale.

Path to stable release: **alpha3 → alpha4 → beta1 → 0.3.0**. Alpha4 does
test-depth additions and toolchain pinning. Beta1 is cut when every CTF
command has ≥3 tests and CI has been stable for 10 pushes. 0.3.0 releases
after a field-soak cycle against beta1.

**T012 `ContextResolver` refactor is deliberately out of this milestone** —
it opens the 0.4.x cycle instead. Blast-radius of a whole-binary refactor
is wrong for a beta-cycle change.

### T020: Migrate Path Completion To Instruction-Based (DONE)

Goal: eliminate silent-fallback behaviour in the shell-completion layer.

Status: Completed in 0.3.0-alpha2 (crane commit bundle). All four completer
helpers now return empty when they lack explicit instruction instead of
guessing. Documented in-tree as the file header of
`src/engine/ctf/completions.rs`.

### T021: Pin Toolchain To Prevent Devshell-CI Drift

Goal: stop the "fmt and clippy differ between local and CI" failure mode that
cost 4 consecutive red CI runs in alpha3.

Deliverables (pick one, document the chosen path):

- Pin `dtolnay/rust-toolchain@<version>` in `.github/workflows/ci.yml` and
  update it deliberately (requires coordinated devshell bump), OR
- Keep `flake.lock` `rust-overlay` current as a hard rule; enforce by adding
  a CI step that diffs `rustc --version` between devshell and CI host.

Exit criterion: 10 consecutive CI pushes green without toolchain-drift
breakage.

### T022: Migrate To Crane For Incremental Nix Builds (DONE)

Goal: stop rebuilding all ~200 dep crates on every wardex source change.

Status: Completed in 0.3.0-alpha3 (`06e6955`). Dependency crates now cache
across source changes via `craneLib.buildDepsOnly`. `nh os switch` on
wardex bumps is typically 80–90% faster.

## Completed First Sprint

All items shipped across 0.2.0-alpha4 through alpha6:

1. ~~T004 Implement challenge metadata schema~~ (alpha4)
2. ~~T006 Add `--cd` flag to `ctf add`~~ (alpha4)
3. ~~T005 Implement `ctf shelve` interactive flow~~ (alpha5)
4. ~~T007 Implement file triage system~~ (alpha5 + alpha6)
5. ~~T002 Write RFC 0002 for shell completion architecture~~ (alpha6)
6. ~~T003 Stabilize shell-oriented output contracts~~ (alpha6)
7. ~~T009 Add a completion entry point~~ (alpha6)
8. ~~T010 Implement static completion for commands and flags~~ (alpha6)

## Completed 0.3.x Sprint

All items shipped across 0.3.0-alpha1 through alpha3:

1. ~~T011 Dynamic completion for events and categories~~ (alpha8 + alpha2 refactor)
2. ~~T014 Dynamic challenge-path completion~~ (0.3-alpha1, refactored alpha2)
3. ~~T019 CI for the flagship path~~ (now stable as of alpha3 `75aacae`)
4. ~~T020 Instruction-based completion refactor~~ (alpha2)
5. ~~T022 Crane-based Nix build~~ (alpha3)
6. ~~T017 phase 1: archive + finish regression pins~~ (alpha3)

## Suggested Next Sprint (0.3.0-alpha4 — stabilization)

Shortest route to a shippable 0.3.0, ordered by risk/reward:

1. **T017 phase 2** — close the remaining single-test lifecycle commands
   (`schedule`, `check`, `recent`, `finish` error paths). Pure test
   additions; no API change. **Required for beta1 cut.**
2. **T021 toolchain pinning** — one-commit hygiene fix; prevents the fmt
   drift class from recurring. **Required for beta1 cut.**
3. **T013 `ctf path --cd` / `ctf add --cd` review** — low risk, already
   escape-hardened (`3d2eb79` escape-single-quotes fix); a docs pass + two
   wrapper examples in README finishes it. **Ships with alpha4 or beta1.**
4. **T018 completion verification** — alpha3 added 10 unit tests covering
   the completers; adding a shell-integration smoke test (actually invoke
   `compgen`/`compadd` from bash) is the remaining bit. **Optional for beta1.**

Deferred past 0.3.0:

- **T012 `ContextResolver` refactor** — opens 0.4.x. Whole-binary refactor
  is wrong shape for a stabilization beta.
