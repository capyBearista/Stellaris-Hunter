//! Integration tests for all 27 IPC routes exposed by the HTTP sidecar.
//!
//! Each route is a POST endpoint at `/ipc/<command>`.  Routes that accept
//! request-body parameters are exercised with minimal valid JSON.  Routes
//! that depend on network or Windows-only resources are asserted to return
//! error status codes rather than succeeding.
//!
//! Uses `stellaris_hunter_scan::serve::build_test_router` to create an axum
//! `Router` backed by a temporary SQLite database.  Because
//! `tower::ServiceExt::oneshot` consumes the router, round-trip tests
//! manually create fresh routers from the same DB path.

use std::path::PathBuf;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::Value;
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A real achievement ID guaranteed to exist in the bundled catalog.
const KNOWN_ACHIEVEMENT_ID: &str = "archaeologist";

/// Initialise a test database at `db_path` (schema + bundled catalog import).
fn init_db(db_path: &std::path::Path) {
    let mut conn =
        stellaris_hunter_scan::db::open_app_db(db_path).expect("open_app_db should succeed");
    stellaris_hunter_scan::db::ensure_catalog_imported(&mut conn)
        .expect("ensure_catalog_imported should succeed");
}

/// Initialise a test database **and** insert a test run row so that FK
/// constraints on `run_folder_path` are satisfied.
fn init_db_with_run(db_path: &std::path::Path) {
    let mut conn =
        stellaris_hunter_scan::db::open_app_db(db_path).expect("open_app_db should succeed");
    stellaris_hunter_scan::db::ensure_catalog_imported(&mut conn)
        .expect("ensure_catalog_imported should succeed");

    // Insert a synthetic run so write operations referencing "/test" don't
    // violate the `runs` FK constraint.
    conn.execute(
        "INSERT OR IGNORE INTO runs (folder_path, run_folder, updated_at)
         VALUES ('/test', 'test', datetime('now'))",
        [],
    )
    .expect("insert test run should succeed");
}

/// Create a temporary directory with an initialised DB and a ready-to-use
/// `Router`.  Returns the `TempDir` (keeps the DB alive) and the router.
fn setup_router() -> (tempfile::TempDir, Router) {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("test.db");
    init_db(&db_path);
    let app = stellaris_hunter_scan::serve::build_test_router(db_path);
    (tmp, app)
}

/// Like `setup_router` but also inserts a test run row into the DB for
/// use in write-operation tests that have FK constraints.
fn setup_router_with_run() -> (tempfile::TempDir, Router) {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("test.db");
    init_db_with_run(&db_path);
    let app = stellaris_hunter_scan::serve::build_test_router(db_path);
    (tmp, app)
}

/// Build a fresh router from an existing DB path.
fn router_for(db_path: PathBuf) -> Router {
    stellaris_hunter_scan::serve::build_test_router(db_path)
}

/// Response from a single request.
struct Response {
    status: StatusCode,
    raw_body: Vec<u8>,
}

impl Response {
    /// Parse the body as JSON (returns `None` for non-JSON responses).
    fn json(&self) -> Option<Value> {
        serde_json::from_slice(&self.raw_body).ok()
    }

    /// Decode the body as UTF-8 text.
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.raw_body).to_string()
    }
}

/// Send a POST request and return the response.
async fn send(router: Router, uri: &str, body: &str) -> Response {
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let raw_body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();

    Response { status, raw_body }
}

/// Convenience: POST with an empty JSON object `{}`.
async fn post_empty(router: Router, uri: &str) -> Response {
    send(router, uri, "{}").await
}

// ---------------------------------------------------------------------------
// 1  /ipc/load_achievements
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_load_achievements_returns_all() {
    let (_tmp, router) = setup_router();
    let res = post_empty(router, "/ipc/load_achievements").await;

    assert_eq!(res.status, StatusCode::OK);
    let val = res.json().expect("body should be present");
    let arr = val.as_array().expect("should be array");
    // Bundled catalog contains 219 achievements
    assert_eq!(
        arr.len(),
        219,
        "should have 219 achievements from bundled catalog"
    );
}

