#!/usr/bin/env python3
"""
catalog-diff.py — Detect upstream achievement catalog drift.

Compares an upstream achievement snapshot (source of truth) against
catalog/latest.json (curated catalog) and produces a structured review
report for new, missing, or changed achievements.

Usage:
    # Default: compare upstream snapshot vs catalog/latest.json
    python3 tools/catalog-diff/catalog_diff.py

    # Explicit paths
    python3 tools/catalog-diff/catalog_diff.py \
        --snapshot tools/catalog-diff/upstream_snapshot.json \
        --catalog catalog/latest.json

    # Output to file instead of stdout
    python3 tools/catalog-diff/catalog_diff.py --output drift-report.md

    # JSON output for machine consumption
    python3 tools/catalog-diff/catalog_diff.py --format json

Exit code:
    0 — no drift detected (upstream fully covered in catalog)
    1 — drift detected or errors
"""

import argparse
import json
import sys
from pathlib import Path
from difflib import SequenceMatcher


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def slugify(name: str) -> str:
    """Derive a catalog-style id slug from an achievement name."""
    s = name.lower().strip()
    # Remove punctuation except hyphens (keep apostrophes for now)
    s = s.replace("'", "")
    s = s.replace("!", "")
    s = s.replace(".", "")
    s = s.replace(",", "")
    s = s.replace(":", "")
    s = s.replace("(", "").replace(")", "")
    s = s.replace("༼", "").replace("つ", "").replace("◕", "")
    s = s.replace("_", "").replace("༽", "")
    # Trim internal spaces to hyphens
    s = "-".join(s.split())
    # Collapse multiple hyphens
    while "--" in s:
        s = s.replace("--", "-")
    return s.strip("-")


def similarity(a: str, b: str) -> float:
    """Return a 0–1 similarity ratio between two strings."""
    return SequenceMatcher(None, a.lower(), b.lower()).ratio()


def match_achievements(upstream_list, catalog_list):
    """Match upstream entries to catalog entries by id, steam_api_name, and name similarity.

    Returns (by_id, by_api_name, by_name_fuzzy):
      by_id: dict of upstream_id -> catalog_entry (exact id match)
      by_api_name: dict of upstream_id -> catalog_entry (steam_api_name match)
      by_name_fuzzy: dict of upstream_id -> (catalog_entry, score) (best fuzzy name match)
    """
    catalog_by_id = {e["id"]: e for e in catalog_list}
    catalog_by_api = {}
    catalog_by_name_lower = {}
    for e in catalog_list:
        api = e.get("steam_api_name")
        if api:
            catalog_by_api[api] = e
        catalog_by_name_lower[e["source"]["name"].lower()] = e

    by_id = {}
    by_api_name = {}
    by_name_fuzzy = {}

    for ue in upstream_list:
        uid = ue["id"]
        # 1. Exact id match
        if uid in catalog_by_id:
            by_id[uid] = catalog_by_id[uid]
            continue
        # 2. steam_api_name match
        uapi = ue.get("steam_api_name")
        if uapi and uapi in catalog_by_api:
            by_api_name[uid] = catalog_by_api[uapi]
            continue
        # 3. Name-based fuzzy match
        uname = ue["name"].lower()
        if uname in catalog_by_name_lower:
            by_name_fuzzy[uid] = (catalog_by_name_lower[uname], 1.0)
            continue
        best_score = 0.0
        best_match = None
        for ce in catalog_list:
            score = similarity(uname, ce["source"]["name"].lower())
            if score > best_score:
                best_score = score
                best_match = ce
        if best_score >= 0.7:
            by_name_fuzzy[uid] = (best_match, best_score)

    return by_id, by_api_name, by_name_fuzzy


def detect_field_changes(upstream_entry, catalog_entry, fields):
    """Compare specified fields between upstream source data and catalog source data."""
    changes = []
    cat_src = catalog_entry.get("source", {})
    for field in fields:
        uv = upstream_entry.get(field)
        cv = cat_src.get(field) if field != "id" else catalog_entry.get("id")
        # Normalize for comparison
        if isinstance(uv, str):
            uv = uv.strip()
        if isinstance(cv, str):
            cv = cv.strip()
        if (uv is None and cv is not None) or (cv is None and uv is not None) or (uv != cv):
            changes.append({
                "field": field,
                "upstream": upstream_entry.get(field),
                "catalog": cat_src.get(field) if field != "id" else catalog_entry.get("id"),
            })
    return changes


# ---------------------------------------------------------------------------
# Diff logic
# ---------------------------------------------------------------------------

