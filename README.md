![banner](https://raw.githubusercontent.com/gi-dellav/speck/main/assets/banner.png)

---

# speck
Spec-driven agentic compiler — keep features, technical specs, and code in sync using AI agents.

*note:* speck is a frontend for [zerostack](https://github.com/gi-dellav/zerostack). It requires a working `zerostack` binary on `PATH`.

## Features

- **Three-layer architecture**: `specs/features/` (what) → `specs/technical/` (how) → `src/` (code). Edit any layer, the others stay in sync.
- **Bidirectional sync**: `speck apply` detects changes via BLAKE3 hashes and propagates them through the layers using specialized AI prompts.
- **Migration from existing projects**: `speck migrate` reverse-engineers `specs/technical/` from any codebase, then derives `specs/features/` from that.
- **Scaffolding**: `speck init` bootstraps a new project with canonical AGENTS.md, ARCHITECTURE.md, and prompt templates.
- **Conflict resolution**: when both code and specs are edited, speck prompts for priority (or use `--prefer-code` / `--prefer-specs`).
- **Code review**: `speck review` runs a comprehensive, structured review via a specialized prompt, outputting a Markdown report.
- **Git hooks**: `speck git-hooks` installs pre-commit, pre-push, post-merge, and post-checkout hooks that run `speck apply`, `speck fmt`, or `speck status` automatically.
- **Tech stack switching**: `speck switch-lang` destructively changes language/framework and rebuilds everything from feature specs.
- **File management**: `speck mv` and `speck rm` move or remove files while keeping the hash database in sync.
- **Change detection**: `speck status` lists edited and unregistered files grouped by layer.

## Installation

### Cargo

```bash
cargo install --path .
```

### From source

```bash
git clone https://github.com/gi-dellav/speck.git
cd speck
cargo build --release
```

## Quick start

```bash
# Ensure zerostack is installed and configured
zerostack --version

# Scaffold a new speck project
speck init

# Or migrate an existing codebase
speck migrate

# Sync edits between specs and code
speck apply

# Check what changed
speck status

# Full code review
speck review --output report.md
```

## Configuration

Speck reads `Speck.toml` from the project root:

| Field | Description |
|-------|-------------|
| `name` | Project name (defaults to directory name) |
| `source_dir` | Source code directory (defaults to `src`) |
| `model` | Optional default model to use |
| `features_dir` | Optional alternate path for high-level specs |
| `fmt_cmd` | Optional formatter command |
| `test_cmd` | Optional test command |

## How it works

Speck maintains a **three-layer architecture** where each layer is a different representation of the same product:

| Layer | Location | Content | Audience |
|-------|----------|---------|----------|
| **Features** | `specs/features/` | High-level, non-technical — user stories, acceptance criteria | PMs, stakeholders |
| **Technical** | `specs/technical/` | Low-level — HOW and WHY, mapped 1:1 onto source files | Developers |
| **Source** | `src/` (configurable) | The actual code | Compiler |

### The `apply` pipeline

`speck apply` runs a 4-step pipeline:

1. **Detect changes** — compare current BLAKE3 hashes against `.speck_hash.toml`
2. **feat2tech** — propagate feature spec edits down to technical specs
3. **tech2code** — propagate technical spec edits down to source code
4. **code2tech** — propagate source code edits up to technical specs

Each step uses a specialized zerostack prompt at the appropriate temperature (0 for deterministic updates, higher for creative spec expansion).

### Conflict resolution

When both a source file and its corresponding technical spec are edited since the last sync, speck cannot know which is authoritative. It prompts interactively:

```
Both spec and code changed for src/auth.rs — which takes priority?
[1] Code (update specs to match)
[2] Specs (update code to match)
```

Use `--prefer-code` or `--prefer-specs` to resolve all conflicts non-interactively.

## Subcommands

| Command | Description |
|---------|-------------|
| `speck init` | Scaffold a new speck project with templates and directory structure |
| `speck migrate` | Reverse-engineer specs from an existing codebase |
| `speck apply` | Sync edits bidirectionally between specs and code |
| `speck status` | List edited and unregistered files grouped by layer |
| `speck review` | Run a comprehensive code review via agent |
| `speck fmt` | Run the configured formatter and refresh hashes |
| `speck chat` | Open a zerostack TUI session, then auto-apply changes |
| `speck force-update` | Reset all hashes to match current file contents |
| `speck reset` | Clear hashes (optionally delete src/ or specs/technical/) |
| `speck switch-lang` | Change tech stack and rebuild everything from features |
| `speck mv` | Move a file and update hash tracking |
| `speck rm` | Remove a file and update hash tracking |
| `speck git-hooks` | Install git hooks (pre-commit, pre-push, post-merge, post-checkout) |

## Prompts

Speck ships with five specialized zerostack prompts in `data/prompts/`:

| Prompt | Purpose |
|--------|---------|
| `speck-code2tech` | Source code → technical specs |
| `speck-tech2code` | Technical specs → source code |
| `speck-feat2tech` | Feature specs → technical specs |
| `speck-tech2feat` | Technical specs → feature specs |
| `speck-review` | Comprehensive code review |

These are installed into your project on `speck init` and can be customized per-project.

## Context files

Speck ships default `AGENTS.md` and `ARCHITECTURE.md` templates that are scaffolded into new projects on `speck init`. These provide shared core knowledge for all agents (including zerostack) working on the codebase.

## License

GPL-3.0-only
