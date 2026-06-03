use std::{fs, fs::File, io::Write, path::Path};

use rusqlite::Connection;
use stellaris_hunter_scan::{
    catalog::initialize_catalog_schema,
    model::{SaveRunSummary, SaveSummary},
    run_state::{
        initialize_run_state_schema, load_persisted_runs, load_run_facts, persist_run_for_tests,
        persist_scan_report, run_exists,
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
