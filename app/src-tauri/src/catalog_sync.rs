use crate::{
    catalog::{import_catalog, load_catalog_metadata, parse_catalog_json},
    model::CatalogSyncResult,
};
use rusqlite::Connection;

/// URL for the latest published catalog on GitHub.
pub const CATALOG_URL: &str =
    "https://raw.githubusercontent.com/capyBearista/Stellaris-Hunter/main/catalog/latest.json";

/// Fetch catalog JSON from a URL via HTTP GET with 30s timeout.
pub async fn fetch_catalog(url: &str) -> Result<String, CatalogSyncError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| CatalogSyncError::Http(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| CatalogSyncError::Http(e.to_string()))?;

    if !response.status().is_success() {
        return Err(CatalogSyncError::Http(format!(
            "HTTP {}",
            response.status()
        )));
    }

    response
        .text()
        .await
        .map_err(|e| CatalogSyncError::Http(e.to_string()))
}

/// Compare semver versions. Returns true if `new` is strictly greater than `old`.
pub fn is_newer_version(old: &str, new: &str) -> bool {
    let parse = |v: &str| -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some((
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ))
    };
    match (parse(old), parse(new)) {
        (Some(o), Some(n)) => n > o,
        _ => false,
    }
}

/// Parse, validate, and import a remote catalog JSON into the local database.
///
/// Checks the version against the currently-stored catalog and rejects
/// downgrades. Returns a [`CatalogSyncResult`] describing what happened.
pub fn sync_catalog_from_json(
    conn: &mut Connection,
    json_text: &str,
) -> Result<CatalogSyncResult, CatalogSyncError> {
    let catalog =
        parse_catalog_json(json_text).map_err(|e| CatalogSyncError::Parse(e.to_string()))?;

    let current_meta =
        load_catalog_metadata(conn).map_err(|e| CatalogSyncError::Database(e.to_string()))?;

    let old_version = current_meta.map(|m| m.catalog_version);
    let new_version = catalog.catalog_version.clone();

    // Reject downgrade or same version
    if let Some(ref old) = old_version {
        if !is_newer_version(old, &new_version) {
            return Ok(CatalogSyncResult {
                updated: false,
                old_version: Some(old.clone()),
                new_version: new_version.clone(),
                message: format!("Catalog is up to date (v{})", new_version),
            });
        }
    }

    // Import the new catalog
    import_catalog(conn, &catalog).map_err(|e| CatalogSyncError::Database(e.to_string()))?;

    Ok(CatalogSyncResult {
        updated: true,
        old_version,
        new_version: new_version.clone(),
        message: format!("Catalog updated to v{}", new_version),
    })
}

#[derive(Debug, thiserror::Error)]
pub enum CatalogSyncError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Database error: {0}")]
    Database(String),
}
