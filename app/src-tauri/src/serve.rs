//! HTTP sidecar binary for the Stellaris Hunter scanner.
//!
//! Serves the compiled React UI as static files and provides 27 IPC-style
//! POST endpoints backed by the existing library.  Allows driving the same
//! UI from a plain browser without the Tauri desktop shell.
//!
//! Compiled with default features (no `desktop`, no `steam` required).

use std::path::PathBuf;

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use clap::Parser;
use serde::Deserialize;
use tower_http::services::ServeDir;

use crate::{
    db,
    error::Result,
    icons::IconSyncResult,
    ipc_helpers,
    model::{
        AchievementCatalogEntry, AchievementOverride, AppConfig, AppInfo, CatalogSyncResult,
        FactOverride, PersistedRunSummary, PlannerAchievementEvaluation, PlannerStatusCounts,
        RunAchievementNote, RunFactSummary, RunNote, SaveRunSummary, SteamSyncResult,
    },
    run_state, scan_all, ScanReport,
};

// ── CLI ────────────────────────────────────────────────────────────────────

#[derive(Debug, Parser)]
pub struct ServeArgs {
    /// Port to listen on.
    #[arg(long, default_value = "8787")]
    pub port: u16,

    /// Path to the app SQLite database.
    #[arg(long)]
    pub db_path: Option<PathBuf>,

    /// Path to the compiled frontend dist directory.
    #[arg(long)]
    pub dist_dir: Option<PathBuf>,
}

// ── State ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    db_path: PathBuf,
    dist_dir: Option<PathBuf>,
}

// ── Error type ─────────────────────────────────────────────────────────────

type ApiError = (StatusCode, String);

