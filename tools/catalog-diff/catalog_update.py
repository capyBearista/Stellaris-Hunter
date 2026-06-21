#!/usr/bin/env python3
"""
catalog_update.py — Maintainer workflow for importing wiki HTML and refreshing catalog.

PARSES  a maintainer-saved Stellaris Wiki Achievements HTML file.
OUTPUTS candidate files under tools/catalog-diff/out/ (no mutation by default).
APPLIES in-place changes only with --apply-snapshot or --apply-catalog-source.

Pipeline (default, no mutation):
    python3 tools/catalog-diff/catalog_update.py path/to/Achievements.html

    Produces:
        out/refreshed_snapshot.json        — upstream snapshot candidate
        out/patched_catalog.json           — catalog with source fields patched
        out/new_achievements_draft.json    — draft entries needing curation
        out/drift_report.md                — human-readable diff report

Apply snapshot mutation:
    python3 tools/catalog-diff/catalog_update.py path/to/Achievements.html --apply-snapshot

Apply catalog source mutation (exact matches only):
    python3 tools/catalog-diff/catalog_update.py path/to/Achievements.html --apply-catalog-source

See tools/catalog-diff/README.md for full workflow.
"""

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

# Import our HTML parser
from wiki_parser import parse_achievements_html

# Import helpers from catalog_diff.py (same directory)
from catalog_diff import slugify, similarity, compute_diff, format_markdown, format_json

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

# Fields in catalog "source" that we are allowed to patch on exact match
PATCHABLE_SOURCE_FIELDS = [
    "name",
    "description",
    "requirement",
    "hint",
    "group",
    "version_added",
    "difficulty",
]

# ---------------------------------------------------------------------------
# Repo path helpers (mirrors catalog_diff.py logic)
# ---------------------------------------------------------------------------

def _repo_root():
    """Return repo root assuming we run from tools/catalog-diff/."""
    return Path(__file__).resolve().parent.parent.parent

def _default_snapshot_path():
    return Path(__file__).resolve().parent / "upstream_snapshot.json"

def _default_catalog_path():
    return _repo_root() / "catalog" / "latest.json"

def _out_dir():
    return Path(__file__).resolve().parent / "out"

# ---------------------------------------------------------------------------
# Snapshot generation
# ---------------------------------------------------------------------------

def wiki_entries_to_snapshot(entries, source_url, stellaris_version):
    """Wrap parsed wiki entries into a full upstream snapshot structure."""
    now = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    return {
        "snapshot_version": "0.0.0",  # placeholder — maintainer bumps this
        "stellaris_version": stellaris_version,
        "source": f"Parsed from saved HTML: {source_url}",
        "source_notes": (
            "Auto-generated from offline wiki HTML via catalog_update.py. "
            "steam_api_name values must be verified separately."
        ),
        "updated_at": now,
        "achievements": entries,
    }


def build_snapshot_entry(parsed, existing_snapshot_entry=None):
    """Build a full snapshot entry dict from a parsed wiki entry.

    If existing_snapshot_entry is provided, carries forward steam_api_name.
    Otherwise steam_api_name is None (needs manual verification).
    """
    entry = {
        "id": slugify(parsed["name"]),
        "steam_api_name": None,
        "name": parsed["name"],
        "description": parsed["description"],
        "requirement": parsed["requirement"],
        "hint": parsed["hint"],
        "group": parsed["group"],
        "version_added": parsed["version_added"],
        "difficulty": parsed["difficulty"] if parsed["difficulty"] else "UC",
    }
    if existing_snapshot_entry:
        entry["steam_api_name"] = existing_snapshot_entry.get("steam_api_name")
    return entry


# ---------------------------------------------------------------------------
# Matching helper (shared between apply and candidate paths)
# ---------------------------------------------------------------------------

