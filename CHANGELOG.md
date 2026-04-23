# Changelog

All notable changes to Wardex will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Wardex uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html) with alpha pre-release tags.

## Versioning Policy

### Development Phases

| Phase | Version pattern | Stability | Example |
|-------|----------------|-----------|---------|
| **Alpha** | `0.x.0-alphaN` | Breaking changes expected. Commands, flags, output, and metadata schemas may change between alphas. | `0.2.0-alpha4` |
| **Beta** | `0.x.0-betaN` | Feature-complete. Breaking changes only for critical bugs. | `0.2.0-beta1` |
| **Release** | `0.x.0` | Stable within the minor version. | `0.2.0` |

### Support Policy

- **Only the latest version is supported.** There are no backports, security patches, or bug fixes for older versions.
- **No backward compatibility guarantee** during alpha. Upgrading may require re-running commands or migrating metadata.
- **Best-effort migration** is provided where practical. Wardex includes auto-migration for metadata schemas (see below) but does not guarantee migration for all changes.
- Users should always upgrade to the latest version.

### Branching Strategy

| Branch | Purpose | Merges to |
|--------|---------|-----------|
| `main` | Current development. All alpha work happens here. | — |
| `release/0.x` | Created at beta. Receives only bug fixes. | — |
| `feat/*`, `fix/*` | Feature and fix branches. Short-lived. | `main` |

During alpha:
- All work lands on `main` directly or via feature branches.
- No release branches exist yet. Tags mark alpha releases (`v0.2.0-alpha4`).

At beta:
- A `release/0.2` branch is cut from `main`.
- `main` continues with next-version work.
- Only critical fixes are cherry-picked to the release branch.

### Metadata Schema Versioning

All metadata files (`.ctf_meta.json`, `.challenge.json`) include a `schema_version` integer field.

| Behavior | Description |
|----------|-------------|
| **Auto-migrate on read** | When Wardex loads a file with an older schema version, it upgrades transparently. |
| **Forward compat warning** | If the file has a newer schema version than the binary, Wardex warns but tries best-effort parsing. |
| **Pre-schema files** | Files without `schema_version` are treated as version 0 and migrated on first access. |
| **flag.txt migration** | Challenges with `flag.txt` but no `.challenge.json` get metadata auto-created on read. |

Schema version history:

| Version | Introduced in | Changes |
|---------|---------------|---------|
| 0 (implicit) | Pre-alpha4 | No schema_version field. flag.txt for flags. |
| 1 | 0.2.0-alpha4 | `.challenge.json` with schema_version, status, flag, solved_by, note, imported_from, shelved_at |

## [0.3.0-alpha2] - 2026-04-23

### Summary

Tab-completion hardening. Fixes a broken `~`-prefix path completion, migrates
the Home Manager module off the deprecated `programs.zsh.initExtra`, and
rewrites the CTF completion helpers to be instruction-based — completions now
reflect explicit configuration (`config.yaml`, `wardex ctf use <event>`) rather
than silently falling back to hard-coded guesses.

### Fixed

| Area | Fix |
|------|-----|
| Path completion | `~/Downloads/x<TAB>` no longer produces garbled output. Added `ValueHint::FilePath` / `AnyPath` to `ctf import`, `--config`, `search`, and `info` so clap_complete routes through its tilde-aware path completer. |
| Home Manager module | `programs.zsh.initExtra` → `programs.zsh.initContent` with `lib.mkAfter`, silencing the Home Manager deprecation warning. |
| Category completion | Case-sensitivity inconsistency removed — `PW<TAB>` now matches `pwn/` like the other completers. |

### Changed

| Area | Change |
|------|--------|
| `challenge_completer` | No longer silently picks the lexically-largest event directory when no active event is set. Returns empty; user must run `wardex ctf use <event>`. |
| `category_completer` | Reads `config.ctf.default_categories` instead of a hard-coded six-category list. Returns empty if no config and no active event. |
| `resolve_ctf_root` | Replaces hand-rolled YAML scan with `Config::load()` — honors `paths.ctf_root` exactly as the main binary does. `WX_PATHS_CTF_ROOT` now tilde-expands. |

## [0.3.0-alpha1] - 2026-04-10

### Summary

First 0.3.x release: CTF workflow polish. Metadata-driven writeups with flag redaction,
config validation, enriched status table with JSON output, challenge path completions,
and multi-event context switching (`ctf use -`, `ctf recent`).

### Added

| Feature | Description | Status |
|---------|-------------|--------|
| Challenge path completer | TAB-complete `category/challenge` in `ctf path` | Stable |
| Config validation | `wardex config validate` checks paths, categories, regex | Stable |
| `ctf recent` | List last 5 recently used events | Stable |
| `ctf use -` | Switch back to previous event (like `cd -`) | Stable |
| Status summary line | Total/solved/unsolved/active counts after status table | Stable |
| Status JSON output | `wardex ctf status --format json` | Stable |
| Status solver/note columns | Shows who solved and notes preview in status table | Stable |
| Writeup metadata | Status icons, solver, flag, notes, timestamps in writeups | Stable |
| Writeup summary | Solve counts at top of generated writeup | Stable |
| Writeup flag redaction | `wardex ctf writeup --no-flags` hides flags | Stable |

