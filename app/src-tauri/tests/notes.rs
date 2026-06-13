use rusqlite::Connection;
use stellaris_hunter_scan::{
    catalog::initialize_catalog_schema,
    run_state::{
        clear_run_achievement_note, clear_run_note, initialize_run_state_schema,
        load_run_achievement_notes, load_run_achievement_statuses, load_run_notes,
        persist_run_for_tests, set_run_achievement_note, set_run_achievement_status, set_run_note,
    },
    SaveRunSummary, SaveSummary,
};
use tempfile::tempdir;

fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .unwrap();
    initialize_catalog_schema(&conn).unwrap();
    initialize_run_state_schema(&conn).unwrap();
    // Insert placeholder achievements so FK constraints on run_achievement_status are satisfied
    for ach_id in &["ach_1", "ach_2"] {
        conn.execute(
            "INSERT OR IGNORE INTO achievements (id, name, source_json, curation_json) VALUES (?1, ?2, '{}', '{}')",
            rusqlite::params![ach_id, format!("Test {ach_id}")],
        )
        .unwrap();
    }
    conn
}

fn setup_test_run(conn: &mut Connection) -> String {
    let tmp = tempdir().unwrap();
    let save_root = tmp.path().to_path_buf();
    let run = SaveRunSummary {
        run_folder: "test_run".to_string(),
        save_count: 1,
        latest_save: Some(SaveSummary {
            path: save_root.join("test_run").join("ironman.sav"),
            version: Some("4.3.0".to_string()),
            date: Some("2200.01.01".to_string()),
            name: Some("Test Save".to_string()),
            ironman: Some(true),
            cheated_on_save: Some(false),
            player_country: Some("country_0".to_string()),
            authority: Some("auth_democratic".to_string()),
            government_type: Some("gov_democratic_republic".to_string()),
            origin: Some("origin_default".to_string()),
            ethics: vec![
                "ethic_fanatic_xenophile".to_string(),
                "ethic_materialist".to_string(),
            ],
            civics: vec!["civic_meritocracy".to_string()],
            founder_species_class: Some("humanoid".to_string()),
            founder_species_portrait: Some("human".to_string()),
            founder_species_traits: vec!["trait_adaptive".to_string()],
            ..Default::default()
        }),
        ..Default::default()
    };
    persist_run_for_tests(conn, &save_root, &run).unwrap();
    save_root
        .join("test_run")
        .to_string_lossy()
        .replace('\\', "/")
}

#[test]
fn set_and_load_run_note() {
    let mut conn = setup_test_db();
    let run_path = setup_test_run(&mut conn);

    // Initially no note
    assert!(load_run_notes(&conn, &run_path).unwrap().is_none());

    // Set a note
    set_run_note(&conn, &run_path, "My test note").unwrap();

    let note = load_run_notes(&conn, &run_path)
        .unwrap()
        .expect("note should exist");
    assert_eq!(note.note_text, "My test note");
    assert_eq!(note.run_folder_path, run_path);
    assert!(!note.created_at.is_empty());
    assert!(!note.updated_at.is_empty());
}

#[test]
fn clear_run_note_works() {
    let mut conn = setup_test_db();
    let run_path = setup_test_run(&mut conn);

    set_run_note(&conn, &run_path, "Temporary note").unwrap();
    assert!(load_run_notes(&conn, &run_path).unwrap().is_some());

    clear_run_note(&conn, &run_path).unwrap();
    assert!(load_run_notes(&conn, &run_path).unwrap().is_none());
}

#[test]
fn run_note_upsert_replaces_text() {
    let mut conn = setup_test_db();
    let run_path = setup_test_run(&mut conn);

    set_run_note(&conn, &run_path, "First version").unwrap();
    set_run_note(&conn, &run_path, "Second version").unwrap();

    let note = load_run_notes(&conn, &run_path)
        .unwrap()
        .expect("note should exist");
    assert_eq!(note.note_text, "Second version");
    // Verify there's exactly one row in the table
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM run_notes", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn set_and_load_run_achievement_note() {
    let mut conn = setup_test_db();
    let run_path = setup_test_run(&mut conn);

    let notes = load_run_achievement_notes(&conn, &run_path).unwrap();
    assert!(notes.is_empty());

    set_run_achievement_note(&conn, &run_path, "ach_1", "Working on this").unwrap();

    let notes = load_run_achievement_notes(&conn, &run_path).unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].achievement_id, "ach_1");
    assert_eq!(notes[0].notes, "Working on this");
    assert!(!notes[0].updated_at.is_empty());
}

#[test]
fn clear_run_achievement_note_works() {
    let mut conn = setup_test_db();
    let run_path = setup_test_run(&mut conn);

    set_run_achievement_note(&conn, &run_path, "ach_1", "Has notes").unwrap();
    assert_eq!(
        load_run_achievement_notes(&conn, &run_path).unwrap().len(),
        1
    );

    clear_run_achievement_note(&conn, &run_path, "ach_1").unwrap();
    assert!(load_run_achievement_notes(&conn, &run_path)
        .unwrap()
        .is_empty());
}

#[test]
fn run_achievement_note_preserves_user_status() {
    let mut conn = setup_test_db();
    let run_path = setup_test_run(&mut conn);

    // Set user_status first
    set_run_achievement_status(&conn, &run_path, "ach_1", Some("planned")).unwrap();

    // Then set a note
    set_run_achievement_note(&conn, &run_path, "ach_1", "Note for planned achievement").unwrap();

    // Verify user_status is still 'planned'
    let statuses = load_run_achievement_statuses(&conn, &run_path).unwrap();
    let ach_1_status = statuses
        .iter()
        .find(|s| s.achievement_id == "ach_1")
        .expect("ach_1 status should exist");
    assert_eq!(ach_1_status.user_status, "planned");
    assert_eq!(
        ach_1_status.notes.as_deref(),
        Some("Note for planned achievement")
    );

    // Now set a note on a new achievement (no existing status) — should create with 'planned'
    set_run_achievement_note(&conn, &run_path, "ach_2", "New achievement note").unwrap();
    let statuses = load_run_achievement_statuses(&conn, &run_path).unwrap();
    let ach_2_status = statuses
        .iter()
        .find(|s| s.achievement_id == "ach_2")
        .expect("ach_2 status should exist");
    assert_eq!(ach_2_status.user_status, "planned");
    assert_eq!(ach_2_status.notes.as_deref(), Some("New achievement note"));
}
