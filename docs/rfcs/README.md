# RFC Process

Wardex uses lightweight RFCs for significant CLI, workflow, and architecture decisions.

The goal is not bureaucracy. The goal is to make high-impact changes easier to reason about before code and docs drift apart.

## When To Write An RFC

Write an RFC for changes that:

- add, remove, or substantially change commands
- change shell integration or completion behavior
- alter context resolution or global state behavior
- introduce a new workflow concept
- affect project direction or release scope

Small fixes and isolated implementation details usually do not need an RFC.

## RFC Workflow

1. Copy [`0000-template.md`](/run/host/mnt/Data/Workspace/1_Projects/Dev-CLI-Wardex/docs/rfcs/0000-template.md) to a new numbered file.
2. Give it a short, descriptive title.
3. Fill in the problem, proposal, alternatives, risks, and rollout sections.
4. Keep the RFC grounded in user workflow, not just implementation detail.
5. Update the RFC status as discussion and implementation progress.

## Suggested Status Values

- `Draft`
- `Proposed`
- `Accepted`
- `Implemented`
- `Superseded`
- `Rejected`

## Numbering

Use ascending numbers:

- `0001-ctf-shell-first.md`
- `0002-shell-completions.md`

Do not renumber accepted RFCs.

## Review Expectations

Good RFCs should answer:

- What user problem are we solving?
- Why is the change needed now?
- What alternatives were considered?
- How does this affect shell usage, help text, and completions?
- What breaks, and how will users migrate?

## Design Guardrails

All RFCs should be checked against [`docs/CLI_DESIGN.md`](/run/host/mnt/Data/Workspace/1_Projects/Dev-CLI-Wardex/docs/CLI_DESIGN.md).

Especially for CLI-facing changes, reviewers should ask:

- Is the workflow clearer after this change?
- Is the command easy to discover and complete?
- Is the output safe for shell usage?
- Does it reduce or increase ambiguity?

## Directory Layout

- `docs/rfcs/README.md`: process overview
- `docs/rfcs/0000-template.md`: RFC template
- `docs/rfcs/*.md`: individual RFCs
