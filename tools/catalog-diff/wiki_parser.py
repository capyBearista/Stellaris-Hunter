#!/usr/bin/env python3
"""
wiki_parser.py — Parse offline Stellaris Wiki Achievements HTML.

Stdlib-only HTML parser for the Stellaris Wiki Achievements page
(https://stellaris.paradoxwikis.com/Achievements).  Extracts achievement
entries from the wikitable structure: section heading → 5-column rows
(Achievement / Requirements / Hints / Ver / DI).

Output is a list of dicts matching the upstream_snapshot entry schema,
minus `steam_api_name` (which the wiki does not provide).  Intended as
a sub‑module of catalog_update.py, not as a standalone CLI.
"""

import re
from html.parser import HTMLParser

# Shared slugify (canonical implementation in catalog_diff.py)
from catalog_diff import slugify

# ---------------------------------------------------------------------------
# HTML tag stripper for extracting readable text
# ---------------------------------------------------------------------------

class _TextCollector(HTMLParser):
    """Collects all plain text content, stripping HTML tags."""
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.parts = []

    def handle_data(self, data):
        self.parts.append(data)

    @property
    def text(self):
        return "".join(self.parts).strip()


def strip_html(html_fragment: str) -> str:
    """Return the plain text content of an HTML fragment with tags removed."""
    collector = _TextCollector()
    try:
        collector.feed(html_fragment)
    except Exception:
        pass
    return collector.text


# ---------------------------------------------------------------------------
# Normalise inline icons in wiki text
# ---------------------------------------------------------------------------

# Pattern for inline icon spans like: <span class="icon">…</span>
_ICON_SPAN_RE = re.compile(r'<span\s+class=["\']icon["\'][^>]*>.*?</span>', re.DOTALL)
# Pattern for <img> tags
_IMG_RE = re.compile(r'<img[^>]*>', re.DOTALL)

def strip_icons(text: str) -> str:
    """Remove inline icon spans and img tags, normalise whitespace."""
    text = _ICON_SPAN_RE.sub("", text)
    text = _IMG_RE.sub("", text)
    # Collapse multiple spaces/newlines
    text = re.sub(r'\s+', ' ', text)
    return text.strip()


# ---------------------------------------------------------------------------
# Main wiki-achievements-page parser
# ---------------------------------------------------------------------------

