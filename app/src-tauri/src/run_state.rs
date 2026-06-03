use std::{
    fs,
    path::{Component, Path, PathBuf},
    time::UNIX_EPOCH,
};

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

use crate::{
    documents::order_saves_by_preference,
    error::Result,
    model::{PersistedRunSummary, RunFactSummary, SaveRunSummary, SaveSummary},
    ScanReport,
};

const FACT_SOURCE_PARSED_SAVE: &str = "parsed_save";
const FACT_CONFIDENCE_HIGH: &str = "high";

#[derive(Debug, Clone)]
struct RunFactInput {
    dimension: String,
    key: String,
    value: Value,
}

pub fn initialize_run_state_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS runs (
          folder_path TEXT PRIMARY KEY,
          run_folder TEXT NOT NULL,
          display_name TEXT,
          latest_save_path TEXT,
          game_version TEXT,
          latest_ingame_date TEXT,
          updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS save_files (
          file_path TEXT PRIMARY KEY,
          run_folder_path TEXT NOT NULL,
          file_name TEXT NOT NULL,
          file_size INTEGER,
          modified_at TEXT,
          is_latest_for_run INTEGER NOT NULL DEFAULT 0,
          parse_status TEXT NOT NULL,
          parse_error TEXT,
          parsed_at TEXT,
          FOREIGN KEY (run_folder_path) REFERENCES runs(folder_path)
        );

        CREATE TABLE IF NOT EXISTS run_facts (
          run_folder_path TEXT NOT NULL,
          dimension TEXT NOT NULL,
          key TEXT NOT NULL,
          value_json TEXT NOT NULL,
          source TEXT NOT NULL,
          confidence TEXT NOT NULL,
          updated_from_save_path TEXT,
          updated_at TEXT NOT NULL DEFAULT (datetime('now')),
          PRIMARY KEY (run_folder_path, dimension, key),
          FOREIGN KEY (run_folder_path) REFERENCES runs(folder_path)
        );

        CREATE TABLE IF NOT EXISTS fact_overrides (
          run_folder_path TEXT NOT NULL,
          dimension TEXT NOT NULL,
          key TEXT NOT NULL,
          value_json TEXT NOT NULL,
          reason TEXT,
          created_at TEXT NOT NULL DEFAULT (datetime('now')),
          updated_at TEXT NOT NULL DEFAULT (datetime('now')),
          PRIMARY KEY (run_folder_path, dimension, key),
          FOREIGN KEY (run_folder_path) REFERENCES runs(folder_path)
        );
        "#,
    )?;
    Ok(())
}

pub fn persist_scan_report(conn: &mut Connection, report: &ScanReport) -> Result<()> {
    let Some(documents) = &report.documents else {
        return Ok(());
    };

    let save_root = documents.root.join("save games");
    let tx = conn.transaction()?;
    for run in &documents.save_runs {
        persist_run(&tx, &save_root, run)?;
    }
    tx.commit()?;
    Ok(())
}

pub fn load_persisted_runs(conn: &Connection) -> Result<Vec<PersistedRunSummary>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
          r.folder_path,
          r.run_folder,
          r.display_name,
          r.latest_save_path,
          sf.file_name,
          r.latest_ingame_date,
          r.game_version,
          sf.parse_status,
          sf.parse_error,
          COALESCE(rf.fact_count, 0) AS fact_count,
          r.updated_at
        FROM runs r
        LEFT JOIN save_files sf ON sf.file_path = r.latest_save_path
        LEFT JOIN (
          SELECT run_folder_path, COUNT(*) AS fact_count
          FROM run_facts
          GROUP BY run_folder_path
        ) rf ON rf.run_folder_path = r.folder_path
        ORDER BY COALESCE(r.latest_ingame_date, '') DESC, r.run_folder COLLATE NOCASE ASC
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        let fact_count: i64 = row.get(9)?;
        Ok(PersistedRunSummary {
            folder_path: row.get(0)?,
            run_folder: row.get(1)?,
            display_name: row.get(2)?,
            latest_save_path: row.get(3)?,
            latest_save_file_name: row.get(4)?,
            latest_ingame_date: row.get(5)?,
            game_version: row.get(6)?,
            parse_status: row.get(7)?,
            parse_error: row.get(8)?,
            fact_count: fact_count.max(0) as usize,
            updated_at: row.get(10)?,
        })
    })?;

    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub fn load_run_facts(conn: &Connection, run_folder_path: &str) -> Result<Vec<RunFactSummary>> {
    let normalized_run_path = normalize_path(Path::new(run_folder_path));
    let mut stmt = conn.prepare(
        r#"
        SELECT run_folder_path, dimension, key, value_json, source, confidence,
               updated_from_save_path, updated_at
        FROM run_facts
        WHERE run_folder_path = ?1
        ORDER BY dimension COLLATE NOCASE ASC, key COLLATE NOCASE ASC
        "#,
    )?;

    let mut rows = stmt.query([normalized_run_path])?;
    let mut facts = Vec::new();
    while let Some(row) = rows.next()? {
        let value_json: String = row.get(3)?;
        let value = serde_json::from_str(&value_json)?;
        facts.push(RunFactSummary {
            run_folder_path: row.get(0)?,
            dimension: row.get(1)?,
            key: row.get(2)?,
            value,
            source: row.get(4)?,
            confidence: row.get(5)?,
            updated_from_save_path: row.get(6)?,
            updated_at: row.get(7)?,
        });
    }

    Ok(facts)
}

