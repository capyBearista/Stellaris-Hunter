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
        catalog::{
            clear_completion_override as clear_completion_override_in_db,
            load_catalog_entries_with_issues, load_catalog_metadata,
            load_completion_overrides as load_completion_overrides_from_db,
            set_completion_override as set_completion_override_in_db,
        },
        db::AppDbPath,
        model::{AchievementCatalogEntry, AchievementOverride, CatalogVersionMetadata},
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

    #[tauri::command]
    pub async fn load_completion_overrides(
        db_path: State<'_, AppDbPath>,
    ) -> Result<Vec<AchievementOverride>, String> {
        let path = db_path.0.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<AchievementOverride>, String> {
            let conn = Connection::open(&path).map_err(|e| format!("open db: {e}"))?;
            load_completion_overrides_from_db(&conn)
                .map_err(|e| format!("load completion overrides: {e}"))
        })
        .await
        .map_err(|e| format!("worker failed: {e}"))?
    }

    #[tauri::command]
    pub async fn set_completion_override(
        db_path: State<'_, AppDbPath>,
        achievement_id: String,
        completed: bool,
    ) -> Result<(), String> {
        let path = db_path.0.clone();
        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let conn = Connection::open(&path).map_err(|e| format!("open db: {e}"))?;
            set_completion_override_in_db(&conn, &achievement_id, completed)
                .map_err(|e| format!("set completion override: {e}"))
        })
        .await
        .map_err(|e| format!("worker failed: {e}"))?
    }

    #[tauri::command]
    pub async fn clear_completion_override(
        db_path: State<'_, AppDbPath>,
        achievement_id: String,
    ) -> Result<(), String> {
        let path = db_path.0.clone();
        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let conn = Connection::open(&path).map_err(|e| format!("open db: {e}"))?;
            clear_completion_override_in_db(&conn, &achievement_id)
                .map_err(|e| format!("clear completion override: {e}"))
        })
        .await
        .map_err(|e| format!("worker failed: {e}"))?
    }
}

#[cfg(feature = "desktop")]
pub use catalog_commands::{
    clear_completion_override, load_achievements, load_catalog_info, load_completion_overrides,
    set_completion_override,
};