def compute_diff(upstream, catalog, strict_id=True):
    """Compare upstream snapshot against curated catalog.

    Returns a dict with sections: new, missing, changed, coverage.
    """
    upstream_entries = upstream["achievements"]
    catalog_entries = catalog["achievements"]

    # Match
    by_id, by_api_name, by_name_fuzzy = match_achievements(upstream_entries, catalog_entries)

    known_ids = set(by_id.keys()) | set(by_api_name.keys()) | set(by_name_fuzzy.keys())
    catalog_ids = {e["id"] for e in catalog_entries}

    # New: upstream entries not found in catalog
    new = []
    for ue in upstream_entries:
        uid = ue["id"]
        if uid in known_ids:
            continue
        new.append({
            "id": uid,
            "name": ue["name"],
            "group": ue.get("group", ""),
            "difficulty": ue.get("difficulty", "UC"),
            "version_added": ue.get("version_added", ""),
            "steam_api_name": ue.get("steam_api_name"),
            "description": ue.get("description", ""),
            "requirement": ue.get("requirement", ""),
            "hint": ue.get("hint", ""),
        })

    # Missing: catalog entries no longer in upstream (possibly deprecated)
    upstream_ids = {ue["id"] for ue in upstream_entries}
    upstream_by_api = {}
    for ue in upstream_entries:
        api = ue.get("steam_api_name")
        if api:
            upstream_by_api[api] = ue

    missing = []
    for ce in catalog_entries:
        cid = ce["id"]
        cid_found = cid in upstream_ids
        api = ce.get("steam_api_name")
        api_found = api and api in upstream_by_api
        if not cid_found and not api_found:
            # Also check fuzzy
            uname = ce["source"]["name"].lower()
            fuzzy_found = any(
                similarity(uname, ue["name"].lower()) >= 0.7
                for ue in upstream_entries
            )
            if not fuzzy_found:
                missing.append({
                    "id": cid,
                    "name": ce["source"]["name"],
                    "group": ce["source"].get("group", ""),
                    "steam_api_name": api,
                    "deprecated": ce.get("deprecated", False),
                })

    # Changed: upstream entries that matched but have different field values
    compare_fields = ["name", "description", "requirement", "group", "difficulty", "version_added"]
    changed = []

    for ue in upstream_entries:
        uid = ue["id"]
        ce = by_id.get(uid) or by_api_name.get(uid)
        if ce is None and uid in by_name_fuzzy:
            ce = by_name_fuzzy[uid][0]
        if ce is None:
            continue
        changes = detect_field_changes(ue, ce, compare_fields)
        if changes:
            changed.append({
                "id": uid,
                "name": ue["name"],
                "changes": changes,
            })

    # Coverage summary
    matched_count = len(by_id) + len(by_api_name)
    fuzzy_count = max(0, len(by_name_fuzzy))
    coverage = {
        "upstream_total": len(upstream_entries),
        "catalog_total": len(catalog_entries),
        "exact_id_matches": len(by_id),
        "api_name_matches": len(by_api_name),
        "fuzzy_name_matches": fuzzy_count,
        "new_achievements": len(new),
        "missing_achievements": len(missing),
        "changed_achievements": len(changed),
        "covered": matched_count + fuzzy_count,
    }

    return {
        "coverage": coverage,
        "new": new,
        "missing": missing,
        "changed": changed,
    }


# ---------------------------------------------------------------------------
# Output formatters
# ---------------------------------------------------------------------------

