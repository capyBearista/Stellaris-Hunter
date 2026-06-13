use crate::{scan_all, ScanReport};

#[cfg_attr(feature = "desktop", tauri::command)]
pub async fn scan_local_state() -> Result<ScanReport, String> {
    tokio::task::spawn_blocking(|| scan_all(None, None))
        .await
        .map_err(|err| format!("scan worker failed: {err}"))
}

#[cfg(feature = "desktop")]
pub(crate) mod catalog_commands {
    use tauri::State;

    use crate::{
        catalog::{
            clear_completion_override as clear_completion_override_in_db,
            load_catalog_entries_with_issues, load_catalog_metadata,
            load_completion_overrides as load_completion_overrides_from_db,
            load_displayed_completion_map,
            set_completion_override as set_completion_override_in_db,
        },
        db::{open_app_db, AppDbPath},
        model::{
            AchievementCatalogEntry, AchievementOverride, CatalogVersionMetadata,
            PersistedRunSummary, PlannerAchievementEvaluation, RunFactSummary,
        },
        rules::evaluate_planner_achievements,
        run_state::{
            load_persisted_runs, load_run_achievement_statuses,
            load_run_facts as load_run_facts_from_db, persist_scan_report,
            set_run_achievement_status as set_run_achievement_status_in_db,
        },
        scan_all,
    };

    #[tauri::command]
    pub async fn load_achievements(
        db_path: State<'_, AppDbPath>,
    ) -> Result<Vec<AchievementCatalogEntry>, String> {
        let path = db_path.0.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<AchievementCatalogEntry>, String> {
            let conn = open_app_db(&path).map_err(|e| format!("open db: {e}"))?;
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
            let conn = open_app_db(&path).map_err(|e| format!("open db: {e}"))?;
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
            let conn = open_app_db(&path).map_err(|e| format!("open db: {e}"))?;
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
            let conn = open_app_db(&path).map_err(|e| format!("open db: {e}"))?;
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
            let conn = open_app_db(&path).map_err(|e| format!("open db: {e}"))?;
            clear_completion_override_in_db(&conn, &achievement_id)
                .map_err(|e| format!("clear completion override: {e}"))
        })
        .await
        .map_err(|e| format!("worker failed: {e}"))?
    }

    #[tauri::command]
    pub async fn load_runs(
        db_path: State<'_, AppDbPath>,
    ) -> Result<Vec<PersistedRunSummary>, String> {
        let path = db_path.0.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<PersistedRunSummary>, String> {
            let conn = open_app_db(&path).map_err(|e| format!("open db: {e}"))?;
            load_persisted_runs(&conn).map_err(|e| format!("load runs: {e}"))
        })
        .await
        .map_err(|e| format!("worker failed: {e}"))?
    }

    #[tauri::command]
    pub async fn load_run_facts(
        db_path: State<'_, AppDbPath>,
        run_folder_path: String,
    ) -> Result<Vec<RunFactSummary>, String> {
        let path = db_path.0.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<RunFactSummary>, String> {
            let conn = open_app_db(&path).map_err(|e| format!("open db: {e}"))?;
            load_run_facts_from_db(&conn, &run_folder_path)
                .map_err(|e| format!("load run facts: {e}"))
        })
        .await
        .map_err(|e| format!("worker failed: {e}"))?
    }

    #[tauri::command]
    pub async fn rescan_saves(
        db_path: State<'_, AppDbPath>,
    ) -> Result<Vec<PersistedRunSummary>, String> {
        let path = db_path.0.clone();
        tokio::task::spawn_blocking(move || -> Result<Vec<PersistedRunSummary>, String> {
            let mut conn = open_app_db(&path).map_err(|e| format!("open db: {e}"))?;
            let report = scan_all(None, None);
            if !report.errors.is_empty() {
                eprintln!("scan errors before persistence: {:?}", report.errors);
            }
            persist_scan_report(&mut conn, &report).map_err(|e| format!("persist scan: {e}"))?;
            load_persisted_runs(&conn).map_err(|e| format!("load runs: {e}"))
        })
        .await
        .map_err(|e| format!("worker failed: {e}"))?
    }

    #[tauri::command]
    pub async fn load_planner_evaluations(
        db_path: State<'_, AppDbPath>,
        run_folder_path: String,
    ) -> Result<Vec<PlannerAchievementEvaluation>, String> {
        let path = db_path.0.clone();
        tokio::task::spawn_blocking(
            move || -> Result<Vec<PlannerAchievementEvaluation>, String> {
                let conn = open_app_db(&path).map_err(|e| format!("open db: {e}"))?;
                let load = load_catalog_entries_with_issues(&conn)
                    .map_err(|e| format!("load achievements: {e}"))?;
                if !load.issues.is_empty() {
                    eprintln!("catalog load issues: {:?}", load.issues);
                }
                let facts = load_run_facts_from_db(&conn, &run_folder_path)
                    .map_err(|e| format!("load run facts: {e}"))?;
                let completed = load_displayed_completion_map(&conn)
                    .map_err(|e| format!("load displayed completion: {e}"))?
                    .into_iter()
                    .filter_map(|(achievement_id, displayed)| displayed.then_some(achievement_id))
                    .collect();
                let user_statuses = load_run_achievement_statuses(&conn, &run_folder_path)
                    .map_err(|e| format!("load run achievement statuses: {e}"))?;

                Ok(evaluate_planner_achievements(
                    load.entries,
                    &facts,
                    &completed,
                    &user_statuses,
                ))
            },
        )
        .await
        .map_err(|e| format!("worker failed: {e}"))?
    }

    #[tauri::command]
    pub async fn set_run_achievement_status(
        db_path: State<'_, AppDbPath>,
        run_folder_path: String,
        achievement_id: String,
        user_status: Option<String>,
    ) -> Result<(), String> {
        let path = db_path.0.clone();
        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let conn = open_app_db(&path).map_err(|e| format!("open db: {e}"))?;
            if let Some(status) = user_status.as_deref() {
                if !matches!(status, "planned" | "ignored") {
                    return Err(format!("unsupported run achievement status: {status}"));
                }
            }
            set_run_achievement_status_in_db(
                &conn,
                &run_folder_path,
                &achievement_id,
                user_status.as_deref(),
            )
            .map_err(|e| format!("set run achievement status: {e}"))
        })
        .await
        .map_err(|e| format!("worker failed: {e}"))?
    }
}
