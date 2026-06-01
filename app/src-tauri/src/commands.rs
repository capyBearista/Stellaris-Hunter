use crate::{scan_all, ScanReport};

#[cfg_attr(feature = "desktop", tauri::command)]
pub async fn scan_local_state() -> Result<ScanReport, String> {
    tokio::task::spawn_blocking(|| scan_all(None, None))
        .await
        .map_err(|err| format!("scan worker failed: {err}"))
}
