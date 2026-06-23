use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::{
    error::Result,
    model::{ChecksumScope, InstallSummary},
    paths,
};

pub fn discover_install(explicit: Option<PathBuf>) -> Result<Option<InstallSummary>> {
    let candidates = paths::install_candidates(explicit);
    let Some(root) = paths::first_existing(&candidates) else {
        return Ok(None);
    };

    let mut summary = InstallSummary {
        root: root.clone(),
        ..Default::default()
    };

    summary.steam_appid = read_steam_appid(&root)?;
    summary.checksum_manifest = read_checksum_manifest(&root)?;

    if let Ok(value) = read_json_file(&root.join("launcher-settings.json")) {
        summary.version = find_first_string(
            &value,
            &["version", "display_version", "game_version", "gameVersion"],
        );
        summary.raw_version = find_first_string(
            &value,
            &[
                "raw_version",
                "rawVersion",
                "raw_game_version",
                "rawGameVersion",
            ],
        )
        .or_else(|| summary.version.clone());
    } else {
        summary
            .issues
            .push("launcher-settings.json missing or unreadable".to_string());
    }

    Ok(Some(summary))
}

fn read_steam_appid(root: &Path) -> Result<Option<u32>> {
    let path = root.join("steam_appid.txt");
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path)?;
    Ok(text.trim().parse::<u32>().ok())
}

fn read_json_file(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

fn read_checksum_manifest(root: &Path) -> Result<Vec<ChecksumScope>> {
    let path = root.join("checksum_manifest.txt");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)?;
    Ok(text.lines().filter_map(parse_checksum_line).collect())
}

fn parse_checksum_line(line: &str) -> Option<ChecksumScope> {
    let raw = line.trim();
    if raw.is_empty() || raw.starts_with('#') {
        return None;
    }

    if raw.contains("directory name") || raw.contains("file_extension") {
        return parse_manifest_key_value_line(raw);
    }

    let mut recursive = false;
    let mut body = raw.to_string();
    if let Some(stripped) = body.strip_suffix(" with subdirectories") {
        recursive = true;
        body = stripped.trim().to_string();
    }

    let split_at = body.find(char::is_whitespace)?;
    let (directory, rest) = body.split_at(split_at);
    let patterns = rest
        .trim()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .collect::<Vec<_>>();

    Some(ChecksumScope {
        raw: raw.to_string(),
        directory: Some(directory.trim_end_matches('/').to_string()),
        patterns,
        recursive,
    })
}

fn parse_manifest_key_value_line(raw: &str) -> Option<ChecksumScope> {
    let directory = find_manifest_value(raw, "directory name")
        .or_else(|| find_manifest_value(raw, "directory"))?;
    let recursive = find_manifest_value(raw, "sub_directories")
        .map(|value| matches!(value.as_str(), "yes" | "true" | "1"))
        .unwrap_or(false);
    let patterns = find_manifest_value(raw, "file_extension")
        .map(|extension| vec![format!("*{extension}")])
        .unwrap_or_default();

    Some(ChecksumScope {
        raw: raw.to_string(),
        directory: Some(directory.trim_end_matches('/').to_string()),
        patterns,
        recursive,
    })
}

fn find_manifest_value(raw: &str, key: &str) -> Option<String> {
    let start = raw.find(key)?;
    let after_key = &raw[start + key.len()..];
    let after_equals = after_key.trim_start().strip_prefix('=')?.trim_start();
    let value = after_equals
        .split_whitespace()
        .next()
        .map(|value| value.trim_matches('"').to_string())?;
    (!value.is_empty()).then_some(value)
}

fn find_first_string(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for key in keys {
                if let Some(found) = map
                    .get(*key)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                {
                    return Some(found);
                }
            }
            for child in map.values() {
                if let Some(found) = find_first_string(child, keys) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(items) => items.iter().find_map(|item| find_first_string(item, keys)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // parse_checksum_line
    // -----------------------------------------------------------------------

    #[test]
    fn parses_simple_format() {
        let scope = parse_checksum_line("common *.txt").expect("should parse simple format");
        assert_eq!(scope.directory.as_deref(), Some("common"));
        assert_eq!(scope.patterns, vec!["*.txt"]);
        assert!(!scope.recursive);
        assert_eq!(scope.raw, "common *.txt");
    }

    #[test]
    fn parses_with_subdirectories() {
        let scope = parse_checksum_line("common *.txt with subdirectories")
            .expect("should parse subdirectories format");
        assert_eq!(scope.directory.as_deref(), Some("common"));
        assert_eq!(scope.patterns, vec!["*.txt"]);
        assert!(scope.recursive);
        assert_eq!(scope.raw, "common *.txt with subdirectories");
    }

    #[test]
    fn parses_multiple_patterns() {
        let scope =
            parse_checksum_line("common *.txt, *.json").expect("should parse multiple patterns");
        assert_eq!(scope.directory.as_deref(), Some("common"));
        assert_eq!(scope.patterns, vec!["*.txt", "*.json"]);
        assert!(!scope.recursive);
    }

    #[test]
    fn ignores_comment_line() {
        assert!(parse_checksum_line("# comment").is_none());
    }

    #[test]
    fn ignores_empty_line() {
        assert!(parse_checksum_line("").is_none());
        assert!(parse_checksum_line("   ").is_none());
    }

    #[test]
    fn parses_key_value_format() {
        let scope = parse_checksum_line(r#"directory name = "common/" file_extension = ".txt""#)
            .expect("should parse key-value format");
        assert_eq!(scope.directory.as_deref(), Some("common"));
        assert_eq!(scope.patterns, vec!["*.txt"]);
        assert!(!scope.recursive);
        assert_eq!(
            scope.raw,
            r#"directory name = "common/" file_extension = ".txt""#
        );
    }

    #[test]
    fn parses_key_value_with_subdirectories() {
        let scope = parse_checksum_line(
            r#"directory name = "events" sub_directories = "yes" file_extension = ".txt""#,
        )
        .expect("should parse key-value with subdirectories");
        assert_eq!(scope.directory.as_deref(), Some("events"));
        assert_eq!(scope.patterns, vec!["*.txt"]);
        assert!(scope.recursive);
    }

    // -----------------------------------------------------------------------
    // find_manifest_value
    // -----------------------------------------------------------------------

    #[test]
    fn finds_existing_key() {
        let result = find_manifest_value(r#"directory name = "common/""#, "directory name");
        assert_eq!(result.as_deref(), Some("common/"));
    }

    #[test]
    fn finds_missing_key() {
        let result = find_manifest_value(r#"file_extension = ".txt""#, "directory name");
        assert_eq!(result, None);
    }

    #[test]
    fn handles_empty_value() {
        let result = find_manifest_value(r#"directory name = """#, "directory name");
        assert_eq!(result, None);
    }
}
