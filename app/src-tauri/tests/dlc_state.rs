use std::{fs, io::Write, path::Path};

use stellaris_hunter_scan::model::{
    classify_dlc_state, dlc_match_keys, normalize_dlc_id, normalize_readable_dlc_name,
};
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

#[test]
fn normalizes_subdirectory_disabled_dlc_path() {
    // dlc_load.json.disabled_dlcs can use subdirectory paths like
    // "dlc/dlc031_astral_planes/dlc031.dlc".  Must extract the DLC directory
    // name, not produce a malformed "dlc031_astral_planes/dlc031".
    assert_eq!(
        normalize_dlc_id("dlc/dlc031_astral_planes/dlc031.dlc"),
        "dlc031_astral_planes"
    );
    assert_eq!(
        normalize_dlc_id("dlc\\dlc032_cybernetics\\dlc032.dlc"),
        "dlc032_cybernetics"
    );
}

#[test]
fn normalizes_flat_dlc_path_unchanged() {
    // Flat .dlc files directly in dlc/ must still work
    assert_eq!(
        normalize_dlc_id("dlc/dlc014_leviathans.dlc"),
        "dlc014_leviathans"
    );
}

// ── Unit tests for normalize_readable_dlc_name ─────────────────────

#[test]
fn normalizes_story_pack_to_underscore_key() {
    assert_eq!(
        normalize_readable_dlc_name("Ancient Relics Story Pack"),
        "ancient_relics"
    );
    assert_eq!(
        normalize_readable_dlc_name("Leviathans Story Pack"),
        "leviathans"
    );
}

#[test]
fn normalizes_species_pack_to_underscore_key() {
    assert_eq!(
        normalize_readable_dlc_name("Plantoids Species Pack"),
        "plantoids"
    );
}

#[test]
fn normalizes_no_suffix_name_to_underscore_key() {
    assert_eq!(normalize_readable_dlc_name("Utopia"), "utopia");
    assert_eq!(normalize_readable_dlc_name("Apocalypse"), "apocalypse");
    assert_eq!(normalize_readable_dlc_name("MegaCorp"), "megacorp");
    assert_eq!(normalize_readable_dlc_name("Federations"), "federations");
}

#[test]
fn normalizes_multi_word_name_without_suffix() {
    assert_eq!(
        normalize_readable_dlc_name("The Machine Age"),
        "the_machine_age"
    );
    assert_eq!(
        normalize_readable_dlc_name("Galactic Paragons"),
        "galactic_paragons"
    );
}

#[test]
fn normalizes_canonical_ids_pass_through() {
    // Canonical/internal IDs should pass through unchanged
    assert_eq!(
        normalize_readable_dlc_name("dlc014_leviathans"),
        "dlc014_leviathans"
    );
    assert_eq!(
        normalize_readable_dlc_name("dlc009_plantoids"),
        "dlc009_plantoids"
    );
}

#[test]
fn dlc_match_keys_normalized_readable_name_matches_launcher_key() {
    // A normalized readable name ("ancient_relics") should have overlapping
    // match keys with the launcher/registry form ("dlc028_ancient_relics").
    let normalized_keys = dlc_match_keys("ancient_relics");
    let launcher_keys = dlc_match_keys("dlc028_ancient_relics");
    assert!(
        normalized_keys.iter().any(|k| launcher_keys.contains(k)),
        "normalized 'ancient_relics' keys {:?} should overlap launcher keys {:?}",
        normalized_keys,
        launcher_keys
    );
    assert!(
        normalized_keys.contains(&"ancient_relics".to_string()),
        "normalized keys should contain 'ancient_relics'"
    );
}

#[test]
fn dlc_match_keys_normalized_leviathans_matches_launcher_key() {
    let normalized_keys = dlc_match_keys("leviathans");
    let launcher_keys = dlc_match_keys("dlc014_leviathans");
    assert!(
        normalized_keys.iter().any(|k| launcher_keys.contains(k)),
        "normalized 'leviathans' keys {:?} should overlap launcher keys {:?}",
        normalized_keys,
        launcher_keys
    );
}

