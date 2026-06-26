use std::{collections::HashSet, fs, fs::File, io::Write, path::Path};

use rusqlite::Connection;
use stellaris_hunter_scan::{
    catalog::{
        import_catalog, initialize_catalog_schema, load_catalog_entries, parse_catalog_json,
    },
    model::{ActionFacts, SaveRunSummary, SaveSummary},
    rules::evaluate_planner_achievements,
    run_state::{
        initialize_run_state_schema, load_persisted_runs, load_run_achievement_statuses,
        load_run_facts, persist_run_for_tests, persist_scan_report, run_exists,
        set_run_achievement_status,
    },
};
use tempfile::tempdir;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[test]
fn run_state_schema_initializes_without_removing_catalog_tables() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    initialize_catalog_schema(&conn).expect("catalog schema");
    initialize_run_state_schema(&conn).expect("run schema");

    let player_table_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'player_achievements'",
            [],
            |row| row.get(0),
        )
        .expect("player table count");
    let runs_table_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'runs'",
            [],
            |row| row.get(0),
        )
        .expect("runs table count");

    assert_eq!(player_table_count, 1);
    assert_eq!(runs_table_count, 1);
}

#[test]
fn persists_parsed_save_facts_for_a_run() {
    let dir = tempdir().expect("tempdir");
    let save_root = dir.path().join("save games");
    let run_root = save_root.join("run_a");
    fs::create_dir_all(&run_root).expect("run dir");
    let save_path = run_root.join("ironman.sav");
    fs::write(&save_path, b"synthetic bytes").expect("save file");

    let mut conn = Connection::open_in_memory().expect("open in-memory db");
    initialize_run_state_schema(&conn).expect("run schema");
    let run = SaveRunSummary {
        run_folder: "run_a".to_string(),
        save_count: 1,
        latest_save: Some(SaveSummary {
            path: save_path.clone(),
            version: Some("Cetus v4.3.7".to_string()),
            date: Some("2532.01.26".to_string()),
            name: Some("Synthetic Run".to_string()),
            ironman: Some(true),
            cheated_on_save: Some(false),
            origin: Some("origin_default".to_string()),
            ethics: vec!["ethic_xenophile".to_string()],
            civics: vec!["civic_meritocracy".to_string()],
            founder_species_class: Some("HUM".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    persist_run_for_tests(&mut conn, &save_root, &run).expect("persist run");

    let runs = load_persisted_runs(&conn).expect("load runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].run_folder, "run_a");
    assert_eq!(
        runs[0].latest_save_file_name.as_deref(),
        Some("ironman.sav")
    );
    assert_eq!(runs[0].parse_status.as_deref(), Some("parsed"));
    assert!(
        runs[0].fact_count >= 7,
        "expected persisted facts: {runs:?}"
    );

    assert!(run_exists(&conn, &run_root).expect("run exists"));
    let facts = load_run_facts(&conn, &runs[0].folder_path).expect("load facts");
    assert!(facts.iter().any(|fact| {
        fact.dimension == "empire" && fact.key == "origin" && fact.value == "origin_default"
    }));
    assert!(facts.iter().any(|fact| {
        fact.dimension == "eligibility" && fact.key == "ironman" && fact.value == true
    }));
}

#[test]
fn rescanning_same_folder_updates_instead_of_duplicating_run() {
    let dir = tempdir().expect("tempdir");
    let save_root = dir.path().join("save games");
    let run_root = save_root.join("run_a");
    fs::create_dir_all(&run_root).expect("run dir");
    let save_path = run_root.join("ironman.sav");
    fs::write(&save_path, b"synthetic bytes").expect("save file");
    let mut conn = Connection::open_in_memory().expect("open in-memory db");
    initialize_run_state_schema(&conn).expect("run schema");

    let first = save_run("run_a", &save_path, "2532.01.26");
    persist_run_for_tests(&mut conn, &save_root, &first).expect("first persist");
    let second = save_run("run_a", &save_path, "2533.02.01");
    persist_run_for_tests(&mut conn, &save_root, &second).expect("second persist");

    let runs = load_persisted_runs(&conn).expect("load runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].latest_ingame_date.as_deref(), Some("2533.02.01"));
}

#[test]
fn scan_persistence_records_failed_latest_save_without_aborting() {
    let dir = tempdir().expect("tempdir");
    let install_root = dir.path().join("install");
    let documents_root = dir.path().join("documents");
    let run_root = documents_root.join("save games").join("run_a");
    fs::create_dir_all(&install_root).expect("install dir");
    fs::create_dir_all(&run_root).expect("run dir");
    fs::write(install_root.join("steam_appid.txt"), "281990\n").expect("appid");
    fs::write(run_root.join("ironman.sav"), b"not a zip").expect("bad save");

    let report = stellaris_hunter_scan::scan_all(Some(install_root), Some(documents_root));
    let mut conn = Connection::open_in_memory().expect("open in-memory db");
    initialize_run_state_schema(&conn).expect("run schema");
    persist_scan_report(&mut conn, &report).expect("persist report");

    let runs = load_persisted_runs(&conn).expect("load runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].parse_status.as_deref(), Some("failed"));
    assert!(runs[0]
        .parse_error
        .as_deref()
        .unwrap_or_default()
        .contains("failed to parse"));
}

#[test]
fn loading_run_facts_rejects_corrupt_json_values() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    initialize_run_state_schema(&conn).expect("run schema");
    conn.execute(
        "INSERT INTO runs (folder_path, run_folder, updated_at) VALUES (?1, ?2, datetime('now'))",
        ["/tmp/run_a", "run_a"],
    )
    .expect("insert run");
    conn.execute(
        "INSERT INTO run_facts (
            run_folder_path, dimension, key, value_json, source, confidence, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))",
        [
            "/tmp/run_a",
            "empire",
            "origin",
            "{broken-json",
            "parsed_save",
            "high",
        ],
    )
    .expect("insert corrupt fact");

    let result = load_run_facts(&conn, "/tmp/run_a");

    assert!(
        result.is_err(),
        "corrupt fact JSON must not be silently displayed"
    );
}

#[test]
fn run_achievement_status_round_trips_and_clears() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    initialize_catalog_schema(&conn).expect("catalog schema");
    initialize_run_state_schema(&conn).expect("run schema");
    conn.execute(
        "INSERT INTO achievements (
            id, steam_app_id, name, source_json, curation_json, created_at, updated_at
        ) VALUES (?1, 281990, ?2, ?3, ?4, datetime('now'), datetime('now'))",
        [
            "ach_1",
            "Achievement One",
            r#"{"name":"Achievement One"}"#,
            r#"{"tags":[],"conditions":[],"warnings":[],"planner_notes":null,"known_limitations":[],"rule_confidence":null}"#,
        ],
    )
    .expect("insert achievement");
    conn.execute(
        "INSERT INTO runs (folder_path, run_folder, updated_at) VALUES (?1, ?2, datetime('now'))",
        ["/tmp/run_a", "run_a"],
    )
    .expect("insert run");

    set_run_achievement_status(&conn, "/tmp/run_a", "ach_1", Some("planned")).expect("set planned");
    let statuses = load_run_achievement_statuses(&conn, "/tmp/run_a").expect("load statuses");
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].achievement_id, "ach_1");
    assert_eq!(statuses[0].user_status, "planned");

    set_run_achievement_status(&conn, "/tmp/run_a", "ach_1", None).expect("clear status");
    let statuses = load_run_achievement_statuses(&conn, "/tmp/run_a").expect("reload statuses");
    assert!(statuses.is_empty());
}