pub fn persist_run_for_tests(
    conn: &mut Connection,
    save_root: &Path,
    run: &SaveRunSummary,
) -> Result<()> {
    let tx = conn.transaction()?;
    persist_run(&tx, save_root, run)?;
    tx.commit()?;
    Ok(())
}

fn persist_run(conn: &Connection, save_root: &Path, run: &SaveRunSummary) -> Result<()> {
    let run_path = save_root.join(&run.run_folder);
    let folder_path = normalize_path(&run_path);
    let failed_latest_save_path = if run.latest_save.is_none() {
        latest_save_path_for_failed_run(save_root, run)
    } else {
        None
    };
    let (latest_save_path, game_version, latest_ingame_date, display_name) = run
        .latest_save
        .as_ref()
        .map(|save| {
            (
                Some(normalize_path(&save.path)),
                save.version.clone(),
                save.date.clone(),
                save.name.clone(),
            )
        })
        .unwrap_or((
            failed_latest_save_path
                .as_ref()
                .map(|path| normalize_path(path)),
            None,
            None,
            None,
        ));

    conn.execute(
        r#"
        INSERT INTO runs (
          folder_path, run_folder, display_name, latest_save_path,
          game_version, latest_ingame_date, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))
        ON CONFLICT(folder_path) DO UPDATE SET
          run_folder = excluded.run_folder,
          display_name = excluded.display_name,
          latest_save_path = excluded.latest_save_path,
          game_version = excluded.game_version,
          latest_ingame_date = excluded.latest_ingame_date,
          updated_at = excluded.updated_at
        "#,
        params![
            folder_path,
            run.run_folder,
            display_name,
            latest_save_path,
            game_version,
            latest_ingame_date,
        ],
    )?;

    conn.execute(
        "UPDATE save_files SET is_latest_for_run = 0 WHERE run_folder_path = ?1",
        [folder_path.as_str()],
    )?;

    if let Some(save) = &run.latest_save {
        persist_save_file(conn, folder_path.as_str(), &save.path, "parsed", None)?;
        replace_run_facts(conn, folder_path.as_str(), save)?;
    } else {
        let parse_error = run.issues.first().cloned();
        if let Some(path) = failed_latest_save_path.as_deref() {
            persist_save_file(
                conn,
                folder_path.as_str(),
                path,
                "failed",
                parse_error.as_deref(),
            )?;
        }
        conn.execute(
            "DELETE FROM run_facts WHERE run_folder_path = ?1",
            [folder_path.as_str()],
        )?;
    }

    Ok(())
}

fn persist_save_file(
    conn: &Connection,
    run_folder_path: &str,
    save_path: &Path,
    parse_status: &str,
    parse_error: Option<&str>,
) -> Result<()> {
    let file_path = normalize_path(save_path);
    let file_name = file_name(save_path);
    let (file_size, modified_at) = file_metadata(save_path);
    conn.execute(
        r#"
        INSERT INTO save_files (
          file_path, run_folder_path, file_name, file_size, modified_at,
          is_latest_for_run, parse_status, parse_error, parsed_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, datetime('now'))
        ON CONFLICT(file_path) DO UPDATE SET
          run_folder_path = excluded.run_folder_path,
          file_name = excluded.file_name,
          file_size = excluded.file_size,
          modified_at = excluded.modified_at,
          is_latest_for_run = excluded.is_latest_for_run,
          parse_status = excluded.parse_status,
          parse_error = excluded.parse_error,
          parsed_at = excluded.parsed_at
        "#,
        params![
            file_path,
            run_folder_path,
            file_name,
            file_size,
            modified_at,
            parse_status,
            parse_error,
        ],
    )?;
    Ok(())
}

