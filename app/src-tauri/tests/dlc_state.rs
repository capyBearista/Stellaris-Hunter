use std::{fs, io::Write, path::Path};

use stellaris_hunter_scan::model::{classify_dlc_state, normalize_dlc_id};
use tempfile::tempdir;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

// ── Unit tests for normalize_dlc_id ───────────────────────────────

#[test]
fn normalizes_dlc_path_with_prefix_and_extension() {
    assert_eq!(
        normalize_dlc_id("dlc/dlc014_leviathans.dlc"),
        "dlc014_leviathans"
    );
}

#[test]
fn normalizes_dlc_path_with_backslash() {
    assert_eq!(
        normalize_dlc_id("dlc\\dlc014_leviathans.dlc"),
        "dlc014_leviathans"
    );
}

#[test]
fn normalizes_short_dlc_id_without_path() {
    assert_eq!(
        normalize_dlc_id("dlc028_ancient_relics"),
        "dlc028_ancient_relics"
    );
}

#[test]
fn normalizes_case_insensitively() {
    assert_eq!(normalize_dlc_id("DLC/DLC_ALPHA.DLC"), "dlc_alpha");
}

#[test]
fn normalizes_save_required_dlc() {
    // Save required_dlcs entries look like "dlc_alpha" or "dlc_beta"
    assert_eq!(normalize_dlc_id("dlc_alpha"), "dlc_alpha");
}

#[test]
fn normalizes_plain_name() {
    // Some DLC entries might not have the dlc_ prefix
    assert_eq!(normalize_dlc_id("leviathans"), "leviathans");
}

#[test]
fn normalizes_trailing_slash_from_dirpath() {
    // Real launcher dirPath values often end with a trailing slash
    assert_eq!(
        normalize_dlc_id("dlc/dlc009_plantoids/"),
        "dlc009_plantoids"
    );
    assert_eq!(
        normalize_dlc_id("dlc/dlc028_ancient_relics/"),
        "dlc028_ancient_relics"
    );
}

#[test]
fn normalizes_trailing_backslash_from_dirpath() {
    assert_eq!(
        normalize_dlc_id("dlc\\dlc009_plantoids\\"),
        "dlc009_plantoids"
    );
}

#[test]
fn normalizes_filename_extracted_from_dirpath() {
    // When only the last path component is passed (e.g., after extracting filename)
    assert_eq!(normalize_dlc_id("dlc009_plantoids/"), "dlc009_plantoids");
}

// ── Unit tests for classify_dlc_state ─────────────────────────────

