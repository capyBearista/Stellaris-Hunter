use std::path::PathBuf;

use rusqlite::Connection;

pub async fn run_blocking_string<T, F>(work: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|err| format!("worker failed: {err}"))?
}

pub async fn run_blocking_split<T, F>(
    work: F,
) -> std::result::Result<std::result::Result<T, String>, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|err| format!("worker failed: {err}"))
}

pub async fn with_app_db<T, F>(path: PathBuf, work: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&Connection) -> Result<T, String> + Send + 'static,
{
    run_blocking_string(move || {
        let conn = crate::db::open_app_db(&path).map_err(|err| format!("open db: {err}"))?;
        work(&conn)
    })
    .await
}

pub async fn with_app_db_mut<T, F>(path: PathBuf, work: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&mut Connection) -> Result<T, String> + Send + 'static,
{
    run_blocking_string(move || {
        let mut conn = crate::db::open_app_db(&path).map_err(|err| format!("open db: {err}"))?;
        work(&mut conn)
    })
    .await
}

pub async fn with_app_db_split<T, F>(
    path: PathBuf,
    work: F,
) -> std::result::Result<std::result::Result<T, String>, String>
where
    T: Send + 'static,
    F: FnOnce(&Connection) -> Result<T, String> + Send + 'static,
{
    run_blocking_split(move || {
        let conn = crate::db::open_app_db(&path).map_err(|err| format!("open db: {err}"))?;
        work(&conn)
    })
    .await
}

pub async fn with_app_db_mut_split<T, F>(
    path: PathBuf,
    work: F,
) -> std::result::Result<std::result::Result<T, String>, String>
where
    T: Send + 'static,
    F: FnOnce(&mut Connection) -> Result<T, String> + Send + 'static,
{
    run_blocking_split(move || {
        let mut conn = crate::db::open_app_db(&path).map_err(|err| format!("open db: {err}"))?;
        work(&mut conn)
    })
    .await
}
