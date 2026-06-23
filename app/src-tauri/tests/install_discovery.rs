use std::path::PathBuf;

use tempfile::tempdir;

/// Synthetic checksum_manifest.txt content used across multiple test scenarios.
fn checksum_content() -> &'static str {
    concat!(
        "# Comment line\n",
        "common *.txt\n",
        "common *.txt with subdirectories\n",
        r#"directory name = "common/" file_extension = ".txt""#,
        "\nevents *.txt\n",
    )
}

/// Synthetic launcher-settings.json content.
fn launcher_settings_content() -> &'static str {
    r#"{"version":"4.4.0","raw_version":"Cetus v4.4.0"}"#
}

/// Helper: creates a synthetic install directory and writes the given files
/// (relative path -> content). Returns the temp dir (keeps it alive).
fn setup_synthetic_install(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempdir().expect("tempdir should succeed");
    for (name, content) in files {
        std::fs::write(dir.path().join(name), content)
            .unwrap_or_else(|e| panic!("failed to write {name}: {e}"));
    }
    dir
}

// ---------------------------------------------------------------------------
// discover_install integration tests
// ---------------------------------------------------------------------------

#[test]
fn full_install_populates_all_fields() {
    let _dir = setup_synthetic_install(&[
        ("steam_appid.txt", "281990"),
        ("launcher-settings.json", launcher_settings_content()),
        ("checksum_manifest.txt", checksum_content()),
    ]);

    let summary = stellaris_hunter_scan::install::discover_install(Some(_dir.path().to_path_buf()))
        .expect("discover_install should not error")
        .expect("discover_install should find the install");

    assert_eq!(summary.steam_appid, Some(281_990));
    assert_eq!(summary.version.as_deref(), Some("4.4.0"));
    assert_eq!(summary.raw_version.as_deref(), Some("Cetus v4.4.0"));

    // 5 lines in manifest, 1 is a comment => 4 scopes
    assert_eq!(summary.checksum_manifest.len(), 4);

    assert!(
        summary.issues.is_empty(),
        "expected no issues, got: {:?}",
        summary.issues
    );
}

#[test]
fn missing_steam_appid_returns_none_for_appid() {
    let _dir = setup_synthetic_install(&[
        ("launcher-settings.json", launcher_settings_content()),
        ("checksum_manifest.txt", checksum_content()),
    ]);

    let summary = stellaris_hunter_scan::install::discover_install(Some(_dir.path().to_path_buf()))
        .expect("discover_install should not error")
        .expect("discover_install should find the install");

    assert_eq!(
        summary.steam_appid, None,
        "steam_appid should be None when file is missing"
    );
    assert_eq!(summary.version.as_deref(), Some("4.4.0"));
    assert_eq!(summary.checksum_manifest.len(), 4);
    // The code does not push an issue for missing steam_appid.txt, so issues should be empty.
    assert!(
        summary.issues.is_empty(),
        "expected no issues, got: {:?}",
        summary.issues
    );
}

#[test]
fn missing_launcher_settings_records_issue_and_none_version() {
    let _dir = setup_synthetic_install(&[
        ("steam_appid.txt", "281990"),
        ("checksum_manifest.txt", checksum_content()),
    ]);

    let summary = stellaris_hunter_scan::install::discover_install(Some(_dir.path().to_path_buf()))
        .expect("discover_install should not error")
        .expect("discover_install should find the install");

    assert_eq!(summary.steam_appid, Some(281_990));
    assert_eq!(
        summary.version, None,
        "version must be None when launcher-settings.json is missing"
    );
    assert_eq!(
        summary.raw_version, None,
        "raw_version must be None when launcher-settings.json is missing"
    );
    assert_eq!(summary.checksum_manifest.len(), 4);
    assert!(
        summary
            .issues
            .iter()
            .any(|i| i.contains("launcher-settings.json")),
        "expected issue mentioning launcher-settings.json, got: {:?}",
        summary.issues
    );
    // Only one issue expected.
    assert_eq!(summary.issues.len(), 1, "expected exactly one issue");
}

#[test]
fn missing_checksum_manifest_yields_empty_vec() {
    let _dir = setup_synthetic_install(&[
        ("steam_appid.txt", "281990"),
        ("launcher-settings.json", launcher_settings_content()),
    ]);

    let summary = stellaris_hunter_scan::install::discover_install(Some(_dir.path().to_path_buf()))
        .expect("discover_install should not error")
        .expect("discover_install should find the install");

    assert_eq!(summary.steam_appid, Some(281_990));
    assert_eq!(summary.version.as_deref(), Some("4.4.0"));
    assert!(
        summary.checksum_manifest.is_empty(),
        "checksum_manifest should be empty when file is missing"
    );
    assert!(
        summary.issues.is_empty(),
        "expected no issues, got: {:?}",
        summary.issues
    );
}

#[test]
fn empty_directory_returns_none_and_issue() {
    let _dir = setup_synthetic_install(&[]);

    let summary = stellaris_hunter_scan::install::discover_install(Some(_dir.path().to_path_buf()))
        .expect("discover_install should not error")
        .expect("discover_install should find the install (dir exists)");

    assert_eq!(summary.steam_appid, None);
    assert!(summary.checksum_manifest.is_empty());
    assert!(
        summary
            .issues
            .iter()
            .any(|i| i.contains("launcher-settings.json")),
        "expected issue mentioning launcher-settings.json, got: {:?}",
        summary.issues
    );
}

#[test]
fn non_existent_path_falls_through_to_default_candidates() {
    // When the explicit path doesn't exist, discover_install falls back to its
    // default candidate list and whatever installation it finds there (or None
    // if there is no Stellaris install at any default location). The important
    // contract is that the function never errors.
    let result = stellaris_hunter_scan::install::discover_install(Some(PathBuf::from(
        "/nonexistent/path/for/testing",
    )));
    assert!(
        result.is_ok(),
        "discover_install should never error on non-existent path"
    );
    // Note: result may be Ok(Some(...)) if a Stellaris install exists at a
    // default candidate path, or Ok(None) if none is found. Both are valid.
}
