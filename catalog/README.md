# Stellaris Hunter Achievement Catalog

This directory holds the reviewed achievement catalog distributed with the app.

## Maintainer Update Tools

When a new Stellaris version ships, upstream achievements may change before the
catalog is updated. Use the tools in `tools/catalog-diff/` to import, compare,
and review changes.

### Quick diff (no HTML needed)

```bash
python3 tools/catalog-diff/catalog_diff.py
```

### Full HTML import pipeline

Save the [wiki Achievements page](https://stellaris.paradoxwikis.com/Achievements)
as offline HTML, then:

```bash
python3 tools/catalog-diff/catalog_update.py path/to/Achievements.html
```

This produces candidate files in `tools/catalog-diff/out/` without mutating
any checked-in files. Add `--apply-snapshot` and/or `--apply-catalog-source`
to apply changes in-place (exact-match patches only — safe rules enforced).

See `tools/catalog-diff/README.md` for full maintainer workflow and update
procedure.

## Source policy

- Primary human-review source: Stellaris Wiki Achievements page.
- Review requirement: catalog text and curated metadata must be reviewed before becoming `latest.json`.
- Steam API names come from Steam achievement enumeration and are needed only to match Steam unlock state to catalog entries.
- Local `common/achievements.txt` keys may be added as `local_key` when confirmed from a read-only local install scan.

## Files

- `schema.json` — baseline JSON shape for catalog authoring and validation. The Rust importer also enforces invariants that portable JSON Schema cannot fully express, especially unique achievement `id` values after parsing.
- `latest.json` — the current reviewed full catalog snapshot shipped with the app and imported into the app-owned SQLite database.

## Import semantics

Catalog JSON must declare `"snapshot_kind": "full"`. Imports are authoritative full snapshots: achievements omitted from a newer snapshot are marked deprecated in the app-owned SQLite catalog instead of being deleted. Do not use this importer for partial or delta updates unless a future merge mode is explicitly added and tested.

The published schema expects canonical curation keys: tags use lowercase hyphen slugs such as `galactic-community`, and condition key fields use lowercase snake case such as `species_class`. The Rust importer can normalize some flexible input during tests/imports, but committed catalog data should already match the schema.

Importer validation remains authoritative for constraints such as duplicate achievement IDs and trimmed non-empty required text. Treat schema validation as necessary but not sufficient before committing catalog content; run the Rust catalog import tests/checks as the final gate.

## Maintenance notes

- The canonical `ascension_path` value for the Machine Age nanotech path is `nanotech`, not `nanite`.

## Licensing and attribution

Stellaris Wiki editorial content is CC-BY-SA 3.0 and must be attributed. Game names, descriptions, images, and icons remain subject to Paradox/Valve ownership. Do not redistribute Steam icon assets in this repo; cache them locally at runtime after read-only Steam sync if that path is enabled.

## Steam API role, briefly

Steam matters because it is the source of what the player has already unlocked. The catalog says what each achievement means; Steam sync maps `steam_api_name` values to the user’s unlocked/locked state and unlock timestamps. The app must only read Steam achievement state and must never call Steam achievement/stat mutators.
