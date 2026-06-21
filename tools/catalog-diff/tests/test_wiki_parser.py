#!/usr/bin/env python3
"""Tests for the Stellaris Wiki HTML parser and safety rules."""

import json
import sys
from pathlib import Path

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from wiki_parser import parse_achievements_html, strip_html, strip_icons


FIXTURE_DIR = Path(__file__).resolve().parent / "fixtures"


def test_parse_fixture():
    """Parse the sample fixture and verify all entries."""
    html = (FIXTURE_DIR / "sample_wiki_Achievements.html").read_text(encoding="utf-8")
    entries, warnings = parse_achievements_html(html, min_entries=0)

    assert len(entries) == 7, f"Expected 7 entries, got {len(entries)}"

    # Check first entry thoroughly
    e = entries[0]
    assert e["name"] == "Archaeologist"
    assert e["description"] == "Successfully investigate an archaeological site."
    assert e["group"] == "Base game"
    assert e["version_added"] == "2.3"
    assert e["difficulty"] == "VE"
    assert e["requirement"] == "Complete all levels of an archaeological site"
    assert "Galactic Doorstep" in e["hint"]

    # Check section grouping
    for i, expected_group in enumerate([
        "Base game", "Base game", "Base game",
        "Leviathans", "Leviathans",
        "Nomads", "Nomads",
    ]):
        assert entries[i]["group"] == expected_group, (
            f"Entry {i} ({entries[i]['name']}): expected group '{expected_group}', "
            f"got '{entries[i]['group']}'"
        )

    # Check known entries
    names = [e["name"] for e in entries]
    assert "Brave New World" in names
    assert "Break On Through..." in names
    assert "Dreadnought" in names
    assert "Infinite Creation" in names
    assert "Khan of Khans" in names
    assert "Forever is a Long Time" in names

    # Check version/difficulty parsing
    assert entries[0]["version_added"] == "2.3"
    assert entries[3]["difficulty"] == "E"
    assert entries[4]["difficulty"] == "M"
    assert entries[5]["version_added"] == "4.4"
    assert entries[5]["difficulty"] == "UC"

    # Check no extra text leaked into name field
    for e in entries:
        assert "\n" not in e["name"], f"Name contains newline: {repr(e['name'])}"
        assert e["description"], f"Empty description for {e['name']}"

    print(f"  PASS: {len(entries)} entries, {len(warnings)} warnings")
    return entries


def test_decoy_table_rejected():
    """Unrelated wikitables with different headers must NOT be parsed."""
    html = (FIXTURE_DIR / "sample_wiki_Achievements.html").read_text(encoding="utf-8")
    entries, warnings = parse_achievements_html(html, min_entries=0)

    # The decoy table has headers: Name, Type, Description — should be rejected.
    # Only the 3 achievement tables should be parsed.
    assert len(entries) == 7, (
        f"Decoy table not rejected: expected 7, got {len(entries)}. "
        f"Names: {[e['name'] for e in entries]}"
    )

    # Verify no decoy content leaked in
    names = [e["name"] for e in entries]
    assert "Dummy" not in names, "Decoy table row was incorrectly parsed"

    # Verify we got a warning about the skipped table
    table_warnings = [w for w in warnings if "Skipping wikitable" in w]
    assert len(table_warnings) >= 1, (
        f"Expected warning about skipped table, got warnings: {warnings}"
    )
    print(f"  PASS: decoy table rejected, {len(table_warnings)} warning(s)")


def test_bad_headers_rejected():
    """A wikitable with wrong headers must produce 0 entries."""
    html = """<html><body>
    <h2>Bad table</h2>
    <table class="wikitable sortable">
    <thead><tr><th>Wrong</th><th>Headers</th><th>Here</th></tr></thead>
    <tbody><tr><td>X</td><td>Y</td><td>Z</td></tr></tbody>
    </table>
    </body></html>"""
    entries, warnings = parse_achievements_html(html, min_entries=0)
    assert len(entries) == 0, f"Expected 0 entries for bad headers, got {len(entries)}"
    assert any("Skipping wikitable" in w for w in warnings), (
        f"No skip warning for bad headers: {warnings}"
    )
    print("  PASS: bad headers produce 0 entries with warning")