#[test]
fn classifies_dlc_entries_from_enabled_mods() {
    let enabled_mods: Vec<String> = vec![
        "dlc/dlc014_leviathans.dlc".to_string(),
        "dlc/dlc028_ancient_relics.dlc".to_string(),
        "mod/alpha.mod".to_string(),
        "mod/beta.mod".to_string(),
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let disabled_dlcs: Vec<String> = vec!["dlc009_plantoids".to_string()];

    let state = classify_dlc_state(&enabled_mods, &disabled_dlcs);

    assert_eq!(
        state.enabled_dlcs,
        vec![
            "dlc014_leviathans".to_string(),
            "dlc028_ancient_relics".to_string(),
        ]
    );
    assert_eq!(state.disabled_dlcs, vec!["dlc009_plantoids".to_string()]);
}

#[test]
fn classifies_no_dlc_when_enabled_mods_empty() {
    let enabled_mods: Vec<String> = vec![];
    let disabled_dlcs: Vec<String> = vec![];

    let state = classify_dlc_state(&enabled_mods, &disabled_dlcs);

    assert!(state.enabled_dlcs.is_empty());
    assert!(state.disabled_dlcs.is_empty());
}

#[test]
fn classifies_no_dlc_when_only_mods_present() {
    let enabled_mods: Vec<String> = vec!["mod/alpha.mod".to_string(), "mod/beta.mod".to_string()];
    let disabled_dlcs: Vec<String> = vec![];

    let state = classify_dlc_state(&enabled_mods, &disabled_dlcs);

    assert!(state.enabled_dlcs.is_empty());
    assert!(state.disabled_dlcs.is_empty());
}

#[test]
fn classifies_dlc_with_mixed_case() {
    let enabled_mods: Vec<String> = vec!["DLC/DLC014_LEVIATHANS.DLC".to_string()];
    let disabled_dlcs: Vec<String> = vec!["DLC028_ANCIENT_RELICS".to_string()];

    let state = classify_dlc_state(&enabled_mods, &disabled_dlcs);

    assert_eq!(state.enabled_dlcs, vec!["dlc014_leviathans"]);
    assert_eq!(state.disabled_dlcs, vec!["dlc028_ancient_relics"]);
}

#[test]
fn classifies_dlc_lists_without_duplicates() {
    let enabled_mods: Vec<String> = vec![
        "dlc/dlc014_leviathans.dlc".to_string(),
        "dlc/dlc014_leviathans.dlc".to_string(),
    ];
    let disabled_dlcs: Vec<String> = vec![
        "dlc009_plantoids".to_string(),
        "dlc009_plantoids".to_string(),
    ];

    let state = classify_dlc_state(&enabled_mods, &disabled_dlcs);

    assert_eq!(state.enabled_dlcs, vec!["dlc014_leviathans"]);
    assert_eq!(state.disabled_dlcs, vec!["dlc009_plantoids"]);
}

// ── Integration test with scan_all ────────────────────────────────

#[test]
fn scan_report_adds_dlc_info_for_runs() {
    let dir = tempdir().unwrap();
    let install_root = dir.path().join("install");
    let documents_root = dir.path().join("documents");
    let run_root = documents_root.join("save games").join("run_a");
    fs::create_dir_all(&install_root).unwrap();
    fs::create_dir_all(&run_root).unwrap();

    // Write install marker
    fs::write(install_root.join("steam_appid.txt"), "281990\n").unwrap();

    // Write dlc_load.json with mixed DLCs and mods
    fs::write(
        documents_root.join("dlc_load.json"),
        r#"{
            "enabled_mods": [
                "dlc/dlc014_leviathans.dlc",
                "dlc/dlc028_ancient_relics.dlc",
                "mod/alpha.mod"
            ],
            "disabled_dlcs": [
                "dlc009_plantoids",
                "dlc016_synthetic_dawn"
            ]
        }"#,
    )
    .unwrap();

    // Write a save file whose required_dlcs overlap the enabled and disabled lists
    write_save_fixture(
        &run_root.join("ironman.sav"),
        &[
            "dlc014_leviathans",
            "dlc009_plantoids",
            "dlc777_unknown_dlc",
        ],
    );

    let report = stellaris_hunter_scan::scan_all(Some(install_root), Some(documents_root));
    let run = report
        .documents
        .as_ref()
        .and_then(|d| d.save_runs.first())
        .expect("scan should discover the synthetic run");

    let dlc_info = run
        .dlc_info
        .as_ref()
        .expect("scan should produce dlc_info for the run");

    // dlc014_leviathans is enabled in dlc_load.json
    assert_eq!(
        dlc_info.enabled_and_required,
        vec!["dlc014_leviathans"],
        "DLC that save requires AND is enabled should be in enabled_and_required"
    );

    // dlc009_plantoids is disabled in dlc_load.json
    assert_eq!(
        dlc_info.disabled_but_required,
        vec!["dlc009_plantoids"],
        "DLC that save requires but is disabled should be in disabled_but_required"
    );

    // dlc777_unknown_dlc is neither enabled nor disabled
    assert_eq!(
        dlc_info.unknown_status_required,
        vec!["dlc777_unknown_dlc"],
        "DLC that save requires but is unknown should be in unknown_status_required"
    );

    // all_enabled_dlcs reflects the full list from dlc_load.json
    assert_eq!(
        dlc_info.all_enabled_dlcs,
        vec!["dlc014_leviathans", "dlc028_ancient_relics"],
        "all_enabled_dlcs should include all DLCs detected from enabled_mods"
    );

    // all_disabled_dlcs reflects the full list from dlc_load.json
    assert_eq!(
        dlc_info.all_disabled_dlcs,
        vec!["dlc009_plantoids", "dlc016_synthetic_dawn"],
        "all_disabled_dlcs should include all entries from disabled_dlcs"
    );
}

