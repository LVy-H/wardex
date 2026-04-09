# Wardex Development Plan

This folder turns the current repository into a staged build and upgrade plan.

Wardex is already beyond a prototype. The CLI, configuration layer, CTF workflow, workspace utilities, optional TUI, and integration tests are present and working. The next step is not "start from zero", but to focus the product around one strong promise: a fast CTF workflow that feels native inside the shell.

## Current Baseline

Based on the repository review:

- The core CLI is implemented in [`src/main.rs`](/run/host/mnt/Data/Workspace/1_Projects/Dev-CLI-Wardex/src/main.rs).
- Business logic is split into focused modules under [`src/engine`](/run/host/mnt/Data/Workspace/1_Projects/Dev-CLI-Wardex/src/engine).
- CTF flows are the most mature product area and should drive near-term product decisions.
- Configuration, global state, watcher support, and optional TUI plumbing already exist.
- The automated baseline is healthy: `cargo test` passed locally on April 9, 2026.

## Near-Term Product Focus

For now, planning effort should concentrate on the CTF experience and shell integration:

- Make the event and challenge lifecycle feel smooth under time pressure.
- Make navigation and command discovery shell-friendly.
- Support tab completion and completion-aware subcommands for event names, categories, and challenge paths.
- Reduce context friction between the current directory, active event state, and shell wrappers.
- Delay broader workspace-manager expansion until the flagship CTF flow is excellent.

## Planning Principles

- Stabilize before expanding.
- Prefer one clearly shipped workflow over many partially finished ones.
- Keep CTF operations as the flagship path until workspace management reaches the same quality bar.
- Design commands so they are easy to type, easy to complete, and easy to script.
- Treat docs, tests, and command UX as product features, not cleanup work.
- Introduce larger bets like TUI polish or "Wardex Brain" only after the CLI contract is dependable.

## Suggested Planning Order

1. Phase 0: define the supported CTF workflow and shell integration contract.
2. Phase 1: harden CTF lifecycle commands and context resolution.
3. Phase 2: ship first-class shell completion, wrappers, and navigation helpers.
4. Phase 3: polish templates, writeups, and power-user workflows.
5. Phase 4: revisit broader workspace features only after the CTF path is excellent.

## Documents In This Folder

- [`review.md`](/run/host/mnt/Data/Workspace/1_Projects/Dev-CLI-Wardex/docs/plan/review.md): repository assessment and planning rationale
- [`roadmap.md`](/run/host/mnt/Data/Workspace/1_Projects/Dev-CLI-Wardex/docs/plan/roadmap.md): phased release and upgrade path
- [`workstreams.md`](/run/host/mnt/Data/Workspace/1_Projects/Dev-CLI-Wardex/docs/plan/workstreams.md): concrete engineering tracks, backlog themes, and success measures
- [`ctf-shell-plan.md`](/run/host/mnt/Data/Workspace/1_Projects/Dev-CLI-Wardex/docs/plan/ctf-shell-plan.md): focused plan for CTF workflow and shell-native ergonomics
- [`task-list.md`](/run/host/mnt/Data/Workspace/1_Projects/Dev-CLI-Wardex/docs/plan/task-list.md): prioritized execution backlog and suggested issue order

Related project-wide guidance:

- [`docs/CLI_DESIGN.md`](/run/host/mnt/Data/Workspace/1_Projects/Dev-CLI-Wardex/docs/CLI_DESIGN.md): CLI and shell design rules
- [`docs/ctf-lifecycle.md`](/run/host/mnt/Data/Workspace/1_Projects/Dev-CLI-Wardex/docs/ctf-lifecycle.md): draft CTF lifecycle and command naming stance
- [`docs/rfcs/README.md`](/run/host/mnt/Data/Workspace/1_Projects/Dev-CLI-Wardex/docs/rfcs/README.md): RFC process for major changes
