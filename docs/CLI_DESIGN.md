# CLI Design Principles

This document defines the design rules for Wardex's command-line experience.

Wardex is not just a collection of commands. It is a shell-native workflow tool, with CTF operations as the flagship experience. New commands and changes should follow these principles so the CLI stays fast, predictable, and scriptable.

## Product Intent

Wardex should feel:

- fast under pressure
- easy to discover from `--help`
- easy to complete with the shell
- safe for filesystem-heavy operations
- stable enough to script

## Core Principles

## 1. Optimize For The Main Workflow

The CTF event and challenge lifecycle is the primary design target.

When a design tradeoff exists:

- prefer the path that improves active CTF use
- avoid adding generic commands that weaken the main workflow
- keep workspace-management features secondary unless they clearly reinforce the flagship path

## 2. Design For Shell Use First

Every command should be evaluated as if it will be used:

- from a shell alias
- inside command substitution
- behind tab completion
- during repeated, time-sensitive terminal work

Commands that print paths or state for shell use must avoid decorative noise.

## 3. Prefer Clear Verbs And Predictable Nouns

Command names should communicate action directly.

Good patterns:

- `ctf init`
- `ctf use`
- `ctf path`
- `ctf add`
- `ctf shelve`

Avoid:

- clever names that hide meaning
- vague verbs such as `work`, `do`, `handle`, or `manage` when a more direct verb exists
- near-duplicate commands without a clear distinction
- subcommands whose purpose cannot be inferred from the help text

If a user is more likely to guess `add` than `work`, the CLI should prefer `add`.

## 4. One Command, One Primary Job

Commands may have side effects, but the primary action should be obvious.

If a command performs multiple steps, the help text must explain the workflow clearly. If two commands overlap heavily, either:

- make one a true alias
- separate them by intent
- remove the weaker interface

## 5. Context Should Help, Not Surprise

Wardex can infer context from:

- the current working directory
- saved active event state
- fuzzy event resolution

This is powerful, but it must stay predictable.

Rules:

- local directory context should have the highest confidence
- active-event fallback should be visible in docs and errors
- fuzzy matching should help with recall, not silently choose the wrong target
- ambiguity should produce a helpful error instead of a guess

## 6. Make Safe Operations Easy And Risky Operations Explicit

Wardex performs real filesystem work. The CLI must protect users from accidental damage.

Rules:

- destructive or stateful operations should have actionable errors
- dry-run support should exist where it meaningfully reduces risk
- shell-evaluable commands must produce safe output
- implicit behavior is acceptable only when it matches strong user expectations

## 7. Interactive By Default, Flags To Skip

Multi-step commands should use navigable interactive prompts by default (arrow keys, space to toggle, enter to confirm). Users should not need to memorize flag combinations to use the tool.

Rules:

- default mode is interactive with smart presets
- every interactive step maps to a flag that skips it
- `--auto` runs with smart defaults and no prompts
- shell wrappers and scripts use flags for non-interactive operation

This applies especially to commands with multiple decisions, such as `ctf shelve`.

## 8. Human-Friendly By Default, Machine-Friendly By Design

Default output should help a human understand what happened quickly.

But commands intended for scripting or shell integration must have stable output rules.

Examples:

- `ctf path --cd` and `ctf add --cd` should stay eval-safe
- path-printing commands should avoid extra prose
- status-like commands may be rich for humans, but should not be reused as hidden machine interfaces unless explicitly designed for that purpose

## 9. Completion Is Part Of The Interface

Tab completion is not a bonus feature. It is part of the command design.

New commands should be reviewed for:

- subcommand completion
- flag completion
- dynamic completion of event names, categories, and challenge paths
- whether the command shape is awkward to complete

If a command is hard to complete, it is probably too awkward to type.

## 10. Help Output Should Teach The Workflow

Help text should do more than list parameters.

Rules:

- describe the command in the user's terms
- include the most common usage shape
- explain workflow relationships when relevant
- avoid internal terminology unless it helps the user act

## 11. Defaults Should Reflect Real Use

Good defaults reduce typing and reduce decision load.

Prefer defaults that match common competition behavior, common shell usage, and the most likely directory context. If defaults would be surprising in a high-pressure situation, require the user to be explicit instead.

## Command Design Rules

## Naming

- Use short, concrete verbs.
- Prefer the verb a new user would guess first.
- Prefer consistency over novelty.
- Keep sibling commands parallel in shape when they do related things.
- Treat vague verbs as a design smell unless they are clearly established and narrowly scoped.
- If an old command has weak naming, keep it only as an alias or compatibility layer, not as the documented primary interface.

## Arguments

- Positional arguments should represent the most important user input.
- Optional behavior should use flags.
- Avoid requiring multiple positional arguments when a single structured path is clearer.

## Naming Review Checklist

Before adding or renaming a command, ask:

- Would a new user guess this verb without reading the docs?
- Does the verb describe the main action, not a feeling or workflow mood?
- Is this name still understandable when shown in shell completion?
- Is there a more natural everyday verb available?

If the answer to the last question is yes, use the more natural verb.

## Flags

- Long flags should be explicit and readable.
- Short flags should be reserved for high-frequency actions.
- Negation flags such as `--no-archive` are acceptable when they modify a default workflow cleanly.

## Output

- Print only what the command needs to communicate.
- Do not mix shell-oriented output with explanatory prose.
- Keep formatting stable for commands that users are likely to wrap.

## Error Messages

- State what failed.
- State which target or path was involved.
- State what the user can do next.
- Prefer ambiguity errors over silent incorrect guesses.

## Shell Integration Rules

- Support Bash and Zsh first.
- Completion support should cover the main CTF workflow before broader features.
- Shell helper docs should be small, copyable, and durable.
- Commands used inside `eval` or shell wrappers must avoid unstable formatting.

## RFC Trigger Checklist

An RFC is recommended when a change does any of the following:

- adds or removes a top-level command or major subcommand
- changes command semantics in a breaking way
- changes output relied on by shell integration
- introduces new context-resolution rules
- adds a major workflow concept
- changes the project's CTF-first product direction

## The Shelve System

Shelving is Wardex's signature concept. It replaces the old `solve` / `done` split with a single emotionally neutral verb that handles all challenge outcomes.

Shelving means: organize it, tag it, clean it up, and put it on the shelf. A shelved challenge is accessible, browsable, and reopenable. It is not "giving up" and it is not "closing forever".

Key properties:

- `ctf shelve` is the one command for finishing work on a challenge
- interactive by default: status, flag, file triage, notes, archive decision
- file triage uses a configurable blacklist (delete to reclaim disk) and whitelist (keep and archive)
- challenge metadata (flag, status, notes) lives in `.challenge.json`, not loose files
- `solve` and `done` are hidden aliases for `shelve`
- `add --cd` replaces the old `work` command

See [`docs/ctf-lifecycle.md`](/run/host/mnt/Data/Workspace/1_Projects/Dev-CLI-Wardex/docs/ctf-lifecycle.md) for the full lifecycle and interactive flow.

## Non-Goals

Wardex does not need to:

- optimize every command equally
- expose every internal capability as a command
- prioritize novelty over reliability
- add broad features before the main workflow is excellent
