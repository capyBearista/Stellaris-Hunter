//! Read-only Steam achievement sync (Mode C: post-game).
//!
//! This module initializes the Steam client under Stellaris App ID 281990,
//! reads achievement unlock state, and writes it to the local database.
//!
//! IMPORTANT: This module must NEVER call mutating Steam APIs.
//! The quarantine test enforces this.

use steamworks::{AppId, Client};

use crate::model::SteamSyncResult;

const STELLARIS_APP_ID: u32 = 281990;

/// State of a single Steam achievement after sync.
#[derive(Debug, Clone)]
pub struct SteamAchievementState {
    pub api_name: String,
    pub unlocked: Option<bool>,
}

#[derive(Debug, thiserror::Error)]
pub enum SteamSyncError {
    #[error("Steam init failed: {0}")]
    Init(String),
    #[error("Steam read failed: {0}")]
    Read(String),
    #[error("Steam sync timeout: {0}")]
    Timeout(String),
    #[error("Database error: {0}")]
    Database(String),
}

/// Initialize Steam client, request stats, and read all achievement states.
pub fn read_steam_achievements() -> Result<Vec<SteamAchievementState>, SteamSyncError> {
    let client = Client::init_app(AppId(STELLARIS_APP_ID))
        .map_err(|e| SteamSyncError::Init(format!("{e:?}")))?;

    // Request stats and pump callbacks until received
    let stats_received = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stats_flag = stats_received.clone();

    let _callback = client.register_callback(move |p: steamworks::UserStatsReceived| {
        if p.result.is_ok() {
            stats_flag.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    });

    let steam_id = client.user().steam_id();
    client.user_stats().request_user_stats(steam_id.raw());

    // Pump callbacks with timeout (10 seconds)
    let start = std::time::Instant::now();
    while !stats_received.load(std::sync::atomic::Ordering::SeqCst) {
        client.run_callbacks();
        std::thread::sleep(std::time::Duration::from_millis(100));
        if start.elapsed() > std::time::Duration::from_secs(10) {
            return Err(SteamSyncError::Timeout("stats request timed out".into()));
        }
    }

    // Read achievement names and unlock states
    let names = client
        .user_stats()
        .get_achievement_names()
        .ok_or_else(|| SteamSyncError::Read("failed to get achievement names".into()))?;

    let mut achievements = Vec::with_capacity(names.len());
    for name in &names {
        let unlocked = client.user_stats().achievement(name).get().ok();
        achievements.push(SteamAchievementState {
            api_name: name.to_string(),
            unlocked,
        });
    }

    Ok(achievements)
}

/// Write Steam achievement states to the database.
/// Maps Steam API names to catalog entries via the `steam_api_name` column.
/// Updates `player_achievements.steam_unlocked`, `steam_last_synced_at`.
/// Also updates `displayed_unlocked` when no manual override is active.
pub fn sync_to_db(
    conn: &rusqlite::Connection,
    achievements: &[SteamAchievementState],
) -> Result<SteamSyncResult, SteamSyncError> {
    let mut synced = 0u32;
    let mut skipped = 0u32;
    let mut unmatched = 0u32;

    for ach in achievements {
        // Skip achievements whose state could not be read from Steam
        let Some(unlocked) = ach.unlocked else {
            skipped += 1;
            continue;
        };

        // Find catalog entry by steam_api_name
        let achievement_id: Option<String> = conn
            .query_row(
                "SELECT id FROM achievements WHERE steam_api_name = ?1",
                rusqlite::params![ach.api_name],
                |row| row.get(0),
            )
            .ok();

        let Some(achievement_id) = achievement_id else {
            unmatched += 1;
            continue;
        };

        let unlocked_i64 = i64::from(unlocked);

        // Upsert player_achievements:
        // - Set steam_unlocked and steam_last_synced_at
        // - Update displayed_unlocked ONLY if no manual_override is active
        //   (COALESCE preserves manual override)
        conn.execute(
            r#"
            INSERT INTO player_achievements (
                achievement_id, steam_unlocked, steam_last_synced_at, displayed_unlocked
            ) VALUES (?1, ?2, datetime('now'), ?2)
            ON CONFLICT(achievement_id) DO UPDATE SET
                steam_unlocked = excluded.steam_unlocked,
                steam_last_synced_at = excluded.steam_last_synced_at,
                displayed_unlocked = CASE
                    WHEN player_achievements.manual_override IS NOT NULL
                    THEN player_achievements.displayed_unlocked
                    ELSE excluded.displayed_unlocked
                END
            "#,
            rusqlite::params![achievement_id, unlocked_i64],
        )
        .map_err(|e| SteamSyncError::Database(e.to_string()))?;

        synced += 1;
    }

    Ok(SteamSyncResult {
        synced,
        skipped,
        unmatched,
        total_steam_achievements: achievements.len() as u32,
        message: format!(
            "Synced {} achievements ({} skipped, {} unmatched in catalog)",
            synced, skipped, unmatched
        ),
    })
}
