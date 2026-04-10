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

### T011: Implement Dynamic Completion For Events And Categories

Goal: reduce typing during real event work.

Deliverables:

- event-name completion for `ctf use`, `ctf path`, and related commands
- category completion for `ctf add`, `ctf import` where applicable
- sensible behavior when no active event exists

Depends on:

- T009 (DONE)

## Milestone 3: Context And Navigation Hardening

### T012: Make Context Resolution Predictable

Goal: ensure shell wrappers and completions can rely on Wardex behavior.

Deliverables:

- document and enforce precedence: local directory > global state > latest event
- improve ambiguity errors
- avoid silent wrong-target selection

Depends on:

- T001

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

### T014: Dynamic Challenge-Path Completion

Goal: complete real challenge targets, not just command names.

Deliverables:

- challenge-path completion based on current or active event context
- support category/name flows cleanly
- handle missing context gracefully

Depends on:

- T011
- T012

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

### T017: Expand CTF Integration Coverage

Goal: protect the flagship workflow with regression tests.

Deliverables:

- tests for `ctf path`, `ctf add --cd`, `ctf import`, `ctf shelve`, `ctf finish`
- tests for challenge metadata lifecycle
- tests for context resolution and fuzzy matching
- tests for shell-oriented output behavior

Depends on:

- T003
- T005
- T012
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

### T019: Add CI For The Flagship Path

Goal: make the CTF-shell surface safe to evolve.

Deliverables:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features`
- `cargo test`
- completion verification if practical

Depends on:

- T017
- T018

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

## Suggested Next Sprint

Shortest route to next visible value:

1. T011 Implement dynamic completion for events and categories
2. T012 Make context resolution predictable
3. T013 Harden `ctf path --cd` and `ctf add --cd`
4. T017 Expand CTF integration coverage
