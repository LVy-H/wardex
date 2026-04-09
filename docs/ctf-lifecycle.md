# CTF Lifecycle

Status: Draft

This document defines the canonical CTF workflow in Wardex and the design decisions behind it.

## Design Principles

### Natural Verbs First

`add` is more natural than `work`.

- `add` describes a clear action and is easier to guess before reading docs
- `work` is vague and does not tell the user whether it creates, navigates, selects, or edits
- use `ctf add` as the primary verb for creating a challenge entry
- do not introduce similarly vague verbs in future CLI design

### Interactive First

Commands should be interactive by default, using navigable prompts (arrow keys, space to toggle, enter to confirm). Flags exist to skip prompts for scripting and power users.

Why:

- CTF players are under time pressure and do not memorize flag combinations
- interactive prompts teach the workflow while the user runs it
- flags like `--auto` or specific overrides provide the non-interactive escape hatch

### The Shelve System

`shelve` is Wardex's signature concept for challenge lifecycle management.

Shelving means: organize it, tag it, clean it up, and put it on the shelf. It is not "giving up" and it is not "archiving forever". A shelved challenge is accessible, browsable, and reopenable.

This replaces the old `solve` / `done` split with a single emotionally neutral verb that works for all outcomes:

- solved it yourself
- teammate solved it
- nobody solved it, moving on
- event ended

## Lifecycle

```
init → use → [add | import] → [hack] → shelve → finish
```

### 1. Event Setup

```bash
wardex ctf init SecretCTF --date 2026-04-12
wardex ctf use SecretCTF
```

`init` creates the event directory, default category folders, `notes.md`, and `.ctf_meta.json`. It sets the event as globally active.

`use` switches between existing events.

### 2. Challenge Creation

```bash
wardex ctf add pwn/heap-overflow        # create challenge, stay in place
wardex ctf add pwn/heap-overflow --cd   # create challenge, print cd command
wardex ctf import challenge.tar.gz      # extract + smart category detection
```

`add` is the primary creation verb. The `--cd` flag outputs a `cd '<path>'` command for shell eval, replacing the old `work` command.

`import` handles downloaded challenge archives with interactive category selection.

### 3. Shelving A Challenge

```bash
wardex ctf shelve                       # interactive (default)
wardex ctf shelve "flag{got_it}"        # quick solve with flag
wardex ctf shelve --auto                # smart defaults, no prompts
```

#### Interactive Flow (Default)

When run without flags, `shelve` walks through a guided flow using navigable prompts:

**Step 1: Status** (Select with arrow keys)

```
┌ What happened with this challenge?
│ ● I solved it
│ ○ Team solved it
│ ○ Unsolved — shelve for later
└ ↑↓ to move, enter to select
```

**Step 2: Flag** (Input, shown if solved/team-solved)

```
┌ Enter the flag:
│ flag{heap_feng_shui_master}
└
```

**Step 3: File Triage** (MultiSelect with smart defaults)

Files are pre-sorted into three tiers:

| Tier | Default state | Examples |
|------|---------------|---------|
| Whitelist (keep) | Checked to keep | `solve.*`, `exploit.*`, `notes.md`, `Dockerfile`, `docker-compose.yml`, imported originals |
| Blacklist (remove) | Checked to remove | `node_modules/`, `.venv/`, `core.*`, `*.o`, `.gdb_history`, `__pycache__/` (if not from challenge) |
| Unknown | Unchecked, user decides | Everything else |

```
┌ File triage — what to keep? (space to toggle, enter to confirm)
│ KEEP:
│ ☑ solve.py                    2 KB
│ ☑ notes.md                    1 KB
│ ☑ heap-overflow.tar.gz        imported original
│ ☑ Dockerfile                  1 KB
│ DELETE (reclaim 623 MB):
│ ☑ node_modules/             420 MB
│ ☑ .venv/                    180 MB
│ ☑ core.48291                 23 MB
│ OTHER:
│ ☐ helper.py                   3 KB
│ ☐ test_payload.bin            1 KB
└ ctrl+a select all, ctrl+n select none
```

