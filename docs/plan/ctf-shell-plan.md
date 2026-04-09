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
5. Solve, note, archive, and finish without losing context.

## Priority Features

## 1. Completion Support

- Bash completion for core `wardex ctf` commands
- Zsh completion for the same surface
- Suggestions for:
  - subcommands
  - active and known event names
  - category names
  - challenge names or category/name paths
  - selected long flags where useful

## 2. Shell Navigation

- Make `ctf path --cd` and `ctf work` stable enough for wrappers and aliases.
- Add recommended shell functions for Bash and Zsh.
- Ensure outputs meant for shell evaluation stay free of extra formatting.

## 3. Command Ergonomics

- Prefer natural verbs over vague ones.
- Treat `add` as the default creation verb unless a stronger alternative is proven.
- Reduce overlap or confusion between `add` and `work`.
- Clarify the role of `solve` versus `done`.
- Make help text read like a workflow, not just isolated commands.
- Prefer defaults that match real competition usage.

## 4. Context Awareness

- Improve interaction between current directory, active global state, and fuzzy event resolution.
- Make context behavior predictable enough for completions and wrappers.
- Show concise context info when it helps prevent mistakes.

## Delivery Phases

### Phase A: Command Contract

- Freeze the intended CTF lifecycle.
- Decide whether `work` survives as an alias, shortcut, or removal candidate.
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

- Tune wrappers, templates, notes, and writeup flow.
- Add examples for timed-event usage.
- Tighten help output and docs around the final workflow.

## Technical Notes

Questions to answer during implementation:

- Should completion be generated directly from Clap or layered with custom logic?
- Is a dedicated `wardex completions <shell>` command needed?
- Which commands need stable machine-readable output modes?
- How should dynamic completion behave when no active event exists?
- How much fuzzy matching is helpful before suggestions become surprising?

## Definition Of Done

The CTF shell experience is "good enough" when:

- A user can install completions in Bash or Zsh in a few minutes.
- Common event and challenge navigation is mostly tab-driven.
- Shell wrappers for `ctf work` and `ctf path` are documented and dependable.
- Core CTF flows feel faster with Wardex than with manual folders and aliases alone.