fn replace_run_facts(conn: &Connection, run_folder_path: &str, save: &SaveSummary) -> Result<()> {
    conn.execute(
        "DELETE FROM run_facts WHERE run_folder_path = ?1",
        [run_folder_path],
    )?;

    let save_path = normalize_path(&save.path);
    for fact in facts_from_save(save) {
        conn.execute(
            r#"
            INSERT INTO run_facts (
              run_folder_path, dimension, key, value_json, source, confidence,
              updated_from_save_path, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))
            "#,
            params![
                run_folder_path,
                fact.dimension,
                fact.key,
                serde_json::to_string(&fact.value)?,
                FACT_SOURCE_PARSED_SAVE,
                FACT_CONFIDENCE_HIGH,
                save_path,
            ],
        )?;
    }
    Ok(())
}

fn facts_from_save(save: &SaveSummary) -> Vec<RunFactInput> {
    let mut facts = Vec::new();
    push_option(&mut facts, "save", "version", &save.version);
    push_option(&mut facts, "save", "date", &save.date);
    push_option(&mut facts, "save", "name", &save.name);
    push_vec(&mut facts, "save", "required_dlcs", &save.required_dlcs);
    push_option(&mut facts, "empire", "player_country", &save.player_country);
    push_option(&mut facts, "empire", "authority", &save.authority);
    push_option(
        &mut facts,
        "empire",
        "government_type",
        &save.government_type,
    );
    push_option(&mut facts, "empire", "origin", &save.origin);
    push_vec(&mut facts, "empire", "ethics", &save.ethics);
    push_vec(&mut facts, "empire", "civics", &save.civics);
    push_option(
        &mut facts,
        "species",
        "founder_species_ref",
        &save.founder_species_ref,
    );
    push_option(
        &mut facts,
        "species",
        "founder_species_class",
        &save.founder_species_class,
    );
    push_option(
        &mut facts,
        "species",
        "founder_species_portrait",
        &save.founder_species_portrait,
    );
    push_vec(
        &mut facts,
        "species",
        "founder_species_traits",
        &save.founder_species_traits,
    );
    push_bool(&mut facts, "eligibility", "ironman", save.ironman);
    push_bool(
        &mut facts,
        "eligibility",
        "cheated_on_save",
        save.cheated_on_save,
    );
    facts
}

fn push_option(facts: &mut Vec<RunFactInput>, dimension: &str, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        facts.push(RunFactInput {
            dimension: dimension.to_string(),
            key: key.to_string(),
            value: Value::String(value.clone()),
        });
    }
}

fn push_vec(facts: &mut Vec<RunFactInput>, dimension: &str, key: &str, values: &[String]) {
    if !values.is_empty() {
        facts.push(RunFactInput {
            dimension: dimension.to_string(),
            key: key.to_string(),
            value: Value::Array(values.iter().cloned().map(Value::String).collect()),
        });
    }
}

fn push_bool(facts: &mut Vec<RunFactInput>, dimension: &str, key: &str, value: Option<bool>) {
    if let Some(value) = value {
        facts.push(RunFactInput {
            dimension: dimension.to_string(),
            key: key.to_string(),
            value: Value::Bool(value),
        });
    }
}

fn latest_save_path_for_failed_run(save_root: &Path, run: &SaveRunSummary) -> Option<PathBuf> {
    let run_path = save_root.join(&run.run_folder);
    let ironman = run_path.join("ironman.sav");
    if ironman.exists() {
        return Some(ironman);
    }

    let mut candidates = fs::read_dir(run_path)
        .ok()?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sav"))
        .collect::<Vec<_>>();
    order_saves_by_preference(&mut candidates);
    candidates.into_iter().next()
}

fn normalize_path(path: &Path) -> String {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    normalized.to_string_lossy().replace('\\', "/")
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string()
}

fn file_metadata(path: &Path) -> (Option<i64>, Option<String>) {
    let Ok(metadata) = fs::metadata(path) else {
        return (None, None);
    };
    let file_size = i64::try_from(metadata.len()).ok();
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs().to_string());
    (file_size, modified_at)
}

pub fn run_exists(conn: &Connection, folder_path: &Path) -> Result<bool> {
    let normalized = normalize_path(folder_path);
    Ok(conn
        .query_row(
            "SELECT 1 FROM runs WHERE folder_path = ?1 LIMIT 1",
            [normalized],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}
