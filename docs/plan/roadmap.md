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

## Phase 1: CTF Lifecycle Hardening (Complete, with ongoing phase 2)

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

## Phase 2: Shell-First Integration (Complete, with path-completion polish in alpha3)

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

## Phase 3: CTF Workflow Polish (Features implemented — 0.3.0-alpha1 through alpha3)

Goal: improve depth and speed for repeat CTF usage once the shell layer is solid.

Status: **features implemented across alpha1–alpha3. Stabilization to 0.3.0
still pending** — need alpha4 (test depth), beta1 (feature freeze), and a
field-soak cycle before the stable 0.3.0 release. See
[`evaluation-alpha3.md`](evaluation-alpha3.md) for the detailed per-commit map.

### Shipped deliverables

- ~~Rich writeup assembly with `.challenge.json` metadata and flag redaction.~~ (alpha1, `0ac5876`)
- ~~Config validation (`wardex config validate`).~~ (alpha1, `c0f143e`)
- ~~Status enrichment: solver, notes preview, summary line, JSON output.~~ (alpha1, `209a41e`)
- ~~Multi-event context handling: `ctf use -`, `ctf recent`.~~ (alpha1, `d5caf87`)
- ~~Challenge-path dynamic completion.~~ (alpha1, `acbaf67`)
- ~~Path-arg tilde handling (`~`, `~/…`) across all `PathBuf` args.~~ (alpha3, `880ca31`)
- ~~Instruction-based completers — no silent fallback to guessed events/categories.~~ (alpha2)
- ~~Crane-based Nix build — dep-cache survives source changes.~~ (alpha3, `06e6955`)

### Intentionally deferred

- Per-category challenge templates: user demand unclear; `.challenge.json`
  already covers the metadata half. Re-open in 0.5+ if field feedback asks.
- TUI role: frozen per `long-term-strategy.md`. Current CLI+editor workflow
  is the right seam for CTF operations.

### Exit Criteria — Met

- Repetitive competition tasks feel streamlined (shelve auto-mode, status JSON,
  challenge-path completion).
- Power users can tailor shell behavior and categories via `config.yaml` without
  patching source.
- The CTF experience is a cohesive product.

## Phase 3 Stabilization: 0.3.0-alpha4 → beta1 → 0.3.0 (Active)

Not in the original plan. Added after the alpha3 retrospective surfaced a
pattern: destructive lifecycle commands (`finish`, `archive`) have thinner
coverage than interactive ones, and CI has been fragile to toolchain drift.
This phase closes those gaps before cutting 0.3.0 stable.

### alpha4 Deliverables (next up)

Pure additive / hygiene. No API changes.

- **T017 phase 2 — lifecycle regression depth**: cover `finish` end-time
  metadata; `schedule`, `check`, `recent` beyond smoke; `finish` without git
  repo; `finish` on unsolved events.
- **T021 — operational hygiene**: pin the devshell-vs-CI toolchain to avoid
  the fmt/clippy drift that caused 4 consecutive red CI runs in alpha3.
  Either pin CI to a specific rust version (`dtolnay/rust-toolchain@1.95.0`)
  or require `flake.lock` rust-overlay stay current and enforce via CI.
- **T013 — shell wrapper examples**: short README pass adding Bash/Zsh
  wrapper examples for `ctf path --cd` and `ctf add --cd`. Already
  escape-hardened in `3d2eb79`; this is documentation only.

### beta1 Cut Criteria

- Every CTF command has ≥3 integration tests.
- CI goes ≥10 consecutive pushes green without toolchain-drift breakage.
- `docs/ctf-lifecycle.md` + `docs/shell-output-contracts.md` match the
  current code exactly.
- Docs pass: `docs/PREVIEW.md`, README, CHANGELOG consistent with shipped
  commands.

### 0.3.0 Release Criteria

- beta1 has soaked for one field-use cycle (a real CTF event or equivalent).
- No open regression reports against beta1.
- CHANGELOG entry for 0.3.0 finalized.

**T012 (`ContextResolver` refactor) is deliberately deferred to 0.4.x** —
it's an internal refactor that touches every command's resolution code,
which is too large a blast-radius for beta-cycle changes.

## Phase 4: Post-0.3.0 Buffer + Broader Expansion (Future)

Goal: land the `ContextResolver` refactor first, then revisit non-CTF expansion.

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

- `0.2.x`: Shelve system, CTF contract, lifecycle hardening, shell completion (**released 0.2.0**)
- `0.3.x`: CTF workflow polish — writeup, status, multi-event, path completion
  (**features implemented in alpha1–alpha3; stabilizing to 0.3.0 via alpha4 → beta1 → release**)
- `0.4.x`: Context-resolver refactor + selective non-CTF expansion (future)
- `0.5.x`: broader expansion beyond the flagship CTF path
- `0.6.x+`: advanced capabilities after core stabilization
