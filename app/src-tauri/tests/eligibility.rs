use std::{fs, fs::File, io::Write, path::Path};

use rusqlite::Connection;
use stellaris_hunter_scan::{
    eligibility::compute_save_eligibility,
    model::{EligibilityConclusion, LauncherModSummary, ModChecksumRisk, SaveSummary},
};
use tempfile::tempdir;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[test]
fn uncheated_save_without_enabled_mods_is_likely_eligible() {
    let save = save_summary(Some(false), Some(false));

    let eligibility = compute_save_eligibility(&save, Some(&[]), Some(&[]), &[]);

    assert_eq!(
        eligibility.conclusion,
        EligibilityConclusion::LikelyEligible
    );
    assert_eq!(eligibility.mod_risk, ModChecksumRisk::None);
}

#[test]
fn cheated_save_is_likely_ineligible() {
    let save = save_summary(Some(true), Some(true));

    let eligibility = compute_save_eligibility(&save, Some(&[]), Some(&[]), &[]);

    assert_eq!(
        eligibility.conclusion,
        EligibilityConclusion::LikelyIneligible
    );
    assert!(eligibility
        .reasons
        .iter()
        .any(|reason| reason.contains("cheated_on_save=yes")));
}

#[test]
fn enabled_mods_keep_eligibility_unknown_until_checksum_impact_is_validated() {
    let save = save_summary(Some(true), Some(false));
    let mods = vec![LauncherModSummary {
        name: Some("Alpha Mod".to_string()),
        ..Default::default()
    }];

    let eligibility = compute_save_eligibility(&save, Some(&mods), Some(&[]), &[]);

    assert_eq!(eligibility.conclusion, EligibilityConclusion::Unknown);
    assert_eq!(eligibility.mod_risk, ModChecksumRisk::Unknown);
    assert!(eligibility
        .warnings
        .iter()
        .any(|warning| warning.contains("checksum impact is unvalidated")));
}

#[test]
fn scan_report_adds_per_run_eligibility_summary() {
    let dir = tempdir().unwrap();
    let install_root = dir.path().join("install");
    let documents_root = dir.path().join("documents");
    let run_root = documents_root.join("save games").join("run_a");
    fs::create_dir_all(&install_root).unwrap();
    fs::create_dir_all(&run_root).unwrap();

    fs::write(install_root.join("steam_appid.txt"), "281990\n").unwrap();
    fs::write(
        install_root.join("checksum_manifest.txt"),
        "directory name = \"common/\" file_extension = \".txt\" sub_directories = yes\n",
    )
    .unwrap();
    fs::write(
        documents_root.join("dlc_load.json"),
        r#"{"enabled_mods":[],"disabled_dlcs":[]}"#,
    )
    .unwrap();
    write_empty_launcher_db(&documents_root.join("launcher-v2.sqlite"));
    write_save_fixture(&run_root.join("ironman.sav"));

    let report = stellaris_hunter_scan::scan_all(Some(install_root), Some(documents_root));
    let run = report
        .documents
        .as_ref()
        .and_then(|documents| documents.save_runs.first())
        .expect("scan should discover the synthetic run");
    let eligibility = run
        .eligibility
        .as_ref()
        .expect("parsed save should receive an eligibility summary");

    assert_eq!(
        eligibility.conclusion,
        EligibilityConclusion::LikelyIneligible
    );
    assert_eq!(eligibility.cheated_on_save, Some(true));
}

#[test]
fn scan_report_treats_launcher_mod_query_failure_as_unknown_mod_state() {
    let dir = tempdir().unwrap();
    let install_root = dir.path().join("install");
    let documents_root = dir.path().join("documents");
    let run_root = documents_root.join("save games").join("run_a");
    fs::create_dir_all(&install_root).unwrap();
    fs::create_dir_all(&run_root).unwrap();

    fs::write(install_root.join("steam_appid.txt"), "281990\n").unwrap();
    write_broken_launcher_db(&documents_root.join("launcher-v2.sqlite"));
    write_save_fixture_with_cheat_flag(&run_root.join("ironman.sav"), false);

    let report = stellaris_hunter_scan::scan_all(Some(install_root), Some(documents_root));
    let eligibility = first_run_eligibility(&report);

    assert_eq!(eligibility.conclusion, EligibilityConclusion::Unknown);
    assert_eq!(eligibility.mod_risk, ModChecksumRisk::Unknown);
}

