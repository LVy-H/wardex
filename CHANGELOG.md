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