def _match_catalog_entry(ce, parsed_by_slug, parsed_by_api, parsed_entries):
    """Match a catalog entry to a parsed wiki entry.

    Returns (matched_parsed, match_type) where match_type is
    'exact' (id or api) or 'fuzzy' (name similarity ≥ 0.7) or None.
    """
    cid = ce["id"]
    api = ce.get("steam_api_name")

    # 1. Exact id match
    if cid in parsed_by_slug:
        return parsed_by_slug[cid], "exact"
    # 2. Exact steam_api_name match (only if verified in snapshot)
    if api and api in parsed_by_api:
        return parsed_by_api[api], "exact"
    # 3. Fuzzy name match (report only)
    ce_name = ce.get("source", {}).get("name", "").lower()
    best_score = 0.0
    best_pe = None
    for pe in parsed_entries:
        score = similarity(ce_name, pe["name"].lower())
        if score > best_score:
            best_score = score
            best_pe = pe
    if best_score >= 0.7:
        return best_pe, "fuzzy"
    return None, None


# ---------------------------------------------------------------------------
# Catalog patching logic
# ---------------------------------------------------------------------------

def build_catalog_source_patch(parsed_entry, catalog_entry, match_type):
    """Build a patched catalog source dict.

    Only patches fields in PATCHABLE_SOURCE_FIELDS.
    Only applies on exact id or steam_api_name match (match_type='exact').
    Fuzzy/name-only matches return the original source fields unchanged
    (report-only).
    """
    original_source = dict(catalog_entry.get("source", {}))
    patched_source = dict(original_source)

    if match_type != "exact":
        # fuzzy matches: report-only, return original
        return {
            "patched_source": original_source,
            "fields_changed": [],
            "match_type": match_type,
        }

    fields_changed = []
    for field in PATCHABLE_SOURCE_FIELDS:
        parsed_val = parsed_entry.get(field)
        original_val = original_source.get(field)
        # Normalize for comparison
        if isinstance(parsed_val, str):
            parsed_val = parsed_val.strip()
        if isinstance(original_val, str):
            original_val = original_val.strip()

        if parsed_val != original_val and parsed_val is not None:
            patched_source[field] = parsed_val
            fields_changed.append({
                "field": field,
                "old": original_val,
                "new": parsed_val,
            })

    return {
        "patched_source": patched_source,
        "fields_changed": fields_changed,
        "match_type": match_type,
    }


# ---------------------------------------------------------------------------
# Main update logic
# ---------------------------------------------------------------------------

