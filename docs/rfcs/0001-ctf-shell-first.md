# RFC 0001: CTF-Shell-First Product Direction

- Status: `Draft`
- Author: `Codex`
- Created: `2026-04-09`
- Updated: `2026-04-09`

## Summary

Wardex should prioritize the CTF workflow as its flagship experience and treat shell-native ergonomics as a core product requirement. This means the near-term roadmap should optimize for event lifecycle clarity, path/navigation speed, and strong Bash/Zsh completion support before broader workspace-manager expansion.

## Motivation

The repository already shows the strongest feature maturity in CTF management. At the same time, the product surface is broad enough that it risks feeling unfocused. A CTF-shell-first direction gives the project a clear identity, a simpler quality bar, and a more coherent roadmap for future CLI changes.

## Goals

- Make the CTF lifecycle the primary design target for new CLI work
- Treat shell integration and completion as first-class interface concerns
- Reduce ambiguity across overlapping CTF commands

## Non-Goals

- Dropping all non-CTF functionality from the codebase
- Finalizing every future workspace-management feature today

## Proposal

Wardex should adopt the following project rules:

- CTF workflow improvements take priority over broader feature expansion
- Bash and Zsh shell integration are part of the release surface
- Commands that support navigation, event selection, and challenge selection should be designed with completion in mind
- Changes to CTF command semantics should be reviewed against the CLI design rules

Planning and contributor docs should reflect this direction.

## Examples

```bash
wardex ctf use<TAB>
wardex ctf work web/<TAB>
eval "$(wardex ctf path --cd)"
```

Expected behavior:

- the user can discover events, categories, and challenge targets through completion
- shell-oriented commands print stable, eval-safe output
- the intended workflow is clear from help text and docs

## Alternatives Considered

- Keep broad workspace-manager work as equal priority
- Focus first on the optional TUI

The proposal is preferred because it builds on the strongest existing product area and creates a clearer release story.

## Risks And Tradeoffs

- Some non-CTF areas may progress more slowly in the short term
- Strong shell guarantees can limit how freely output formats evolve

## Rollout Plan

1. Publish CLI design principles and RFC process
2. Define the canonical CTF workflow and shell contract
3. Implement completion and shell integration improvements
4. Reassess broader expansion after the flagship path is strong

## Open Questions

- Should completion support be generated, scripted, or hybrid?
- Which commands need explicit machine-readable modes beyond current shell helpers?

## Design Checklist

- Checked against [`docs/CLI_DESIGN.md`](/run/host/mnt/Data/Workspace/1_Projects/Dev-CLI-Wardex/docs/CLI_DESIGN.md)
- Shell and completion implications reviewed
- Help text and output behavior considered
- Migration story documented if behavior changes