#[test]
fn dlc_match_keys_normalized_with_suffix_still_matches() {
    // "the_machine_age" should produce keys overlapping with a hypothetical
    // "dlc031_machine_age" via the shared "machine_age" suffix.
    let normalized_keys = dlc_match_keys("the_machine_age");
    let launcher_keys = dlc_match_keys("dlc031_machine_age");
    assert!(
        normalized_keys.iter().any(|k| launcher_keys.contains(k)),
        "normalized 'the_machine_age' keys {:?} should overlap launcher keys {:?}",
        normalized_keys,
        launcher_keys
    );
}

// ── Unit tests for dlc_match_keys ─────────────────────────────────

#[test]
fn dlc_match_keys_canonical_id_yields_full_and_short() {
    let keys = dlc_match_keys("dlc014_leviathans");
    assert!(
        keys.contains(&"dlc014_leviathans".to_string()),
        "should contain canonical form"
    );
    assert!(
        keys.contains(&"leviathans".to_string()),
        "should contain short suffix"
    );
}

#[test]
fn dlc_match_keys_short_id_yields_only_short() {
    let keys = dlc_match_keys("leviathans");
    assert!(
        keys.contains(&"leviathans".to_string()),
        "should contain short form"
    );
    assert_eq!(keys.len(), 1, "short ID should not generate extra keys");
}

#[test]
fn dlc_match_keys_plantoid_alias_round_trip() {
    // canonical suffix "plantoids" should produce alias "plantoid"
    let canonical_keys = dlc_match_keys("dlc009_plantoids");
    assert!(canonical_keys.contains(&"plantoids".to_string()));
    assert!(
        canonical_keys.contains(&"plantoid".to_string()),
        "plantoids should alias to plantoid"
    );

    // launcher short "plantoid" should reverse-alias to "plantoids"
    let launcher_keys = dlc_match_keys("plantoid");
    assert!(launcher_keys.contains(&"plantoid".to_string()));
    assert!(
        launcher_keys.contains(&"plantoids".to_string()),
        "plantoid should reverse-alias to plantoids"
    );
}

#[test]
fn dlc_match_keys_normalizes_input() {
    let keys = dlc_match_keys("DLC/DLC014_LEVIATHANS.DLC");
    assert!(keys.contains(&"dlc014_leviathans".to_string()));
    assert!(keys.contains(&"leviathans".to_string()));
}

#[test]
fn dlc_match_keys_utopia_no_alias() {
    let keys = dlc_match_keys("utopia");
    assert!(keys.contains(&"utopia".to_string()));
    assert_eq!(keys.len(), 1, "utopia has no known alias");
}

// ── Unit tests for DLC alias pairs ────────────────────────────────

#[test]
fn dlc_match_keys_anniversary_portraits_aliases_anniversary() {
    // Save form "anniversary_portraits" should alias to launcher "anniversary"
    let save_keys = dlc_match_keys("anniversary_portraits");
    assert!(save_keys.contains(&"anniversary_portraits".to_string()));
    assert!(
        save_keys.contains(&"anniversary".to_string()),
        "anniversary_portraits should alias to anniversary"
    );

    // Launcher form "anniversary" should reverse-alias to "anniversary_portraits"
    let launcher_keys = dlc_match_keys("anniversary");
    assert!(launcher_keys.contains(&"anniversary".to_string()));
    assert!(
        launcher_keys.contains(&"anniversary_portraits".to_string()),
        "anniversary should reverse-alias to anniversary_portraits"
    );
}

#[test]
fn dlc_match_keys_first_contact_aliases_firstcontact() {
    // Save form "first_contact" should alias to launcher "firstcontact"
    let save_keys = dlc_match_keys("first_contact");
    assert!(save_keys.contains(&"first_contact".to_string()));
    assert!(
        save_keys.contains(&"firstcontact".to_string()),
        "first_contact should alias to firstcontact"
    );

    // Launcher form "firstcontact" should reverse-alias to "first_contact"
    let launcher_keys = dlc_match_keys("firstcontact");
    assert!(launcher_keys.contains(&"firstcontact".to_string()));
    assert!(
        launcher_keys.contains(&"first_contact".to_string()),
        "firstcontact should reverse-alias to first_contact"
    );
}

