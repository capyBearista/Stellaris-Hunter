use std::collections::HashSet;

use rusqlite::Connection;
use serde_json::json;
use stellaris_hunter_scan::{
    catalog::{
        import_catalog, initialize_catalog_schema, load_catalog_entries, parse_catalog_json,
    },
    rules::evaluate_planner_achievements,
    run_state::{
        clear_fact_override, initialize_run_state_schema, load_fact_overrides,
        load_run_achievement_statuses, load_run_facts, persist_run_for_tests, set_fact_override,
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
fn set_and_load_fact_override() {
    let mut conn = setup_test_db();
    let run_path = setup_test_run(&mut conn);

    let value = json!("origin_synaptic");
    set_fact_override(
        &conn,
        &run_path,
        "empire",
        "origin",
        &value,
        Some("corrected manually"),
    )
    .unwrap();

    let overrides = load_fact_overrides(&conn, &run_path).unwrap();
    assert_eq!(overrides.len(), 1);
    assert_eq!(overrides[0].dimension, "empire");
    assert_eq!(overrides[0].key, "origin");
    assert_eq!(overrides[0].value, json!("origin_synaptic"));
    assert_eq!(overrides[0].reason, Some("corrected manually".to_string()));
}

#[test]
fn override_clear() {
    let mut conn = setup_test_db();
    let run_path = setup_test_run(&mut conn);

    let value = json!("origin_synaptic");
    set_fact_override(&conn, &run_path, "empire", "origin", &value, None).unwrap();
    assert_eq!(load_fact_overrides(&conn, &run_path).unwrap().len(), 1);

    clear_fact_override(&conn, &run_path, "empire", "origin").unwrap();
    assert_eq!(load_fact_overrides(&conn, &run_path).unwrap().len(), 0);
}

#[test]
fn override_upsert_replaces_value() {
    let mut conn = setup_test_db();
    let run_path = setup_test_run(&mut conn);

    set_fact_override(
        &conn,
        &run_path,
        "empire",
        "origin",
        &json!("origin_a"),
        None,
    )
    .unwrap();
    set_fact_override(
        &conn,
        &run_path,
        "empire",
        "origin",
        &json!("origin_b"),
        Some("updated"),
    )
    .unwrap();

    let overrides = load_fact_overrides(&conn, &run_path).unwrap();
    assert_eq!(overrides.len(), 1);
    assert_eq!(overrides[0].value, json!("origin_b"));
    assert_eq!(overrides[0].reason, Some("updated".to_string()));
}

#[test]
fn load_run_facts_includes_override_flag() {
    let mut conn = setup_test_db();
    let run_path = setup_test_run(&mut conn);

    // The run has origin_default from the save. Override it.
    set_fact_override(
        &conn,
        &run_path,
        "empire",
        "origin",
        &json!("origin_synaptic"),
        None,
    )
    .unwrap();

    let facts = load_run_facts(&conn, &run_path).unwrap();
    let origin_fact = facts
        .iter()
        .find(|f| f.dimension == "empire" && f.key == "origin")
        .expect("origin fact should exist");
    assert!(origin_fact.is_override);
    assert_eq!(origin_fact.value, json!("origin_synaptic"));
    assert_eq!(origin_fact.source, "user_override");
}

#[test]
fn override_synthesizes_missing_fact() {
    let mut conn = setup_test_db();
    let run_path = setup_test_run(&mut conn);

    // Override a dimension/key that doesn't exist in parsed facts
    set_fact_override(
        &conn,
        &run_path,
        "progression",
        "fleet_power",
        &json!(50000),
        Some("from in-game observation"),
    )
    .unwrap();

    let facts = load_run_facts(&conn, &run_path).unwrap();
    let fleet_fact = facts
        .iter()
        .find(|f| f.dimension == "progression" && f.key == "fleet_power")
        .expect("synthesized override fact should exist");
    assert!(fleet_fact.is_override);
    assert_eq!(fleet_fact.value, json!(50000));
    assert_eq!(fleet_fact.source, "user_override");
    assert_eq!(fleet_fact.confidence, "high");
}

#[test]
fn planner_evaluation_uses_override() {
    let mut conn = setup_test_db();
    let run_path = setup_test_run(&mut conn);

    // Import a minimal catalog with one achievement that requires origin_synaptic
    let catalog_json = r#"{
        "catalog_version": "1.0.0",
        "snapshot_kind": "full",
        "stellaris_version": "4.3",
        "source_url": "",
        "source_hash": "",
        "updated_at": "2026-01-01",
        "achievements": [{
            "id": "test-achievement",
            "steam_app_id": 281990,
            "steam_api_name": "test_achievement",
            "local_key": null,
            "deprecated": false,
            "source": {
                "name": "Test Achievement",
                "description": "Test",
                "requirement": "Use synaptic origin",
                "hint": null,
                "group": "test",
                "version_added": "1.0",
                "difficulty": "E"
            },
            "curation": {
                "tags": ["test"],
                "conditions": [{
                    "condition_type": "required",
                    "dimension": "origin",
                    "operator": "equals",
                    "value": "origin_synaptic",
                    "timing": "setup",
                    "mutability": "immutable",
                    "severity": "hard",
                    "source": "test",
                    "notes": null
                }],
                "warnings": [],
                "known_limitations": [],
                "rule_confidence": "high"
            }
        }]
    }"#;

    let catalog = parse_catalog_json(catalog_json).unwrap();
    import_catalog(&mut conn, &catalog).unwrap();

    // Without override: origin is "origin_default" -> should be Incompatible
    let facts_no_override = load_run_facts(&conn, &run_path).unwrap();
    let entries = load_catalog_entries(&conn).unwrap();
    let completed: HashSet<String> = HashSet::new();
    let statuses = load_run_achievement_statuses(&conn, &run_path).unwrap();
    let evals =
        evaluate_planner_achievements(entries.clone(), &facts_no_override, &completed, &statuses);
    assert_eq!(evals[0].computed_status, "Impossible");

    // With override: origin is "origin_synaptic" -> should be Possible
    set_fact_override(
        &conn,
        &run_path,
        "empire",
        "origin",
        &json!("origin_synaptic"),
        None,
    )
    .unwrap();
    let facts_with_override = load_run_facts(&conn, &run_path).unwrap();
    let evals = evaluate_planner_achievements(entries, &facts_with_override, &completed, &statuses);
    assert_eq!(evals[0].computed_status, "Possible");
}
