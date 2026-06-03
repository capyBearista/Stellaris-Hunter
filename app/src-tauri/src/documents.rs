use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};

use rusqlite::Connection;

use crate::{
    error::{Error, Result},
    model::{
        ContinueGameTarget, DlcLoadSummary, DocumentsSummary, LauncherModSummary,
        LauncherPlaysetSummary, LauncherStateSummary, SaveRunSummary,
    },
    paths, save,
};

pub fn discover_documents(explicit: Option<PathBuf>) -> Result<Option<DocumentsSummary>> {
    let candidates = paths::documents_candidates(explicit);
    let Some(root) = paths::first_existing(&candidates) else {
        return Ok(None);
    };

    let mut summary = DocumentsSummary {
        root: root.clone(),
        ..Default::default()
    };

    summary.continue_game = read_continue_game(&root)?;
    summary.dlc_load = read_dlc_load(&root)?;
    summary.save_runs = discover_save_runs(&root)?;
    summary.launcher = read_launcher_state(&root)?;

    Ok(Some(summary))
}

fn read_continue_game(root: &Path) -> Result<Option<ContinueGameTarget>> {
    let path = root.join("continue_game.json");
    if !path.exists() {
        return Ok(None);
    }
    let target = serde_json::from_str::<ContinueGameTarget>(&fs::read_to_string(path)?)?;
    Ok(Some(target))
}

fn read_dlc_load(root: &Path) -> Result<Option<DlcLoadSummary>> {
    let path = root.join("dlc_load.json");
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&text)?;
    let enabled_mods = value
        .get("enabled_mods")
        .map(normalize_string_list)
        .unwrap_or_default();
    let disabled_dlcs = value
        .get("disabled_dlcs")
        .map(normalize_string_list)
        .unwrap_or_default();
    Ok(Some(DlcLoadSummary {
        enabled_mods,
        disabled_dlcs,
    }))
}