#[test]
fn scan_report_dlc_info_is_none_when_no_save_parsed() {
    let dir = tempdir().unwrap();
    let documents_root = dir.path().join("documents");
    let bad_run_root = documents_root.join("save games").join("bad_run");
    fs::create_dir_all(&bad_run_root).unwrap();

    // Write a broken save file
    fs::write(bad_run_root.join("ironman.sav"), b"not a zip").unwrap();

    // Write dlc_load.json
    fs::write(
        documents_root.join("dlc_load.json"),
        r#"{"enabled_mods":["dlc/dlc014_leviathans.dlc"],"disabled_dlcs":[]}"#,
    )
    .unwrap();

    let report = stellaris_hunter_scan::scan_all(None, Some(documents_root));
    let run = report
        .documents
        .as_ref()
        .and_then(|d| d.save_runs.first())
        .expect("scan should discover the bad run");

    // Dlc info should be None because no save was parsed
    assert!(
        run.dlc_info.is_none(),
        "dlc_info should be None when latest_save is None"
    );
}

#[test]
fn scan_report_marks_required_dlcs_unknown_when_no_local_dlc_state_exists() {
    let dir = tempdir().unwrap();
    let documents_root = dir.path().join("documents");
    let run_root = documents_root.join("save games").join("run_a");
    fs::create_dir_all(&run_root).unwrap();

    // No dlc_load.json
    write_save_fixture(&run_root.join("ironman.sav"), &["dlc_alpha"]);

    let report = stellaris_hunter_scan::scan_all(None, Some(documents_root));
    let run = report
        .documents
        .as_ref()
        .and_then(|d| d.save_runs.first())
        .expect("scan should discover the run");

    let dlc_info = run
        .dlc_info
        .as_ref()
        .expect("save-required DLCs should still surface as unknown without local DLC state");

    assert!(dlc_info.enabled_and_required.is_empty());
    assert!(dlc_info.disabled_but_required.is_empty());
    assert_eq!(dlc_info.unknown_status_required, vec!["dlc_alpha"]);
}

#[test]
fn scan_report_uses_launcher_dlc_state_when_dlc_load_only_lists_mods() {
    let dir = tempdir().unwrap();
    let documents_root = dir.path().join("documents");
    let run_root = documents_root.join("save games").join("run_a");
    fs::create_dir_all(&run_root).unwrap();

    fs::write(
        documents_root.join("dlc_load.json"),
        r#"{
            "enabled_mods": [
                "mod/ugc_682090850.mod",
                "mod/ugc_697938908.mod"
            ],
            "disabled_dlcs": []
        }"#,
    )
    .unwrap();
    write_launcher_db(&documents_root.join("launcher-v2.sqlite"));
    write_save_fixture(
        &run_root.join("ironman.sav"),
        &[
            "dlc014_leviathans",
            "dlc009_plantoids",
            "dlc777_unknown_dlc",
        ],
    );

    let report = stellaris_hunter_scan::scan_all(None, Some(documents_root));
    let run = report
        .documents
        .as_ref()
        .and_then(|d| d.save_runs.first())
        .expect("scan should discover the synthetic run");

    let dlc_info = run
        .dlc_info
        .as_ref()
        .expect("scan should still produce dlc_info from launcher tables");

    assert_eq!(dlc_info.enabled_and_required, vec!["dlc014_leviathans"]);
    assert_eq!(dlc_info.disabled_but_required, vec!["dlc009_plantoids"]);
    assert_eq!(dlc_info.unknown_status_required, vec!["dlc777_unknown_dlc"]);
}