class WikiAchievementsParser(HTMLParser):
    """Parse the Stellaris Wiki Achievements page HTML.

    Attributes:
        achievements: list of dicts — extracted entries (no steam_api_name).
        errors: list of str — parse warnings/issues.
        warnings_: list of str — non-fatal structural notes.
    """

    # Column indices in the achievement table
    COL_NAME_DESC = 0
    COL_REQUIREMENT = 1
    COL_HINT = 2
    COL_VERSION = 3
    COL_DIFFICULTY = 4

    # Canonical achievement-table header labels (lowercased, stripped)
    EXPECTED_HEADERS = ["achievement", "requirements", "hints", "ver", "di"]

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.achievements = []
        self.errors = []
        self.warnings_ = []

        # State tracking
        self._section_name = ""
        self._in_heading = False        # inside <h2>/<h3>
        self._in_table = False          # inside <table class="wikitable">
        self._table_is_ach = False      # validated as achievement table
        self._in_thead = False          # inside <thead>
        self._in_th = False             # inside <th>
        self._header_cells = []         # collected header cell text
        self._in_tr = False             # inside a data <tr> (not header)
        self._in_td = False             # inside a <td>
        self._td_index = -1
        self._current_row: list = [None] * 5  # raw HTML per column
        self._td_buffer = ""

        # Column-0 (iconbox) sub-state
        self._in_iconbox_title = False
        self._in_iconbox_text = False
        self._in_ach_header = False
        self._in_ach_desc = False
        self._iconbox_name = ""
        self._iconbox_desc = ""

        # Heading recognition
        self._heading_tags = {"h2", "h3"}

    # ----- Tag handlers -----

    def handle_starttag(self, tag, attrs):
        attr_dict = dict(attrs)

        # Track sections via <h2>/<h3>
        if tag in self._heading_tags:
            self._in_heading = True
            self._section_name = ""  # reset; will be filled by id attr or text
            # Try to get section from id attribute (e.g. id="Base_game")
            # Some wikis put id on <h2>/<h3>, others on child <span>
            hid = attr_dict.get("id", "")
            if hid:
                self._section_name = hid.replace("_", " ").strip()
            return

        # Table detection
        if tag == "table":
            raw_class = attr_dict.get("class") or ""
            classes = raw_class.split()
            if "wikitable" in classes:
                self._in_table = True
                self._header_cells = []  # reset for potential achievement table
            return

        if tag == "thead":
            self._in_thead = True
            self._header_cells = []
            return
        if tag == "tbody":
            self._in_thead = False
            return

        # Track <th> cells inside thead to validate header row
        if tag == "th" and self._in_thead:
            self._in_th = True
            self._td_buffer = ""
            return

        if tag == "tr" and self._in_table and not self._in_thead:
            if not self._table_is_ach:
                return  # skip rows in non-achievement tables
            self._in_tr = True
            self._current_row = [None] * 5
            self._td_index = -1
            self._iconbox_name = ""
            self._iconbox_desc = ""
            return

        if tag == "td" and self._in_tr:
            self._in_td = True
            self._td_index += 1
            self._td_buffer = ""
            self._iconbox_name = ""
            self._iconbox_desc = ""
            self._in_iconbox_title = False
            self._in_iconbox_text = False
            self._in_ach_header = False
            self._in_ach_desc = False
            return

        # Section heading child span — may carry the id attribute
        if self._in_heading and tag == "span":
            cls = attr_dict.get("class") or ""
            if "mw-headline" in cls:
                hid = attr_dict.get("id", "")
                if hid:
                    self._section_name = hid.replace("_", " ").strip()
            return

        # Column 0 iconbox structure
        if self._in_td and self._td_index == self.COL_NAME_DESC:
            cls = attr_dict.get("class") or ""
            if "iconbox-title" in cls:
                self._in_iconbox_title = True
                return
            if "iconbox-text" in cls:
                self._in_iconbox_text = True
                return
            if "achievement-header" in cls:
                self._in_ach_header = True
                return
            if "achievement-desc" in cls:
                self._in_ach_desc = True
                return

    def handle_endtag(self, tag):
        if tag in self._heading_tags:
            self._in_heading = False
            return

        if tag == "th" and self._in_th:
            self._in_th = False
            text = self._td_buffer.strip().lower()
            self._header_cells.append(text)
            self._td_buffer = ""
            return

        if tag == "thead":
            self._in_thead = False
            # Validate: must have exactly our 5 expected headers
            if self._header_cells == self.EXPECTED_HEADERS:
                self._table_is_ach = True
            else:
                self._table_is_ach = False
                if self._header_cells:
                    self.warnings_.append(
                        f"Skipping wikitable with unexpected headers: "
                        f"{self._header_cells}"
                    )
            self._header_cells = []
            return

        if tag == "table" and self._in_table:
            self._in_table = False
            self._table_is_ach = False
            self._in_thead = False
            self._in_tr = False
            self._in_td = False
            return

        if tag == "tr" and self._in_tr:
            self._finalise_row()
            self._in_tr = False
            self._in_td = False
            return

        if tag == "td" and self._in_td:
            # Store collected HTML for this cell
            self._store_td()
            self._in_td = False
            self._in_iconbox_title = False
            self._in_iconbox_text = False
            self._in_ach_header = False
            self._in_ach_desc = False
            return

        # Reset iconbox sub-flags when their container div closes
        if tag == "div":
            if self._in_iconbox_title:
                self._in_iconbox_title = False
            elif self._in_iconbox_text:
                self._in_iconbox_text = False

    def handle_data(self, data):
        if self._in_heading:
            # Section heading text — use as fallback if no id attr
            if not self._section_name:
                self._section_name = data.strip()
            return

        # Collect header cell text for validation
        if self._in_th:
            self._td_buffer += data
            return

        if not self._in_td:
            return

        if self._in_iconbox_title or self._in_ach_header:
            self._iconbox_name += data
            return

        if self._in_iconbox_text or self._in_ach_desc:
            self._iconbox_desc += data
            return

        # For all columns: buffer data for post-processing
        self._td_buffer += data

    # NOTE: handle_entityref is intentionally absent.
    # HTMLParser(convert_charrefs=True) automatically decodes all named and
    # numeric character references before calling handle_data, so iconbox
    # text, header cells, and _td_buffer all receive decoded text already.

    # ----- Internal helpers -----

    def _store_td(self):
        """Store the collected content for the current table cell."""
        if self._td_index < 0 or self._td_index > 4:
            return

        if self._td_index == self.COL_NAME_DESC:
            # Column 0: name + description from iconbox
            name = self._iconbox_name.strip()
            desc = self._iconbox_desc.strip()
            if not name:
                # Fallback: use the buffer directly
                buf = self._td_buffer.strip()
                name = buf
            self._current_row[self.COL_NAME_DESC] = {
                "name": name,
                "description": desc,
            }
        else:
            content = self._td_buffer.strip()
            # Strip icons and collapse whitespace
            content = strip_icons(content)
            content = strip_html(content)
            self._current_row[self._td_index] = content

    def _finalise_row(self):
        """Process a completed row into an achievement entry."""
        col0 = self._current_row[self.COL_NAME_DESC]
        if col0 is None:
            return

        name = col0.get("name", "").strip()
        desc = col0.get("description", "").strip()
        req = self._current_row[self.COL_REQUIREMENT] or ""
        hint = self._current_row[self.COL_HINT] or ""
        version = self._current_row[self.COL_VERSION] or ""
        difficulty = self._current_row[self.COL_DIFFICULTY] or "UC"

        if not name:
            self.warnings_.append(f"Skipped row with empty name (section: {self._section_name})")
            return

        entry = {
            "name": name,
            "description": desc,
            "requirement": req,
            "hint": hint,
            "group": self._section_name or "Unknown",
            "version_added": version,
            "difficulty": difficulty if difficulty else "UC",
        }
        self.achievements.append(entry)

    # ----- Public API -----

    def get_warnings(self):
        return list(self.warnings_)

    def get_errors(self):
        return list(self.errors)


