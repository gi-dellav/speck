# ARCHITECTURE.md

<!-- Speck default ARCHITECTURE.md — customize for your project -->

## Project Structure

```
.
├── AGENTS.md              # Agent instructions for the entire project
├── ARCHITECTURE.md        # This file — high-level architecture overview
├── Speck.toml             # Project configuration (name, source dir, model, etc.)
├── .speck_hash.toml       # File hash tracking (auto-generated, do not edit manually)
├── .zerostack/
│   └── prompts/           # Speck's system prompts for agentic operations
├── src/                   # Source code directory (configurable via Speck.toml)
│   └── ...
└── specs/
    ├── TECH_STACK.md      # Core tech stack definition (language, framework, dependencies)
    ├── features/          # High-level feature specifications (WHAT the product does)
    │   └── *.md
    └── technical/         # Low-level technical specifications (HOW it works, 1:1 with src/)
        └── *.md
```

## Layered Specification System

Speck uses a three-layer architecture:

| Layer | Location | Content | Audience |
|-------|----------|---------|----------|
| **Features** | `specs/features/` | User-facing capabilities, acceptance criteria | PMs, stakeholders |
| **Technical** | `specs/technical/` | Implementation details, data structures, design rationale (1:1 with src/) | Developers |
| **Code** | `src/` | Actual source code | Compiler, runtime |

### Data Flow

```
specs/features/ ──(feat2tech)──→ specs/technical/ ──(tech2code)──→ src/
src/ ──(code2tech)──→ specs/technical/ ──(tech2feat)──→ specs/features/
```

All conversions are managed by `speck apply`, which compares file hashes to determine what changed and propagates updates through the pipeline.

## Tech Stack
See `specs/TECH_STACK.md` for the full tech stack definition, including language, version, framework, and key dependencies.

## Hash Tracking
`.speck_hash.toml` stores SHA-256 hashes of all tracked files across three categories:
- `features_hash` — files in `specs/features/`
- `technical_hash` — files in `specs/technical/`
- `src_hash` — files in the source directory

Speck compares current file hashes against stored hashes to detect changes and determine what needs to be synced. Files matching `.gitignore` patterns and `specs/_*.md` files are excluded from tracking.