// ---------------------------------------------------------------------------
// 2  /ipc/load_catalog_info
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_load_catalog_info_returns_metadata() {
    let (_tmp, router) = setup_router();
    let res = post_empty(router, "/ipc/load_catalog_info").await;

    assert_eq!(res.status, StatusCode::OK);
    let meta = res
        .json()
        .expect("body should not be null — catalog was imported");
    assert!(meta.is_object(), "body should be a JSON object");
    assert!(
        meta["catalog_version"]
            .as_str()
            .is_some_and(|v| !v.is_empty()),
        "catalog_version should be a non-empty string"
    );
}

// ---------------------------------------------------------------------------
// 3  /ipc/load_completion_overrides
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_load_completion_overrides_empty() {
    let (_tmp, router) = setup_router();
    let res = post_empty(router, "/ipc/load_completion_overrides").await;

    assert_eq!(res.status, StatusCode::OK);
    let val = res.json().expect("body should be present");
    let arr = val.as_array().expect("should be array");
    assert!(arr.is_empty(), "no overrides set yet");
}

// ---------------------------------------------------------------------------
// 4  /ipc/set_completion_override
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_set_completion_override_ok() {
    // This route calls `ensure_achievement_exists` which needs a real ID.
    let (_tmp, router) = setup_router();
    let res = send(
        router,
        "/ipc/set_completion_override",
        &format!(
            r#"{{"achievementId":"{0}","completed":true}}"#,
            KNOWN_ACHIEVEMENT_ID
        ),
    )
    .await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(
        res.json(),
        Some(Value::Null),
        "response body should be null"
    );
}

// ---------------------------------------------------------------------------
// 5  /ipc/clear_completion_override
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_clear_completion_override_ok() {
    let (_tmp, router) = setup_router();

    // Set first (uses a real achievement ID)
    let res1 = send(
        router,
        "/ipc/set_completion_override",
        &format!(
            r#"{{"achievementId":"{0}","completed":true}}"#,
            KNOWN_ACHIEVEMENT_ID
        ),
    )
    .await;
    assert_eq!(res1.status, StatusCode::OK);

    // Fresh router for clear
    let router2 = router_for(_tmp.path().join("test.db"));
    let res2 = send(
        router2,
        "/ipc/clear_completion_override",
        &format!(r#"{{"achievementId":"{0}"}}"#, KNOWN_ACHIEVEMENT_ID),
    )
    .await;
    assert_eq!(res2.status, StatusCode::OK);
    assert_eq!(res2.json(), Some(Value::Null));
}

// ---------------------------------------------------------------------------
// 6  /ipc/load_runs
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_load_runs_empty() {
    let (_tmp, router) = setup_router();
    let res = post_empty(router, "/ipc/load_runs").await;

    assert_eq!(res.status, StatusCode::OK);
    let val = res.json().expect("body should be present");
    let arr = val.as_array().expect("should be array");
    assert!(arr.is_empty(), "no runs yet");
}

// ---------------------------------------------------------------------------
// 7  /ipc/load_run_facts
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_load_run_facts_nonexistent() {
    let (_tmp, router) = setup_router();
    let res = send(
        router,
        "/ipc/load_run_facts",
        r#"{"runFolderPath":"/nonexistent"}"#,
    )
    .await;

    assert_eq!(res.status, StatusCode::OK);
    let val = res.json().expect("body should be present");
    let arr = val.as_array().expect("should be array");
    assert!(arr.is_empty(), "no facts for nonexistent run");
}

// ---------------------------------------------------------------------------
// 8  /ipc/rescan_saves
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_rescan_saves_ok() {
    let (_tmp, router) = setup_router();

    // On this system there is probably no Stellaris install, so rescan_saves
    // should succeed with an empty run list.
    let res = post_empty(router, "/ipc/rescan_saves").await;

    assert_eq!(res.status, StatusCode::OK);
    let val = res.json().expect("body should be present");
    assert!(val.is_array(), "should be a JSON array");
}

// ---------------------------------------------------------------------------
// 9  /ipc/load_planner_evaluations
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_load_planner_evaluations_nonexistent() {
    let (_tmp, router) = setup_router();
    let res = send(
        router,
        "/ipc/load_planner_evaluations",
        r#"{"runFolderPath":"/nonexistent"}"#,
    )
    .await;

    assert_eq!(res.status, StatusCode::OK);
    let val = res.json().expect("body should be present");
    let arr = val.as_array().expect("should be array");
    // The planner evaluates all 219 achievements even for a nonexistent run
    assert_eq!(arr.len(), 219, "should evaluate all 219 achievements");
}

// ---------------------------------------------------------------------------
// 10 /ipc/set_run_achievement_status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_set_run_achievement_status_planned() {
    let (_tmp, router) = setup_router_with_run();
    let res = send(
        router,
        "/ipc/set_run_achievement_status",
        &format!(
            r#"{{"runFolderPath":"/test","achievementId":"{0}","userStatus":"planned"}}"#,
            KNOWN_ACHIEVEMENT_ID,
        ),
    )
    .await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json(), Some(Value::Null));
}