def format_markdown(diff, snapshot_meta, catalog_meta):
    """Render the diff report as human-readable Markdown."""
    c = diff["coverage"]
    lines = []
    lines.append("# Catalog Drift Report\n")
    lines.append(f"- **Upstream**: {snapshot_meta.get('stellaris_version', '?')} "
                 f"(snapshot {snapshot_meta.get('snapshot_version', '?')}, "
                 f"{c['upstream_total']} achievements)")
    lines.append(f"- **Catalog**:  v{catalog_meta.get('catalog_version', '?')} "
                 f"({catalog_meta.get('stellaris_version', '?')}, "
                 f"{c['catalog_total']} achievements)")
    lines.append(f"- **Generated**: {diff.get('generated_at', 'N/A')}")
    lines.append(f"- **Source**: {snapshot_meta.get('source', 'N/A')}")
    src_notes = snapshot_meta.get("source_notes")
    if src_notes:
        lines.append(f"- **Source Notes**: {src_notes}")
    lines.append("")

    # Coverage bar
    covered_pct = (c["covered"] / c["upstream_total"] * 100) if c["upstream_total"] > 0 else 0
    lines.append(f"## Coverage: {c['covered']}/{c['upstream_total']} ({covered_pct:.1f}%)\n")
    lines.append(f"| Metric | Count |")
    lines.append(f"|--------|------:|")
    lines.append(f"| Exact ID matches | {c['exact_id_matches']} |")
    lines.append(f"| Steam API name matches | {c['api_name_matches']} |")
    lines.append(f"| Fuzzy name matches | {c['fuzzy_name_matches']} |")
    lines.append(f"| **Total covered** | **{c['covered']}** |")
    lines.append(f"| New (upstream only) | {c['new_achievements']} |")
    lines.append(f"| Missing (catalog only) | {c['missing_achievements']} |")
    lines.append(f"| Changed (field diffs) | {c['changed_achievements']} |")
    lines.append("")

    # New achievements
    if diff["new"]:
        lines.append(f"## 🆕 New Achievements ({len(diff['new'])})\n")
        lines.append("| ID | Name | API Name | Group | Difficulty | Version |")
        lines.append("|----|------|----------|-------|------------|---------|")
        for a in sorted(diff["new"], key=lambda x: x["id"]):
            api = a.get("steam_api_name") or "—"
            lines.append(f"| {a['id']} | {a['name']} | `{api}` | "
                         f"{a['group']} | {a['difficulty']} | {a['version_added']} |")
        lines.append("")
        lines.append("### Details\n")
        for a in sorted(diff["new"], key=lambda x: x["id"]):
            lines.append(f"#### {a['name']} (`{a['id']}`)\n")
            lines.append(f"- **Group**: {a['group']}")
            lines.append(f"- **Steam API Name**: `{a.get('steam_api_name', '?')}`")
            lines.append(f"- **Difficulty**: {a['difficulty']}")
            lines.append(f"- **Added**: {a['version_added']}")
            lines.append(f"- **Description**: {a.get('description', '')}")
            lines.append(f"- **Requirement**: {a.get('requirement', '')}")
            if a.get("hint"):
                lines.append(f"- **Hint**: {a['hint']}")
            lines.append("")
    else:
        lines.append("## 🆕 New Achievements\n\n*None detected. All upstream achievements are represented in the catalog.*\n")

    # Missing achievements
    if diff["missing"]:
        lines.append(f"## ❌ Missing from Upstream ({len(diff['missing'])})\n")
        lines.append("These catalog entries were not found in the upstream snapshot. "
                     "They may have been removed from the wiki or renamed.\n")
        lines.append("| ID | Name | Steam API Name | Deprecated |")
        lines.append("|----|------|----------------|------------|")
        for a in sorted(diff["missing"], key=lambda x: x["id"]):
            api = a.get("steam_api_name") or "—"
            dep = "yes" if a.get("deprecated") else "no"
            lines.append(f"| {a['id']} | {a['name']} | `{api}` | {dep} |")
        lines.append("")
    else:
        lines.append("## ❌ Missing from Upstream\n\n*None detected.*\n")

    # Changed achievements
    if diff["changed"]:
        lines.append(f"## 🔄 Changed Fields ({len(diff['changed'])})\n")
        lines.append("| ID | Name | Field | Upstream | Catalog |")
        lines.append("|----|------|-------|----------|---------|")
        for a in sorted(diff["changed"], key=lambda x: x["id"]):
            for ch in a["changes"]:
                uv = str(ch.get("upstream", "") or "")
                cv = str(ch.get("catalog", "") or "")
                if len(uv) > 80:
                    uv = uv[:77] + "..."
                if len(cv) > 80:
                    cv = cv[:77] + "..."
                lines.append(f"| {a['id']} | {a['name']} | {ch['field']} | {uv} | {cv} |")
        lines.append("")
    else:
        lines.append("## 🔄 Changed Fields\n\n*None detected. All matched fields are consistent.*\n")

    # Section for steam_api_name verification
    new_entries_with_estimated_api = [a for a in diff["new"] if a.get("steam_api_name", "").startswith("achievement_")]
    if new_entries_with_estimated_api:
        lines.append("## ⚠️  Estimated Steam API Names\n")
        lines.append("The following `steam_api_name` values are estimated (prefixed `achievement_` from the catalog "
                     "naming convention). They **must be verified** from game files or Steam before the catalog entry "
                     "can be trusted:\n")
        for a in new_entries_with_estimated_api:
            lines.append(f"- `{a['steam_api_name']}` — {a['name']}")
        lines.append("")

    # Overall verdict
    if c["new_achievements"] > 0 or c["changed_achievements"] > 0:
        summary_parts = []
        if c["new_achievements"] > 0:
            summary_parts.append(f"{c['new_achievements']} new achievement(s) to curate")
        if c["changed_achievements"] > 0:
            summary_parts.append(f"{c['changed_achievements']} achievement(s) with changed fields to review")
        if c["missing_achievements"] > 0:
            summary_parts.append(f"{c['missing_achievements']} achievement(s) missing from upstream")
        lines.append(f"## Verdict\n\n**Drift detected**: {'; '.join(summary_parts)}.\n")
    else:
        lines.append("## Verdict\n\n**No drift detected.** The catalog is up to date with the upstream snapshot.\n")

    return "\n".join(lines)


