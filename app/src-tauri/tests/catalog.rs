use rusqlite::Connection;
use stellaris_hunter_scan::catalog::{
    clear_completion_override, import_catalog, initialize_catalog_schema, load_catalog_entries,
    load_catalog_entries_with_issues, load_catalog_metadata, load_completion_overrides,
    parse_catalog_json, set_completion_override,
};

const CATALOG_FIXTURE: &str = include_str!("fixtures/catalog/catalog.json");

#[test]
fn parses_catalog_fixture() {
    let catalog = parse_catalog_json(CATALOG_FIXTURE).expect("fixture should parse");

    assert_eq!(catalog.catalog_version, "1.0.0");
    assert_eq!(catalog.snapshot_kind, "full");
    assert_eq!(catalog.achievements.len(), 2);
    assert_eq!(catalog.achievements[0].id, "rock_beats_paper");
    assert_eq!(
        catalog.achievements[0].steam_api_name.as_deref(),
        Some("achievement_rock_beats_paper")
    );
    assert_eq!(catalog.achievements[0].curation.conditions.len(), 1);
}

#[test]
fn rejects_catalog_without_full_snapshot_marker() {
    let missing_marker = CATALOG_FIXTURE.replace("  \"snapshot_kind\": \"full\",\n", "");

    let error = parse_catalog_json(&missing_marker)
        .expect_err("catalog imports must explicitly declare full snapshot semantics");

    assert!(
        error.to_string().contains("missing field `snapshot_kind`")
            || error.to_string().contains("snapshot_kind"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_tags_with_punctuation_or_non_ascii() {
    let invalid = CATALOG_FIXTURE.replace(
        "\"lithoid\", \"galactic-community\"",
        "\"lithoid!\", \"galactic-community\"",
    );

    let error = parse_catalog_json(&invalid).expect_err("punctuation should be rejected in tags");

    assert!(
        error.to_string().contains("achievement.curation.tags[]"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_condition_keys_with_punctuation() {
    let invalid = CATALOG_FIXTURE.replace(
        "\"dimension\": \"species_class\"",
        "\"dimension\": \"species_class!\"",
    );

    let error =
        parse_catalog_json(&invalid).expect_err("punctuation should be rejected in condition keys");

    assert!(
        error.to_string().contains("condition.dimension"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_whitespace_only_required_text() {
    let invalid = CATALOG_FIXTURE.replace("\"id\": \"rock_beats_paper\"", "\"id\": \"   \"");

    let error = parse_catalog_json(&invalid).expect_err("blank id should fail");

    assert!(
        error.to_string().contains("achievement.id"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_duplicate_achievement_ids() {
    let duplicate = CATALOG_FIXTURE.replace("\"humble_pie\"", "\"rock_beats_paper\"");

    let error = parse_catalog_json(&duplicate).expect_err("duplicate ids should fail");

    assert!(
        error.to_string().contains("duplicate achievement id"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_unsupported_difficulty() {
    let invalid = CATALOG_FIXTURE.replace("\"difficulty\": \"M\"", "\"difficulty\": \"BAD\"");

    let error = parse_catalog_json(&invalid).expect_err("bad difficulty should fail");

    assert!(
        error.to_string().contains("unsupported difficulty"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_unknown_fields() {
    let invalid = CATALOG_FIXTURE.replace(
        "\"catalog_version\": \"1.0.0\"",
        "\"catalog_version\": \"1.0.0\", \"unexpected\": true",
    );

    let error = parse_catalog_json(&invalid).expect_err("unknown fields should fail");

    assert!(
        error.to_string().contains("unknown field"),
        "unexpected error: {error}"
    );
}

#[test]
fn normalizes_tags_and_condition_keys() {
    let raw = CATALOG_FIXTURE
        .replace(
            "\"lithoid\", \"galactic-community\"",
            "\" Lithoid \", \"galactic community\", \"lithoid\"",
        )
        .replace("\"species_class\"", "\"Species Class\"")
        .replace("\"immutable\"", "\"Immutable\"");

    let catalog = parse_catalog_json(&raw).expect("normalizable values should parse");
    let entry = &catalog.achievements[0];

    assert_eq!(entry.curation.tags, vec!["lithoid", "galactic-community"]);
    assert_eq!(entry.curation.conditions[0].dimension, "species_class");
    assert_eq!(entry.curation.conditions[0].mutability, "immutable");
}

#[test]
fn initializes_catalog_schema_idempotently() {
    let conn = Connection::open_in_memory().expect("in-memory sqlite should open");

    initialize_catalog_schema(&conn).expect("first init should pass");
    initialize_catalog_schema(&conn).expect("second init should pass");

    let table_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('catalog_versions', 'achievements', 'achievement_tags', 'achievement_conditions', 'player_achievements')",
            [],
            |row| row.get(0),
        )
        .expect("table count query should pass");

    assert_eq!(table_count, 5);
}

#[test]
fn initializes_catalog_schema_migrates_pre_metadata_versions_table() {
    let catalog = parse_catalog_json(CATALOG_FIXTURE).expect("fixture should parse");
    let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");

    conn.execute_batch(
        r#"
        CREATE TABLE catalog_versions (
          id TEXT PRIMARY KEY,
          catalog_version TEXT NOT NULL,
          stellaris_version TEXT,
          source_url TEXT,
          source_hash TEXT,
          imported_at TEXT NOT NULL
        );
        "#,
    )
    .expect("old catalog_versions schema should be created");

    initialize_catalog_schema(&conn).expect("schema migration should pass");
    import_catalog(&mut conn, &catalog).expect("import after migration should pass");

    let metadata = load_catalog_metadata(&conn)
        .expect("metadata load should pass")
        .expect("metadata should exist after import");

    assert_eq!(metadata.updated_at, "2026-06-01T00:00:00Z");
}

#[test]
fn initializes_catalog_schema_backfills_legacy_metadata_before_reimport() {
    let conn = Connection::open_in_memory().expect("in-memory sqlite should open");

    conn.execute_batch(
        r#"
        CREATE TABLE catalog_versions (
          id TEXT PRIMARY KEY,
          catalog_version TEXT NOT NULL,
          stellaris_version TEXT,
          source_url TEXT,
          source_hash TEXT,
          imported_at TEXT NOT NULL
        );
        INSERT INTO catalog_versions (
          id, catalog_version, stellaris_version, source_url, source_hash, imported_at
        ) VALUES (
          '1.0.0', '1.0.0', 'v4.3.7', 'https://stellaris.paradoxwikis.com/Achievements',
          'legacy-fixture', '2026-05-31 12:00:00'
        );
        "#,
    )
    .expect("old catalog_versions row should be created");

    initialize_catalog_schema(&conn).expect("schema migration should pass");
    let metadata = load_catalog_metadata(&conn)
        .expect("metadata load should pass")
        .expect("legacy metadata should exist after migration");

    assert_eq!(metadata.updated_at, "2026-05-31 12:00:00");
}

#[test]
fn initializes_catalog_schema_repairs_blank_existing_updated_at() {
    let conn = Connection::open_in_memory().expect("in-memory sqlite should open");

    conn.execute_batch(
        r#"
        CREATE TABLE catalog_versions (
          id TEXT PRIMARY KEY,
          catalog_version TEXT NOT NULL,
          stellaris_version TEXT,
          source_url TEXT,
          source_hash TEXT,
          updated_at TEXT NOT NULL,
          imported_at TEXT NOT NULL
        );
        INSERT INTO catalog_versions (
          id, catalog_version, stellaris_version, source_url, source_hash, updated_at, imported_at
        ) VALUES (
          '1.0.0', '1.0.0', 'v4.3.7', 'https://stellaris.paradoxwikis.com/Achievements',
          'legacy-fixture', '   ', '2026-05-31 12:00:00'
        );
        "#,
    )
    .expect("blank updated_at catalog_versions row should be created");

    initialize_catalog_schema(&conn).expect("schema repair should pass");
    let metadata = load_catalog_metadata(&conn)
        .expect("metadata load should pass")
        .expect("metadata should exist after repair");

    assert_eq!(metadata.updated_at, "2026-05-31 12:00:00");
}

#[test]
fn rejects_non_semver_catalog_version() {
    let invalid = CATALOG_FIXTURE.replace(
        "\"catalog_version\": \"1.0.0\"",
        "\"catalog_version\": \"v1\"",
    );

    let error = parse_catalog_json(&invalid).expect_err("non-semver catalog version should fail");

    assert!(
        error.to_string().contains("catalog_version"),
        "unexpected error: {error}"
    );
}

#[test]
fn imports_catalog_and_loads_entries() {
    let catalog = parse_catalog_json(CATALOG_FIXTURE).expect("fixture should parse");
    let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");

    import_catalog(&mut conn, &catalog).expect("import should pass");
    let entries = load_catalog_entries(&conn).expect("load should pass");

    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|entry| entry.id == "rock_beats_paper"));

    let tag_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM achievement_tags", [], |row| {
            row.get(0)
        })
        .expect("tag count query should pass");
    let condition_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM achievement_conditions", [], |row| {
            row.get(0)
        })
        .expect("condition count query should pass");

    assert_eq!(tag_count, 4);
    assert_eq!(condition_count, 1);
}

#[test]
fn loads_catalog_entries_with_issues_and_skips_corrupt_rows() {
    let catalog = parse_catalog_json(CATALOG_FIXTURE).expect("fixture should parse");
    let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");

    import_catalog(&mut conn, &catalog).expect("import should pass");
    conn.execute(
        "UPDATE achievements SET curation_json = '{invalid' WHERE id = 'humble_pie'",
        [],
    )
    .expect("corrupting one row should succeed");

    let loaded = load_catalog_entries_with_issues(&conn).expect("load with issues should pass");

    assert_eq!(loaded.entries.len(), 1);
    assert!(loaded
        .entries
        .iter()
        .any(|entry| entry.id == "rock_beats_paper"));
    assert_eq!(loaded.issues.len(), 1);
    assert!(
        loaded.issues[0].contains("humble_pie"),
        "unexpected issue: {:?}",
        loaded.issues
    );
    assert!(
        loaded.issues[0].contains("curation_json"),
        "unexpected issue: {:?}",
        loaded.issues
    );
}

#[test]
fn load_catalog_entries_with_issues_skips_semantically_invalid_rows() {
    let catalog = parse_catalog_json(CATALOG_FIXTURE).expect("fixture should parse");
    let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");

    import_catalog(&mut conn, &catalog).expect("import should pass");
    conn.execute(
        r#"
        UPDATE achievements
        SET source_json = '{"name":"Humble Pie","difficulty":"BAD"}'
        WHERE id = 'humble_pie'
        "#,
        [],
    )
    .expect("making one row semantically invalid should succeed");

    let loaded = load_catalog_entries_with_issues(&conn).expect("load with issues should pass");

    assert_eq!(loaded.entries.len(), 1);
    assert!(loaded
        .entries
        .iter()
        .any(|entry| entry.id == "rock_beats_paper"));
    assert_eq!(loaded.issues.len(), 1);
    assert!(
        loaded.issues[0].contains("humble_pie"),
        "unexpected issue: {:?}",
        loaded.issues
    );
    assert!(
        loaded.issues[0].contains("unsupported difficulty"),
        "unexpected issue: {:?}",
        loaded.issues
    );
}

#[test]
fn imports_and_loads_catalog_metadata() {
    let catalog = parse_catalog_json(CATALOG_FIXTURE).expect("fixture should parse");
    let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");

    import_catalog(&mut conn, &catalog).expect("import should pass");
    let metadata = load_catalog_metadata(&conn)
        .expect("metadata load should pass")
        .expect("metadata should exist after import");

    assert_eq!(metadata.catalog_version, "1.0.0");
    assert_eq!(metadata.stellaris_version.as_deref(), Some("v4.3.7"));
    assert_eq!(
        metadata.source_url.as_deref(),
        Some("https://stellaris.paradoxwikis.com/Achievements")
    );
    assert_eq!(metadata.source_hash.as_deref(), Some("synthetic-fixture"));
    assert_eq!(metadata.updated_at, "2026-06-01T00:00:00Z");
    assert!(!metadata.imported_at.is_empty());
    assert_ne!(metadata.imported_at, metadata.updated_at);
}

#[test]
fn import_upserts_existing_achievements() {
    let catalog = parse_catalog_json(CATALOG_FIXTURE).expect("fixture should parse");
    let mut updated = catalog.clone();
    updated.achievements[0].source.name = "Rock Beats Updated Paper".to_string();
    updated.achievements[0].curation.tags = vec!["updated".to_string()];

    let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");

    import_catalog(&mut conn, &catalog).expect("first import should pass");
    import_catalog(&mut conn, &updated).expect("second import should pass");

    let loaded = load_catalog_entries(&conn).expect("load should pass");
    let entry = loaded
        .iter()
        .find(|entry| entry.id == "rock_beats_paper")
        .expect("updated entry should exist");

    assert_eq!(entry.source.name, "Rock Beats Updated Paper");
    assert_eq!(entry.curation.tags, vec!["updated".to_string()]);
}

#[test]
fn full_snapshot_import_marks_missing_existing_achievements_deprecated() {
    let catalog = parse_catalog_json(CATALOG_FIXTURE).expect("fixture should parse");
    let mut reduced = catalog.clone();
    reduced
        .achievements
        .retain(|entry| entry.id == "rock_beats_paper");

    let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");

    import_catalog(&mut conn, &catalog).expect("first import should pass");
    import_catalog(&mut conn, &reduced).expect("second import should pass");

    let humble_deprecated: bool = conn
        .query_row(
            "SELECT deprecated FROM achievements WHERE id = 'humble_pie'",
            [],
            |row| row.get(0),
        )
        .expect("deprecated query should pass");
    let rock_deprecated: bool = conn
        .query_row(
            "SELECT deprecated FROM achievements WHERE id = 'rock_beats_paper'",
            [],
            |row| row.get(0),
        )
        .expect("deprecated query should pass");

    assert!(humble_deprecated);
    assert!(!rock_deprecated);
}

#[test]
fn completion_overrides_round_trip_manual_completed_state() {
    let catalog = parse_catalog_json(CATALOG_FIXTURE).expect("fixture should parse");
    let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");

    import_catalog(&mut conn, &catalog).expect("import should pass");
    set_completion_override(&conn, "rock_beats_paper", true).expect("override should save");

    let overrides = load_completion_overrides(&conn).expect("overrides should load");

    assert_eq!(overrides.len(), 1);
    assert_eq!(overrides[0].achievement_id, "rock_beats_paper");
    assert!(overrides[0].completed);
}

#[test]
fn completion_overrides_can_be_cleared() {
    let catalog = parse_catalog_json(CATALOG_FIXTURE).expect("fixture should parse");
    let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");

    import_catalog(&mut conn, &catalog).expect("import should pass");
    set_completion_override(&conn, "rock_beats_paper", true).expect("override should save");
    clear_completion_override(&conn, "rock_beats_paper").expect("override should clear");

    let overrides = load_completion_overrides(&conn).expect("overrides should load");

    assert!(overrides.is_empty());
}

#[test]
fn completion_overrides_can_force_incomplete() {
    let catalog = parse_catalog_json(CATALOG_FIXTURE).expect("fixture should parse");
    let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");

    import_catalog(&mut conn, &catalog).expect("import should pass");
    set_completion_override(&conn, "rock_beats_paper", false).expect("override should save");

    let overrides = load_completion_overrides(&conn).expect("overrides should load");
    let displayed_unlocked: bool = conn
        .query_row(
            "SELECT displayed_unlocked FROM player_achievements WHERE achievement_id = 'rock_beats_paper'",
            [],
            |row| row.get(0),
        )
        .expect("displayed status should load");

    assert_eq!(overrides.len(), 1);
    assert_eq!(overrides[0].achievement_id, "rock_beats_paper");
    assert!(!overrides[0].completed);
    assert!(!displayed_unlocked);
}

#[test]
fn completion_overrides_reject_unknown_achievement_ids() {
    let catalog = parse_catalog_json(CATALOG_FIXTURE).expect("fixture should parse");
    let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");

    import_catalog(&mut conn, &catalog).expect("import should pass");
    let error = set_completion_override(&conn, "missing", true)
        .expect_err("unknown achievement id should fail");

    assert!(
        error.to_string().contains("unknown achievement id"),
        "unexpected error: {error}"
    );
}

#[test]
fn clearing_completion_overrides_rejects_unknown_achievement_ids() {
    let catalog = parse_catalog_json(CATALOG_FIXTURE).expect("fixture should parse");
    let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");

    import_catalog(&mut conn, &catalog).expect("import should pass");
    let error = clear_completion_override(&conn, "missing")
        .expect_err("unknown achievement id should fail");

    assert!(
        error.to_string().contains("unknown achievement id"),
        "unexpected error: {error}"
    );
}