#[test]
fn dlc_match_keys_galactic_paragons_aliases_paragon() {
    // Save form "galactic_paragons" should alias to launcher "paragon"
    let save_keys = dlc_match_keys("galactic_paragons");
    assert!(save_keys.contains(&"galactic_paragons".to_string()));
    assert!(
        save_keys.contains(&"paragon".to_string()),
        "galactic_paragons should alias to paragon"
    );

    // Launcher form "paragon" should reverse-alias to "galactic_paragons"
    let launcher_keys = dlc_match_keys("paragon");
    assert!(launcher_keys.contains(&"paragon".to_string()));
    assert!(
        launcher_keys.contains(&"galactic_paragons".to_string()),
        "paragon should reverse-alias to galactic_paragons"
    );
}

#[test]
fn dlc_match_keys_shadows_of_the_shroud_aliases_shadows_shroud() {
    // Save form "shadows_of_the_shroud" should alias to launcher "shadows_shroud"
    let save_keys = dlc_match_keys("shadows_of_the_shroud");
    assert!(save_keys.contains(&"shadows_of_the_shroud".to_string()));
    assert!(
        save_keys.contains(&"shadows_shroud".to_string()),
        "shadows_of_the_shroud should alias to shadows_shroud"
    );

    // Launcher form "shadows_shroud" should reverse-alias to "shadows_of_the_shroud"
    let launcher_keys = dlc_match_keys("shadows_shroud");
    assert!(launcher_keys.contains(&"shadows_shroud".to_string()));
    assert!(
        launcher_keys.contains(&"shadows_of_the_shroud".to_string()),
        "shadows_shroud should reverse-alias to shadows_of_the_shroud"
    );
}

#[test]
fn dlc_match_keys_rick_the_cube_species_portrait_aliases_rick_the_cube() {
    // Save form "rick_the_cube_species_portrait" should alias to launcher "rick_the_cube"
    let save_keys = dlc_match_keys("rick_the_cube_species_portrait");
    assert!(save_keys.contains(&"rick_the_cube_species_portrait".to_string()));
    assert!(
        save_keys.contains(&"rick_the_cube".to_string()),
        "rick_the_cube_species_portrait should alias to rick_the_cube"
    );

    // Launcher form "rick_the_cube" should reverse-alias to "rick_the_cube_species_portrait"
    let launcher_keys = dlc_match_keys("rick_the_cube");
    assert!(launcher_keys.contains(&"rick_the_cube".to_string()));
    assert!(
        launcher_keys.contains(&"rick_the_cube_species_portrait".to_string()),
        "rick_the_cube should reverse-alias to rick_the_cube_species_portrait"
    );
}

