# Shell Output Contracts

This document specifies which commands produce shell-stable output and what that output looks like. Shell wrappers, aliases, and scripts may depend on these contracts.

## Stability Levels

| Level | Meaning |
|-------|---------|
| **Stable** | Output format is guaranteed within a minor version. Breaking changes require a version bump and migration note. |
| **Unstable** | Output is human-oriented and may change at any time. Do not parse or eval. |

## Stable Output Commands

### `ctf path [event] [challenge]`

Prints the absolute path to an event or challenge directory. No trailing newline decoration, no prose.

```
/home/user/workspace/1_Projects/CTFs/2026_SecretCTF
```

Stability: **Stable**. One line, absolute path, no quotes.

### `ctf path --cd [event] [challenge]`

Prints a shell-safe `cd` command for eval.

```
cd '/home/user/workspace/1_Projects/CTFs/2026_SecretCTF'
```

Stability: **Stable**. Format: `cd '<path>'` with single quotes. Safe for `eval "$(wardex ctf path --cd)"`.

### `ctf add <cat/name> --cd`

After creating the challenge, prints a `cd` command.

```
Created challenge: pwn/heap-overflow
Created solve.py template
cd '/home/user/workspace/1_Projects/CTFs/2026_SecretCTF/pwn/heap-overflow'
```

Stability: The `cd` line is **Stable** (same format as `path --cd`). The info lines above it go to stdout but are **Unstable** — do not parse them. Shell wrappers should use `eval "$(wardex ctf add pwn/heap --cd 2>/dev/null)"` or extract the last line.

Note: In a future release, info lines may move to stderr so that only the `cd` line is on stdout.

### `config goto <folder>`

Prints the absolute path to a workspace folder.

```
/home/user/workspace/1_Projects
```

Stability: **Stable**. One line, absolute path, no quotes.

## Unstable Output Commands

All other commands produce human-oriented output that may change between versions:

| Command | Output type |
|---------|------------|
| `ctf init` | Info messages |
| `ctf list` | Table |
| `ctf info` | Context summary |
| `ctf status` | Challenge table |
| `ctf shelve` | Interactive prompts + status |
| `ctf finish` | Multi-step workflow output |
| `ctf check` | Event expiry list |
| `ctf import` | Import progress |
| `ctf writeup` | File path |
| `ctf schedule` | Confirmation |
| `ctf archive` | Confirmation |
| `ctf solve` | Legacy solve output |

## Rules For New Commands

When adding a command that produces shell-evaluable output:

1. Document the output format in this file.
2. Mark it as Stable.
3. Add a test that verifies the exact output format.
4. Prefer moving informational output to stderr so stdout stays clean.
5. Use single-quoted paths in `cd` commands for eval safety.