def run_update(html_path, snapshot_path, catalog_path,
               apply_snapshot=False, apply_catalog_source=False,
               stellaris_version=None, min_entries=50,
               out_dir=None):
    """Execute the full catalog update pipeline.

    Args:
        out_dir: Output directory for candidate files.
                 Default: tools/catalog-diff/out/

    Returns a dict with paths to all outputs and diff statistics.
    Returns None on fatal errors.
    """
    out = Path(out_dir) if out_dir else _out_dir()
    out.mkdir(parents=True, exist_ok=True)

    results = {
        "refreshed_snapshot_path": None,
        "patched_catalog_path": None,
        "new_draft_path": None,
        "drift_report_path": None,
        "statistics": {},
        "warnings": [],
        "errors": [],
    }

    # ---- Step 1: Read inputs ----
    html_text = html_path.read_text(encoding="utf-8")

    try:
        parsed_entries, parse_warnings = parse_achievements_html(html_text, min_entries=min_entries)
    except ValueError as e:
        results["errors"].append(str(e))
        print(f"FATAL: {e}", file=sys.stderr)
        return None

    for w in parse_warnings:
        results["warnings"].append(w)
        print(f"WARNING: {w}", file=sys.stderr)

    if not parsed_entries:
        results["errors"].append("Parser returned 0 entries")
        return None

    # Determine stellaris_version
    if not stellaris_version:
        # Try to guess from a common pattern in the HTML
        # Fallback: ask the maintainer to specify
        stellaris_version = "unknown"

    # ---- Step 2: Load existing snapshot for steam_api_name carry-forward ----
    existing_snapshot = {}
    existing_snapshot_entries = []
    snapshot_path_resolved = Path(snapshot_path) if snapshot_path else _default_snapshot_path()
    if snapshot_path_resolved.exists():
        try:
            with open(snapshot_path_resolved, "r") as f:
                existing_snapshot = json.load(f)
            existing_snapshot_entries = existing_snapshot.get("achievements", [])
        except (json.JSONDecodeError, OSError) as e:
            results["warnings"].append(f"Could not read existing snapshot: {e}")

    # Index existing by id and exact name for carry-forward.
    # NO fuzzy carry-forward for steam_api_name — only exact matches.
    existing_by_id = {}
    existing_by_name = {}
    for e in existing_snapshot_entries:
        existing_by_id[e["id"]] = e
        existing_by_name[e["name"].lower()] = e

    # ---- Step 3: Build refreshed snapshot entries ----
    snapshot_entries = []
    for pe in parsed_entries:
        pid = slugify(pe["name"])
        # Carry forward steam_api_name only on:
        #   1. exact id match (slug = existing snapshot entry id)
        #   2. exact name match (parsed name = existing entry name)
        carry = existing_by_id.get(pid) or existing_by_name.get(pe["name"].lower())

        entry = build_snapshot_entry(pe, existing_snapshot_entry=carry)
        snapshot_entries.append(entry)

    # ---- Step 4: Build refreshed snapshot ----
    snapshot_doc = wiki_entries_to_snapshot(
        snapshot_entries,
        source_url="https://stellaris.paradoxwikis.com/Achievements",
        stellaris_version=stellaris_version,
    )
    # Try to carry forward snapshot_version from existing
    if existing_snapshot.get("snapshot_version"):
        ver_parts = existing_snapshot["snapshot_version"].split(".")
        try:
            ver_parts[-1] = str(int(ver_parts[-1]) + 1)
        except (ValueError, IndexError):
            ver_parts = ["0", "0", "1"]
        snapshot_doc["snapshot_version"] = ".".join(ver_parts)
    else:
        snapshot_doc["snapshot_version"] = "1.0.0"

    # ---- Step 5: Apply snapshot (if --apply-snapshot) ----
    if apply_snapshot:
        target = _default_snapshot_path()
        with open(target, "w") as f:
            json.dump(snapshot_doc, f, indent=2, ensure_ascii=False)
            f.write("\n")
        results["refreshed_snapshot_path"] = str(target)
        results["snapshot_applied"] = True
        print(f"Snapshot applied: {target}")
    else:
        # Write candidate to out/
        candidate_path = out / "refreshed_snapshot.json"
        with open(candidate_path, "w") as f:
            json.dump(snapshot_doc, f, indent=2, ensure_ascii=False)
            f.write("\n")
        results["refreshed_snapshot_path"] = str(candidate_path)
        print(f"Candidate snapshot written: {candidate_path}")

    # ---- Step 6: Load catalog ----
    catalog_path_resolved = Path(catalog_path) if catalog_path else _default_catalog_path()
    catalog_doc = {}
    catalog_entries = []
    try:
        with open(catalog_path_resolved, "r") as f:
            catalog_doc = json.load(f)
        catalog_entries = catalog_doc.get("achievements", [])
    except (json.JSONDecodeError, OSError) as e:
        results["errors"].append(f"Could not read catalog: {e}")
        print(f"ERROR: {e}", file=sys.stderr)
        # Still continue — we can produce a partial output

    # ---- Step 7: Match catalog — compute once for all paths ----
    # Build intermediate result list so both apply and candidate paths reuse it.
    match_results = []
    patch_log = {
        "exact_matches_patched": 0,
        "fuzzy_matches_skipped": 0,
        "unmatched_new_entries": 0,
    }
    new_draft_entries = []

    # Index parsed entries by slug for matching.
    # Also build reverse index from carry-forward steam_api_name
    # (from snapshot entries) to the corresponding parsed entry.
    parsed_by_slug = {}
    parsed_by_api = {}
    for se, pe in zip(snapshot_entries, parsed_entries):
        parsed_by_slug[se["id"]] = pe
        api = se.get("steam_api_name")
        if api:
            parsed_by_api[api] = pe

    for ce in catalog_entries:
        matched_parsed, match_type = _match_catalog_entry(
            ce, parsed_by_slug, parsed_by_api, parsed_entries
        )
        if matched_parsed:
            patch_result = build_catalog_source_patch(matched_parsed, ce, match_type)
        else:
            patch_result = {"fields_changed": [], "match_type": None}

        if match_type == "fuzzy":
            patch_log["fuzzy_matches_skipped"] += 1

        match_results.append({
            "catalog_entry": ce,
            "matched_parsed": matched_parsed,
            "match_type": match_type,
            "patch_result": patch_result,
        })

    # ---- Step 8: Identify new entries (not in catalog) ----
    catalog_slugs = {e["id"] for e in catalog_entries}
    catalog_api_names = {e.get("steam_api_name") for e in catalog_entries if e.get("steam_api_name")}
    catalog_names_lower = {e.get("source", {}).get("name", "").lower() for e in catalog_entries}

    for se in snapshot_entries:
        sid = se["id"]
        if sid in catalog_slugs:
            continue
        # Also check if the name is already in catalog (might have different slug)
        if se["name"].lower() in catalog_names_lower:
            continue
        # Check if steam_api_name already in catalog (new slug, known API name)
        api = se.get("steam_api_name")
        if api and api in catalog_api_names:
            continue
        new_draft_entries.append({
            "status": "new",
            "id": sid,
            "steam_api_name": api,
            "source": {
                "name": se["name"],
                "description": se["description"],
                "requirement": se["requirement"],
                "hint": se["hint"],
                "group": se["group"],
                "version_added": se["version_added"],
                "difficulty": se["difficulty"] if se["difficulty"] else "UC",
            },
            "curation": None,
            "notes": "DRAFT — requires steam_api_name verification and curation review",
        })
        patch_log["unmatched_new_entries"] += 1

    # ---- Step 9: Write patched catalog (apply or candidate) ----
    catalog_out = dict(catalog_doc) if catalog_doc else {}

    if apply_catalog_source:
        patched = []
        for mr in match_results:
            ce = mr["catalog_entry"]
            mtype = mr["match_type"]
            if mtype == "exact" and mr["patch_result"]["fields_changed"]:
                new_ce = dict(ce)
                new_source = dict(ce.get("source", {}))
                for fc in mr["patch_result"]["fields_changed"]:
                    new_source[fc["field"]] = fc["new"]
                new_ce["source"] = new_source
                patched.append(new_ce)
                patch_log["exact_matches_patched"] += 1
            else:
                patched.append(ce)
        catalog_out["achievements"] = patched
        target = _default_catalog_path()
        with open(target, "w") as f:
            json.dump(catalog_out, f, indent=2, ensure_ascii=False)
            f.write("\n")
        results["patched_catalog_path"] = str(target)
        results["catalog_applied"] = True
        print(f"Catalog patched: {target}")
    else:
        # Candidate mode — always show patches in output for review
        candidate = []
        for mr in match_results:
            ce = mr["catalog_entry"]
            fc = mr["patch_result"]["fields_changed"]
            if fc:
                patched_ce = dict(ce)
                new_source = dict(ce.get("source", {}))
                for field_change in fc:
                    new_source[field_change["field"]] = field_change["new"]
                patched_ce["source"] = new_source
                candidate.append(patched_ce)
            else:
                candidate.append(ce)
        catalog_out["achievements"] = candidate
        candidate_path = out / "patched_catalog.json"
        with open(candidate_path, "w") as f:
            json.dump(catalog_out, f, indent=2, ensure_ascii=False)
            f.write("\n")
        results["patched_catalog_path"] = str(candidate_path)
        print(f"Candidate patched catalog: {candidate_path}")

    # ---- Step 10: Write new-achievements draft ----
    if new_draft_entries:
        draft_path = out / "new_achievements_draft.json"
        draft_doc = {
            "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "source": "Parsed from wiki HTML",
            "count": len(new_draft_entries),
            "achievements": new_draft_entries,
        }
        with open(draft_path, "w") as f:
            json.dump(draft_doc, f, indent=2, ensure_ascii=False)
            f.write("\n")
        results["new_draft_path"] = str(draft_path)
        print(f"New achievement drafts: {draft_path}")
    else:
        results["new_draft_path"] = None

    # ---- Step 11: Run the drift report ----
    try:
        diff = compute_diff(snapshot_doc, catalog_out)
        diff["generated_at"] = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

        snapshot_meta = {
            "snapshot_version": snapshot_doc.get("snapshot_version", "?"),
            "stellaris_version": snapshot_doc.get("stellaris_version", "?"),
            "source": snapshot_doc.get("source", "N/A"),
            "source_notes": snapshot_doc.get("source_notes", ""),
        }
        catalog_meta = {
            "catalog_version": catalog_doc.get("catalog_version", "?"),
            "stellaris_version": catalog_doc.get("stellaris_version", "?"),
        }

        md_report = format_markdown(diff, snapshot_meta, catalog_meta)
        json_report = format_json(diff, snapshot_meta, catalog_meta)

        report_path = out / "drift_report.md"
        report_path.write_text(md_report)
        results["drift_report_path"] = str(report_path)
        print(f"Drift report: {report_path}")

        json_report_path = out / "drift_report.json"
        json_report_path.write_text(json_report)

        results["statistics"] = diff["coverage"]
        results["statistics"]["patch_log"] = patch_log
        results["statistics"]["new_draft_count"] = len(new_draft_entries)

    except Exception as e:
        results["errors"].append(f"Drift report generation failed: {e}")
        print(f"WARNING: Drift report failed: {e}", file=sys.stderr)

    return results


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(
        description="Import wiki HTML and refresh catalog data",
    )
    parser.add_argument(
        "html_file",
        type=str,
        help="Path to saved Stellaris Wiki Achievements HTML file",
    )
    parser.add_argument(
        "--apply-snapshot",
        action="store_true",
        help="Mutate tools/catalog-diff/upstream_snapshot.json in-place",
    )
    parser.add_argument(
        "--apply-catalog-source",
        action="store_true",
        help="Mutate catalog/latest.json source fields in-place (exact id/api matches only)",
    )
    parser.add_argument(
        "--stellaris-version",
        type=str,
        default=None,
        help="Override Stellaris version (e.g. '4.4'). Default: from HTML or 'unknown'",
    )
    parser.add_argument(
        "--out",
        type=str,
        default=None,
        help="Output directory (default: tools/catalog-diff/out/)",
    )
    parser.add_argument(
        "--min-entries",
        type=int,
        default=50,
        help="Minimum expected HTML entries (default 50; 0 disables check)",
    )

    args = parser.parse_args()

    html_path = Path(args.html_file)
    if not html_path.exists():
        print(f"Error: HTML file not found: {html_path}", file=sys.stderr)
        sys.exit(1)

    # Run the pipeline
    results = run_update(
        html_path=html_path,
        snapshot_path=None,
        catalog_path=None,
        apply_snapshot=args.apply_snapshot,
        apply_catalog_source=args.apply_catalog_source,
        stellaris_version=args.stellaris_version,
        min_entries=args.min_entries,
        out_dir=args.out,
    )

    if results is None:
        sys.exit(1)

    # Print summary
    c = results.get("statistics", {})
    print()
    print("=" * 60)
    print("UPDATE SUMMARY")
    print("=" * 60)
    print(f"  Parsed wiki entries:       {c.get('upstream_total', '?')}")
    print(f"  Catalog entries before:    {c.get('catalog_total', '?')}")
    print(f"  New achievements detected: {c.get('new_achievements', 0)}")
    print(f"  Changed fields detected:   {c.get('changed_achievements', 0)}")
    print(f"  Missing (catalog orphans): {c.get('missing_achievements', 0)}")
    print()
    print(f"  Snapshot applied:          {results.get('snapshot_applied', False)}")
    print(f"  Catalog source patched:    {results.get('catalog_applied', False)}")
    print()
    p = c.get("patch_log", {})
    if p:
        print(f"  Exact matches patched:     {p.get('exact_matches_patched', 0)}")
        print(f"  Fuzzy matches (report):    {p.get('fuzzy_matches_skipped', 0)}")
        print(f"  New draft entries:         {p.get('unmatched_new_entries', 0)}")
    print()
    print("Output files:")
    for label, key in [
        ("Refreshed snapshot", "refreshed_snapshot_path"),
        ("Patched catalog candidate", "patched_catalog_path"),
        ("New achievement drafts", "new_draft_path"),
        ("Drift report (md)", "drift_report_path"),
    ]:
        path = results.get(key)
        if path:
            print(f"  {label}: {path}")
        else:
            print(f"  {label}: (none)")
    if results.get("warnings"):
        print()
        print("Warnings:")
        for w in results["warnings"]:
            print(f"  - {w}")
    if results.get("errors"):
        print()
        print("Errors:")
        for e in results["errors"]:
            print(f"  - {e}")

    # Exit code
    if results.get("errors"):
        sys.exit(1)
    if c.get("new_achievements", 0) > 0:
        # Non-fatal: drift is expected
        pass
    sys.exit(0)


if __name__ == "__main__":
    main()
