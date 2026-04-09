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

### T002: Write RFC 0002 For Shell Completion Architecture

Goal: choose the completion approach before implementation diverges.

Deliverables:

- define whether completion is Clap-generated, custom, or hybrid
- define whether Wardex needs `wardex completions <shell>`
- define how dynamic completion will source events, categories, and challenges
- define Bash and Zsh support expectations

Definition of done:

- RFC accepted or ready for acceptance
- completion architecture is clear enough to implement without rework

### T003: Stabilize Shell-Oriented Output Contracts

Goal: make shell-facing commands safe to wrap and evaluate.

Deliverables:

- specify output contracts for `ctf path`, `ctf path --cd`, `ctf add --cd`, and `ctf use`
- review whether any commands need a quieter or machine-oriented mode
- document what output must remain stable across versions

Definition of done:

- shell-facing commands have explicit output rules
- risky output ambiguity is removed from the backlog of unknowns

## Milestone 1: Shelve System Implementation

### T004: Implement Challenge Metadata Schema

Goal: replace `flag.txt` with structured per-challenge metadata.

Deliverables:

- define and implement `ChallengeMetadata` struct (`name`, `category`, `status`, `flag`, `solved_by`, `note`, `imported_from`, `shelved_at`, `created_at`)
- `.challenge.json` read/write alongside existing `.ctf_meta.json`
- status values: `active`, `solved`, `team-solved`, `unsolved`
- migration: read `flag.txt` if `.challenge.json` is absent (backwards compat)
- `ctf add` creates `.challenge.json` with `active` status

Depends on:

- T001

### T005: Implement `ctf shelve` Interactive Flow

Goal: build the signature shelve command with interactive-first design.

Deliverables:

- status prompt (Select: solved / team-solved / unsolved)
- flag input (Input, conditional on status)
- file triage (MultiSelect with blacklist/whitelist pre-sorting and sizes)
- note prompt (Input, optional)
- archive prompt (Confirm: move to archives?)
- each step skippable with flags (`--no-clean`, `--note`, `--move`/`--no-move`, `--auto`)
- update `.challenge.json` with solve state
- `solve` and `done` as hidden aliases

Depends on:

- T004

### T006: Add `--cd` Flag To `ctf add`

Goal: make the primary creation verb also serve shell navigation.

Deliverables:

- `--cd` flag on `ctf add` that outputs `cd '<path>'` after creation
- `work` as hidden alias for `add --cd`
- update help text to document `add` as the primary verb

Depends on:

- T001

### T007: Implement File Triage System

Goal: smart file cleanup during shelve with configurable patterns.

Deliverables:

- default blacklist patterns: `node_modules/`, `.venv/`, `venv/`, `core.*`, `*.o`, `.gdb_history`, `peda-*`, `__pycache__/` (when not from challenge)
- default whitelist patterns: `solve.*`, `exploit.*`, `notes.md`, `Dockerfile`, `docker-compose.yml`, imported originals (from `.challenge.json` `imported_from`)
- `config.yaml` keys for custom blacklist and whitelist
- file size display in triage prompt
- invert mode: select-to-keep instead of select-to-delete

Depends on:

- T005

## Milestone 2: Shell Completion

### T008: Write RFC 0002 For Shell Completion Architecture

Same as T002. Moved here for ordering clarity.

### T009: Add A Completion Entry Point

Goal: give users a standard way to install completions.

Deliverables:

- add a completion command or equivalent supported install path
- support Bash and Zsh first
- document the install flow in project docs

Depends on:

- T002

### T010: Implement Static Completion For Commands And Flags

Goal: complete the command tree and common options.

Deliverables:

- subcommand completion (including `shelve` and hidden aliases)
- flag completion
- shell install examples for Bash and Zsh

Depends on:

- T009

### T011: Implement Dynamic Completion For Events And Categories

Goal: reduce typing during real event work.

Deliverables:

- event-name completion for `ctf use`, `ctf path`, and related commands
- category completion for `ctf add`, `ctf import` where applicable
- sensible behavior when no active event exists

Depends on:

- T002
- T009

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

## Suggested First Sprint

Shortest route to visible user value:

1. T004 Implement challenge metadata schema
2. T006 Add `--cd` flag to `ctf add`
3. T005 Implement `ctf shelve` interactive flow
4. T007 Implement file triage system
5. T002 Write RFC 0002 for shell completion architecture
6. T003 Stabilize shell-oriented output contracts
