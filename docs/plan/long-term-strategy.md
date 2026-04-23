# Wardex Long-Term Strategy

Originally drafted at 0.2.0-beta1. Refreshed at 0.3.0-alpha3 — see
[`evaluation-alpha3.md`](evaluation-alpha3.md) for the detailed checkpoint.

## Version Roadmap

Per the versioning policy in `CHANGELOG.md`: each minor version goes through
alpha (breaking changes expected) → beta (feature-complete, bug fixes only)
→ `0.x.0` stable release.

| Version | Theme | Status |
|---------|-------|--------|
| **0.2.x** | Beta — CTF + shell reliability | **Released (0.2.0)** |
| **0.3.x** | CTF polish — writeup, config validation, status enrichment | **Active — themes implemented across alpha1–alpha3; beta + 0.3.0 pending** |
| **0.4.x** | Post-0.3.0 buffer — `ContextResolver` refactor, non-CTF enhancements | Planned |
| **0.5.x** | Non-CTF commands active support, team features | Planned |
| **0.6+** | Expansion — web UI, cross-platform, advanced features | Future |

## 0.3.x: CTF Workflow Polish (Active)

### Writeup Generation Improvements

Current `generate_writeup()` ignores `.challenge.json` metadata entirely.

Improvements:
- Pull flag, status, solved_by, shelved_at into writeup headers
- Summary statistics: solved count, solve rate, time-to-first-flag
- Optional flag redaction (`--no-flags`)
- Template support via `config.ctf.writeup_template`

### Config Validation

No validation today — bad paths fail at runtime with cryptic errors.

Add `Config::validate()` called after `Config::load()`:
- Check workspace path exists or is creatable
- Validate category names (alphanumeric + underscore)
- Pre-compile regex patterns in clean rules
- New command: `wardex config validate [--fix]`

### CTF Status Enrichment

Current table: 3 columns (Category, Challenge, Status).

Add optional columns: solved_by, solve time, notes preview.
Add summary line: `15 total | 10 solved (me: 7, team: 3) | 5 unsolved`.
Add flags: `--format json`, `--filter solved`, `--sort time`.

### Multi-Event Context Handling

Only one active event in global state. Users in overlapping CTFs get surprised.

Add:
- `wardex ctf use -` — switch back to previous event (like `cd -`)
- `wardex ctf recent` — list last 3-5 events
- Warning when CWD context differs from global state
- `activity_status` field in `.ctf_meta.json` (active/paused/archived)

### Plugin/Extension System

Too early for a full plugin system. Instead:
- Document hook points (challenge_created, challenge_solved)
- Proof of concept: auto-run `checksec` for pwn challenges via config flag
- Design RFC for 0.4+ if demand exists

## 0.3.x: Stabilization Path (alpha4 → beta → 0.3.0)

The strategic themes above are implemented across alpha1–alpha3. The
remaining work to reach stable 0.3.0 is test depth and operational hygiene,
not new features. Per the versioning policy, alphas may still make breaking
changes; betas are feature-complete bug-fix only.

### alpha4 (next): test-depth additions

Pure additive work — no breaking changes, no new features.

- **Lifecycle test hardening (T017 phase 2)**: alpha3 pinned `archive`/`finish`
  basics. Still open: `finish` end-time-metadata, `schedule`, `check`,
  `recent`, the `finish` error paths when the event has no git repo / has
  unsolved challenges.
- **Operational hygiene (T021)**: devshell-vs-CI toolchain drift caused 4
  consecutive red CI runs during alpha3. Either pin CI to a specific rust
  version or keep the `flake.lock` `rust-overlay` current; pick one and
  document.

### beta1: feature freeze

Cut when:

- Every CTF command has ≥3 integration tests.
- CI has gone ≥10 consecutive pushes green without toolchain drift.
- `docs/ctf-lifecycle.md` and `docs/shell-output-contracts.md` match the
  current code exactly.

No new features accepted once the beta branch is cut.

### 0.3.0: stable release

Cut when beta1 has soaked for one field-use cycle (a real CTF event or
equivalent) with zero regression reports.

## 0.4.x: Post-0.3.0 Buffer (Future)

Deferred to after 0.3.0 stable. The original "buffer release" slot is now
earmarked for the `ContextResolver` refactor (T012) — an internal cleanup
that warrants its own alpha gate because it touches every command's
resolution code and deserves clear blast-radius isolation from the 0.3.0
stabilization.

### Confirmed early workstream

- **`ContextResolver` refactor (T012)**: current context resolution
  (`local cwd > global state > latest event`) is reimplemented per-command.
  Every alpha has found a fresh bug in this area (alpha2 `~/` handling, alpha2
  silent-fallback in challenge completer, alpha3 zsh user-name fallback on
  bare `~`). Collapse into one resolver + unit tests.

### Buffer slots (if field feedback warrants)

- Non-CTF command enhancements (cleaner --preview, status --stale)
- Team member tracking basics (solved_by names, points field)
- Web UI proof-of-concept (if TUI deemed insufficient)

## 0.5.x: Non-CTF Commands Active Support

### Module Quality Assessment

| Module | Quality | Action |
|--------|---------|--------|
| `cleaner.rs` (110 lines) | ~70% | **Keep** — add `--preview` mode, size freed display |
| `search.rs` (368 lines) | ~60% | **Keep** — later add full-text indexing |
| `status.rs` (151 lines) | ~85% | **Keep** — add stale repo detection, `--fetch` |
| `auditor.rs` (104 lines) | ~65% | **Keep** — add symlink safety, large file warnings |
| `stats.rs` (140 lines) | ~50% | **Refactor** — add CTF-specific metrics |
| `scaffold.rs` (91 lines) | ~40% | **Defer/drop** — brittle, no tests |
| `undo.rs` (141 lines) | ~55% | **Keep as-is** — narrow scope is correct |

### Team Features

- **0.5 (easy):** Proper team member tracking in .challenge.json (`solved_by: "alice"`, `points: 500`)
- **0.6 (medium):** Git-backed sync (`ctf pull`/`ctf push` — wrapping git)
- **Skip:** CTF platform API integration (out of scope)

### TUI Decision

**Freeze.** Current TUI (~300 lines) renders a dashboard but can't do CTF work. CTF workflows are long-form — CLI + text editor is the right interface. If a visual UI is wanted later, a web dashboard (axum + WebSocket) would serve better.

## Cross-Platform

| Platform | Support | Notes |
|----------|---------|-------|
| Linux | Full | Primary development platform |
| macOS | Full | Works on M1/Intel |
| Windows | Partial | Works but: file locking issues, slower watch, needs PowerShell docs |

Don't over-invest in Windows parity unless users request it. Add Windows CI in 0.5+ if demand exists.

## Competitive Landscape

No direct competitor in "CLI workspace organizer for CTF players." Wardex's differentiators:
- Shelve system (emotionally neutral, interactive)
- Git-native (auto-commit, history preservation)
- Nix/Home Manager first-class support
- Composable shell integration

Stay differentiated by staying CLI/file-system native. Don't become a web platform or library.

## Dropped

- Challenge templates per-category — CTFs too diverse for opinionated scaffolding
- CTF platform API integration (CTFd, PicoCTF) — out of scope
- Full plugin system pre-0.5 — too early, document hook points instead