fn normalize_string_list(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::Array(items) => items
            .iter()
            .map(|item| match item {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Object(map) => map
                    .get("path")
                    .or_else(|| map.get("name"))
                    .or_else(|| map.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                other => other.to_string(),
            })
            .filter(|s| !s.is_empty())
            .collect(),
        serde_json::Value::Object(map) => map
            .values()
            .filter_map(|value| value.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    }
}

fn discover_save_runs(root: &Path) -> Result<Vec<SaveRunSummary>> {
    let mut grouped: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    let save_root = root.join("save games");
    if !save_root.exists() {
        return Ok(Vec::new());
    }

    for entry in walkdir::WalkDir::new(&save_root).min_depth(2).max_depth(2) {
        let entry = entry.map_err(|err| Error::Parse(err.to_string()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("sav") {
            continue;
        }
        let run_folder = entry
            .path()
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        grouped
            .entry(run_folder)
            .or_default()
            .push(entry.path().to_path_buf());
    }

    let mut runs = Vec::new();
    for (run_folder, mut files) in grouped {
        order_saves_by_preference(&mut files);
        let mut issues = Vec::new();
        let latest_save = files.first().and_then(|path| match save::parse_save(path) {
            Ok(summary) => Some(summary),
            Err(err) => {
                issues.push(format!("failed to parse {}: {err}", path.display()));
                None
            }
        });
        runs.push(SaveRunSummary {
            run_folder,
            save_count: files.len(),
            latest_save,
            eligibility: None,
            issues,
        });
    }

    Ok(runs)
}

pub(crate) fn order_saves_by_preference(files: &mut Vec<PathBuf>) {
    files.sort_by_key(|path| {
        std::fs::metadata(path)
            .and_then(|meta| meta.modified())
            .ok()
    });
    files.reverse();
    if let Some(ironman_index) = files
        .iter()
        .position(|path| path.file_name().and_then(|name| name.to_str()) == Some("ironman.sav"))
    {
        let ironman = files.remove(ironman_index);
        files.insert(0, ironman);
    }
}

fn read_launcher_state(root: &Path) -> Result<Option<LauncherStateSummary>> {
    let path = root.join("launcher-v2.sqlite");
    if !path.exists() {
        return Ok(None);
    }

    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut issues = Vec::new();

    let active_playset = read_active_playset(&conn).unwrap_or_else(|err| {
        issues.push(err.to_string());
        None
    });
    let mod_catalog = read_mod_catalog(&conn).unwrap_or_else(|err| {
        issues.push(err.to_string());
        HashMap::new()
    });
    let enabled_mods = read_enabled_mods(&conn, active_playset.as_ref(), &mod_catalog)
        .unwrap_or_else(|err| {
            issues.push(err.to_string());
            Vec::new()
        });

    Ok(Some(LauncherStateSummary {
        active_playset,
        enabled_mods,
        issues,
    }))
}

fn read_active_playset(conn: &Connection) -> Result<Option<LauncherPlaysetSummary>> {
    let mut stmt = conn.prepare("SELECT * FROM playsets WHERE isActive = 1 LIMIT 1")?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let playset = LauncherPlaysetSummary {
            uuid: string_column(row, &["uuid", "id"]),
            name: string_column(row, &["name", "display_name"]),
            sync_state: string_column(row, &["syncState", "sync_state"]),
            state: string_column(row, &["state"]),
        };
        Ok(Some(playset))
    } else {
        Ok(None)
    }
}

fn read_mod_catalog(conn: &Connection) -> Result<HashMap<String, LauncherModSummary>> {
    let mut stmt = conn.prepare("SELECT * FROM mods")?;
    let mut rows = stmt.query([])?;
    let mut catalog = HashMap::new();

    while let Some(row) = rows.next()? {
        let id = string_column(row, &["id", "uuid", "mod_id"]);
        let summary = LauncherModSummary {
            position: None,
            id: id.clone(),
            name: string_column(row, &["displayName", "name", "display_name", "title"]),
            registry_id: string_column(row, &["gameRegistryId", "registry_id"]),
            path: string_column(
                row,
                &[
                    "dirPath",
                    "archivePath",
                    "descriptorPath",
                    "repositoryPath",
                    "path",
                    "mod_path",
                    "descriptor_path",
                    "gameRegistryId",
                ],
            ),
        };
        if let Some(id) = id {
            catalog.insert(id, summary);
        }
    }

    Ok(catalog)
}

fn read_enabled_mods(
    conn: &Connection,
    active_playset: Option<&LauncherPlaysetSummary>,
    mod_catalog: &HashMap<String, LauncherModSummary>,
) -> Result<Vec<LauncherModSummary>> {
    let mut stmt = conn.prepare("SELECT * FROM playsets_mods ORDER BY position ASC")?;
    let mut rows = stmt.query([])?;
    let mut mods = Vec::new();
    while let Some(row) = rows.next()? {
        if let Some(enabled) = integer_column(row, &["enabled", "isEnabled"]) {
            if enabled == 0 {
                continue;
            }
        }

        if let Some(active) = active_playset {
            if let Some(playset_key) =
                string_column(row, &["playset_uuid", "playset_id", "playsetId", "playset"])
            {
                if active
                    .uuid
                    .as_deref()
                    .map(|uuid| uuid != playset_key)
                    .unwrap_or(false)
                {
                    continue;
                }
            }
        }

        let mod_id = string_column(row, &["mod_id", "mod_uuid", "modId", "id"]);
        let position = integer_column(row, &["position"]);

        let mut mod_summary = mod_id
            .as_ref()
            .and_then(|id| mod_catalog.get(id))
            .cloned()
            .unwrap_or_default();
        mod_summary.position = position;
        if mod_summary.id.is_none() {
            mod_summary.id = mod_id;
        }

        mods.push(mod_summary);
    }
    Ok(mods)
}

fn string_column(row: &rusqlite::Row<'_>, names: &[&str]) -> Option<String> {
    for name in names {
        if let Ok(Some(value)) = row.get::<_, Option<String>>(*name) {
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

fn integer_column(row: &rusqlite::Row<'_>, names: &[&str]) -> Option<i64> {
    for name in names {
        if let Ok(Some(value)) = row.get::<_, Option<i64>>(*name) {
            return Some(value);
        }
    }
    None
}