#[tokio::test]
async fn test_set_run_achievement_status_ignored() {
    let (_tmp, router) = setup_router_with_run();
    let res = send(
        router,
        "/ipc/set_run_achievement_status",
        &format!(
            r#"{{"runFolderPath":"/test","achievementId":"{0}","userStatus":"ignored"}}"#,
            KNOWN_ACHIEVEMENT_ID,
        ),
    )
    .await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json(), Some(Value::Null));
}

#[tokio::test]
async fn test_set_run_achievement_status_clear() {
    let (_tmp, router) = setup_router_with_run();
    let res = send(
        router,
        "/ipc/set_run_achievement_status",
        &format!(
            r#"{{"runFolderPath":"/test","achievementId":"{0}","userStatus":null}}"#,
            KNOWN_ACHIEVEMENT_ID,
        ),
    )
    .await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json(), Some(Value::Null));
}

#[tokio::test]
async fn test_set_run_achievement_status_unsupported() {
    let (_tmp, router) = setup_router_with_run();
    let res = send(
        router,
        "/ipc/set_run_achievement_status",
        &format!(
            r#"{{"runFolderPath":"/test","achievementId":"{0}","userStatus":"bogus"}}"#,
            KNOWN_ACHIEVEMENT_ID,
        ),
    )
    .await;

    // Unsupported status → 400
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// 11 /ipc/load_fact_overrides
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_load_fact_overrides_empty() {
    let (_tmp, router) = setup_router();
    let res = send(
        router,
        "/ipc/load_fact_overrides",
        r#"{"runFolderPath":"/nonexistent"}"#,
    )
    .await;

    assert_eq!(res.status, StatusCode::OK);
    let val = res.json().expect("body should be present");
    let arr = val.as_array().expect("should be array");
    assert!(arr.is_empty(), "no overrides yet");
}

// ---------------------------------------------------------------------------
// 12 /ipc/set_fact_override
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_set_fact_override_ok() {
    let (_tmp, router) = setup_router_with_run();
    let res = send(
        router,
        "/ipc/set_fact_override",
        r#"{
            "runFolderPath":"/test",
            "dimension":"empire",
            "key":"origin",
            "valueJson":"\"test_origin\"",
            "reason":"test"
        }"#,
    )
    .await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json(), Some(Value::Null));
}

// ---------------------------------------------------------------------------
// 13 /ipc/clear_fact_override
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_clear_fact_override_ok() {
    let (_tmp, router) = setup_router_with_run();

    // Set first
    let res1 = send(
        router,
        "/ipc/set_fact_override",
        r#"{
            "runFolderPath":"/test",
            "dimension":"empire",
            "key":"origin",
            "valueJson":"\"test_origin\"",
            "reason":"test"
        }"#,
    )
    .await;
    assert_eq!(res1.status, StatusCode::OK);

    // Fresh router for clear
    let router2 = router_for(_tmp.path().join("test.db"));
    let res2 = send(
        router2,
        "/ipc/clear_fact_override",
        r#"{"runFolderPath":"/test","dimension":"empire","key":"origin"}"#,
    )
    .await;

    assert_eq!(res2.status, StatusCode::OK);
    assert_eq!(res2.json(), Some(Value::Null));
}

// ---------------------------------------------------------------------------
// 14 /ipc/load_run_notes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_load_run_notes_null() {
    let (_tmp, router) = setup_router();

    let res = send(
        router,
        "/ipc/load_run_notes",
        r#"{"runFolderPath":"/nonexistent"}"#,
    )
    .await;

    assert_eq!(res.status, StatusCode::OK);
    // Non-existent run returns JSON null (Option<RunNote> = None)
    assert_eq!(res.json(), Some(Value::Null));
}

