# Catalog Drift Detection & Update Tool

Isolated from shipped runtime code — for maintainer use only.

## Tools

| Script | Purpose |
|--------|---------|
| `catalog_diff.py` | Compare upstream snapshot vs catalog; produce drift report |
| `catalog_update.py` | Import offline wiki HTML; generate refreshed snapshot, catalog candidate, and draft entries |
| `wiki_parser.py` | Stdlib-only HTML parser for the Stellaris Wiki Achievements page |

## Quick Start — Drift Check

Compare the current upstream snapshot against the curated catalog:

```bash
python3 tools/catalog-diff/catalog_diff.py
```

Run after any snapshot update to check what changed.

## Quick Start — HTML Import

```bash
# Default: parse HTML → write candidates under out/ (no mutation)
python3 tools/catalog-diff/catalog_update.py path/to/Achievements.html

# Apply snapshot in-place (updates upstream_snapshot.json)
python3 tools/catalog-diff/catalog_update.py path/to/Achievements.html --apply-snapshot

# Apply catalog source in-place (exact id/API matches only)
python3 tools/catalog-diff/catalog_update.py path/to/Achievements.html --apply-catalog-source

# Full pipeline: import + apply both
python3 tools/catalog-diff/catalog_update.py path/to/Achievements.html \
    --apply-snapshot --apply-catalog-source
```

All commands produce:
```
out/refreshed_snapshot.json       — upstream snapshot candidate
out/patched_catalog.json          — catalog with source fields patched
out/new_achievements_draft.json   — draft entries needing curation
out/drift_report.md               — human-readable diff report
```

## Maintainer Update Workflow

When a new Stellaris version ships:

### 1. Save the Wiki HTML

1. Navigate to https://stellaris.paradoxwikis.com/Achievements
2. Save the full page as HTML (Ctrl+S / Cmd+S, "Webpage, Complete")
3. Place the file at e.g. `tools/catalog-diff/out/Achievements-4.5.html`

### 2. Run the Import Pipeline (no mutation)

```bash
python3 tools/catalog-diff/catalog_update.py tools/catalog-diff/out/Achievements-4.5.html
```

Review the outputs in `tools/catalog-diff/out/`:
- How many entries were parsed?
- How many new achievements were detected?
- Are the field changes reasonable?

### 3. Review and Verify

Check the drift report and new-achievement drafts. Pay attention to:
- **steam_api_name**: estimated values need verification from game files or Steam
- **Group names**: match the wiki section headings exactly
- **Difficulty**: uncategorized (`UC`) entries need wiki review

### 4. Apply the Snapshot (if correct)

```bash
python3 tools/catalog-diff/catalog_update.py tools/catalog-diff/out/Achievements-4.5.html \
    --apply-snapshot
```

This updates `tools/catalog-diff/upstream_snapshot.json` in-place.

### 5. Apply Catalog Source Patches (if confident)

```bash
python3 tools/catalog-diff/catalog_update.py tools/catalog-diff/out/Achievements-4.5.html \
    --apply-catalog-source
```

**Safe patching rules:**
- Only patches on **exact `id` match** or **exact `steam_api_name` match**
- Fuzzy/name-only matches are report-only (no mutation)
- Only patches: `source.name`, `source.description`, `source.requirement`, `source.hint`, `source.group`, `source.version_added`, `source.difficulty`
- Never touches: `id`, `local_key`, `deprecated`, `curation.*`, or catalog-level version metadata
- `steam_api_name` is **never blindly replaced** from fuzzy matches

### 6. Curate New Achievements

New achievements are written as draft JSON files under `out/`. Each draft entry:
- Has `"status": "new"` and `"curation": null`
- Contains source data from the wiki (name, description, requirement, hint, group, version_added, difficulty)
- Needs: steam_api_name verification, full curation block, manual review

### 7. Final Diff

```bash
python3 tools/catalog-diff/catalog_diff.py
```

Should show 0 new / 0 changed if everything was applied.

## Architecture

```
                        ┌──────────────────┐
                        │  Achievements.html│  (maintainer-saved wiki page)
                        └────────┬─────────┘
                                 │
                                 ▼
                    ┌────────────────────────┐
                    │   wiki_parser.py        │  (stdlib HTML parser)
                    │   parse_achievements_   │
                    │   html(html_text)       │
                    └────────┬─────────┘
                             │
                             ▼ parsed entries
                    ┌────────────────────────┐
                    │   catalog_update.py    │  (main pipeline)
                    │                       │
                    │   ┌─────────────────┐  │
                    │   │ Match vs catalog │  │
                    │   │ Build snapshot   │──┼──► out/refreshed_snapshot.json
                    │   │ Patch source     │──┼──► out/patched_catalog.json
                    │   │ Detect new       │──┼──► out/new_achievements_draft.json
                    │   │ Generate report  │──┼──► out/drift_report.md
                    │   └─────────────────┘  │
                    └────────┬─────────┘
                             │
                 ┌───────────┴───────────┐
                 │                       │
                 ▼                       ▼
       upstream_snapshot.json    catalog/latest.json
       (--apply-snapshot)        (--apply-catalog-source)

    ─────────────────────────────────────────────────
    catalog_diff.py  (standalone diff/report)
    upstream_snapshot.json ──┐
                             ├──► drift report
    catalog/latest.json ─────┘
```

## Testing

```bash
python3 tools/catalog-diff/tests/test_wiki_parser.py
```

Run this after any changes to the parser or before importing a new HTML version.

## Current Catalog State

The curated catalog (`catalog/latest.json`) and upstream snapshot
(`tools/catalog-diff/upstream_snapshot.json`) are both at Stellaris **4.4**
with **219 achievements** each.

| Metric | Value |
|--------|-------|
| Catalog version | 1.2.0 |
| Upstream snapshot | 1.0.0 |
| Upstream achievements | 219 |
| Catalog achievements | 219 |
| **New (to curate)** | **0** |
| Missing (catalog orphans) | 0 |
| Coverage | 100% (219/219) |

The 8 **Nomads** DLC achievements were curated in catalog version 1.2.0
(as of 2026-06-21). If the upstream snapshot is ahead again, the drift
report will flag it. |

## Dependencies

- Python 3.8+ (standard library only — no pip dependencies)

## Constraints

- The wiki is behind Cloudflare challenge protection as of 2026-06, so live
  scraping is not viable. The maintainer must save the HTML manually.
- `steam_api_name` for new achievements is estimated until verified from game
  files or Steam API. (The 8 Nomads entries are verified from
  `common/achievements.txt` as of catalog v1.2.0.)
- The import tool never overwrites `curation` blocks or `id` fields.
- Catalog source fields are only patched on exact `id` or `steam_api_name` match.
