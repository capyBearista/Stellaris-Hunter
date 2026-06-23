# Stellaris Achievement Tracker & Playthrough Planner

Help Stellaris players plan achievement-hunting playthroughs by analyzing local
saves, launcher state, and Steam achievement data — all **read-only**.

**Never mutates** game files, Steam state, mods, or launcher databases.

## Features

- **Curated catalog** of all 219 Stellaris achievements with tags, conditions,
  warnings, and planner notes
- **Save parsing** via a Clausewitz tokenizer + recursive descent parser that
  extracts 119 planner dimensions from ZIP-compressed save files
- **Rule engine** that evaluates achievement conditions against parsed run facts
  and classifies each achievement as Possible, Incompatible, Impossible, or Unknown
- **Planner UI** with per-run achievement grouping, planned toggles, and notes
- **Steam sync** (Windows-only) for read-only achievement state and icon caching
- **Catalog sync** from GitHub raw JSON for updates without app updates
- **Fact overrides** so users can correct or supply missing facts
- **HTTP sidecar** binary for driving the same UI from a plain browser

## Repo Layout

```
├── app/                  # Tauri v2 desktop shell (Rust backend + React/Vite frontend)
│   └── src-tauri/        #   Rust crate: discovery, parsing, planner, catalog
├── catalog/              # Curated achievement catalog & schema
├── tools/
│   ├── validate.sh       #   One-command validation suite (from repo root)
│   └── catalog-diff/     #   Catalog drift detection & wiki import tools
└── .github/workflows/    # CI pipeline
```

## Build

### Prerequisites

- Rust stable toolchain
- Node.js 22+ (Vite 8 requirement)
- Python 3 (for catalog drift tools)

### Backend (Rust)

```bash
cargo build --manifest-path app/src-tauri/Cargo.toml
cargo test --manifest-path app/src-tauri/Cargo.toml
```

### Frontend (React + Vite)

```bash
npm --prefix app install
npm --prefix app run typecheck
npm --prefix app run test
npm --prefix app run build
```

### Desktop Shell (Windows)

```bash
npm --prefix app run tauri:dev    # development
npm --prefix app run tauri:build  # production build
```

### HTTP Sidecar

```bash
cargo build --bin stellaris-hunter-serve --manifest-path app/src-tauri/Cargo.toml
```

## Validation

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

CI runs the full suite on every push and PR via `.github/workflows/ci.yml`.

## Core Constraints

- **Read-only**: never writes to Stellaris/Steam/Documents files
- **Steam API quarantine**: mutating Steam APIs (`SetAchievement`, etc.)
  are forbidden everywhere except the allowlisted guard module
- **Graceful degradation**: per-run parse failures don't abort full scans
- **Synthetic fixtures only**: tests use in-repo fixtures, never live saves

## Further Reading

| Document | What it covers |
|---|---|
| [`catalog/README.md`](catalog/README.md) | Catalog format, import semantics, maintenance |
| [`tools/catalog-diff/README.md`](tools/catalog-diff/README.md) | Catalog drift detection & update workflow |

## Attribution

This is an unofficial fan-made achievement tracker and playthrough planner for
Stellaris.

Stellaris is developed by Paradox Development Studio and published by Paradox
Interactive. This project is not affiliated with or endorsed by Paradox
Interactive or Valve.

Achievement strategy data may be derived from the Stellaris Wiki / Paradox Wikis
and is attributed accordingly. Application source code is licensed under MPL-2.0.
Game content, achievement names, descriptions, icons, and wiki-derived content
remain subject to their respective owners and licenses.

## License

MPL-2.0 — see [`LICENSE.txt`](LICENSE.txt)