#[test]
fn run_achievement_status_normalizes_run_paths() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    initialize_catalog_schema(&conn).expect("catalog schema");
    initialize_run_state_schema(&conn).expect("run schema");
    conn.execute(
        "INSERT INTO achievements (
            id, steam_app_id, name, source_json, curation_json, created_at, updated_at
        ) VALUES (?1, 281990, ?2, ?3, ?4, datetime('now'), datetime('now'))",
        [
            "ach_1",
            "Achievement One",
            r#"{"name":"Achievement One"}"#,
            r#"{"tags":[],"conditions":[],"warnings":[],"planner_notes":null,"known_limitations":[],"rule_confidence":null}"#,
        ],
    )
    .expect("insert achievement");
    conn.execute(
        "INSERT INTO runs (folder_path, run_folder, updated_at) VALUES (?1, ?2, datetime('now'))",
        ["/tmp/run_a", "run_a"],
    )
    .expect("insert run");

    set_run_achievement_status(&conn, "/tmp/other/../run_a", "ach_1", Some("ignored"))
        .expect("set ignored");
    let statuses = load_run_achievement_statuses(&conn, "/tmp/run_a").expect("load statuses");

    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].user_status, "ignored");
}

#[test]
fn scan_prefers_ironman_save_over_newer_folder_snapshots() {
    let dir = tempdir().expect("tempdir");
    let install_root = dir.path().join("install");
    let documents_root = dir.path().join("documents");
    let run_root = documents_root.join("save games").join("run_a");
    fs::create_dir_all(&install_root).expect("install dir");
    fs::create_dir_all(&run_root).expect("run dir");
    fs::write(install_root.join("steam_appid.txt"), "281990\n").expect("appid");
    write_save_fixture(&run_root.join("autosave_2300.sav"));
    write_save_fixture(&run_root.join("ironman.sav"));

    let report = stellaris_hunter_scan::scan_all(Some(install_root), Some(documents_root));
    let latest_path = report
        .documents
        .as_ref()
        .and_then(|documents| documents.save_runs.first())
        .and_then(|run| run.latest_save.as_ref())
        .map(|save| save.path.as_path())
        .expect("latest save");

    assert_eq!(
        latest_path.file_name().and_then(|name| name.to_str()),
        Some("ironman.sav")
    );
}