### Changed

| Change | Before | After |
|--------|--------|-------|
| `ctf status` signature | No args | `--format table\|json` |
| `ctf writeup` signature | No args | `--no-flags` optional |
| `generate_writeup()` | Ignores metadata | Reads .challenge.json for each challenge |
| `challenge_status()` | 3-column table | 5-column table with summary line |
| `AppState` | Single event | Tracks previous + recent 5 events |

### Migration Guide

#### From 0.2.0-beta1

**No breaking changes.** All new features are additive. Existing state.json files auto-upgrade via `#[serde(default)]`.

## [0.2.0-beta1] - 2026-04-10

### Summary

First beta release. The CTF workflow and shell integration are feature-complete.
Breaking changes from this point only for critical bugs.

### Changed

| Change | Description |
|--------|-------------|
| Version policy | Beta: feature-complete, breaking changes only for critical bugs |
| CI | GitHub Actions workflow: fmt, clippy, test on every push and PR |
| CTF lifecycle status | Promoted from "Implemented" to "Stable" |
| Nix flake | Version updated, overlay added, HM module gets shell completion support |

### Migration Guide

#### From 0.2.0-alpha8

**No breaking changes.** This release formalizes the beta stability contract.

Nix Home Manager users: shell completions now auto-install. Update your flake input and rebuild.

## [0.2.0-alpha8] - 2026-04-10

### Summary

Ships **dynamic shell completions** for event names and categories via `clap_complete`'s
runtime completion engine. Hardens context resolution with edge-case tests, fixes a bug
where `ctf info` silently succeeded after event deletion, and adds shell output contract tests.

### Added

| Feature | Description | Status |
|---------|-------------|--------|
| Dynamic completions | Event name TAB-completion for `ctf use`, `path`, `archive`, `schedule`, `finish` | Experimental |
| Category completions | Category TAB-completion for `ctf add` (falls back to defaults) | Experimental |
| `CompleteEnv` integration | `source <(COMPLETE=bash wardex)` activates dynamic completions | Experimental |
| Context edge-case tests | Tests for deleted events, spaces in names, bare output format | - |
| Shell output contract tests | Verify `--cd` format and bare path output | - |

### Fixed

| Fix | Description |
|-----|-------------|
| `ctf info` after event deletion | Now returns error instead of silently succeeding |

### Changed

| Change | Before | After |
|--------|--------|-------|
| `clap_complete` feature | Static only | `unstable-dynamic` enabled for runtime completions |

### Migration Guide

#### From 0.2.0-alpha7

**No breaking changes.** Static completions (`wardex completions bash/zsh`) continue to work.

To enable dynamic completions (optional):
```bash
# Bash — add to ~/.bashrc
source <(COMPLETE=bash wardex)

# Zsh — add to ~/.zshrc
source <(COMPLETE=zsh wardex)
```

## [0.2.0-alpha7] - 2026-04-10

### Summary

Safety and correctness release: fixes panic risks, shell quoting, marks non-CTF
commands as experimental, and expands test coverage.

### Fixed

| Fix | Description |
|-----|-------------|
| Panic on non-UTF-8 paths | `.unwrap()` calls in challenge.rs, shelve.rs, archive.rs replaced with proper error handling |
| Shell quoting in `--cd` output | Paths with single quotes no longer break `eval $(wardex ctf ...)` |
| Panic in stats.rs | `Arc::try_unwrap` chain replaced with error propagation |
| Clippy warnings | Removed needless borrows in test code, removed unused import |

### Changed

| Change | Before | After |
|--------|--------|-------|
| Non-CTF commands | No label | `[experimental]` prefix in help text |

### Added

| Feature | Description |
|---------|-------------|
| New tests | Integration tests for `ctf use`, `ctf info`, `ctf writeup`, `ctf archive`, `ctf solve` |
| Shell quoting test | Tests that paths with special characters produce valid shell output |
| Experimental labels test | Verifies non-CTF commands show `[experimental]` in help |

### Migration Guide

#### From 0.2.0-alpha6

**No breaking changes.** Non-CTF commands now show `[experimental]` in help text but behavior is unchanged.

## [0.2.0-alpha6] - 2026-04-09

### Summary

Ships **shell completions** for Bash and Zsh, **configurable file triage** patterns,
and design docs (RFC 0002, output contracts).

### Added

| Feature | Description | Status |
|---------|-------------|--------|
| `wardex completions <shell>` | Generate Bash/Zsh completion scripts | Stable |
| Static completion | Subcommands and flags complete via TAB | Stable |
| Configurable triage patterns | `ctf.shelve.blacklist` and `ctf.shelve.whitelist` in config.yaml | Stable |
| RFC 0002 | Shell completion architecture (Accepted) | - |
| Shell output contracts | Documented stable vs unstable output in `docs/shell-output-contracts.md` | - |

### Changed

| Change | Before | After |
|--------|--------|-------|
| File triage patterns | Hardcoded in shelve.rs | Configurable via `ctf.shelve.blacklist`/`whitelist` in config.yaml |

