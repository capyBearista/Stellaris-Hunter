use std::{fs, path::Path};

use walkdir::WalkDir;

#[test]
fn steam_source_contains_no_mutating_api_names_outside_guard() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let forbidden = [
        "SetAchievement",
        "ClearAchievement",
        "StoreStats",
        "ResetAllStats",
        "SetStat",
        "UpdateAvgRateStat",
        "IndicateAchievementProgress",
    ];

    for entry in WalkDir::new(&root) {
        let entry = entry.unwrap();
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.file_name().and_then(|name| name.to_str()) == Some("guard.rs") {
            continue;
        }
        let text = fs::read_to_string(path).unwrap();
        for needle in forbidden {
            assert!(
                !text.contains(needle),
                "found forbidden Steam mutator {needle} in {}",
                path.display()
            );
        }
    }
}