#[test]
fn scan_report_uses_launcher_with_playsetid_column() {
    // Real Windows launcher schema uses "playsetId" (not "id") as the
    // playsets primary key.  This test verifies the `playsetId` fallback
    // in read_active_playset() and the trailing-slash normalization in
    // launcher_dlc_match_key() / normalize_dlc_id().
    let dir = tempdir().unwrap();
    let documents_root = dir.path().join("documents");
    let run_root = documents_root.join("save games").join("run_a");
    fs::create_dir_all(&run_root).unwrap();

    fs::write(
        documents_root.join("dlc_load.json"),
        r#"{
            "enabled_mods": [
                "mod/ugc_682090850.mod",
                "mod/ugc_697938908.mod"
            ],
            "disabled_dlcs": []
        }"#,
    )
    .unwrap();
    write_real_windows_launcher_db(&documents_root.join("launcher-v2.sqlite"));
    write_save_fixture(
        &run_root.join("ironman.sav"),
        &[
            "dlc014_leviathans",
            "dlc009_plantoids",
            "dlc777_unknown_dlc",
        ],
    );

    let report = stellaris_hunter_scan::scan_all(None, Some(documents_root));
    let run = report
        .documents
        .as_ref()
        .and_then(|d| d.save_runs.first())
        .expect("scan should discover the synthetic run");

    let dlc_info = run
        .dlc_info
        .as_ref()
        .expect("scan should produce dlc_info from launcher tables");

    // Leviathans is enabled (enabled=1 in playsets_dlcs)
    assert_eq!(
        dlc_info.enabled_and_required,
        vec!["dlc014_leviathans"],
        "enabled DLC from real-windows schema should appear as enabled_and_required"
    );

    // Plantoids is disabled (enabled=0 in playsets_dlcs)
    assert_eq!(
        dlc_info.disabled_but_required,
        vec!["dlc009_plantoids"],
        "disabled DLC from real-windows schema should appear as disabled_but_required"
    );

    // Unknown DLC should still be unknown
    assert_eq!(
        dlc_info.unknown_status_required,
        vec!["dlc777_unknown_dlc"],
        "DLC not in launcher should appear as unknown"
    );
}

// ── Helpers ────────────────────────────────────────────────────────

fn write_save_fixture(path: &Path, required_dlcs: &[&str]) {
    let file = fs::File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("meta", options).unwrap();
    let dlc_list: String = required_dlcs
        .iter()
        .map(|d| format!(" \"{}\"", d))
        .collect();
    let meta = format!(
        r#"version="Cetus v4.3.7"
date="2532.01.26"
name="Test Run"
required_dlcs={{ {} }}
ironman=yes"#,
        dlc_list
    );
    zip.write_all(meta.as_bytes()).unwrap();

    zip.start_file("gamestate", options).unwrap();
    zip.write_all(b"cheated_on_save=no\n").unwrap();

    zip.finish().unwrap();
}

fn write_launcher_db(path: &Path) {
    let conn = rusqlite::Connection::open(path).unwrap();
    conn.execute_batch(include_str!("fixtures/launcher/launcher.sql"))
        .unwrap();
}

/// Write a launcher DB matching the real Windows schema that the user
/// reported: `playsets(playsetId, name, isActive)` and
/// `dlc(id, name, dirPath)` with no `gameRegistryId` column,
/// `dirPath` values ending with `/` (not `.dlc` extension).
fn write_real_windows_launcher_db(path: &Path) {
    let conn = rusqlite::Connection::open(path).unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE playsets (
            playsetId TEXT,
            name TEXT,
            isActive INTEGER
        );

        CREATE TABLE dlc (
            id TEXT,
            name TEXT,
            dirPath TEXT
        );

        CREATE TABLE playsets_dlcs (
            playsetId TEXT,
            dlcId TEXT,
            enabled INTEGER
        );

        INSERT INTO playsets (playsetId, name, isActive) VALUES
            ('playset-A', 'Active Playset', 1);

        INSERT INTO dlc (id, name, dirPath) VALUES
            ('dlc-1', 'Leviathans Story Pack', 'dlc/dlc014_leviathans/'),
            ('dlc-2', 'Plantoids Species Pack', 'dlc/dlc009_plantoids/'),
            ('dlc-3', 'Ancient Relics Story Pack', 'dlc/dlc028_ancient_relics/');

        INSERT INTO playsets_dlcs (playsetId, dlcId, enabled) VALUES
            ('playset-A', 'dlc-1', 1),
            ('playset-A', 'dlc-2', 0),
            ('playset-A', 'dlc-3', 1);
        "#,
    )
    .unwrap();
}