#[test]
fn scan_report_uses_dlc_load_mods_when_launcher_has_no_enabled_mods() {
    let dir = tempdir().unwrap();
    let install_root = dir.path().join("install");
    let documents_root = dir.path().join("documents");
    let run_root = documents_root.join("save games").join("run_a");
    fs::create_dir_all(&install_root).unwrap();
    fs::create_dir_all(&run_root).unwrap();

    fs::write(install_root.join("steam_appid.txt"), "281990\n").unwrap();
    fs::write(
        documents_root.join("dlc_load.json"),
        r#"{"enabled_mods":["mod/alpha.mod"],"disabled_dlcs":[]}"#,
    )
    .unwrap();
    write_empty_launcher_db(&documents_root.join("launcher-v2.sqlite"));
    write_save_fixture_with_cheat_flag(&run_root.join("ironman.sav"), false);

    let report = stellaris_hunter_scan::scan_all(Some(install_root), Some(documents_root));
    let eligibility = first_run_eligibility(&report);

    assert_eq!(eligibility.conclusion, EligibilityConclusion::Unknown);
    assert_eq!(eligibility.mod_risk, ModChecksumRisk::Unknown);
}

fn save_summary(ironman: Option<bool>, cheated_on_save: Option<bool>) -> SaveSummary {
    SaveSummary {
        ironman,
        cheated_on_save,
        ..Default::default()
    }
}

fn write_empty_launcher_db(path: &Path) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(
        "CREATE TABLE playsets (
            id TEXT,
            name TEXT,
            isActive INTEGER,
            syncState TEXT,
            state TEXT
        );

        CREATE TABLE mods (
            id TEXT,
            displayName TEXT,
            gameRegistryId TEXT,
            dirPath TEXT
        );

        CREATE TABLE playsets_mods (
            playsetId TEXT,
            modId TEXT,
            enabled INTEGER,
            position INTEGER
        );

        INSERT INTO playsets (id, name, isActive, syncState, state) VALUES
            ('playset-1', 'No Mods', 1, 'READY', 'private');",
    )
    .unwrap();
}

fn write_broken_launcher_db(path: &Path) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(
        "CREATE TABLE playsets (
            id TEXT,
            name TEXT,
            isActive INTEGER,
            syncState TEXT,
            state TEXT
        );

        CREATE TABLE mods (
            id TEXT,
            displayName TEXT,
            gameRegistryId TEXT,
            dirPath TEXT
        );

        INSERT INTO playsets (id, name, isActive, syncState, state) VALUES
            ('playset-1', 'Broken Mods Table', 1, 'READY', 'private');",
    )
    .unwrap();
}

fn write_save_fixture(path: &Path) {
    write_save_fixture_with_cheat_flag(path, true);
}

fn write_save_fixture_with_cheat_flag(path: &Path, cheated_on_save: bool) {
    let file = File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("meta", options).unwrap();
    zip.write_all(include_str!("fixtures/save/meta.txt").as_bytes())
        .unwrap();

    zip.start_file("gamestate", options).unwrap();
    let gamestate = include_str!("fixtures/save/gamestate.txt").replace(
        "cheated_on_save=yes",
        if cheated_on_save {
            "cheated_on_save=yes"
        } else {
            "cheated_on_save=no"
        },
    );
    zip.write_all(gamestate.as_bytes()).unwrap();

    zip.finish().unwrap();
}

fn first_run_eligibility(
    report: &stellaris_hunter_scan::ScanReport,
) -> &stellaris_hunter_scan::model::SaveEligibility {
    report
        .documents
        .as_ref()
        .and_then(|documents| documents.save_runs.first())
        .and_then(|run| run.eligibility.as_ref())
        .expect("scan should produce first-run eligibility")
}
