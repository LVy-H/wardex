# Wardex

**Ward & index your workspace** - CTF management, project organization, and more.

## Features

- 🚩 **CTF Management** - Full event lifecycle: init, add, import, shelve, finish
- 📦 **Interactive Shelve** - Guided challenge completion with file triage, flags, and notes
- 🐚 **Shell Completions** - Bash/Zsh tab completion for commands and flags
- 📥 **Inbox Sorting** - Auto-organize files using regex rules
- 🔍 **Flag Search** - Hunt CTF flags in files and archives (respects `.gitignore`)
- 📊 **Git Dashboard** - Status of all repos at a glance (parallelized)
- ↩️ **Undo Support** - Safely revert file moves
- 👁️ **Watch Mode** - Real-time inbox monitoring
- 🛡️ **Workspace Audit** - Find empty folders and file extension mismatches

## Installation

### Nix


```bash
nix run github:LVy-H/wardex
# Or add to your flake inputs
```

### Cargo

```bash
cargo install --path .
```

## Usage

```bash
# Sort inbox items
wardex clean

# Config management
wardex config init        # Initialize config with defaults
wardex config show        # View current settings
wardex config edit        # Edit in $EDITOR
wardex config goto inbox  # Print path (for shell integration)

# Watch inbox in real-time
wardex watch

# CTF event management
wardex ctf init Defcon2025   # Create event (auto-activates)
wardex ctf list
wardex ctf use Defcon2025    # Switch active event context
wardex ctf info              # Show current event context
wardex ctf import file.zip   # Smart import (moves file, auto-detects category)
wardex ctf add web/chall1    # Create challenge
wardex ctf path              # Print path to current event
wardex ctf path --cd         # Output as 'cd <path>' for eval

# Shell navigation shortcuts
eval $(wardex ctf add pwn/heap-overflow --cd)  # Create challenge + cd into it

# Challenge completion (shelve system)
wardex ctf shelve                        # Interactive: status, flag, file triage, notes
wardex ctf shelve "flag{found_it}"       # Quick solve with flag
wardex ctf shelve --auto                 # Smart defaults, no prompts
wardex ctf shelve --no-clean --no-move   # Skip file triage and archival
wardex ctf status                        # Challenge progress table

# Search for flags
wardex search /path/to/ctf

# Workspace health check
wardex status
wardex audit

# Shell completions
wardex completions bash > ~/.local/share/bash-completion/completions/wardex
wardex completions zsh > ~/.zfunc/_wardex

# Undo last moves
wardex undo -c 3
```

### Context Awareness & Persistence

Wardex knows where you are and what you're working on.

**1. Context Detection**:
- Run commands from **any subdirectory** (e.g., inside `web/chall1`).
- Wardex automatically walks up the tree to find the event root.

**2. Global State**:
- Wardex remembers your active event globally.
- Switch contexts with `wardex ctf use <event>`.
- Run commands from anywhere (e.g., `~/Downloads`), and they will apply to the active event.

**3. Shell Integration**:

**Quick setup** — add to `.bashrc` or `.zshrc`:

```bash
# Navigate to CTF paths (uses --cd flag for eval-safe output)
ctf-goto() { eval "$(wardex ctf path --cd "$@" 2>/dev/null)" || echo "Path not found."; }
# Create challenge and cd into it
ctf-work() { eval "$(wardex ctf work "$@" 2>/dev/null)" || echo "Failed to create challenge."; }
```

**Full wrapper** (optional, wraps all `wardex ctf` commands):

```bash
function ctf() {
    if [ "$1" = "goto" ]; then
        shift
        eval "$(wardex ctf path --cd "$@" 2>/dev/null)" || echo "Path not found."
    elif [ "$1" = "work" ]; then
        shift
        eval "$(wardex ctf work "$@" 2>/dev/null)" || echo "Failed."
    else
        wardex ctf "$@"
    fi
}
```

Usage: 
- `ctf goto` (navigates to the active event root)
- `ctf goto MyEvent` (fuzzy matches event 'MyEvent')
- `ctf goto web/chall1` (navigates to web/chall1 in the active event)
- `ctf goto MyEvent chall1` (navigates to chall1 within MyEvent)

## Configuration

Wardex uses a **layered configuration system** with three priority levels (highest to lowest):

1. **Environment Variables** (override everything)
2. **Config Files** (explicit settings)
3. **Defaults** (fallback values)

### Configuration File Locations

Wardex searches for configuration files in this order:

1. `./config.yaml` (current directory)
2. `~/.config/wardex/config.yaml` (XDG config dir)
3. Built-in defaults if neither exists

Create `~/.config/wardex/config.yaml`:

```yaml
paths:
  workspace: ~/workspace
  inbox: ~/workspace/0_Inbox
  projects: ~/workspace/1_Projects
  areas: ~/workspace/2_Areas
  resources: ~/workspace/3_Resources
  archives: ~/workspace/4_Archives

rules:
  clean:
    - pattern: ".*\\.pdf$"
      target: resources/Documents
    - pattern: ".*\\.zip$"
      target: projects

organize:
  ctf_dir: projects/CTFs

ctf:
  default_categories:
    - web
    - pwn
    - crypto
    - rev
    - misc
  shelve:
    blacklist:
      - node_modules
      - .venv
      - __pycache__
    whitelist:
      - solve.*
      - notes.md
      - Dockerfile
```

### Environment Variables

**Override any config path** using `WX_` prefix with uppercase paths:

| Environment Variable | Overrides | Example |
|---------------------|-----------|---------|
| `WX_PATHS_WORKSPACE` | `paths.workspace` | `/tmp/workspace` |
| `WX_PATHS_INBOX` | `paths.inbox` | `~/Downloads` |
| `WX_PATHS_PROJECTS` | `paths.projects` | `~/dev` |
| `WX_PATHS_ARCHIVES` | `paths.archives` | `~/archives` |
| `WX_ORGANIZE_CTF_DIR` | `organize.ctf_dir` | `~/ctfs` |

**Examples:**

```bash
# Temporarily use different workspace
WX_PATHS_WORKSPACE=/tmp/test wardex status

# Override inbox location
WX_PATHS_INBOX=~/Downloads wardex clean

# Use custom CTF directory
WX_ORGANIZE_CTF_DIR=~/sec/ctfs wardex ctf list
```

### Default Paths

If no configuration is provided, Wardex uses these defaults:

- **workspace**: `~/workspace`
- **inbox**: `{workspace}/0_Inbox`
- **projects**: `{workspace}/1_Projects`
- **areas**: `{workspace}/2_Areas`
- **resources**: `{workspace}/3_Resources`
- **archives**: `{workspace}/4_Archives`
- **ctf_root**: `{projects}/CTFs`

## Documentation

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — code layout and data flows
- [`docs/CLI_DESIGN.md`](docs/CLI_DESIGN.md) — command and shell UX rules
- [`docs/ctf-lifecycle.md`](docs/ctf-lifecycle.md) — CTF workflow and shelve system design
- [`docs/shell-output-contracts.md`](docs/shell-output-contracts.md) — shell integration output specs
- [`docs/PREVIEW.md`](docs/PREVIEW.md) — product direction and future features
- [`docs/rfcs/`](docs/rfcs/README.md) — RFC process and accepted proposals
- [`docs/plan/`](docs/plan/README.md) — staged development plan and task backlog

## License

MIT