# ---------------------------------------------------------------------------
# Convenience: parse an HTML string into achievement entries
# ---------------------------------------------------------------------------

def parse_achievements_html(html_text: str, min_entries: int = 50):
    """Parse Stellaris Wiki Achievements HTML into a list of dicts.

    Args:
        html_text: The HTML content to parse.
        min_entries: Minimum expected entries (default 50 for full page).
                     Pass 0 to disable the check (for test fixtures).

    Returns:
        (entries, warnings): where entries is a list of dicts with keys
        name, description, requirement, hint, group, version_added,
        difficulty.

    Raises:
        ValueError: if parsing yields fewer than min_entries entries
                    (likely structural change).
    """
    parser = WikiAchievementsParser()
    try:
        parser.feed(html_text)
    except Exception as e:
        raise ValueError(f"HTML parse error: {e}") from e

    parser.close()

    entries = parser.achievements
    warnings = parser.get_warnings()

    # Sanity check: we should find a reasonable number of achievements
    # (Stellaris has 211 known + new ones; use min_entries parameter)
    if min_entries > 0 and len(entries) < min_entries:
        # Include the HTML structure in the error message for debugging
        sample = html_text[:500] if len(html_text) > 500 else html_text
        raise ValueError(
            f"Parser extracted only {len(entries)} entries (expected >= {min_entries}). "
            f"The HTML structure may have changed. "
            f"Warnings: {'; '.join(warnings[:5]) if warnings else 'none'}. "
            f"HTML preview: {sample[:200]}..."
        )

    # CHECK: duplicate derived IDs (slug collisions) are a hard failure.
    # Different wiki entries that slugify to the same ID must be resolved
    # before the data can be used.
    id_counts = {}
    for e in entries:
        sid = slugify(e["name"])
        id_counts[sid] = id_counts.get(sid, 0) + 1
    dup_ids = {k: v for k, v in id_counts.items() if v > 1}
    if dup_ids:
        dup_names = []
        for e in entries:
            sid = slugify(e["name"])
            if sid in dup_ids:
                dup_names.append(f"{e['name']} -> {sid}")
        raise ValueError(
            f"Duplicate derived IDs detected ({len(dup_ids)} collisions). "
            f"Affected: {dup_names}. "
            f"Resolve naming conflicts before importing."
        )

    # CHECK: duplicate names within the same group (potential parse error)
    name_counts = {}
    for e in entries:
        key = (e["name"], e["group"])
        name_counts[key] = name_counts.get(key, 0) + 1
    dupes = {k: v for k, v in name_counts.items() if v > 1}
    if dupes:
        warnings.append(
            f"Duplicate names found within the same group: {dupes}"
        )

    return entries, warnings