#[test]
fn dlc_match_keys_stargazer_species_portrait_aliases_stargazer() {
    // Save form "stargazer_species_portrait" should alias to launcher "stargazer"
    let save_keys = dlc_match_keys("stargazer_species_portrait");
    assert!(save_keys.contains(&"stargazer_species_portrait".to_string()));
    assert!(
        save_keys.contains(&"stargazer".to_string()),
        "stargazer_species_portrait should alias to stargazer"
    );

    // Launcher form "stargazer" should reverse-alias to "stargazer_species_portrait"
    let launcher_keys = dlc_match_keys("stargazer");
    assert!(launcher_keys.contains(&"stargazer".to_string()));
    assert!(
        launcher_keys.contains(&"stargazer_species_portrait".to_string()),
        "stargazer should reverse-alias to stargazer_species_portrait"
    );
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

#[test]
fn scan_report_works_with_empty_dlc_table_and_populated_playsets_dlcs() {
    // Real launcher databases can have 0 rows in `dlc` but many rows in
    // `playsets_dlcs` with short IDs (e.g. "leviathans", "utopia", "plantoid").
    // The app must still produce launcher DLC summaries and correctly match
    // save required DLCs against them.
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
    write_empty_dlc_table_db(&documents_root.join("launcher-v2.sqlite"));
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
        .expect("scan should produce dlc_info even when dlc table is empty");

    // Leviathans should match via short ID "leviathans" → canonical "dlc014_leviathans"
    assert!(
        dlc_info
            .enabled_and_required
            .contains(&"dlc014_leviathans".to_string()),
        "leviathans (enabled in playsets_dlcs) should match save-required dlc014_leviathans"
    );

    // Plantoids should match via alias "plantoid" → "plantoids" → "dlc009_plantoids"
    assert!(
        dlc_info.disabled_but_required.contains(&"dlc009_plantoids".to_string()),
        "plantoid (disabled in playsets_dlcs) should match save-required dlc009_plantoids via alias"
    );

    // Unknown DLC should still be unknown
    assert!(
        dlc_info
            .unknown_status_required
            .contains(&"dlc777_unknown_dlc".to_string()),
        "DLC not in launcher should appear as unknown"
    );
}

#[test]
fn scan_report_launcher_dlc_count_with_empty_dlc_table() {
    // When `dlc` table is empty but `playsets_dlcs` has entries, the
    // launcher state summary should still report the correct number of DLCs.
    let dir = tempdir().unwrap();
    let documents_root = dir.path().join("documents");
    fs::create_dir_all(&documents_root).unwrap();

    fs::write(
        documents_root.join("dlc_load.json"),
        r#"{"enabled_mods":[],"disabled_dlcs":[]}"#,
    )
    .unwrap();
    write_empty_dlc_table_db(&documents_root.join("launcher-v2.sqlite"));

    let report = stellaris_hunter_scan::scan_all(None, Some(documents_root));
    let launcher = report
        .documents
        .as_ref()
        .and_then(|d| d.launcher.as_ref())
        .expect("launcher state should be present");

    // Should still report 3 DLCs (from playsets_dlcs entries)
    assert_eq!(
        launcher.dlcs.len(),
        3,
        "should have 3 DLC summaries even when dlc table is empty"
    );
}

#[test]
fn scan_report_matches_enablement_when_catalog_and_playset_keys_differ() {
    // `dlc` table uses opaque/UUID-like IDs that don't match
    // `playsets_dlcs.dlcId` (which uses short launcher IDs like "leviathans").
    // Enablement must be resolved via match keys derived from catalog row
    // fields (gameRegistryId/dirPath/name/id), not via raw hash key lookup.
    let dir = tempdir().unwrap();
    let documents_root = dir.path().join("documents");
    let run_root = documents_root.join("save games").join("run_a");
    fs::create_dir_all(&run_root).unwrap();

    fs::write(
        documents_root.join("dlc_load.json"),
        r#"{
            "enabled_mods": ["mod/ugc_123.mod"],
            "disabled_dlcs": []
        }"#,
    )
    .unwrap();
    write_mismatched_dlc_ids_db(&documents_root.join("launcher-v2.sqlite"));
    write_save_fixture(
        &run_root.join("ironman.sav"),
        &[
            "dlc014_leviathans",
            "dlc009_plantoids",
            "dlc028_ancient_relics",
            "dlc777_unknown_dlc",
        ],
    );

    let report = stellaris_hunter_scan::scan_all(None, Some(documents_root));
    let launcher = report
        .documents
        .as_ref()
        .and_then(|d| d.launcher.as_ref())
        .expect("launcher state should be present");

    // Must not produce duplicates: 3 catalog rows, 3 playsets_dlcs rows,
    // but each playsets_dlcs key is already covered by a catalog entry
    // via match keys — so total should be 3, not 6.
    assert_eq!(
        launcher.dlcs.len(),
        3,
        "should have 3 launcher DLCs, not {} — supplement must not duplicate catalog DLCs that already cover the same playset key via match key",
        launcher.dlcs.len()
    );

    // Leviathans — match via gameRegistryId "dlc014_leviathans"
    let leviathans = launcher
        .dlcs
        .iter()
        .find(|d| d.name.as_deref() == Some("Leviathans Story Pack"))
        .expect("leviathans should be in launcher DLCs");
    assert_eq!(
        leviathans.enabled_in_active_playset,
        Some(true),
        "leviathans enabled_in_active_playset should be Some(true) via match key"
    );

    // Plantoids — match via gameRegistryId "dlc009_plantoids" → short "plantoid" alias
    let plantoids = launcher
        .dlcs
        .iter()
        .find(|d| d.name.as_deref() == Some("Plantoids Species Pack"))
        .expect("plantoids should be in launcher DLCs");
    assert_eq!(
        plantoids.enabled_in_active_playset,
        Some(false),
        "plantoids enabled_in_active_playset should be Some(false) via alias match"
    );

    // Ancient Relics — match via dirPath "dlc/dlc028_ancient_relics/"
    let relics = launcher
        .dlcs
        .iter()
        .find(|d| d.name.as_deref() == Some("Ancient Relics Story Pack"))
        .expect("ancient relics should be in launcher DLCs");
    assert_eq!(
        relics.enabled_in_active_playset,
        Some(true),
        "ancient relics enabled_in_active_playset should be Some(true) via path match"
    );

    // Also verify save-required DLC classification works end-to-end
    let run = report
        .documents
        .as_ref()
        .and_then(|d| d.save_runs.first())
        .expect("scan should discover the synthetic run");
    let dlc_info = run.dlc_info.as_ref().expect("scan should produce dlc_info");

    assert!(
        dlc_info
            .enabled_and_required
            .contains(&"dlc014_leviathans".to_string()),
        "leviathans should be enabled_and_required"
    );
    assert!(
        dlc_info
            .disabled_but_required
            .contains(&"dlc009_plantoids".to_string()),
        "plantoids should be disabled_but_required"
    );
    assert!(
        dlc_info
            .enabled_and_required
            .contains(&"dlc028_ancient_relics".to_string()),
        "ancient relics should be enabled_and_required"
    );
    assert!(
        dlc_info
            .unknown_status_required
            .contains(&"dlc777_unknown_dlc".to_string()),
        "unknown dlc should remain unknown"
    );
}

