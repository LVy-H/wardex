# Wardex Development Plan

This folder turns the current repository into a staged build and upgrade plan.

Wardex is already beyond a prototype. The CLI, configuration layer, CTF workflow, workspace utilities, optional TUI, and integration tests are present and working. The next step is not "start from zero", but to focus the product around one strong promise: a fast CTF workflow that feels native inside the shell.

## Current Baseline

Based on the repository review:

- The core CLI is implemented in [`src/main.rs`](../../src/main.rs).
- Business logic is split into focused modules under [`src/engine`](../../src/engine).
- CTF flows are the most mature product area and should drive near-term product decisions.
- Configuration, global state, watcher support, and optional TUI plumbing already exist.
- The automated baseline is healthy: `cargo test` passed locally on April 9, 2026.

## Near-Term Product Focus

0.3.x feature work is implemented across alpha1–alpha3. Current focus is
**stabilizing to 0.3.0** — not starting 0.4.x. Per
[`../../CHANGELOG.md`](../../CHANGELOG.md) versioning policy, 0.3.x still
needs alpha4 (test depth) → beta1 (feature freeze) → field-soak → 0.3.0
stable release.

- **Close the lifecycle-test gap** (T017 phase 2): `archive` + `finish` got
  regression tests in alpha3; `schedule`, `check`, `recent`, and `finish`
  error paths still have only smoke coverage. Required for beta1 cut.
- **Toolchain hygiene** (T021): devshell-vs-CI drift cost 4 red CI runs in
  alpha3. Pin both sides or enforce alignment via CI. Required for beta1 cut.
- **Shell-wrapper docs** (T013): README pass adding Bash/Zsh wrapper
  examples for `ctf path --cd` / `ctf add --cd`. Code is already
  escape-hardened (`3d2eb79`); this is documentation only.

Backlog items to preserve but not prioritize yet:

- **T012 `ContextResolver` refactor** is deferred to 0.4.x. It touches every
  command's resolution code — wrong shape for a stabilization beta.
- Broader workspace-manager expansion stays deferred until 0.4.x.
- Per-category challenge templates — deferred per `long-term-strategy.md`
  (user demand unclear; existing `.challenge.json` covers the metadata half).

## Planning Principles

- Stabilize before expanding.
- Prefer one clearly shipped workflow over many partially finished ones.
- Keep CTF operations as the flagship path until workspace management reaches the same quality bar.
- Design commands so they are easy to type, easy to complete, and easy to script.
- Treat docs, tests, and command UX as product features, not cleanup work.
- Introduce larger bets like TUI polish or "Wardex Brain" only after the CLI contract is dependable.

## Planning Order — Current Status

1. ~~Phase 0: define the supported CTF workflow and shell integration contract.~~ **(alpha4–alpha6)**
2. ~~Phase 1: harden CTF lifecycle commands and context resolution.~~ **(alpha7–beta1; phase-2 hardening in 0.3.x stabilization)**
3. ~~Phase 2: ship first-class shell completion, wrappers, and navigation helpers.~~ **(alpha6 → 0.3-alpha3)**
4. ~~Phase 3: polish templates, writeups, and power-user workflows — features implemented.~~ **(0.3-alpha1 through 0.3-alpha3)**
5. **Phase 3 stabilization**: test-depth + toolchain hygiene, then beta1, then field-soak, then **0.3.0 stable release**. **(active)**
6. Phase 4: `ContextResolver` refactor + selective broader workspace features. **(future, 0.4.x)**

## Documents In This Folder

- [`evaluation-alpha3.md`](evaluation-alpha3.md): **current status snapshot** — what's shipped, what's gap, next-phase recommendation as of 0.3.0-alpha3.
- [`review.md`](review.md): repository assessment and planning rationale
- [`roadmap.md`](roadmap.md): phased release and upgrade path
- [`long-term-strategy.md`](long-term-strategy.md): version roadmap 0.2–0.6+ with per-version themes
- [`workstreams.md`](workstreams.md): concrete engineering tracks, backlog themes, and success measures
- [`ctf-shell-plan.md`](ctf-shell-plan.md): focused plan for CTF workflow and shell-native ergonomics
- [`task-list.md`](task-list.md): prioritized execution backlog and suggested issue order

Related project-wide guidance:

- [`docs/CLI_DESIGN.md`](../CLI_DESIGN.md): CLI and shell design rules
- [`docs/ctf-lifecycle.md`](../ctf-lifecycle.md): CTF lifecycle and command naming (implemented in alpha4–alpha6)
- [`docs/rfcs/README.md`](../rfcs/README.md): RFC process for major changes
