# CTF And Shell Integration Plan

This plan narrows Wardex's immediate mission to one outcome: make the CTF workflow excellent from inside the shell.

## Product Goal

Wardex should help a user move through an event with minimal manual navigation, minimal typing, and minimal uncertainty about what command comes next.

## User Experience Target

A strong shell-native CTF flow should look like this:

1. Create or switch to an event quickly.
2. Tab-complete event names, categories, and challenge paths.
3. Jump into the right folder without hand-typing deep paths.
4. Import or create challenges with useful defaults.
5. Shelve challenges interactively — clean up, tag, note, and move on.
6. Finish events with all remaining work shelved automatically.

## Priority Features

## 1. The Shelve System

The signature feature. `ctf shelve` is the one command for finishing work on a challenge:

- Interactive by default with navigable prompts (arrow keys, space, enter)
- Handles all outcomes: self-solved, team-solved, unsolved
- File triage with blacklist (delete for disk) and whitelist (keep and archive)
- Challenge metadata in `.challenge.json` (flag, status, notes, imported_from)
- Flags to skip any interactive step for scripting

See [`docs/ctf-lifecycle.md`](/run/host/mnt/Data/Workspace/1_Projects/Dev-CLI-Wardex/docs/ctf-lifecycle.md) for the full interactive flow.

## 2. Completion Support

- Bash completion for core `wardex ctf` commands
- Zsh completion for the same surface
- Suggestions for:
  - subcommands
  - active and known event names
  - category names
  - challenge names or category/name paths
  - selected long flags where useful

## 3. Shell Navigation

- Make `ctf path --cd` and `ctf add --cd` stable enough for wrappers and aliases.
- Add recommended shell functions for Bash and Zsh.
- Ensure outputs meant for shell evaluation stay free of extra formatting.

## 4. Command Ergonomics

- Use natural verbs: `add`, `shelve`, `import`, `init`, `use`, `finish`.
- `add` is the primary creation verb with `--cd` for shell navigation.
- `shelve` replaces `solve` and `done` as the primary challenge completion verb.
- `solve`, `done`, and `work` remain as hidden aliases for muscle memory.
- Interactive-first: commands with multiple decisions use navigable prompts by default.
- Make help text read like a workflow, not just isolated commands.

## 5. Context Awareness

- Improve interaction between current directory, active global state, and fuzzy event resolution.
- Make context behavior predictable enough for completions and wrappers.
- Show concise context info when it helps prevent mistakes.

## Delivery Phases

### Phase A: Command Contract

- Freeze the intended CTF lifecycle around the shelve system.
- Implement `ctf shelve` with interactive flow and challenge metadata.
- Add `--cd` flag to `ctf add`, making `work` a hidden alias.
- Define output guarantees for shell-oriented commands.
- Decide how completions will source event and challenge candidates.

### Phase B: Completion MVP

- Ship subcommand and static flag completion.
- Add dynamic completion for event names and categories.
- Document installation for Bash and Zsh.

### Phase C: Dynamic Challenge Completion

- Complete challenge paths based on current or active event context.
- Support fuzzy or prefix-friendly suggestions where practical.
- Add tests for completion-related command behavior.

### Phase D: Workflow Polish

- Tune file triage defaults (blacklist and whitelist patterns).
- Improve templates, notes, and writeup flow.
- Add examples for timed-event usage.
- Tighten help output and docs around the final workflow.

## Technical Notes

Questions to answer during implementation:

- Should completion be generated directly from Clap or layered with custom logic?
- Is a dedicated `wardex completions <shell>` command needed?
- Which commands need stable machine-readable output modes?
- How should dynamic completion behave when no active event exists?
- How much fuzzy matching is helpful before suggestions become surprising?
- What should the default blacklist and whitelist patterns be?
- How should `.challenge.json` interact with the existing `.ctf_meta.json`?

## Definition Of Done

The CTF shell experience is "good enough" when:

- A user can install completions in Bash or Zsh in a few minutes.
- Common event and challenge navigation is mostly tab-driven.
- `ctf shelve` interactively handles solved, team-solved, and unsolved challenges.
- Shell wrappers for `ctf add --cd` and `ctf path` are documented and dependable.
- Core CTF flows feel faster with Wardex than with manual folders and aliases alone.