def format_json(diff, snapshot_meta, catalog_meta):
    """Render the diff as structured JSON."""
    output = {
        "generated_at": diff.get("generated_at"),
        "upstream": {
            "snapshot_version": snapshot_meta.get("snapshot_version"),
            "stellaris_version": snapshot_meta.get("stellaris_version"),
            "source": snapshot_meta.get("source"),
        },
        "catalog": {
            "catalog_version": catalog_meta.get("catalog_version"),
            "stellaris_version": catalog_meta.get("stellaris_version"),
        },
        "diff": diff,
    }
    return json.dumps(output, indent=2, ensure_ascii=False)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(
        description="Detect upstream achievement catalog drift",
    )
    parser.add_argument(
        "--snapshot",
        default=None,
        help="Path to upstream snapshot JSON (default: tools/catalog-diff/upstream_snapshot.json)",
    )
    parser.add_argument(
        "--catalog",
        default=None,
        help="Path to curated catalog JSON (default: catalog/latest.json)",
    )
    parser.add_argument(
        "--output", "-o",
        default=None,
        help="Write report to file instead of stdout",
    )
    parser.add_argument(
        "--format", "-f",
        choices=["markdown", "json"],
        default="markdown",
        help="Output format (default: markdown)",
    )
    parser.add_argument(
        "--no-fuzzy",
        action="store_true",
        help="Disable fuzzy name matching (only match by id and steam_api_name)",
    )

    args = parser.parse_args()

    # Resolve paths relative to repo root (two levels up from script dir, or CWD)
    script_dir = Path(__file__).resolve().parent
    repo_root = script_dir.parent.parent  # tools/catalog-diff/../.. = repo root

    snapshot_path = Path(args.snapshot) if args.snapshot else (
        script_dir / "upstream_snapshot.json"
    )
    catalog_path = Path(args.catalog) if args.catalog else (
        repo_root / "catalog" / "latest.json"
    )

    # Load data
    for label, path in [("Snapshot", snapshot_path), ("Catalog", catalog_path)]:
        if not path.exists():
            print(f"Error: {label} file not found: {path}", file=sys.stderr)
            sys.exit(1)

    try:
        with open(snapshot_path, "r") as f:
            upstream = json.load(f)
    except json.JSONDecodeError as e:
        print(f"Error parsing snapshot JSON: {e}", file=sys.stderr)
        sys.exit(1)

    try:
        with open(catalog_path, "r") as f:
            catalog = json.load(f)
    except json.JSONDecodeError as e:
        print(f"Error parsing catalog JSON: {e}", file=sys.stderr)
        sys.exit(1)

    # Compute diff
    diff = compute_diff(upstream, catalog, strict_id=not args.no_fuzzy)
    from datetime import datetime, timezone
    diff["generated_at"] = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    # Metadata for report headers
    snapshot_meta = {
        "snapshot_version": upstream.get("snapshot_version", "?"),
        "stellaris_version": upstream.get("stellaris_version", "?"),
        "source": upstream.get("source", "N/A"),
        "source_notes": upstream.get("source_notes", ""),
    }
    catalog_meta = {
        "catalog_version": catalog.get("catalog_version", "?"),
        "stellaris_version": catalog.get("stellaris_version", "?"),
    }

    # Format
    if args.format == "json":
        report = format_json(diff, snapshot_meta, catalog_meta)
    else:
        report = format_markdown(diff, snapshot_meta, catalog_meta)

    # Output
    if args.output:
        out_path = Path(args.output)
        out_path.write_text(report)
        print(f"Report written to {out_path}")
    else:
        print(report)

    # Exit code
    if diff["coverage"]["new_achievements"] > 0 or diff["coverage"]["changed_achievements"] > 0:
        sys.exit(1)
    sys.exit(0)


if __name__ == "__main__":
    main()