#[test]
fn scan_report_matches_human_readable_save_dlc_against_canonical_launcher_keys() {
    // Save required_dlcs use human-readable names like "Ancient Relics Story Pack"
    // which get normalized to "ancient_relics" in save.rs.  The launcher uses
    // canonical IDs like "dlc028_ancient_relics".  add_dlc_info_summaries must
    // expand both sides (not just the save side) to find the overlap.
    let dir = tempdir().unwrap();
    let documents_root = dir.path().join("documents");
    let run_root = documents_root.join("save games").join("run_a");
    fs::create_dir_all(&run_root).unwrap();

    fs::write(
        documents_root.join("dlc_load.json"),
        r#"{
            "enabled_mods": ["mod/ugc_123.mod"],
            "disabled_dlcs": []
        }"#,
    )
    .unwrap();
    // Launcher with canonical dirPath values → match keys like "dlc014_leviathans"
    write_real_windows_launcher_db(&documents_root.join("launcher-v2.sqlite"));
    // Save fixture with HUMAN-READABLE required_dlcs (not canonical IDs)
    write_save_fixture(
        &run_root.join("ironman.sav"),
        &[
            "Leviathans Story Pack",
            "Ancient Relics Story Pack",
            "Plantoids Species Pack",
            "Some Unknown DLC",
        ],
    );

    let report = stellaris_hunter_scan::scan_all(None, Some(documents_root));
    let launcher = report
        .documents
        .as_ref()
        .and_then(|d| d.launcher.as_ref())
        .expect("launcher state should be present");
    assert_eq!(launcher.dlcs.len(), 3, "launcher should have 3 DLCs");

    let run = report
        .documents
        .as_ref()
        .and_then(|d| d.save_runs.first())
        .expect("scan should discover the synthetic run");
    let dlc_info = run.dlc_info.as_ref().expect("scan should produce dlc_info");

    // After save normalization: "Leviathans Story Pack" → "leviathans"
    // Enabled list has "dlc014_leviathans" → expanded via dlc_match_keys includes "leviathans"
    assert!(
        dlc_info
            .enabled_and_required
            .contains(&"leviathans".to_string()),
        "leviathans (normalized from 'Leviathans Story Pack') should match enabled launcher dlc014_leviathans"
    );

    // After save normalization: "Ancient Relics Story Pack" → "ancient_relics"
    // Enabled list has "dlc028_ancient_relics" → expanded includes "ancient_relics"
    assert!(
        dlc_info
            .enabled_and_required
            .contains(&"ancient_relics".to_string()),
        "ancient_relics (normalized from 'Ancient Relics Story Pack') should match enabled launcher dlc028_ancient_relics"
    );

    // After save normalization: "Plantoids Species Pack" → "plantoids"
    // Disabled list has "dlc009_plantoids" → expanded includes "plantoids"
    assert!(
        dlc_info
            .disabled_but_required
            .contains(&"plantoids".to_string()),
        "plantoids (normalized from 'Plantoids Species Pack') should match disabled launcher dlc009_plantoids"
    );

    // Unknown DLC should remain unknown
    assert!(
        dlc_info
            .unknown_status_required
            .contains(&"some_unknown_dlc".to_string()),
        "unknown DLC should remain unknown"
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

/// Write a launcher DB with an empty `dlc` table but populated `playsets_dlcs`
/// with short launcher IDs (e.g. "leviathans", "utopia", "plantoid").
/// This reproduces the real-world scenario where the `dlc` table has 0 rows.
fn write_empty_dlc_table_db(path: &Path) {
    let conn = rusqlite::Connection::open(path).unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE playsets (
            playsetId TEXT,
            name TEXT,
            isActive INTEGER,
            syncState TEXT,
            state TEXT
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

        INSERT INTO playsets (playsetId, name, isActive, syncState, state) VALUES
            ('playset-A', 'Active Playset', 1, 'NOT_ELIGIBLE', 'private');

        -- dlc table deliberately left empty

        INSERT INTO playsets_dlcs (playsetId, dlcId, enabled) VALUES
            ('playset-A', 'leviathans', 1),
            ('playset-A', 'utopia', 1),
            ('playset-A', 'plantoid', 0);
        "#,
    )
    .unwrap();
}

/// Launcher DB where `dlc.id` uses opaque IDs (e.g. "opaque-001") that
/// do NOT match `playsets_dlcs.dlcId` (which uses short keys like "leviathans").
/// Enablement must be resolved via match keys derived from catalog row
/// fields (gameRegistryId, dirPath, name, id) rather than raw hash key lookup.
fn write_mismatched_dlc_ids_db(path: &Path) {
    let conn = rusqlite::Connection::open(path).unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE playsets (
            playsetId TEXT,
            name TEXT,
            isActive INTEGER,
            syncState TEXT,
            state TEXT
        );

        CREATE TABLE dlc (
            id TEXT,
            name TEXT,
            gameRegistryId TEXT,
            dirPath TEXT
        );

        CREATE TABLE playsets_dlcs (
            playsetId TEXT,
            dlcId TEXT,
            enabled INTEGER
        );

        INSERT INTO playsets (playsetId, name, isActive, syncState, state) VALUES
            ('ps-main', 'Main', 1, 'NOT_ELIGIBLE', 'private');

        -- dlc.id uses opaque IDs; match keys come from gameRegistryId or dirPath
        INSERT INTO dlc (id, name, gameRegistryId, dirPath) VALUES
            ('opaque-001', 'Leviathans Story Pack', 'dlc014_leviathans', 'dlc/dlc014_leviathans/'),
            ('opaque-002', 'Plantoids Species Pack', 'dlc009_plantoids', 'dlc/dlc009_plantoids/'),
            ('opaque-003', 'Ancient Relics Story Pack', 'dlc028_ancient_relics', 'dlc/dlc028_ancient_relics/');

        -- playsets_dlcs.dlcId uses SHORT IDs, NOT matching dlc.id
        INSERT INTO playsets_dlcs (playsetId, dlcId, enabled) VALUES
            ('ps-main', 'leviathans', 1),
            ('ps-main', 'plantoid', 0),
            ('ps-main', 'ancient_relics', 1);
        "#,
    )
    .unwrap();
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