// ---------------------------------------------------------------------------
// 15 /ipc/set_run_note
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_set_run_note_ok() {
    let (_tmp, router) = setup_router_with_run();
    let res = send(
        router,
        "/ipc/set_run_note",
        r#"{"runFolderPath":"/test","noteText":"hello world"}"#,
    )
    .await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json(), Some(Value::Null));
}

// ---------------------------------------------------------------------------
// 16 /ipc/clear_run_note
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_clear_run_note_ok() {
    let (_tmp, router) = setup_router_with_run();

    // Set first
    let res1 = send(
        router,
        "/ipc/set_run_note",
        r#"{"runFolderPath":"/test","noteText":"will be cleared"}"#,
    )
    .await;
    assert_eq!(res1.status, StatusCode::OK);

    let router2 = router_for(_tmp.path().join("test.db"));
    let res2 = send(
        router2,
        "/ipc/clear_run_note",
        r#"{"runFolderPath":"/test"}"#,
    )
    .await;

    assert_eq!(res2.status, StatusCode::OK);
    assert_eq!(res2.json(), Some(Value::Null));
}

// ---------------------------------------------------------------------------
// 17 /ipc/load_run_achievement_notes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_load_run_achievement_notes_empty() {
    let (_tmp, router) = setup_router();
    let res = send(
        router,
        "/ipc/load_run_achievement_notes",
        r#"{"runFolderPath":"/nonexistent"}"#,
    )
    .await;

    assert_eq!(res.status, StatusCode::OK);
    let val = res.json().expect("body should be present");
    let arr = val.as_array().expect("should be array");
    assert!(arr.is_empty(), "no achievement notes yet");
}

// ---------------------------------------------------------------------------
// 18 /ipc/set_run_achievement_note
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_set_run_achievement_note_ok() {
    let (_tmp, router) = setup_router_with_run();
    let res = send(
        router,
        "/ipc/set_run_achievement_note",
        &format!(
            r#"{{"runFolderPath":"/test","achievementId":"{0}","notes":"a note"}}"#,
            KNOWN_ACHIEVEMENT_ID,
        ),
    )
    .await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json(), Some(Value::Null));
}

// ---------------------------------------------------------------------------
// 19 /ipc/clear_run_achievement_note
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_clear_run_achievement_note_ok() {
    let (_tmp, router) = setup_router_with_run();

    // Set first
    let res1 = send(
        router,
        "/ipc/set_run_achievement_note",
        &format!(
            r#"{{"runFolderPath":"/test","achievementId":"{0}","notes":"a note"}}"#,
            KNOWN_ACHIEVEMENT_ID,
        ),
    )
    .await;
    assert_eq!(res1.status, StatusCode::OK);

    let router2 = router_for(_tmp.path().join("test.db"));
    let res2 = send(
        router2,
        "/ipc/clear_run_achievement_note",
        &format!(
            r#"{{"runFolderPath":"/test","achievementId":"{0}"}}"#,
            KNOWN_ACHIEVEMENT_ID,
        ),
    )
    .await;

    assert_eq!(res2.status, StatusCode::OK);
    assert_eq!(res2.json(), Some(Value::Null));
}

// ---------------------------------------------------------------------------
// 20 /ipc/load_app_config
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_load_app_config_default() {
    let (_tmp, router) = setup_router();
    let res = post_empty(router, "/ipc/load_app_config").await;

    assert_eq!(res.status, StatusCode::OK);
    let config = res.json().expect("body should be present");
    assert_eq!(config["installPathOverride"], Value::Null);
    assert_eq!(config["documentsPathOverride"], Value::Null);
}

// ---------------------------------------------------------------------------
// 21 /ipc/save_app_config
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_save_app_config_null() {
    let (_tmp, router) = setup_router();
    let res = send(
        router,
        "/ipc/save_app_config",
        r#"{"config":{"installPathOverride":null,"documentsPathOverride":null}}"#,
    )
    .await;

    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json(), Some(Value::Null));
}