// ── Request structs (camelCase to match Tauri `invoke()` conventions) ──────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SingleFolderReq {
    run_folder_path: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunIdReq {
    run_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AchievementIdReq {
    achievement_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetCompletionOverrideReq {
    achievement_id: String,
    completed: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetRunAchievementStatusReq {
    run_folder_path: String,
    achievement_id: String,
    user_status: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetFactOverrideReq {
    run_folder_path: String,
    dimension: String,
    key: String,
    value_json: String,
    reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClearFactOverrideReq {
    run_folder_path: String,
    dimension: String,
    key: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetRunNoteReq {
    run_folder_path: String,
    note_text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetRunAchievementNoteReq {
    run_folder_path: String,
    achievement_id: String,
    notes: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClearRunAchievementNoteReq {
    run_folder_path: String,
    achievement_id: String,
}

#[derive(Deserialize)]
struct SaveAppConfigReq {
    config: AppConfig,
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Execute a blocking DB operation, returning JSON or an error tuple.
async fn db_call<T, F>(path: PathBuf, f: F) -> std::result::Result<Json<T>, ApiError>
where
    T: serde::Serialize + Send + 'static,
    F: FnOnce(&rusqlite::Connection) -> Result<T> + Send + 'static,
{
    let inner: std::result::Result<T, crate::error::Error> =
        tokio::task::spawn_blocking(move || {
            let conn = db::open_app_db(&path)?;
            f(&conn)
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("worker failed: {e}"),
            )
        })?;

    Ok(Json(
        inner.map_err(|e| (StatusCode::BAD_REQUEST, format!("{e}")))?,
    ))
}

async fn run_call<T, F>(work: F) -> std::result::Result<Json<T>, ApiError>
where
    T: serde::Serialize + Send + 'static,
    F: FnOnce() -> std::result::Result<T, String> + Send + 'static,
{
    let inner = ipc_helpers::run_blocking_split(work)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;

    Ok(Json(inner.map_err(|err| (StatusCode::BAD_REQUEST, err))?))
}

fn default_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".local/share/com.stellaris-hunter.scan/stellaris-hunter.db")
}

fn default_dist_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map(|path| path.join("dist"))
        .unwrap_or_else(|| manifest.join("dist"))
}

// ── IPC handlers ───────────────────────────────────────────────────────────

async fn handle_scan_local_state() -> std::result::Result<Json<ScanReport>, ApiError> {
    run_call(|| Ok(scan_all(None, None)))
        .await
        .map_err(|(status, err)| {
            (
                status,
                err.replacen("worker failed", "scan worker failed", 1),
            )
        })
}

async fn handle_sync_catalog(
    State(state): State<AppState>,
) -> std::result::Result<Json<CatalogSyncResult>, ApiError> {
    let path = state.db_path;
    let json_text = crate::catalog_sync::fetch_catalog(crate::catalog_sync::CATALOG_URL)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("fetch catalog: {e}")))?;

    Ok(Json(
        ipc_helpers::with_app_db_mut_split(path, move |conn| {
            let sync_result = crate::catalog_sync::sync_catalog_from_json(conn, &json_text)
                .map_err(|e| format!("sync catalog: {e}"))?;
            let _ = run_state::invalidate_all_evaluations(conn);
            Ok(sync_result)
        })
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?
        .map_err(|err| (StatusCode::BAD_REQUEST, err))?,
    ))
}

async fn handle_load_achievements(
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<AchievementCatalogEntry>>, ApiError> {
    db_call(state.db_path, crate::catalog::load_catalog_entries).await
}

async fn handle_load_catalog_info(
    State(state): State<AppState>,
) -> std::result::Result<Json<Option<crate::model::CatalogVersionMetadata>>, ApiError> {
    db_call(state.db_path, crate::catalog::load_catalog_metadata).await
}

async fn handle_load_completion_overrides(
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<AchievementOverride>>, ApiError> {
    db_call(state.db_path, crate::catalog::load_completion_overrides).await
}

async fn handle_set_completion_override(
    State(state): State<AppState>,
    Json(req): Json<SetCompletionOverrideReq>,
) -> std::result::Result<Json<()>, ApiError> {
    let id = req.achievement_id;
    let completed = req.completed;
    db_call(state.db_path, move |conn| {
        crate::catalog::set_completion_override(conn, &id, completed)?;
        run_state::invalidate_all_evaluations(conn)?;
        Ok(())
    })
    .await
}

async fn handle_clear_completion_override(
    State(state): State<AppState>,
    Json(req): Json<AchievementIdReq>,
) -> std::result::Result<Json<()>, ApiError> {
    let id = req.achievement_id;
    db_call(state.db_path, move |conn| {
        crate::catalog::clear_completion_override(conn, &id)?;
        run_state::invalidate_all_evaluations(conn)?;
        Ok(())
    })
    .await
}

async fn handle_load_runs(
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<PersistedRunSummary>>, ApiError> {
    db_call(state.db_path, run_state::load_persisted_runs).await
}

async fn handle_load_run_facts(
    State(state): State<AppState>,
    Json(req): Json<SingleFolderReq>,
) -> std::result::Result<Json<Vec<RunFactSummary>>, ApiError> {
    let folder = req.run_folder_path;
    db_call(state.db_path, move |conn| {
        run_state::load_run_facts(conn, &folder)
    })
    .await
}

async fn handle_rescan_saves(
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<PersistedRunSummary>>, ApiError> {
    let path = state.db_path;
    Ok(Json(
        ipc_helpers::with_app_db_mut_split(path, move |conn| {
            let report = scan_all(None, None);
            if !report.errors.is_empty() {
                eprintln!("scan errors before persistence: {:?}", report.errors);
            }
            run_state::persist_scan_report(conn, &report)
                .map_err(|e| format!("persist scan: {e}"))?;
            let _ = run_state::invalidate_all_evaluations(conn);
            run_state::load_persisted_runs(conn).map_err(|e| format!("load runs: {e}"))
        })
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?
        .map_err(|err| (StatusCode::BAD_REQUEST, err))?,
    ))
}

async fn handle_load_planner_evaluations(
    State(state): State<AppState>,
    Json(req): Json<SingleFolderReq>,
) -> std::result::Result<Json<Vec<PlannerAchievementEvaluation>>, ApiError> {
    let path = state.db_path;
    let folder = req.run_folder_path;
    Ok(Json(
        ipc_helpers::with_app_db_split(path, move |conn| {
            // Try cache first
            if let Some(cached) = run_state::load_evaluations(conn, &folder)
                .map_err(|e| format!("load cache: {e}"))?
            {
                return Ok(cached);
            }

            let load = crate::catalog::load_catalog_entries_with_issues(conn)
                .map_err(|e| format!("load achievements: {e}"))?;
            if !load.issues.is_empty() {
                eprintln!("catalog load issues: {:?}", load.issues);
            }
            let facts = run_state::load_run_facts(conn, &folder)
                .map_err(|e| format!("load run facts: {e}"))?;
            let completed = crate::catalog::load_displayed_completion_map(conn)
                .map_err(|e| format!("load displayed completion: {e}"))?
                .into_iter()
                .filter_map(|(id, displayed)| displayed.then_some(id))
                .collect();
            let user_statuses = run_state::load_run_achievement_statuses(conn, &folder)
                .map_err(|e| format!("load run achievement statuses: {e}"))?;
            let evaluations = crate::rules::evaluate_planner_achievements(
                load.entries,
                &facts,
                &completed,
                &user_statuses,
            );
            let _ = run_state::save_evaluations(conn, &folder, &evaluations);
            Ok(evaluations)
        })
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?
        .map_err(|err| (StatusCode::BAD_REQUEST, err))?,
    ))
}

async fn handle_load_planner_status_counts(
    State(state): State<AppState>,
    Json(req): Json<RunIdReq>,
) -> std::result::Result<Json<PlannerStatusCounts>, ApiError> {
    let run_id = req.run_id;
    db_call(state.db_path, move |conn| {
        run_state::load_planner_status_counts(conn, &run_id)
    })
    .await
}

async fn handle_reparse_run_save(
    State(state): State<AppState>,
    Json(req): Json<RunIdReq>,
) -> std::result::Result<Json<SaveRunSummary>, ApiError> {
    let path = state.db_path;
    let run_id = req.run_id;
    Ok(Json(
        ipc_helpers::with_app_db_mut_split(path, move |conn| {
            let run = run_state::reparse_run_save(conn, &run_id)
                .map_err(|e| format!("reparse run save: {e}"))?;
            let _ = run_state::invalidate_evaluations(conn, &run_id);
            Ok(run)
        })
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?
        .map_err(|err| (StatusCode::BAD_REQUEST, err))?,
    ))
}

async fn handle_set_run_achievement_status(
    State(state): State<AppState>,
    Json(req): Json<SetRunAchievementStatusReq>,
) -> std::result::Result<Json<()>, ApiError> {
    let folder = req.run_folder_path;
    let id = req.achievement_id;
    let status = req.user_status;
    db_call(state.db_path, move |conn| {
        // Validate user status value (copied from commands.rs)
        if let Some(ref s) = status {
            if !matches!(s.as_str(), "planned" | "ignored") {
                return Err(crate::error::Error::Validation(format!(
                    "unsupported run achievement status: {s}"
                )));
            }
        }
        run_state::set_run_achievement_status(conn, &folder, &id, status.as_deref())?;
        run_state::invalidate_evaluations(conn, &folder)?;
        Ok(())
    })
    .await
}

async fn handle_load_fact_overrides(
    State(state): State<AppState>,
    Json(req): Json<SingleFolderReq>,
) -> std::result::Result<Json<Vec<FactOverride>>, ApiError> {
    let folder = req.run_folder_path;
    db_call(state.db_path, move |conn| {
        run_state::load_fact_overrides(conn, &folder)
    })
    .await
}

async fn handle_set_fact_override(
    State(state): State<AppState>,
    Json(req): Json<SetFactOverrideReq>,
) -> std::result::Result<Json<()>, ApiError> {
    let folder = req.run_folder_path;
    let dim = req.dimension;
    let key = req.key;
    let value_json = req.value_json;
    let reason = req.reason;
    db_call(state.db_path, move |conn| {
        let value: serde_json::Value = serde_json::from_str(&value_json)
            .map_err(|e| crate::error::Error::Parse(format!("invalid value_json: {e}")))?;
        run_state::set_fact_override(conn, &folder, &dim, &key, &value, reason.as_deref())?;
        run_state::invalidate_evaluations(conn, &folder)?;
        Ok(())
    })
    .await
}

async fn handle_clear_fact_override(
    State(state): State<AppState>,
    Json(req): Json<ClearFactOverrideReq>,
) -> std::result::Result<Json<()>, ApiError> {
    let folder = req.run_folder_path;
    let dim = req.dimension;
    let key = req.key;
    db_call(state.db_path, move |conn| {
        run_state::clear_fact_override(conn, &folder, &dim, &key)?;
        run_state::invalidate_evaluations(conn, &folder)?;
        Ok(())
    })
    .await
}

async fn handle_load_run_notes(
    State(state): State<AppState>,
    Json(req): Json<SingleFolderReq>,
) -> std::result::Result<Json<Option<RunNote>>, ApiError> {
    let folder = req.run_folder_path;
    db_call(state.db_path, move |conn| {
        run_state::load_run_notes(conn, &folder)
    })
    .await
}

async fn handle_set_run_note(
    State(state): State<AppState>,
    Json(req): Json<SetRunNoteReq>,
) -> std::result::Result<Json<()>, ApiError> {
    let folder = req.run_folder_path;
    let text = req.note_text;
    db_call(state.db_path, move |conn| {
        run_state::set_run_note(conn, &folder, &text)
    })
    .await
}

async fn handle_clear_run_note(
    State(state): State<AppState>,
    Json(req): Json<SingleFolderReq>,
) -> std::result::Result<Json<()>, ApiError> {
    let folder = req.run_folder_path;
    db_call(state.db_path, move |conn| {
        run_state::clear_run_note(conn, &folder)
    })
    .await
}

async fn handle_load_run_achievement_notes(
    State(state): State<AppState>,
    Json(req): Json<SingleFolderReq>,
) -> std::result::Result<Json<Vec<RunAchievementNote>>, ApiError> {
    let folder = req.run_folder_path;
    db_call(state.db_path, move |conn| {
        run_state::load_run_achievement_notes(conn, &folder)
    })
    .await
}

async fn handle_set_run_achievement_note(
    State(state): State<AppState>,
    Json(req): Json<SetRunAchievementNoteReq>,
) -> std::result::Result<Json<()>, ApiError> {
    let folder = req.run_folder_path;
    let id = req.achievement_id;
    let notes = req.notes;
    db_call(state.db_path, move |conn| {
        run_state::set_run_achievement_note(conn, &folder, &id, &notes)
    })
    .await
}

async fn handle_clear_run_achievement_note(
    State(state): State<AppState>,
    Json(req): Json<ClearRunAchievementNoteReq>,
) -> std::result::Result<Json<()>, ApiError> {
    let folder = req.run_folder_path;
    let id = req.achievement_id;
    db_call(state.db_path, move |conn| {
        run_state::clear_run_achievement_note(conn, &folder, &id)
    })
    .await
}

async fn handle_load_app_config(
    State(state): State<AppState>,
) -> std::result::Result<Json<AppConfig>, ApiError> {
    db_call(state.db_path, db::load_app_config).await
}

async fn handle_save_app_config(
    State(state): State<AppState>,
    Json(req): Json<SaveAppConfigReq>,
) -> std::result::Result<Json<()>, ApiError> {
    let config = req.config;
    db_call(state.db_path, move |conn| {
        db::save_app_config(conn, &config)
    })
    .await
}

async fn handle_load_app_info(
    State(state): State<AppState>,
) -> std::result::Result<Json<AppInfo>, ApiError> {
    db_call(state.db_path, db::load_app_info).await
}

async fn handle_get_achievement_icon(
    State(state): State<AppState>,
    Json(req): Json<AchievementIdReq>,
) -> std::result::Result<Response, ApiError> {
    let path = state.db_path;
    let achievement_id = req.achievement_id;

    let result = ipc_helpers::run_blocking_split(move || {
        let app_data_dir = path.parent().ok_or("invalid db path")?.to_path_buf();
        let cache = crate::icons::IconCache::new(&app_data_dir);
        cache
            .read(&achievement_id)
            .map_err(|e| format!("read icon: {e}"))
    })
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?
    .map_err(|err| (StatusCode::BAD_REQUEST, err))?;

    match result {
        Some(bytes) => Response::builder()
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .body(axum::body::Body::from(bytes))
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("build icon response: {err}"),
                )
            }),
        None => Ok(StatusCode::NO_CONTENT.into_response()),
    }
}

async fn handle_sync_icons(
    State(state): State<AppState>,
) -> std::result::Result<Json<IconSyncResult>, ApiError> {
    let path = state.db_path;

    let result = ipc_helpers::run_blocking_split(move || {
            let app_data_dir = path.parent().ok_or("invalid db path")?.to_path_buf();
            let db_file = path;
            let cache = crate::icons::IconCache::new(&app_data_dir);
            cache
                .ensure_dir()
                .map_err(|e| format!("create cache dir: {e}"))?;

            let conn =
                db::open_app_db(&db_file).map_err(|e| format!("open db: {e}"))?;
            let mut stmt = conn
                .prepare(
                    "SELECT id, steam_api_name FROM achievements WHERE steam_api_name IS NOT NULL AND deprecated = 0",
                )
                .map_err(|e| format!("prepare: {e}"))?;
            let names: Vec<(String, String)> = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|e| format!("query: {e}"))?
                .filter_map(|r| r.ok())
                .collect();

            crate::icons::sync_icons_from_steam(&cache, &names)
                .map_err(|e| format!("sync icons: {e}"))
        })
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?
        .map_err(|err| (StatusCode::BAD_REQUEST, err))?;

    Ok(Json(result))
}

/// WSL stub — returns the same error string as `commands::sync_steam_achievements`.
async fn handle_sync_steam_achievements() -> std::result::Result<Json<SteamSyncResult>, ApiError> {
    Err((
        StatusCode::BAD_REQUEST,
        "Steam sync requires Windows with Steam client running".to_string(),
    ))
}

// ── Router construction ────────────────────────────────────────────────────

fn build_router(state: AppState) -> Router {
    let ipc_routes = Router::new()
        .route("/ipc/scan_local_state", post(handle_scan_local_state))
        .route("/ipc/sync_catalog", post(handle_sync_catalog))
        .route("/ipc/load_achievements", post(handle_load_achievements))
        .route("/ipc/load_catalog_info", post(handle_load_catalog_info))
        .route(
            "/ipc/load_completion_overrides",
            post(handle_load_completion_overrides),
        )
        .route(
            "/ipc/set_completion_override",
            post(handle_set_completion_override),
        )
        .route(
            "/ipc/clear_completion_override",
            post(handle_clear_completion_override),
        )
        .route("/ipc/load_runs", post(handle_load_runs))
        .route("/ipc/load_run_facts", post(handle_load_run_facts))
        .route("/ipc/rescan_saves", post(handle_rescan_saves))
        .route(
            "/ipc/load_planner_evaluations",
            post(handle_load_planner_evaluations),
        )
        .route(
            "/ipc/load_planner_status_counts",
            post(handle_load_planner_status_counts),
        )
        .route("/ipc/reparse_run_save", post(handle_reparse_run_save))
        .route(
            "/ipc/set_run_achievement_status",
            post(handle_set_run_achievement_status),
        )
        .route("/ipc/load_fact_overrides", post(handle_load_fact_overrides))
        .route("/ipc/set_fact_override", post(handle_set_fact_override))
        .route("/ipc/clear_fact_override", post(handle_clear_fact_override))
        .route("/ipc/load_run_notes", post(handle_load_run_notes))
        .route("/ipc/set_run_note", post(handle_set_run_note))
        .route("/ipc/clear_run_note", post(handle_clear_run_note))
        .route(
            "/ipc/load_run_achievement_notes",
            post(handle_load_run_achievement_notes),
        )
        .route(
            "/ipc/set_run_achievement_note",
            post(handle_set_run_achievement_note),
        )
        .route(
            "/ipc/clear_run_achievement_note",
            post(handle_clear_run_achievement_note),
        )
        .route("/ipc/load_app_config", post(handle_load_app_config))
        .route("/ipc/save_app_config", post(handle_save_app_config))
        .route("/ipc/load_app_info", post(handle_load_app_info))
        .route(
            "/ipc/get_achievement_icon",
            post(handle_get_achievement_icon),
        )
        .route("/ipc/sync_icons", post(handle_sync_icons))
        .route(
            "/ipc/sync_steam_achievements",
            post(handle_sync_steam_achievements),
        )
        .with_state(state.clone());

    // Static file serving as fallback
    let static_router = match &state.dist_dir {
        Some(dir) if dir.join("index.html").exists() => {
            let svc = ServeDir::new(dir).not_found_service(
                tower_http::services::fs::ServeFile::new(dir.join("index.html")),
            );
            Router::new().fallback_service(svc)
        }
        _ => Router::new().fallback(|| async {
            (
                StatusCode::OK,
                [("content-type", "text/html; charset=utf-8")],
                concat!(
                    "<!DOCTYPE html><html><body>",
                    "<h1>Build the frontend first</h1>",
                    "<pre>npm --prefix app run build</pre>",
                    "</body></html>"
                ),
            )
        }),
    };

    ipc_routes.merge(static_router)
}

// ── Public entry point ─────────────────────────────────────────────────────

pub async fn run(args: ServeArgs) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let db_path = args.db_path.unwrap_or_else(default_db_path);
    let dist_dir = args.dist_dir.or_else(|| {
        let def = default_dist_dir();
        Some(def)
    });

    eprintln!("DB path: {}", db_path.display());
    if let Some(ref d) = dist_dir {
        eprintln!("Dist dir: {}", d.display());
    }

    // Mirror run_app() DB setup
    {
        let mut conn =
            db::open_app_db(&db_path).map_err(|e| format!("failed to open app db: {e}"))?;

        match db::ensure_catalog_imported(&mut conn) {
            Ok(true) => eprintln!("imported bundled catalog into app db"),
            Ok(false) => eprintln!("catalog already imported"),
            Err(e) => eprintln!("warning: catalog import failed: {e}"),
        }
    }

    let state = AppState { db_path, dist_dir };
    let app = build_router(state);

    let addr = format!("127.0.0.1:{}", args.port);
    eprintln!("Listening on http://{addr}");
    eprintln!("IPC routes at http://{addr}/ipc/<command>");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Build a test router with a given DB path (for integration tests).
///
/// Creates a router backed by a disposable `AppState` (no dist directory).
pub fn build_test_router(db_path: PathBuf) -> Router {
    build_router(AppState {
        db_path,
        dist_dir: None,
    })
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_load_run_notes_empty_db() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("test.db");

        // Initialize the DB schema as run_app() does
        {
            let mut conn = db::open_app_db(&db_path).unwrap();
            let _ = db::ensure_catalog_imported(&mut conn);
        }

        let state = AppState {
            db_path,
            dist_dir: None,
        };
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/ipc/load_run_notes")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"runFolderPath":"/nonexistent"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        // An empty DB with a nonexistent run should return JSON null
        assert_eq!(value, serde_json::Value::Null);
    }
}
