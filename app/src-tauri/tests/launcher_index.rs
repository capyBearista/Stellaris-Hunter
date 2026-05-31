use std::{fs, path::Path};

use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn reads_launcher_state_from_synthetic_fixtures() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(
        root.join("continue_game.json"),
        include_str!("fixtures/launcher/continue_game.json"),
    )
    .unwrap();
    fs::write(
        root.join("dlc_load.json"),
        include_str!("fixtures/launcher/dlc_load.json"),
    )
    .unwrap();
    write_launcher_db(&root.join("launcher-v2.sqlite"));
    let bad_run = root.join("save games").join("bad_run");
    fs::create_dir_all(&bad_run).unwrap();
    fs::write(bad_run.join("ironman.sav"), b"not a zip").unwrap();

    let summary = stellaris_hunter_scan::documents::discover_documents(Some(root.to_path_buf()))
        .expect("documents discovery should not error")
        .expect("documents should be found");

    assert_eq!(
        summary
            .continue_game
            .as_ref()
            .and_then(|t| t.title.as_deref()),
        Some("save games/run_a/ironman")
    );
    assert_eq!(
        summary.dlc_load.as_ref().map(|d| d.enabled_mods.len()),
        Some(2)
    );
    assert_eq!(
        summary
            .launcher
            .as_ref()
            .and_then(|l| l.active_playset.as_ref())
            .and_then(|p| p.name.as_deref()),
        Some("Bare Bones")
    );
    assert_eq!(
        summary.launcher.as_ref().map(|l| l.enabled_mods.len()),
        Some(2)
    );
    let bad_run = summary
        .save_runs
        .iter()
        .find(|run| run.run_folder == "bad_run")
        .expect("bad run should still be indexed");
    assert!(bad_run.latest_save.is_none());
    assert_eq!(bad_run.issues.len(), 1);
}

fn write_launcher_db(path: &Path) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(include_str!("fixtures/launcher/launcher.sql"))
        .unwrap();
}
