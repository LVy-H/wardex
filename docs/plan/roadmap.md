# Roadmap

This roadmap assumes Wardex continues as an alpha Rust CLI with a CTF-first identity and shell-native workflows.

## Phase 0: CTF Product Contract (Complete)

Goal: define the exact CTF and shell experience Wardex wants to own.

Status: **All deliverables shipped across alpha4–alpha6.**

### Deliverables

- ~~Define the supported CTF lifecycle around the shelve system.~~ (alpha4, `docs/ctf-lifecycle.md`)
- ~~Implement `ctf shelve` with interactive-first design and `.challenge.json` metadata.~~ (alpha5)
- ~~Add `--cd` to `ctf add`; make `work`, `solve`, `done` hidden aliases.~~ (alpha4)
- ~~Decide which commands must be completion-aware and what each one should suggest.~~ (RFC 0002, alpha6)
- ~~Document the intended shell story for Bash and Zsh first.~~ (output contracts, alpha6)

### Exit Criteria — Met

- There is one documented CTF workflow that the repo clearly optimizes for.
- Completion targets and shell integration behavior are specified before implementation work expands.
- User-facing docs communicate the CTF-first product direction clearly.

## Phase 1: CTF Lifecycle Hardening (Complete)

Goal: make the core event and challenge workflow reliable enough for active competition use.

### Deliverables

- Expand integration coverage for `ctf init`, `use`, `path`, `add --cd`, `import`, `shelve`, `finish`, and `archive`.
- Add regression tests for context resolution, active-event fallback, fuzzy matching, and shell-oriented output.
- Review error messages for time-sensitive or stateful CTF commands.
- Audit path handling, archive handling, cross-device moves, and challenge name normalization.
- Establish a minimal CI bar: `fmt`, `clippy`, `test`.

### Exit Criteria

- Competition-critical flows are covered by tests, not just happy paths.
- Common CTF mistakes produce actionable errors and recovery hints.
- Path-printing commands are safe and predictable for shell use.

## Phase 2: Shell-First Integration (In Progress)

Goal: make Wardex feel native in the terminal instead of merely callable from it.

### Deliverables

- ~~Implement shell completion generation or maintained completion scripts for Bash and Zsh.~~ (alpha6, `wardex completions <shell>`)
- Support tab completion for event names, categories, challenge paths (dynamic completion — not yet implemented).
- Refine shell integration around `ctf path`, `ctf add --cd`, `ctf use`, and related wrappers.
- ~~Add shell-install docs and example aliases/functions that feel low-friction.~~ (CHANGELOG, README)
- ~~Ensure machine-friendly output modes stay stable for shell evaluation and scripting.~~ (`docs/shell-output-contracts.md`)

### Exit Criteria

- A user can navigate common flows mostly through completion and short commands.
- ~~Bash and Zsh users can install completion without reverse-engineering the repo.~~
- Shell wrappers no longer feel like fragile add-ons.

## Phase 3: CTF Workflow Polish

Goal: improve depth and speed for repeat CTF usage once the shell layer is solid.

### Deliverables

- Improve command output consistency across `status`, `info`, `writeup`, and shelve flows.
- Standardize challenge templates and per-category scaffolding behavior.
- Add richer writeup assembly and notes conventions using `.challenge.json` metadata.
- Add structured config options for categories, templates, blacklist/whitelist patterns, and shell behavior.
- Clarify whether the TUI has a role in CTF operations or remains optional.

### Exit Criteria

- Repetitive competition tasks feel streamlined instead of improvised.
- Power users can tailor templates and shell behavior without patching the source.
- The CTF experience feels like a cohesive product, not a bundle of commands.

## Phase 4: Broader Expansion

Goal: revisit non-CTF expansion only after the flagship workflow is strong.

### Candidate Epics

- Mature workspace-manager features outside the CTF path.
- Semantic or indexed search beyond regex and grep.
- Session resume or flow-state features.
- Richer TUI dashboards and action workflows.
- Knowledge graph or dependency visualization.

### Exit Criteria

- Non-CTF work does not dilute the shell-first CTF identity.
- New capabilities do not weaken core filesystem safety or command clarity.

## Recommended Release Cadence

- `0.2.x`: Shelve system, CTF contract, lifecycle hardening, and shell completion
- `0.3.x`: CTF workflow polish and template/config depth
- `0.4.x`: selective expansion beyond the flagship CTF path
- `0.5.x+`: advanced capabilities after core stabilization
