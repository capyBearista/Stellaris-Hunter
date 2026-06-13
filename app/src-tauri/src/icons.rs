//! Achievement icon cache with multi-tier fallback.
//!
//! Icons are cached as PNG files in `app_data/icons/`.
//! Tier 1: Steam API (Windows-only, requires `steam` feature)
//! Tier 2: Steam CDN (deferred — requires icon hash mapping)
//! Tier 3: Local DDS files (deferred — requires DDS naming convention research)
//! Tier 4: Placeholder (always available)

use std::path::{Path, PathBuf};

/// Manages the icon cache directory.
pub struct IconCache {
    cache_dir: PathBuf,
}

/// Sanitize an achievement ID for safe use as a filename component.
/// Returns `None` if the ID is empty or contains path traversal characters.
fn sanitize_id(id: &str) -> Option<String> {
    if id.is_empty() {
        return None;
    }
    if id.contains('/') || id.contains('\\') || id.contains("..") || id.contains('\0') {
        return None;
    }
    // Only allow alphanumeric, underscore, hyphen, and period
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return None;
    }
    Some(id.to_string())
}

impl IconCache {
    /// Create a new IconCache rooted at the given app data directory.
    /// The `icons/` subdirectory will be used for cached PNG files.
    pub fn new(app_data_dir: &Path) -> Self {
        let cache_dir = app_data_dir.join("icons");
        Self { cache_dir }
    }

    /// Ensure the cache directory exists.
    pub fn ensure_dir(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.cache_dir)
    }

    /// Get the cached icon path for an achievement, if it exists.
    /// Returns `None` if the achievement ID contains invalid characters.
    pub fn get(&self, achievement_id: &str) -> Option<PathBuf> {
        let safe_id = sanitize_id(achievement_id)?;
        let path = self.cache_dir.join(format!("{safe_id}.png"));
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Store icon bytes as a PNG file in the cache.
    /// Returns an error if the achievement ID contains invalid characters.
    pub fn store(&self, achievement_id: &str, png_bytes: &[u8]) -> std::io::Result<PathBuf> {
        let safe_id = sanitize_id(achievement_id).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("invalid achievement ID: {achievement_id}"),
            )
        })?;
        self.ensure_dir()?;
        let path = self.cache_dir.join(format!("{safe_id}.png"));
        std::fs::write(&path, png_bytes)?;
        Ok(path)
    }

    /// Read a cached icon as PNG bytes.
    /// Returns `None` if the achievement ID contains invalid characters or the icon is not cached.
    pub fn read(&self, achievement_id: &str) -> std::io::Result<Option<Vec<u8>>> {
        let safe_id = match sanitize_id(achievement_id) {
            Some(id) => id,
            None => return Ok(None),
        };
        match self.get(&safe_id) {
            Some(path) => Ok(Some(std::fs::read(path)?)),
            None => Ok(None),
        }
    }

    /// List all cached achievement IDs.
    pub fn cached_ids(&self) -> Vec<String> {
        let Ok(entries) = std::fs::read_dir(&self.cache_dir) else {
            return Vec::new();
        };
        entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.strip_suffix(".png").map(|s| s.to_string())
            })
            .collect()
    }

    /// Count cached icons.
    pub fn count(&self) -> usize {
        self.cached_ids().len()
    }

    /// Clear the entire icon cache.
    pub fn clear(&self) -> std::io::Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }
}

/// Result of an icon sync operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconSyncResult {
    pub synced: u32,
    pub failed: u32,
    pub total: u32,
    pub message: String,
}

/// Sync icons for all catalog entries using available tiers.
/// On Windows with `steam` feature: uses Steam API (Tier 1).
/// Otherwise: returns an error (icons will be placeholders until synced on Windows).
#[cfg(all(target_os = "windows", feature = "steam"))]
pub fn sync_icons_from_steam(
    cache: &IconCache,
    steam_api_names: &[(String, String)],
) -> Result<IconSyncResult, crate::steam::sync::SteamSyncError> {
    use steamworks::{AppId, Client};

    let (client, single) = Client::init_app(AppId(281990))
        .map_err(|e| crate::steam::sync::SteamSyncError::Init(format!("{e:?}")))?;

    // Request stats to enable icon access
    let stats_received = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stats_flag = stats_received.clone();
    let _cb = client.register_callback(move |p: steamworks::UserStatsReceived| {
        if p.result.is_ok() {
            stats_flag.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    });
    client.user_stats().request_current_stats();

    let start = std::time::Instant::now();
    while !stats_received.load(std::sync::atomic::Ordering::SeqCst) {
        single.run_callbacks();
        std::thread::sleep(std::time::Duration::from_millis(100));
        if start.elapsed() > std::time::Duration::from_secs(10) {
            break; // Continue anyway, icons may still work
        }
    }

    let mut synced = 0u32;
    let mut failed = 0u32;

    for (achievement_id, api_name) in steam_api_names {
        // Skip if already cached
        if cache.get(achievement_id).is_some() {
            synced += 1;
            continue;
        }

        // Try Steam API icon — get raw RGBA data and convert to PNG
        if let Some(img) = client.user_stats().achievement(api_name).get_icon() {
            let width = img.width();
            let height = img.height();
            if width > 0 && height > 0 {
                let rgba = img.rgba();
                if let Some(rgba_img) = image::RgbaImage::from_raw(width, height, rgba.to_vec()) {
                    let mut png_buf = Vec::new();
                    let mut cursor = std::io::Cursor::new(&mut png_buf);
                    if rgba_img
                        .write_to(&mut cursor, image::ImageFormat::Png)
                        .is_ok()
                        && cache.store(achievement_id, &png_buf).is_ok()
                    {
                        synced += 1;
                        continue;
                    }
                }
            }
        }
        failed += 1;
    }

    Ok(IconSyncResult {
        synced,
        failed,
        total: steam_api_names.len() as u32,
        message: format!("Synced {} icons ({} failed)", synced, failed),
    })
}

/// Non-Windows stub: icon sync requires Windows + Steam.
#[cfg(not(all(target_os = "windows", feature = "steam")))]
pub fn sync_icons_from_steam(
    _cache: &IconCache,
    _steam_api_names: &[(String, String)],
) -> Result<IconSyncResult, String> {
    Err("Icon sync from Steam requires Windows with Steam client running".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_valid_id() {
        assert_eq!(sanitize_id("achievement_1"), Some("achievement_1".into()));
        assert_eq!(sanitize_id("test-123"), Some("test-123".into()));
        assert_eq!(sanitize_id("ach.3.0"), Some("ach.3.0".into()));
        assert_eq!(sanitize_id("A"), Some("A".into()));
    }

    #[test]
    fn sanitize_rejects_empty() {
        assert_eq!(sanitize_id(""), None);
    }

    #[test]
    fn sanitize_rejects_path_traversal() {
        assert_eq!(sanitize_id("../etc/passwd"), None);
        assert_eq!(sanitize_id("foo/bar"), None);
        assert_eq!(sanitize_id("foo\\bar"), None);
        assert_eq!(sanitize_id(".."), None);
    }

    #[test]
    fn sanitize_rejects_null_byte() {
        assert_eq!(sanitize_id("foo\0bar"), None);
    }

    #[test]
    fn sanitize_rejects_spaces() {
        assert_eq!(sanitize_id("id with spaces"), None);
    }
}