def test_no_achievement_table_rejected():
    """No achievement table at all should still pass with 0 entries."""
    html = "<html><body><p>No tables here</p></body></html>"
    entries, warnings = parse_achievements_html(html, min_entries=0)
    assert len(entries) == 0, f"Expected 0 entries, got {len(entries)}"
    print("  PASS: no achievement table → 0 entries")


def test_duplicate_id_hard_failure():
    """Two entries that slugify to the same ID must raise ValueError."""
    html = """<html><body>
    <h2>Test</h2>
    <table class="wikitable sortable">
    <thead><tr><th>Achievement</th><th>Requirements</th><th>Hints</th><th>Ver</th><th>DI</th></tr></thead>
    <tbody>
    <tr>
    <td><div class="iconbox"><div class="iconbox-title">Alpha</div><div class="iconbox-text">First one.</div></div></td>
    <td>Do X</td><td>Hint X</td><td>1.0</td><td>VE</td>
    </tr>
    <tr>
    <td><div class="iconbox"><div class="iconbox-title">Alpha</div><div class="iconbox-text">Second one.</div></div></td>
    <td>Do Y</td><td>Hint Y</td><td>1.0</td><td>VE</td>
    </tr>
    </tbody>
    </table>
    </body></html>"""
    try:
        parse_achievements_html(html, min_entries=0)
        assert False, "Expected ValueError for duplicate IDs"
    except ValueError as e:
        assert "Duplicate derived IDs" in str(e), f"Wrong error: {e}"
    print("  PASS: duplicate ID raises ValueError")


def test_slug_collision_hard_failure():
    """Different names that produce the same slug must also fail."""
    html = """<html><body>
    <h2>Test</h2>
    <table class="wikitable sortable">
    <thead><tr><th>Achievement</th><th>Requirements</th><th>Hints</th><th>Ver</th><th>DI</th></tr></thead>
    <tbody>
    <tr>
    <td><div class="iconbox"><div class="iconbox-title">Nice Job!</div><div class="iconbox-text">First.</div></div></td>
    <td>Do X</td><td>H</td><td>1.0</td><td>VE</td>
    </tr>
    <tr>
    <td><div class="iconbox"><div class="iconbox-title">Nice Job</div><div class="iconbox-text">Second — slug collides with first (no '!').</div></div></td>
    <td>Do Y</td><td>H</td><td>1.0</td><td>VE</td>
    </tr>
    </tbody>
    </table>
    </body></html>"""
    try:
        parse_achievements_html(html, min_entries=0)
        assert False, "Expected ValueError for slug collision"
    except ValueError as e:
        assert "Duplicate derived IDs" in str(e), f"Wrong error: {e}"
    print("  PASS: slug collision raises ValueError")


def test_strip_html():
    """Test HTML tag stripping."""
    assert strip_html("<b>Hello</b> World") == "Hello World"
    assert strip_html("<a href='x'>link</a> text") == "link text"
    assert strip_html("Plain text") == "Plain text"
    assert strip_html("") == ""
    print("  PASS: strip_html")


def test_strip_icons():
    """Test inline icon stripping."""
    assert strip_icons("Build <span class='icon'>...</span> a thing") == "Build a thing"
    assert strip_icons("Text <img src='icon.png' /> more") == "Text more"
    assert strip_icons("No icons") == "No icons"
    assert strip_icons("") == ""
    print("  PASS: strip_icons")


def test_empty_html():
    """Parser should fail on empty/non-HTML input."""
    try:
        entries, _ = parse_achievements_html("<html></html>", min_entries=0)
        assert len(entries) == 0, "Expected 0 entries from empty HTML"
    except ValueError:
        pass  # acceptable
    print("  PASS: empty_html")


if __name__ == "__main__":
    print("test_wiki_parser.py")
    test_strip_html()
    test_strip_icons()
    test_empty_html()
    test_no_achievement_table_rejected()
    test_bad_headers_rejected()
    test_decoy_table_rejected()
    test_duplicate_id_hard_failure()
    test_slug_collision_hard_failure()
    test_parse_fixture()
    print("\nAll tests passed.")
