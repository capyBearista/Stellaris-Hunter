use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::{
    catalog::{
        import_catalog, initialize_catalog_schema, load_catalog_metadata, parse_catalog_json,
    },
    error::Result,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalog_parses_without_errors() {
        let catalog = parse_bundled_catalog().expect("bundled catalog should parse");
        assert_eq!(catalog.achievements.len(), 211);
        assert_eq!(catalog.snapshot_kind, "full");
        assert_eq!(catalog.catalog_version, "1.1.0");
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

        let imported = ensure_catalog_imported(&mut conn).expect("import");
        assert!(imported, "should import into empty db");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM achievements", [], |row| row.get(0))
            .expect("count");
        assert_eq!(count, 211);
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
}
