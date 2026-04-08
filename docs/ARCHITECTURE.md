# Wardex Architecture

## Overview

Wardex is a CLI tool for workspace management and CTF event tracking. It follows a layered architecture with a thin CLI dispatcher, engine modules for business logic, and shared utilities.

## Module Structure

```
src/
  main.rs              CLI argument parsing and command dispatch (clap)
  lib.rs               Module exports
  config.rs            Layered config: file → env vars → defaults
  output.rs            Display formatting for command reports

  core/
    state.rs           Global CTF context (active event) in ~/.local/share/wardex/
    templates.rs       Solve script templates (pwn, web, generic)
    watcher.rs         Real-time inbox file monitoring (notify-debouncer-mini)

  engine/
    cleaner.rs         Inbox sorting via regex rules
    scaffold.rs        Project scaffolding (rust, python, node)
    auditor.rs         Workspace health (empty folders, extension mismatches)
    status.rs          Git dashboard (parallel repo scanning)
    search.rs          Flag search, fuzzy project find, content grep
    stats.rs           Workspace analytics (file counts, sizes)
    undo.rs            Transaction log for reversible file moves

    ctf/
      mod.rs           CtfMeta struct, re-exports, solve script creation
      event.rs         Event lifecycle: create, list, schedule, finish, context
      challenge.rs     Challenge add/solve/status/writeup
      import.rs        Smart archive import with category detection
      archive.rs       Event archival and zip creation
      resolve.rs       Fuzzy path resolution for events and challenges

  tui/                 (feature-gated behind --features tui)
    mod.rs, app.rs, event.rs, ui.rs, update.rs

  utils/
    fs.rs              Cross-device file moves via fs_extra
```

## Key Data Flows

### CTF Init → Solve → Finish

```
wardex ctf init MyEvent
  → create_event() → mkdir {ctf_root}/{year}_{name}/
  → create category dirs (pwn, web, crypto, rev)
  → write .ctf_meta.json
  → set_active_event() → save to global state

wardex ctf add pwn/heap-overflow
  → get_active_event_root() → walk up CWD or check global state
  → mkdir {event}/pwn/heap-overflow/
  → write solve.py template

wardex ctf solve "flag{found_it}"
  → save flag.txt
  → detect solve.py → append to notes.md
  → git add + commit (skippable with --no-commit)
  → create solution.zip (skippable with --no-archive)
  → move challenge to 4_Archives/CTFs/{year}/{event}/{category}/{name}

wardex ctf finish
  → git clean -dXn → interactive cleanup
  → git add + commit
  → mark end_time in metadata
  → archive_event() → move to 4_Archives/CTFs/{year}/
```

### Context Resolution

When a CTF command runs without an explicit event name:
1. Walk up from CWD looking for `.ctf_meta.json` (local context)
2. Check `~/.local/share/wardex/state.json` for saved active event
3. Fall back to latest event by year in CTF root

### Configuration

Layered (highest priority first):
1. `WX_*` environment variables (e.g. `WX_PATHS_WORKSPACE`)
2. Config file: `--config <path>` or `~/.config/wardex/config.yaml`
3. Built-in defaults (`~/workspace`, categories: pwn/web/crypto/rev)

CTF commands work without any config file — defaults are sufficient.

## Design Decisions

- **Parallelism**: `rayon` for workspace audit and git status scanning
- **Error handling**: `anyhow` with context, `fs_err` for path-aware fs errors
- **Gitignore respect**: `ignore` crate for file walking
- **Archive format detection**: Content-based category guessing with confidence levels
- **State**: Simple JSON files, no database