### Migration Guide

#### From 0.2.0-alpha5

**No breaking changes.** New features only.

Install completions:
```bash
# Bash
wardex completions bash > ~/.local/share/bash-completion/completions/wardex
# Zsh
wardex completions zsh > ~/.zfunc/_wardex
```

Optional: add custom triage patterns to config.yaml:
```yaml
ctf:
  shelve:
    blacklist:
      - node_modules
      - .venv
      - __pycache__
    whitelist:
      - solve.
      - notes.md
      - Dockerfile
```

## [0.2.0-alpha5] - 2026-04-09

### Summary

Ships the **`ctf shelve` command** — Wardex's signature interactive challenge completion flow.

### Added

| Feature | Description | Status |
|---------|-------------|--------|
| `ctf shelve` command | Interactive challenge completion with status, flag, file triage, notes, archive | Experimental |
| File triage system | Blacklist/whitelist cleanup during shelve (node_modules, .venv, core, etc.) | Experimental |
| Interactive prompts | Select, MultiSelect, Input, Confirm via dialoguer | Stable |
| `--auto` flag | Skip all prompts with smart defaults | Stable |
| `--note` flag | Add note without prompting | Stable |
| `--no-clean` flag | Skip file triage | Stable |
| `--move` / `--no-move` flags | Control archive behavior | Stable |
| `--no-commit` flag | Skip git commit | Stable |

### Changed

| Change | Before | After |
|--------|--------|-------|
| `ctf solve` role | Primary solve command | Hidden alias for `ctf shelve` (still functional) |

### Migration Guide

#### From 0.2.0-alpha4

**No breaking changes.** `ctf solve` continues to work as before.

New: use `ctf shelve` for the full interactive flow, or `ctf shelve "flag{...}" --auto` for quick scripted use.

## [0.2.0-alpha4] - 2026-04-09

### Summary

Lays the groundwork for the shelve system: per-challenge metadata, `add --cd` for shell
navigation, and comprehensive planning docs establishing the CTF-shell-first direction.

### Added

| Feature | Description | Status |
|---------|-------------|--------|
| `ctf add --cd` flag | Print `cd '<path>'` after creation for shell eval | Stable |
| `.challenge.json` metadata | Per-challenge structured metadata (flag, status, notes, imported_from) | Experimental |
| Schema versioning | `schema_version` field in metadata files for auto-migration | Stable |
| CLI design principles | 11 core principles documented in `docs/CLI_DESIGN.md` | - |
| Development plan | Phased roadmap, workstreams, task list in `docs/plan/` | - |
| RFC process | Lightweight RFC process for CLI changes in `docs/rfcs/` | - |
| RFC 0001 | CTF-shell-first product direction | Accepted |
| CTF lifecycle doc | Canonical lifecycle with shelve system design in `docs/ctf-lifecycle.md` | - |

### Changed

| Change | Before | After |
|--------|--------|-------|
| Challenge creation navigation | `ctf work <cat/name>` | `ctf add <cat/name> --cd` |
| Flag storage (new challenges) | No metadata | `.challenge.json` with `active` status |
| Challenge status tracking | Directory location only | `.challenge.json` metadata + directory |
| `add_challenge()` return | `Result<()>` | `Result<PathBuf>` |

### Deprecated

| Command | Replacement | Notes |
|---------|-------------|-------|
| `ctf work <cat/name>` | `ctf add <cat/name> --cd` | Hidden alias, still functional |
| `ctf done <flag>` | `ctf shelve [flag]` (coming in alpha5) | Hidden from help, still functional |
| `flag.txt` | `.challenge.json` | Read for backwards compat, no longer written by new commands |

### Experimental

These features are functional but their interface may change:

- `.challenge.json` schema (fields may be added)
- `ChallengeStatus` enum values

### Migration Guide

#### From 0.2.0-alpha3

**No breaking changes.** All existing commands continue to work:

- `ctf work` still works as a hidden alias for `ctf add --cd`
- `ctf solve` and `ctf done` still work unchanged
- Challenges without `.challenge.json` are handled transparently
- If `flag.txt` exists and `.challenge.json` does not, metadata is migrated on read

**Recommended updates:**

1. Replace `ctf work` with `ctf add --cd` in shell aliases and scripts
2. Update shell wrappers that parse `ctf work` output (format unchanged: `cd '<path>'`)

### Internal

- Restructured documentation: `docs/plan/`, `docs/rfcs/`, `docs/CLI_DESIGN.md`
- Added `ChallengeMetadata` and `ChallengeStatus` types in `src/engine/ctf/mod.rs`
- `add_challenge()` now returns `Result<PathBuf>` instead of `Result<()>`

## [0.2.0-alpha3] - 2026-04-09

### Added

- Git added as test dependency
- Version bump from alpha2

## [0.2.0-alpha2] - 2026-04-09

### Changed

- Default workspace path set to user's home directory
- Refined config loading behavior

## [0.2.0-alpha1] - 2026-04-09

### Added

- Centralized command output handling (`src/output.rs`)
- CTF workflow shortcuts and enhanced shell integration
- Architecture documentation