The blacklist and whitelist are configurable in `config.yaml`.

**Step 4: Note** (Input, optional)

```
┌ Add a note? (enter to skip)
│ classic tcache poison, used house of force
└
```

**Step 5: Archive** (Confirm)

```
┌ Move to archives? [y/n]
└
```

#### Non-Interactive Overrides

Every interactive step maps to a flag:

| Flag | Skips |
|------|-------|
| `"flag{...}"` (positional) | Status + flag prompt |
| `--no-clean` | File triage prompt |
| `--note "text"` | Note prompt |
| `--move` / `--no-move` | Archive prompt |
| `--auto` | All prompts, use smart defaults |

### 4. Finishing An Event

```bash
wardex ctf finish
```

`finish` shelves all remaining unshelved challenges (interactively or with `--auto`), makes a final commit, marks `end_time` in metadata, and optionally archives the event.

### 5. Direct Archive

```bash
wardex ctf archive SecretCTF
```

`archive` moves an event to archives without the shelve ceremony. Use for manual archival of events that are already cleaned up.

## Challenge Metadata

Challenge state lives in structured metadata, not loose files. No more `flag.txt`.

Per-challenge metadata (`.challenge.json`):

```json
{
  "name": "heap-overflow",
  "category": "pwn",
  "status": "solved",
  "flag": "flag{heap_feng_shui_master}",
  "solved_by": "me",
  "note": "classic tcache poison, used house of force",
  "imported_from": "heap-overflow.tar.gz",
  "shelved_at": "2026-04-12T18:30:00",
  "created_at": "2026-04-12T10:15:00"
}
```

Status values: `active`, `solved`, `team-solved`, `unsolved`.

Benefits:

- flags are queryable by `ctf status` without reading loose files
- all solve-state is in one place
- metadata travels with the archived challenge
- clean challenge directories

## Command Reference

### Primary Commands (The Core Six)

| Command | Role |
|---------|------|
| `ctf init <name>` | Create event |
| `ctf use <event>` | Switch active event |
| `ctf add <cat/name>` | Create challenge (`--cd` for shell navigation) |
| `ctf import <file>` | Import challenge archive |
| `ctf shelve [flag]` | Shelve a challenge (interactive by default) |
| `ctf finish [event]` | End event, shelve remaining, archive |

### Supporting Commands

| Command | Role |
|---------|------|
| `ctf path [event] [challenge]` | Print path (`--cd` for shell eval) |
| `ctf info` | Show current context |
| `ctf status` | Challenge progress table |
| `ctf list` | List all events with solve counts |
| `ctf check` | Show expired/active events |
| `ctf schedule [event]` | Set event times |
| `ctf writeup` | Generate writeup from notes |
| `ctf archive <name>` | Direct archive without ceremony |

### Aliases (Hidden, For Muscle Memory)

| Alias | Target |
|-------|--------|
| `ctf solve` | `ctf shelve` |
| `ctf done` | `ctf shelve` |
| `ctf work <cat/name>` | `ctf add <cat/name> --cd` |

## Context Resolution

When a command needs an event context and none is specified:

1. Local directory: walk up from CWD looking for `.ctf_meta.json`
2. Global state: check `~/.local/share/wardex/state.json`
3. Latest event: fall back to most recent by year

Local context always wins. This prevents accidental cross-event operations.

## Naming Rules

- Prefer verbs that describe an action directly
- Prefer the verb a user would try first without reading docs
- Do not rely on shell completion to explain a confusing verb
- Do not make a vague command the documented primary path
- If a convenience alias exists, document the natural verb first
- Treat vague verbs as a design smell

## Migration Notes

The shelve system changes these behaviors from the current implementation:

- `flag.txt` is replaced by `flag` field in `.challenge.json`
- `solve` becomes an alias for `shelve` instead of a primary command
- `done` becomes an alias for `shelve`
- `work` becomes an alias for `add --cd`
- challenge archival is no longer automatic on solve; it is an interactive choice during shelve
- file cleanup is part of shelve, not finish
