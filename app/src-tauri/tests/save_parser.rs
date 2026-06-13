use std::{fs::File, io::Write};

use tempfile::tempdir;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[test]
fn parses_synthetic_save_zip() {
    let dir = tempdir().unwrap();
    let save_path = dir.path().join("synthetic.sav");
    write_save_fixture(&save_path);

    let summary = stellaris_hunter_scan::save::parse_save(&save_path).expect("save should parse");

    assert_eq!(summary.version.as_deref(), Some("Cetus v4.3.7"));
    assert_eq!(summary.date.as_deref(), Some("2532.01.26"));
    assert_eq!(summary.name.as_deref(), Some("Synthetic Run"));
    assert_eq!(summary.required_dlcs, vec!["dlc_alpha", "dlc_beta"]);
    assert_eq!(summary.ironman, Some(true));
    assert_eq!(summary.cheated_on_save, Some(true));
    assert_eq!(summary.player_country.as_deref(), Some("42"));
    assert_eq!(summary.authority.as_deref(), Some("auth_democratic"));
    assert_eq!(
        summary.government_type.as_deref(),
        Some("gov_cyber_democracy_individualist")
    );
    assert_eq!(summary.origin.as_deref(), Some("origin_default"));
    assert_eq!(
        summary.ethics,
        vec!["ethic_fanatic_egalitarian", "ethic_xenophile"]
    );
    assert_eq!(
        summary.civics,
        vec!["civic_beacon_of_liberty", "civic_meritocracy"]
    );
    assert_eq!(summary.founder_species_ref.as_deref(), Some("99"));
    assert_eq!(summary.founder_species_class.as_deref(), Some("HUM"));
    assert_eq!(summary.founder_species_portrait.as_deref(), Some("human"));
    assert_eq!(
        summary.founder_species_traits,
        vec!["trait_adaptive", "trait_nomadic"]
    );

    // Verify extraction modules produce non-None results (fixture is minimal
    // so individual fields will be mostly defaults, but the wiring is live)
    assert!(
        summary.discovery.is_some(),
        "discovery facts should be extracted"
    );
    assert!(
        summary.progression.is_some(),
        "progression facts should be extracted"
    );
    assert!(
        summary.actions.is_some(),
        "action facts should be extracted"
    );
}

fn write_save_fixture(path: &std::path::Path) {
    let file = File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("meta", options).unwrap();
    zip.write_all(include_str!("fixtures/save/meta.txt").as_bytes())
        .unwrap();

    zip.start_file("gamestate", options).unwrap();
    zip.write_all(include_str!("fixtures/save/gamestate.txt").as_bytes())
        .unwrap();

    zip.finish().unwrap();
}
