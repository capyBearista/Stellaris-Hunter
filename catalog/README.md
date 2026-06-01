# Stellaris Hunter Achievement Catalog

This directory will hold the reviewed achievement catalog distributed with the app.

## Source policy

- Primary human-review source: Stellaris Wiki Achievements page.
- Review requirement: catalog text and curated metadata must be reviewed before becoming `latest.json`.
- Steam API names come from Steam achievement enumeration and are needed only to match Steam unlock state to catalog entries.
- Local `common/achievements.txt` keys may be added as `local_key` when confirmed from a read-only local install scan.

## Files

- `schema.json` — baseline JSON shape for catalog authoring and validation. The Rust importer also enforces invariants that portable JSON Schema cannot fully express, especially unique achievement `id` values after parsing.
- `latest.json` — intentionally not committed yet; add once the full reviewed all-achievement catalog is available.

## Import semantics

Catalog JSON must declare `"snapshot_kind": "full"`. Imports are authoritative full snapshots: achievements omitted from a newer snapshot are marked deprecated in the app-owned SQLite catalog instead of being deleted. Do not use this importer for partial or delta updates unless a future merge mode is explicitly added and tested.

The published schema expects canonical curation keys: tags use lowercase hyphen slugs such as `galactic-community`, and condition key fields use lowercase snake case such as `species_class`. The Rust importer can normalize some flexible input during tests/imports, but committed catalog data should already match the schema.

Importer validation remains authoritative for constraints such as duplicate achievement IDs and trimmed non-empty required text. Treat schema validation as necessary but not sufficient before committing catalog content; run the Rust catalog import tests/checks as the final gate.

## Licensing and attribution

Stellaris Wiki editorial content is CC-BY-SA 3.0 and must be attributed. Game names, descriptions, images, and icons remain subject to Paradox/Valve ownership. Do not redistribute Steam icon assets in this repo; cache them locally at runtime after read-only Steam sync if that path is enabled.

## Steam API role, briefly

Steam matters because it is the source of what the player has already unlocked. The catalog says what each achievement means; Steam sync maps `steam_api_name` values to the user’s unlocked/locked state and unlock timestamps. The app must only read Steam achievement state and must never call Steam achievement/stat mutators.