#[test]
fn phase7_boolean_dimensions_end_to_end() {
    let dir = tempdir().expect("tempdir");
    let save_root = dir.path().join("save games");
    let run_root = save_root.join("run_phase7");
    fs::create_dir_all(&run_root).expect("run dir");
    let save_path = run_root.join("ironman.sav");
    fs::write(&save_path, b"synthetic bytes").expect("save file");

    let mut conn = Connection::open_in_memory().expect("open in-memory db");
    initialize_catalog_schema(&conn).expect("catalog schema");
    initialize_run_state_schema(&conn).expect("run schema");

    // Build a SaveSummary with the three Phase 7 boolean dimensions set to true
    let actions = ActionFacts {
        blazing_scourge_decisions: Some(true),
        pre_ftl_invasion_occurred: Some(true),
        artificial_military_ships_built: Some(true),
        ..Default::default()
    };

    let run = SaveRunSummary {
        run_folder: "run_phase7".to_string(),
        save_count: 1,
        latest_save: Some(SaveSummary {
            path: save_path,
            version: Some("Cetus v4.3.7".to_string()),
            date: Some("2532.01.26".to_string()),
            name: Some("Phase 7 Test".to_string()),
            ironman: Some(true),
            cheated_on_save: Some(false),
            origin: Some("origin_default".to_string()),
            ethics: vec!["ethic_xenophile".to_string()],
            civics: vec!["civic_fire_cult".to_string()],
            actions: Some(actions),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Step 1: Persist the run → facts are extracted and stored in DB
    persist_run_for_tests(&mut conn, &save_root, &run).expect("persist run");

    let runs = load_persisted_runs(&conn).expect("load runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].run_folder, "run_phase7");

    // Step 2: Load facts and verify Phase 7 dimensions were persisted
    let facts = load_run_facts(&conn, &runs[0].folder_path).expect("load facts");

    let blazing = facts
        .iter()
        .find(|f| f.dimension == "action" && f.key == "blazing_scourge_decisions")
        .expect("blazing_scourge_decisions fact persisted");
    assert_eq!(blazing.value, serde_json::json!(true));
    assert_eq!(blazing.source, "parsed_save");

    let pre_ftl = facts
        .iter()
        .find(|f| f.dimension == "action" && f.key == "pre_ftl_invasion_occurred")
        .expect("pre_ftl_invasion_occurred fact persisted");
    assert_eq!(pre_ftl.value, serde_json::json!(true));

    let artificial = facts
        .iter()
        .find(|f| f.dimension == "action" && f.key == "artificial_military_ships_built")
        .expect("artificial_military_ships_built fact persisted");
    assert_eq!(artificial.value, serde_json::json!(true));

    // Step 3: Verify that dimensions NOT set (left as None/default) are absent
    assert!(
        !facts.iter().any(|f| f.key == "colossus_built"),
        "colossus_built should not be persisted when None"
    );

    // Step 4: Import a catalog entry that uses blazing_scourge_decisions
    let catalog_json = r#"{
        "catalog_version": "1.0.0",
        "snapshot_kind": "full",
        "stellaris_version": "4.3",
        "source_url": "",
        "source_hash": "",
        "updated_at": "2026-01-01",
        "achievements": [{
            "id": "from_bad_to_worse",
            "steam_app_id": 281990,
            "steam_api_name": "achievement_from_bad_to_worse",
            "local_key": null,
            "deprecated": false,
            "source": {
                "name": "From Bad to Worse",
                "description": "Use 5 Blazing Scourge decisions on a Tomb World",
                "requirement": "Use 5 Blazing Scourge decisions on a Tomb World",
                "hint": "The Blazing Scourge decision requires the Fire Cult civic.",
                "group": "Infernals",
                "version_added": "4.2",
                "difficulty": "E"
            },
            "curation": {
                "tags": ["infernals"],
                "conditions": [{
                    "condition_type": "required",
                    "dimension": "blazing_scourge_decisions",
                    "operator": "equals",
                    "value": true,
                    "timing": "eventual",
                    "mutability": "normal_change",
                    "severity": "soft",
                    "source": "wiki-reviewed",
                    "notes": "Requires using the Blazing Scourge decision on a Tomb World."
                }],
                "warnings": [],
                "known_limitations": [],
                "rule_confidence": "medium"
            }
        }]
    }"#;
    let catalog = parse_catalog_json(catalog_json).expect("parse catalog");
    import_catalog(&mut conn, &catalog).expect("import catalog");

    // Step 5: Evaluate planner achievements — dimension is true → Possible
    let entries = load_catalog_entries(&conn).expect("load catalog entries");
    let completed: HashSet<String> = HashSet::new();
    let statuses = load_run_achievement_statuses(&conn, &runs[0].folder_path)
        .expect("load achievement statuses");
    let evals = evaluate_planner_achievements(entries, &facts, &completed, &statuses);
    assert_eq!(evals.len(), 1);
    assert_eq!(evals[0].achievement.id, "from_bad_to_worse");
    assert_eq!(
        evals[0].computed_status, "Possible",
        "blazing_scourge_decisions=true should yield Possible"
    );

    // Step 6: Verify that the planner evaluation cache is used
    use stellaris_hunter_scan::run_state::{load_evaluations, save_evaluations};
    let evaluations = evals.clone();
    save_evaluations(&conn, &runs[0].folder_path, &evaluations).expect("save evaluations to cache");
    let cached = load_evaluations(&conn, &runs[0].folder_path)
        .expect("load cached evaluations")
        .expect("cache should have entry");
    assert_eq!(cached.len(), 1);
    assert_eq!(cached[0].achievement.id, "from_bad_to_worse");
    assert_eq!(cached[0].computed_status, "Possible");
}

fn save_run(run_folder: &str, save_path: &Path, date: &str) -> SaveRunSummary {
    SaveRunSummary {
        run_folder: run_folder.to_string(),
        save_count: 1,
        latest_save: Some(SaveSummary {
            path: save_path.to_path_buf(),
            date: Some(date.to_string()),
            name: Some("Synthetic Run".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn write_save_fixture(path: &Path) {
    let file = File::create(path).expect("create save");
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("meta", options).expect("meta entry");
    zip.write_all(include_str!("fixtures/save/meta.txt").as_bytes())
        .expect("write meta");

    zip.start_file("gamestate", options)
        .expect("gamestate entry");
    zip.write_all(include_str!("fixtures/save/gamestate.txt").as_bytes())
        .expect("write gamestate");

    zip.finish().expect("finish zip");
}
