use crate::{scan_all, ScanReport};

#[cfg_attr(feature = "desktop", tauri::command)]
pub async fn scan_local_state() -> Result<ScanReport, String> {
    tokio::task::spawn_blocking(|| scan_all(None, None))
        .await
        .map_err(|err| format!("scan worker failed: {err}"))
}

#[cfg(feature = "desktop")]
mod catalog_commands {
    use rusqlite::Connection;
    use tauri::State;

    use crate::{
        catalog::{load_catalog_entries_with_issues, load_catalog_metadata},
        db::AppDbPath,
        model::{AchievementCatalogEntry, CatalogVersionMetadata},
    };

    #[tauri::command]
    pub async fn load_achievements(
        db_path: State<'_, AppDbPath>,
    ) -> Result<Vec<AchievementCatalogEntry>, String> {
        let path = db_path.0.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<AchievementCatalogEntry>, String> {
            let conn = Connection::open(&path).map_err(|e| format!("open db: {e}"))?;
            let load = load_catalog_entries_with_issues(&conn)
                .map_err(|e| format!("load achievements: {e}"))?;
            if !load.issues.is_empty() {
                eprintln!("catalog load issues: {:?}", load.issues);
            }
            Ok(load.entries)
        })
        .await
        .map_err(|e| format!("worker failed: {e}"))?
    }

    #[tauri::command]
    pub async fn load_catalog_info(
        db_path: State<'_, AppDbPath>,
    ) -> Result<Option<CatalogVersionMetadata>, String> {
        let path = db_path.0.clone();
        tokio::task::spawn_blocking(move || -> Result<Option<CatalogVersionMetadata>, String> {
            let conn = Connection::open(&path).map_err(|e| format!("open db: {e}"))?;
            load_catalog_metadata(&conn).map_err(|e| format!("load catalog info: {e}"))
        })
        .await
        .map_err(|e| format!("worker failed: {e}"))?
    }
}

#[cfg(feature = "desktop")]
pub use catalog_commands::{load_achievements, load_catalog_info};