#[tokio::test]
async fn test_save_app_config_with_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("test.db");
    init_db(&db_path);

    let router = router_for(db_path.clone());
    let res_save = send(
        router,
        "/ipc/save_app_config",
        r#"{"config":{"installPathOverride":"/custom/install","documentsPathOverride":"/custom/docs"}}"#,
    )
    .await;
    assert_eq!(res_save.status, StatusCode::OK);
    assert_eq!(res_save.json(), Some(Value::Null));

    // Verify by opening a fresh router against the same DB
    let router2 = router_for(db_path);
    let res_load = post_empty(router2, "/ipc/load_app_config").await;

    assert_eq!(res_load.status, StatusCode::OK);
    let cfg = res_load.json().expect("body should be present");
    assert_eq!(cfg["installPathOverride"], "/custom/install");
    assert_eq!(cfg["documentsPathOverride"], "/custom/docs");
}

// ---------------------------------------------------------------------------
// 22 /ipc/load_app_info
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_load_app_info_ok() {
    let (_tmp, router) = setup_router();

    let res = post_empty(router, "/ipc/load_app_info").await;
    assert_eq!(res.status, StatusCode::OK);

    let info = res.json().expect("body should be present");
    assert!(
        info["appVersion"].as_str().is_some_and(|v| !v.is_empty()),
        "appVersion should be a non-empty string"
    );
    assert!(
        info["catalogVersion"]
            .as_str()
            .is_some_and(|v| !v.is_empty()),
        "catalogVersion should be non-empty after catalog import"
    );
}

// ---------------------------------------------------------------------------
// 23 /ipc/get_achievement_icon
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_achievement_icon_missing() {
    let (_tmp, router) = setup_router();
    let res = send(
        router,
        "/ipc/get_achievement_icon",
        r#"{"achievementId":"nonexistent_icon_ach"}"#,
    )
    .await;

    // Missing icon → 204 No Content
    assert_eq!(res.status, StatusCode::NO_CONTENT);
}

// ---------------------------------------------------------------------------
// 24 /ipc/scan_local_state
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_scan_local_state_ok() {
    let (_tmp, router) = setup_router();
    let res = post_empty(router, "/ipc/scan_local_state").await;

    // scan_all should always succeed (even with no Stellaris install)
    assert_eq!(res.status, StatusCode::OK);
    let report = res.json().expect("body should be present");
    assert!(report.is_object(), "ScanReport should be a JSON object");
    assert!(
        report.get("install").is_some(),
        "ScanReport should have an install field"
    );
    assert!(
        report.get("documents").is_some(),
        "ScanReport should have a documents field"
    );
    assert!(
        report.get("errors").is_some(),
        "ScanReport should have an errors field"
    );
}

// ---------------------------------------------------------------------------
// 25 /ipc/sync_catalog  (fails without network)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_sync_catalog_network_fail() {
    let (_tmp, router) = setup_router();
    let res = post_empty(router, "/ipc/sync_catalog").await;

    // sync_catalog tries to fetch from GitHub; without network it
    // returns BAD_REQUEST (the inner error is mapped through ApiError).
    assert!(
        res.status == StatusCode::BAD_REQUEST || res.status == StatusCode::INTERNAL_SERVER_ERROR,
        "sync_catalog should fail without network, got {}",
        res.status,
    );
}

// ---------------------------------------------------------------------------
// 26 /ipc/sync_icons  (fails on Linux / without Steam CDN)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_sync_icons_fails() {
    let (_tmp, router) = setup_router();
    let res = post_empty(router, "/ipc/sync_icons").await;

    // sync_icons tries to contact the Steam CDN; on Linux this fails.
    assert!(
        res.status == StatusCode::BAD_REQUEST || res.status == StatusCode::INTERNAL_SERVER_ERROR,
        "sync_icons should fail on Linux, got {}",
        res.status,
    );
}

// ---------------------------------------------------------------------------
// 27 /ipc/sync_steam_achievements  (always 400 on Linux)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_sync_steam_achievements_fails() {
    let (_tmp, router) = setup_router();
    let res = post_empty(router, "/ipc/sync_steam_achievements").await;

    // Always returns 400 with a specific message on Linux
    assert_eq!(res.status, StatusCode::BAD_REQUEST);

    // The error is returned as a plain-text body, not JSON.
    let text = res.text();
    assert!(
        text.contains("Steam sync requires Windows"),
        "unexpected error message: {text}"
    );
}
