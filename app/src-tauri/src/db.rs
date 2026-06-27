use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::{
    catalog::{
        import_catalog, initialize_catalog_schema, load_catalog_metadata, parse_catalog_json,
    },
    error::Result,
    model::{AppConfig, AppInfo},
    run_state::initialize_run_state_schema,
};

/// Path to the bundled catalog JSON, embedded at compile time.
const BUNDLED_CATALOG_JSON: &str = include_str!("../../../catalog/latest.json");

/// Newtype wrapper for the app database path, stored in Tauri managed state.
#[derive(Debug, Clone)]
pub struct AppDbPath(pub PathBuf);

/// Parse the compile-time bundled catalog JSON.
pub fn parse_bundled_catalog() -> Result<crate::model::AchievementCatalog> {
    parse_catalog_json(BUNDLED_CATALOG_JSON)
}

/// Open (or create) the app SQLite database at the given path.
///
/// Initializes app-owned schemas on first open.
pub fn open_app_db(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    initialize_catalog_schema(&conn)?;
    initialize_run_state_schema(&conn)?;
    initialize_config_schema(&conn)?;
    conn.execute_batch("PRAGMA user_version = 3;")?;
    Ok(conn)
}

/// Import the bundled catalog into the database if needed.
///
/// Imports when:
/// - The achievements table is empty (first launch), OR
/// - The bundled catalog version differs from the DB's stored version
///
/// Returns `true` if an import was performed, `false` if the DB is already up to date.
pub fn ensure_catalog_imported(conn: &mut Connection) -> Result<bool> {
    let bundled = parse_bundled_catalog()?;

    // Check if DB already has this version
    if let Some(metadata) = load_catalog_metadata(conn)? {
        if metadata.catalog_version == bundled.catalog_version {
            return Ok(false);
        }
        eprintln!(
            "catalog version mismatch: db={}, bundled={} — re-importing",
            metadata.catalog_version, bundled.catalog_version
        );
    }

    import_catalog(conn, &bundled)?;
    Ok(true)
}

/// Initialize the app_config key-value table.
pub fn initialize_config_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS app_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;
    Ok(())
}

/// Load app config from the database.
pub fn load_app_config(conn: &Connection) -> Result<AppConfig> {
    initialize_config_schema(conn)?;
    let mut config = AppConfig::default();

    let mut stmt = conn.prepare("SELECT key, value FROM app_config")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    for row in rows {
        let (key, value) = row?;
        match key.as_str() {
            "install_path_override" => config.install_path_override = Some(value),
            "documents_path_override" => config.documents_path_override = Some(value),
            _ => {}
        }
    }

    Ok(config)
}

/// Save app config to the database (clear + re-insert).
pub fn save_app_config(conn: &Connection, config: &AppConfig) -> Result<()> {
    initialize_config_schema(conn)?;

    conn.execute("DELETE FROM app_config", [])?;

    if let Some(ref val) = config.install_path_override {
        conn.execute(
            "INSERT INTO app_config (key, value) VALUES ('install_path_override', ?1)",
            rusqlite::params![val],
        )?;
    }
    if let Some(ref val) = config.documents_path_override {
        conn.execute(
            "INSERT INTO app_config (key, value) VALUES ('documents_path_override', ?1)",
            rusqlite::params![val],
        )?;
    }

    Ok(())
}

/// Load app info (versions, timestamps) from the database.
pub fn load_app_info(conn: &Connection) -> Result<AppInfo> {
    let catalog_meta = load_catalog_metadata(conn).ok().flatten();

    let last_steam_sync: Option<String> = conn
        .query_row(
            "SELECT MAX(steam_last_synced_at) FROM player_achievements",
            [],
            |row| row.get(0),
        )
        .ok();

    // runs table has updated_at (not scanned_at)
    let last_save_scan: Option<String> = conn
        .query_row("SELECT MAX(updated_at) FROM runs", [], |row| row.get(0))
        .ok();

    let last_steam_sync_event: Option<(String, Option<String>)> = conn
        .query_row(
            r#"
            SELECT status, error_message
            FROM steam_sync_events
            ORDER BY synced_at DESC
            LIMIT 1
            "#,
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    Ok(AppInfo {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        catalog_version: catalog_meta.as_ref().map(|m| m.catalog_version.clone()),
        stellaris_version: catalog_meta
            .as_ref()
            .and_then(|m| m.stellaris_version.clone()),
        last_catalog_sync: catalog_meta.map(|m| m.imported_at),
        last_steam_sync,
        last_steam_sync_status: last_steam_sync_event.as_ref().map(|event| event.0.clone()),
        last_steam_sync_error: last_steam_sync_event.and_then(|event| event.1),
        last_save_scan,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalog_parses_without_errors() {
        let catalog = parse_bundled_catalog().expect("bundled catalog should parse");
        assert!(
            catalog.achievements.len() > 50,
            "bundled catalog should contain a realistic number of achievements"
        );
        assert_eq!(catalog.snapshot_kind, "full");
        assert!(
            !catalog.catalog_version.is_empty(),
            "bundled catalog_version should be populated"
        );
    }

    #[test]
    fn bundled_catalog_all_entries_have_steam_api_name() {
        let catalog = parse_bundled_catalog().expect("bundled catalog should parse");
        let unmapped: Vec<&str> = catalog
            .achievements
            .iter()
            .filter(|a| a.steam_api_name.is_none())
            .map(|a| a.id.as_str())
            .collect();
        assert!(
            unmapped.is_empty(),
            "all entries should have steam_api_name, but these don't: {unmapped:?}"
        );
    }

    #[test]
    fn ensure_catalog_imports_into_empty_db() {
        let mut conn = Connection::open_in_memory().expect("open in-memory db");
        initialize_catalog_schema(&conn).expect("schema init");

        let bundled = parse_bundled_catalog().expect("bundled catalog should parse");

        let imported = ensure_catalog_imported(&mut conn).expect("import");
        assert!(imported, "should import into empty db");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM achievements", [], |row| row.get(0))
            .expect("count");
        assert_eq!(count as usize, bundled.achievements.len());
    }

    #[test]
    fn ensure_catalog_skips_reimport_when_version_matches() {
        let mut conn = Connection::open_in_memory().expect("open in-memory db");
        initialize_catalog_schema(&conn).expect("schema init");

        ensure_catalog_imported(&mut conn).expect("first import");
        let imported_again = ensure_catalog_imported(&mut conn).expect("second check");
        assert!(
            !imported_again,
            "should skip when bundled version matches DB version"
        );
    }

    #[test]
    fn load_app_info_includes_latest_steam_sync_event() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        initialize_catalog_schema(&conn).expect("catalog schema");
        initialize_run_state_schema(&conn).expect("run schema");
        initialize_config_schema(&conn).expect("config schema");
        conn.execute_batch(
            r#"
            CREATE TABLE steam_sync_events (
              status TEXT NOT NULL,
              error_message TEXT,
              synced_at TEXT NOT NULL
            );

            INSERT INTO steam_sync_events (status, error_message, synced_at)
            VALUES
              ('ok', NULL, '2026-01-01 00:00:00'),
              ('failed', 'steam unavailable', '2026-01-02 00:00:00');
            "#,
        )
        .expect("insert steam sync events");

        let info = load_app_info(&conn).expect("load app info");
        assert_eq!(info.last_steam_sync_status.as_deref(), Some("failed"));
        assert_eq!(
            info.last_steam_sync_error.as_deref(),
            Some("steam unavailable")
        );
    }
}
