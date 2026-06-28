# speck

A fully spec-based AI agentic compiler, built on top of [zerostack](https://github.com/gi-dellav/zerostack).

Speck treats Markdown specifications as the single source of truth for a codebase. Instead of writing specs by hand and code separately, you edit one side and let Speck keep the other in sync through an agent. It maintains two layers of specs (high-level features and low-level technical docs) that map 1:1 onto your source code, and uses file hashes to detect what changed so it only regenerates what is needed.

## Requirements

- A working `zerostack` binary on your `PATH` (Speck drives all agentic operations through it).
- A configured model/provider for zerostack.

## Installation

```bash
cargo install --path .
```

## Quick start

**Existing project** — generate specs from your current code:

```bash
cd your-project
speck migrate
```

`migrate` reverse-engineers `specs/technical/` from your source, derives `specs/features/` from that, and writes the project configuration. You review the generated feature specs, then run `speck apply` to confirm specs and code are in sync.

**New project** — start from an empty Speck scaffold:

```bash
speck init
```

## How it works

Speck tracks three layers and keeps them consistent:

- `specs/features/` — high-level specifications: what the product does, from a user's perspective.
- `specs/technical/` — low-level specifications: how each source file works and why, mapped 1:1 onto the source tree.
- your source directory — the actual code.

When you edit any layer and run `speck apply`, Speck detects the change via stored hashes and propagates it:

- Edit **code** → `specs/technical/` is updated to match.
- Edit a **feature spec** → `specs/technical/`, then the **code**, are updated to implement it.

So adding a feature means describing it in `specs/features/` in plain language; the technical detail and implementation are filled in for you.

## Project structure

A Speck project contains:

| Path | Purpose |
| --- | --- |
| `Speck.toml` | Project configuration. |
| `.speck_hash.toml` | Stored file hashes used to detect edits. |
| `AGENTS.md` | Project-wide instructions for the agent. |
| `ARCHITECTURE.md` | High-level architecture overview. |
| `specs/TECH_STACK.md` | Language, stack, and core dependencies. |
| `specs/features/` | High-level feature specifications. |
| `specs/technical/` | Low-level technical specifications (1:1 with source). |
| `.zerostack/prompts/` | The specialized prompts Speck uses for each transform. |

Files named `specs/_*.md` and anything matched by `.gitignore` are ignored by the hash system.

## Commands

| Command | Description |
| --- | --- |
| `speck init` | Create a new Speck project. Pass `--name` and `--source-path` to skip the prompts, `--skip-git` to skip git setup. |
| `speck migrate` | Migrate an existing project to Speck by generating specs from source. Use `--custom <msg>` to append an instruction to the agent. |
| `speck status` | List edited and unregistered files, grouped into features, technicals, and code. |
| `speck apply` | Propagate edits between specs and code. See options below. |
| `speck review` | Run a full code review. Use `--output <file>` to save the report as Markdown instead of printing it. |
| `speck chat` | Launch the zerostack TUI, then run `speck apply` afterwards. |
| `speck fmt` | Run the configured formatter and refresh source hashes. |
| `speck force-update` | Reset all stored hashes to the current files' hashes. |
| `speck reset` | Remove stored hashes. `--full/-f` also removes `specs/technical/`, `--hard/-h` also removes the source directory, `--rebuild/-r` runs `speck apply` afterwards. |
| `speck switch-lang` | Change the project's tech stack and rebuild. `--safe` keeps `specs/technical/`. |
| `speck mv <source> <dest>` | Move a file and update `.speck_hash.toml`. |
| `speck rm <path>` | Remove a file and update `.speck_hash.toml`. |
| `speck git-hooks` | Set up git hooks for the project. |

### `speck apply` options

| Flag | Description |
| --- | --- |
| `--custom <msg>` | Append a custom instruction to the agent. |
| `--only-direct`, `-d` | Run only the specs-to-code pipeline. |
| `--only-inverse`, `-i` | Run only the code-to-specs pipeline. |
| `--update-features` | Also update `specs/features/` from `specs/technical/`. |
| `--prefer-code`, `-C` | Resolve spec/code conflicts in favor of code. |
| `--prefer-specs`, `-S` | Resolve spec/code conflicts in favor of specs. |
| `--gen-temperature <val>` | Temperature used for code generation. |

If there is nothing to update, `apply` and `status` simply report that there is nothing to do.

## Configuration

`Speck.toml` fields:

| Field | Description |
| --- | --- |
| `name` | Project name (defaults to the directory name). |
| `source_dir` | Source code directory (defaults to `src`). |
| `model` | Optional. Default model to use. |
| `features_dir` | Optional. Alternate path for high-level specs, for sharing multiple codebases in one product. |
| `fmt_cmd` | Optional. Command used by `speck fmt`. |
| `test_cmd` | Optional. Command used to run unit tests. |

## License

GPL-3.0-only.
