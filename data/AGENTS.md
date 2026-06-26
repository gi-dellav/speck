# AGENTS.md

<!-- Speck default AGENTS.md — customize for your project -->

## Project Overview
_Describe what this project does in 2-3 sentences._

## Core Rules
- `specs/features/` is the **source of truth** for what the product does.
- `specs/technical/` is the **source of truth** for how it is implemented.
- `specs/technical/` mirrors the source code 1:1 (same file names, same structure).
- `specs/TECH_STACK.md` defines the language, framework, and key dependencies.
- `Speck.toml` holds project configuration; `.speck_hash.toml` tracks file hashes.
- Files starting with `_` in `specs/` are ignored by Speck's hash tracking.

## Workflow
1. **Before coding** — read the relevant `specs/features/` and `specs/technical/` files.
2. **When coding** — implement exactly what `specs/technical/` prescribes.
3. **After coding** — update `specs/technical/` to reflect what was built and why.
4. **When adding features** — write or update `specs/features/` first, then run `speck apply`.

## Writing specs/technical/
- One file per source file, same relative path.
- Document every public function: signature, parameters, return value, behavior.
- Describe key data structures and their fields.
- Capture edge cases and error handling.
- Explain **WHY** each technical decision was made.
- Keep inline code snippets to **5 lines maximum** each.

## Writing specs/features/
- Describe what the feature does from the user's perspective.
- Avoid implementation details, code, or technical jargon.
- Focus on user stories, acceptance criteria, and expected behavior.

## Code Style
- Keep changes minimal and focused on one concern.
- Follow existing code patterns and idioms of the tech stack.
- Do not introduce new dependencies without updating `specs/TECH_STACK.md`.
- Never commit secrets, keys, or sensitive configuration.

## Testing
- Write tests for all non-trivial changes.
- Run the test suite before considering work complete (`speck test` or the command in Speck.toml).
- Tests should cover edge cases described in the specs.

## Commands Reference
- `speck init` — create a new Speck project.
- `speck migrate` — migrate an existing codebase to Speck.
- `speck apply` — sync specs and code bidirectionally.
- `speck review` — run a full project review.
- `speck status` — show which files have unapplied changes.
- `speck fmt` — run the project formatter.
- `speck chat` — open an agentic chat session.
- `speck mv` / `speck rm` — move or remove tracked files.
- `speck reset` — clear hash tracking (optionally rebuild).
