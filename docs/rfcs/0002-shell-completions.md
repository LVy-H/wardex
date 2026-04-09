# RFC 0002: Shell Completion Architecture

- Status: `Accepted`
- Author: `Hoang`
- Created: `2026-04-09`
- Updated: `2026-04-09`

## Summary

Wardex should provide shell completions via `clap_complete`, exposed through a `wardex completions <shell>` command. Static completions (subcommands and flags) ship first. Dynamic completions (event names, categories, challenge paths) follow in a later release.

## Motivation

Tab completion is a core part of the shell-native CTF experience. Without it, users must remember exact command names, flag spellings, and event names. The plan documents (`CLI_DESIGN.md` principle 9, `ctf-shell-plan.md`) identify completion as a priority feature.

## Goals

- Ship static completion for all commands and flags in Bash and Zsh
- Provide a single command to generate completion scripts
- Keep the architecture simple enough that dynamic completions can be added later

## Non-Goals

- Dynamic completion for event names, categories, or challenges (deferred to alpha7+)
- Fish or PowerShell support (Bash and Zsh first)
- IDE or editor integration

## Proposal

### Approach: Clap-Generated via `clap_complete`

Use `clap_complete` to generate shell completion scripts from the existing Clap command tree. This is the simplest approach that gives correct subcommand and flag completion with zero maintenance overhead.

### Command

```
wardex completions <shell>
```

Where `<shell>` is `bash` or `zsh`. The command prints the completion script to stdout.

### Install Flow

**Bash:**

```bash
wardex completions bash > ~/.local/share/bash-completion/completions/wardex
# or
wardex completions bash >> ~/.bashrc
```

**Zsh:**

```bash
wardex completions zsh > ~/.zfunc/_wardex
# then ensure ~/.zfunc is in fpath and compinit is called
```

### What Completes

In this release (static only):

| Context | Completes |
|---------|-----------|
| `wardex <TAB>` | Top-level subcommands: ctf, config, init, clean, ... |
| `wardex ctf <TAB>` | CTF subcommands: init, add, shelve, path, ... |
| `wardex ctf shelve --<TAB>` | Flags: --auto, --note, --no-clean, --move, ... |
| `wardex ctf add --<TAB>` | Flags: --cd |

Hidden commands (`work`, `done`) are excluded from completion automatically by Clap.

### Future: Dynamic Completions

For a later release, dynamic completion can be added using `clap_complete`'s `CompleteEnv` or custom shell functions that call `wardex` subcommands to list events/categories. This RFC does not prescribe the approach — it will depend on how well `clap_complete` handles custom value hints.

## Alternatives Considered

**Maintained shell scripts**: Hand-written completion scripts for Bash and Zsh. More control but high maintenance burden — every command change requires manual script updates.

**Hybrid**: Clap-generated base with hand-written dynamic additions. Viable for dynamic completions but unnecessary for static-only scope.

The Clap-generated approach is preferred because it is zero-maintenance for the static case and Clap 4.5 already supports `clap_complete` well.

## Risks And Tradeoffs

- Generated completions may not be as polished as hand-written ones for edge cases
- Hidden aliases (work, done) correctly excluded, but `solve` still shows (it should — it's not hidden yet, only deprecated in docs)
- Dynamic completion will require revisiting this architecture

## Rollout Plan

1. Add `clap_complete` dependency
2. Add `wardex completions <shell>` command
3. Verify Bash and Zsh completion output
4. Document install flow in README or dedicated shell doc
5. Add dynamic completions in a future release

## Design Checklist

- Checked against `docs/CLI_DESIGN.md` — principle 9 (completion is part of interface)
- Shell and completion implications reviewed — Bash and Zsh first
- Help text and output behavior considered — completions command outputs script to stdout
- Migration story documented — no breaking changes, additive feature
