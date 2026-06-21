#!/usr/bin/env python3
"""Tests for the catalog update pipeline (matching, output paths)."""

import json
import sys
import tempfile
from pathlib import Path

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from catalog_update import run_update, _match_catalog_entry


FIXTURE_DIR = Path(__file__).resolve().parent / "fixtures"


def test_run_update_produces_outputs():
    """Pipeline with --out should place all 4 output files in that directory."""
    with tempfile.TemporaryDirectory() as tmp:
        tmpdir = Path(tmp)
        html = FIXTURE_DIR / "sample_wiki_Achievements.html"
        results = run_update(
            html_path=html,
            snapshot_path=None,
            catalog_path=None,
            stellaris_version="4.5",
            min_entries=0,
            out_dir=tmpdir,
        )
        assert results is not None, "Pipeline returned None"
        assert results["refreshed_snapshot_path"] is not None
        assert results["patched_catalog_path"] is not None
        assert results["drift_report_path"] is not None
        # new_draft_path may be None if no new achievements found

        # Verify all output files are inside the requested output directory
        always_keys = ["refreshed_snapshot_path", "patched_catalog_path",
                       "drift_report_path"]
        for key in always_keys:
            path = Path(results[key])
            assert path.exists(), f"{key} missing: {path}"
            assert str(path).startswith(str(tmpdir)), (
                f"{key} not in output dir: {path}"
            )

    print("  PASS: pipeline produces all outputs in --out dir")


def test_candidate_catalog_shows_patches():
    """Candidate catalog (non-apply) should show patched source fields
    for entries matched by id OR api name."""
    with tempfile.TemporaryDirectory() as tmp:
        tmpdir = Path(tmp)
        html = FIXTURE_DIR / "sample_wiki_Achievements.html"
        results = run_update(
            html_path=html,
            snapshot_path=None,
            catalog_path=None,
            stellaris_version="4.5",
            min_entries=0,
            out_dir=tmpdir,
        )
        assert results["patched_catalog_path"] is not None
        with open(results["patched_catalog_path"]) as f:
            catalog_out = json.load(f)

        # "Dreadnought" id matches exactly between fixture slug and catalog.
        # Fixture description differs from catalog — should show patched version.
        found = False
        for entry in catalog_out.get("achievements", []):
            if entry["id"] == "dreadnought":
                found = True
                src = entry.get("source", {})
                assert "Automated Dreadnought" in src.get("description", ""), (
                    f"Expected patched 'Automated Dreadnought', got: {src.get('description')}"
                )
                break
        assert found, "dreadnought not found in candidate catalog"

    print("  PASS: candidate catalog shows patched source fields")


def test_match_catalog_entry_exact_id():
    """_match_catalog_entry returns exact for matching id."""
    parsed_by_slug = {"test-ach": {"name": "Test Ach"}}
    parsed_by_api = {}
    parsed_entries = [{"name": "Test Ach"}]
    ce = {"id": "test-ach", "source": {"name": "Test Ach"}}

    matched, mtype = _match_catalog_entry(ce, parsed_by_slug, parsed_by_api, parsed_entries)
    assert matched is not None, "Should match exact id"
    assert mtype == "exact", f"Expected exact, got {mtype}"
    print("  PASS: _match_catalog_entry exact id")


def test_match_catalog_entry_exact_api():
    """_match_catalog_entry returns exact for matching api name."""
    parsed_by_slug = {}
    parsed_by_api = {"achievement_test_ach": {"name": "Test Ach"}}
    parsed_entries = [{"name": "Test Ach"}]
    ce = {"id": "other-slug", "steam_api_name": "achievement_test_ach",
          "source": {"name": "Test Ach"}}

    matched, mtype = _match_catalog_entry(ce, parsed_by_slug, parsed_by_api, parsed_entries)
    assert matched is not None, "Should match exact api"
    assert mtype == "exact", f"Expected exact, got {mtype}"
    print("  PASS: _match_catalog_entry exact api")


def test_match_catalog_entry_fuzzy():
    """_match_catalog_entry returns fuzzy for name similarity ≥0.7."""
    parsed_by_slug = {}
    parsed_by_api = {}
    parsed_entries = [{"name": "Almost the Same Name Extended Version Here"}]
    ce = {"id": "x", "source": {"name": "Almost the Same Name Extended Version"}}

    matched, mtype = _match_catalog_entry(ce, parsed_by_slug, parsed_by_api, parsed_entries)
    assert matched is not None, "Should match fuzzy"
    assert mtype == "fuzzy", f"Expected fuzzy, got {mtype}"
    print("  PASS: _match_catalog_entry fuzzy")


def test_match_catalog_entry_none():
    """_match_catalog_entry returns None for no match."""
    parsed_by_slug = {}
    parsed_by_api = {}
    parsed_entries = [{"name": "Something Completely Different"}]
    ce = {"id": "x", "source": {"name": "No Relation At All"}}

    matched, mtype = _match_catalog_entry(ce, parsed_by_slug, parsed_by_api, parsed_entries)
    assert matched is None, "Should not match"
    assert mtype is None, f"Expected None type, got {mtype}"
    print("  PASS: _match_catalog_entry no match")


if __name__ == "__main__":
    print("test_catalog_update.py")
    test_match_catalog_entry_exact_id()
    test_match_catalog_entry_exact_api()
    test_match_catalog_entry_fuzzy()
    test_match_catalog_entry_none()
    test_candidate_catalog_shows_patches()
    test_run_update_produces_outputs()
    print("\nAll tests passed.")
