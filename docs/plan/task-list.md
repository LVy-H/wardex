# Task List

This file turns the current CTF-shell strategy into an execution-ready backlog.

The recommendation is to build Wardex in this order:

1. Lock the CTF command contract.
2. Implement shell completion and navigation support.
3. Harden shell-safe output and context behavior.
4. Improve challenge templates and writeup flow.
5. Add release-quality verification around the flagship path.

## Immediate Next Jobs

These are the best next tasks to start with.

### T001: Define The Canonical CTF Lifecycle

Goal: remove ambiguity from the current command surface.

Deliverables:

- document the intended flow from `ctf init` to `ctf finish`
- define the role of `add`, `work`, `solve`, `done`, `archive`, and `finish`
- review command names against the "natural verb first" rule
- treat `work` as a likely alias or deprecation candidate unless it earns a clear distinct role
- identify which commands are primary, alias-like, or redundant
- decide which commands need shell-stable output guarantees

Definition of done:

- one clear lifecycle is documented
- command overlap is explained or scheduled for cleanup
- vague command names are explicitly accepted, aliased, or targeted for replacement
- help text changes needed for the lifecycle are identified

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

- specify output contracts for `ctf path`, `ctf path --cd`, `ctf work`, and `ctf use`
- review whether any commands need a quieter or machine-oriented mode
- document what output must remain stable across versions

Definition of done:

- shell-facing commands have explicit output rules
- risky output ambiguity is removed from the backlog of unknowns

## Milestone 1: Completion MVP

### T004: Add A Completion Entry Point

Goal: give users a standard way to install completions.

Deliverables:

- add a completion command or equivalent supported install path
- support Bash and Zsh first
- document the install flow in project docs

Depends on:

- T002

### T005: Implement Static Completion For Commands And Flags

Goal: complete the command tree and common options.

Deliverables:

- subcommand completion
- flag completion
- shell install examples for Bash and Zsh

Depends on:

- T004

### T006: Implement Dynamic Completion For Events And Categories

Goal: reduce typing during real event work.

Deliverables:

- event-name completion for `ctf use`, `ctf path`, and related commands
- category completion for `ctf add`, `ctf work`, and `ctf import` where applicable
- sensible behavior when no active event exists

Depends on:

- T002
- T004

## Milestone 2: Context And Navigation Hardening

### T007: Make Context Resolution Predictable

Goal: ensure shell wrappers and completions can rely on Wardex behavior.

Deliverables:

- document and enforce precedence between current directory, active event, and fuzzy resolution
- improve ambiguity errors
- avoid silent wrong-target selection

Depends on:

- T001

### T008: Harden `ctf path --cd` And `ctf work`

Goal: make navigation commands dependable enough for daily shell use.

Deliverables:

- verify eval-safe output
- align behavior with the command contract
- add wrapper examples for Bash and Zsh
- if `work` remains, treat it as shortcut-grade behavior and not the primary named path

Depends on:

- T003
- T007

### T009: Dynamic Challenge-Path Completion

Goal: complete real challenge targets, not just command names.

Deliverables:

- challenge-path completion based on current or active event context
- support category/name flows cleanly
- handle missing context gracefully

Depends on:

- T006
- T007

## Milestone 3: CTF Workflow Polish

### T010: Clarify `add` Versus `work`

Goal: make creation and navigation feel intentional.

Deliverables:

- prefer `add` as the documented primary verb unless evidence strongly supports `work`
- decide whether `work` remains only as an alias or power-user shortcut
- make one clearly primary if both survive
- align help text and examples with that choice

Depends on:

- T001

### T011: Clarify `solve` Versus `done`

Goal: remove ambiguity around solve-time behavior.

Deliverables:

- define whether `done` is a true alias or a workflow shortcut
- align help text, docs, and output
- review defaults around commit and archive behavior

Depends on:

- T001

### T012: Improve Challenge Templates

Goal: make new challenge folders useful immediately.

Deliverables:

- better category-specific scaffold defaults
- a clear notes structure
- tighter connection between templates and writeup generation

Depends on:

- T010
- T011

### T013: Improve Notes And Writeup Flow

Goal: make solve artifacts more consistent.

Deliverables:

- define a default notes convention
- improve `writeup` expectations and examples
- ensure solve-time data feeds writeup assembly cleanly

Depends on:

- T012

## Milestone 4: Tests And Release Readiness

### T014: Expand CTF Integration Coverage

Goal: protect the flagship workflow with regression tests.

Deliverables:

- tests for `ctf path`, `ctf work`, `ctf import`, `ctf solve`, `ctf finish`, and `ctf archive`
- tests for context resolution and fuzzy matching
- tests for shell-oriented output behavior

Depends on:

- T003
- T007
- T008

### T015: Add Completion Verification

Goal: ensure shell integration remains shippable.

Deliverables:

- verification for generated or maintained completion artifacts
- docs for how contributors validate completion behavior

Depends on:

- T004
- T005
- T006
- T009

### T016: Add CI For The Flagship Path

Goal: make the CTF-shell surface safe to evolve.

Deliverables:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features`
- `cargo test`
- completion verification if practical

Depends on:

- T014
- T015

## Suggested Issue Creation Order

If you want to turn this into GitHub issues, create them in this order:

1. T001 Define the canonical CTF lifecycle
2. T002 RFC 0002 for shell completion architecture
3. T003 Stabilize shell-oriented output contracts
4. T004 Add a completion entry point
5. T006 Implement dynamic completion for events and categories
6. T007 Make context resolution predictable
7. T008 Harden `ctf path --cd` and `ctf work`
8. T009 Dynamic challenge-path completion
9. T010 Clarify `add` versus `work`
10. T011 Clarify `solve` versus `done`
11. T012 Improve challenge templates
12. T013 Improve notes and writeup flow
13. T014 Expand CTF integration coverage
14. T015 Add completion verification
15. T016 Add CI for the flagship path

## Suggested First Sprint

If you want the shortest route to visible user value, do these first:

1. T001 Define the canonical CTF lifecycle
2. T002 RFC 0002 for shell completion architecture
3. T003 Stabilize shell-oriented output contracts
4. T004 Add a completion entry point
5. T006 Implement dynamic completion for events and categories
