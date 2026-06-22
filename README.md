# Stellaris Achievement Tracker & Playthrough Planner

Help Stellaris players plan achievement-hunting playthroughs by analyzing local
saves, launcher state, and Steam achievement data — all **read-only**.

**Never mutates** game files, Steam state, mods, or launcher databases.

## Repo Layout

```
├── app/                  # Tauri v2 desktop shell (Rust backend + React/Vite frontend)
│   └── src-tauri/        #   Rust crate: discovery, parsing, planner, catalog
├── catalog/              # Curated achievement catalog & schema
├── tools/
│   ├── validate.sh       #   One-command validation suite (from repo root)
│   └── catalog-diff/     #   Catalog drift detection & wiki import tools
├── docs/                 # Spike documents, research, handoff notes
└── AGENTS.md             # Full development guide & architecture (start here)
```

## Quick Start — Validation

Run all checks from the repo root with one command:

```bash
bash ./tools/validate.sh
```

Or run a subset:

```bash
bash ./tools/validate.sh rust        # backend: fmt, clippy, tests
bash ./tools/validate.sh frontend    # frontend: typecheck, vitest, build
bash ./tools/validate.sh catalog     # catalog drift detection + Python tests
```

## Core Constraints

- **Read-only**: never writes to Stellaris/Steam/Documents files
- **Steam API quarantine**: mutating Steam APIs (`SetAchievement`, etc.)
  are forbidden everywhere except the allowlisted guard module
- **Graceful degradation**: per-run parse failures don't abort full scans
- **Synthetic fixtures only**: tests use in-repo fixtures, never live saves

## Further Reading

| Document | What it covers |
|---|---|
| [`AGENTS.md`](AGENTS.md) | Complete development guide, workflow, architecture |
| [`WINDOWS_WSL_DEV.md`](WINDOWS_WSL_DEV.md) | Windows vs WSL development split |
| [`catalog/README.md`](catalog/README.md) | Catalog format, import semantics, maintenance |
| [`tools/catalog-diff/README.md`](tools/catalog-diff/README.md) | Catalog drift detection & update workflow |
| [`app/src-tauri/AGENTS.md`](app/src-tauri/AGENTS.md) | Rust backend internals, parsing, Steam quarantine |
