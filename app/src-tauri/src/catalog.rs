use std::collections::HashSet;

use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    error::{Error, Result},
    model::{
        AchievementCatalog, AchievementCatalogEntry, AchievementCondition,
        AchievementCurationFields, AchievementOverride, AchievementSourceFields,
        CatalogEntriesLoad, CatalogVersionMetadata,
    },
};

const SUPPORTED_DIFFICULTIES: &[&str] = &["VE", "E", "M", "H", "VH", "I", "UC"];

pub fn parse_catalog_json(text: &str) -> Result<AchievementCatalog> {
    let catalog: AchievementCatalog = normalize_catalog(serde_json::from_str(text)?)?;
    validate_catalog(&catalog)?;
    Ok(catalog)
}

pub fn initialize_catalog_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS catalog_versions (
          id TEXT PRIMARY KEY,
          catalog_version TEXT NOT NULL,
          stellaris_version TEXT,
          source_url TEXT,
          source_hash TEXT,
          updated_at TEXT NOT NULL,
          imported_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS achievements (
          id TEXT PRIMARY KEY,
          steam_app_id INTEGER NOT NULL DEFAULT 281990,
          steam_api_name TEXT,
          local_key TEXT,
          name TEXT NOT NULL,
          steam_description TEXT,
          wiki_requirement TEXT,
          wiki_hint TEXT,
          catalog_group TEXT,
          version_added TEXT,
          difficulty TEXT CHECK (difficulty IN ('VE', 'E', 'M', 'H', 'VH', 'I', 'UC') OR difficulty IS NULL),
          deprecated INTEGER NOT NULL DEFAULT 0,
          source_json TEXT NOT NULL,
          curation_json TEXT NOT NULL,
          created_at TEXT NOT NULL DEFAULT (datetime('now')),
          updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS achievement_tags (
          achievement_id TEXT NOT NULL,
          tag TEXT NOT NULL,
          PRIMARY KEY (achievement_id, tag),
          FOREIGN KEY (achievement_id) REFERENCES achievements(id)
        );

        CREATE TABLE IF NOT EXISTS achievement_conditions (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          achievement_id TEXT NOT NULL,
          condition_type TEXT NOT NULL,
          dimension TEXT NOT NULL,
          operator TEXT NOT NULL,
          value_json TEXT NOT NULL,
          timing TEXT NOT NULL,
          mutability TEXT NOT NULL,
          severity TEXT NOT NULL,
          source TEXT,
          notes TEXT,
          FOREIGN KEY (achievement_id) REFERENCES achievements(id)
        );

        CREATE TABLE IF NOT EXISTS player_achievements (
          achievement_id TEXT PRIMARY KEY,
          steam_unlocked INTEGER,
          steam_unlocked_at TEXT,
          steam_last_synced_at TEXT,
          manual_override TEXT CHECK (
            manual_override IN ('force_completed', 'force_incomplete') OR manual_override IS NULL
          ),
          manual_override_updated_at TEXT,
          displayed_unlocked INTEGER NOT NULL DEFAULT 0,
          FOREIGN KEY (achievement_id) REFERENCES achievements(id)
        );
        "#,
    )?;
    migrate_catalog_versions_schema(conn)?;
    Ok(())
}

fn migrate_catalog_versions_schema(conn: &Connection) -> Result<()> {
    if !table_has_column(conn, "catalog_versions", "updated_at")? {
        conn.execute(
            "ALTER TABLE catalog_versions ADD COLUMN updated_at TEXT NOT NULL DEFAULT ''",
            [],
        )?;
    }
    conn.execute(
        r#"
        UPDATE catalog_versions
        SET updated_at = imported_at
        WHERE trim(updated_at) = ''
        "#,
        [],
    )?;
    Ok(())
}

fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for existing_column in columns {
        if existing_column? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn import_catalog(conn: &mut Connection, catalog: &AchievementCatalog) -> Result<()> {
    let catalog = normalize_catalog(catalog.clone())?;
    validate_catalog(&catalog)?;
    initialize_catalog_schema(conn)?;

    let tx = conn.transaction()?;
    tx.execute(
        r#"
        INSERT INTO catalog_versions (
          id, catalog_version, stellaris_version, source_url, source_hash, updated_at, imported_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))
        ON CONFLICT(id) DO UPDATE SET
          catalog_version = excluded.catalog_version,
          stellaris_version = excluded.stellaris_version,
          source_url = excluded.source_url,
          source_hash = excluded.source_hash,
          updated_at = excluded.updated_at,
          imported_at = excluded.imported_at
        "#,
        params![
            catalog.catalog_version,
            catalog.catalog_version,
            catalog.stellaris_version,
            catalog.source_url,
            catalog.source_hash,
            catalog.updated_at,
        ],
    )?;

    let active_ids: HashSet<_> = catalog
        .achievements
        .iter()
        .map(|entry| entry.id.as_str())
        .collect();

    for entry in &catalog.achievements {
        upsert_achievement(&tx, entry)?;
    }
    deprecate_missing_achievements(&tx, &active_ids)?;

    tx.commit()?;
    Ok(())
}

pub fn load_catalog_metadata(conn: &Connection) -> Result<Option<CatalogVersionMetadata>> {
    Ok(conn
        .query_row(
            r#"
            SELECT catalog_version, stellaris_version, source_url, source_hash, updated_at, imported_at
            FROM catalog_versions
            ORDER BY imported_at DESC, catalog_version DESC
            LIMIT 1
            "#,
            [],
            |row| {
                Ok(CatalogVersionMetadata {
                    catalog_version: row.get(0)?,
                    stellaris_version: row.get(1)?,
                    source_url: row.get(2)?,
                    source_hash: row.get(3)?,
                    updated_at: row.get(4)?,
                    imported_at: row.get(5)?,
                })
            },
        )
        .optional()?)
}

pub fn load_catalog_entries(conn: &Connection) -> Result<Vec<AchievementCatalogEntry>> {
    Ok(load_catalog_entries_with_issues(conn)?.entries)
}

pub fn load_catalog_entries_with_issues(conn: &Connection) -> Result<CatalogEntriesLoad> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, steam_app_id, steam_api_name, local_key, deprecated, source_json, curation_json
        FROM achievements
        ORDER BY name COLLATE NOCASE ASC
        "#,
    )?;

    let mut rows = stmt.query([])?;
    let mut entries = Vec::new();
    let mut issues = Vec::new();
    let mut ids = HashSet::new();

    while let Some(row) = rows.next()? {
        let id = match row.get::<_, String>(0) {
            Ok(id) => id,
            Err(error) => {
                issues.push(format!(
                    "catalog row with unknown achievement id has invalid id column: {error}"
                ));
                continue;
            }
        };

        let steam_app_id = match row.get::<_, u32>(1) {
            Ok(value) => value,
            Err(error) => {
                issues.push(format!(
                    "catalog row {id} has invalid steam_app_id column: {error}"
                ));
                continue;
            }
        };
        let steam_api_name = match row.get::<_, Option<String>>(2) {
            Ok(value) => value,
            Err(error) => {
                issues.push(format!(
                    "catalog row {id} has invalid steam_api_name column: {error}"
                ));
                continue;
            }
        };
        let local_key = match row.get::<_, Option<String>>(3) {
            Ok(value) => value,
            Err(error) => {
                issues.push(format!(
                    "catalog row {id} has invalid local_key column: {error}"
                ));
                continue;
            }
        };
        let deprecated = match row.get::<_, bool>(4) {
            Ok(value) => value,
            Err(error) => {
                issues.push(format!(
                    "catalog row {id} has invalid deprecated column: {error}"
                ));
                continue;
            }
        };
        let source_json = match row.get::<_, String>(5) {
            Ok(value) => value,
            Err(error) => {
                issues.push(format!(
                    "catalog row {id} has invalid source_json column: {error}"
                ));
                continue;
            }
        };
        let curation_json = match row.get::<_, String>(6) {
            Ok(value) => value,
            Err(error) => {
                issues.push(format!(
                    "catalog row {id} has invalid curation_json column: {error}"
                ));
                continue;
            }
        };

        let source: AchievementSourceFields = match serde_json::from_str(&source_json) {
            Ok(value) => value,
            Err(error) => {
                issues.push(format!("catalog row {id} has invalid source_json: {error}"));
                continue;
            }
        };
        let curation: AchievementCurationFields = match serde_json::from_str(&curation_json) {
            Ok(value) => value,
            Err(error) => {
                issues.push(format!(
                    "catalog row {id} has invalid curation_json: {error}"
                ));
                continue;
            }
        };

        let mut entry = AchievementCatalogEntry {
            id,
            steam_app_id,
            steam_api_name,
            local_key,
            deprecated,
            source,
            curation,
        };

        if let Err(error) =
            normalize_entry(&mut entry).and_then(|_| validate_entry(&entry, &mut ids))
        {
            issues.push(format!(
                "catalog row {} has invalid catalog data: {error}",
                entry.id
            ));
            continue;
        }

        entries.push(entry);
    }

    Ok(CatalogEntriesLoad { entries, issues })
}

fn upsert_achievement(conn: &Connection, entry: &AchievementCatalogEntry) -> Result<()> {
    let source_json = serde_json::to_string(&entry.source)?;
    let curation_json = serde_json::to_string(&entry.curation)?;

    conn.execute(
        r#"
        INSERT INTO achievements (
          id, steam_app_id, steam_api_name, local_key, name, steam_description,
          wiki_requirement, wiki_hint, catalog_group, version_added, difficulty,
          deprecated, source_json, curation_json, created_at, updated_at
        ) VALUES (
          ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
          datetime('now'), datetime('now')
        )
        ON CONFLICT(id) DO UPDATE SET
          steam_app_id = excluded.steam_app_id,
          steam_api_name = excluded.steam_api_name,
          local_key = excluded.local_key,
          name = excluded.name,
          steam_description = excluded.steam_description,
          wiki_requirement = excluded.wiki_requirement,
          wiki_hint = excluded.wiki_hint,
          catalog_group = excluded.catalog_group,
          version_added = excluded.version_added,
          difficulty = excluded.difficulty,
          deprecated = excluded.deprecated,
          source_json = excluded.source_json,
          curation_json = excluded.curation_json,
          updated_at = excluded.updated_at
        "#,
        params![
            entry.id,
            entry.steam_app_id,
            entry.steam_api_name,
            entry.local_key,
            entry.source.name,
            entry.source.description,
            entry.source.requirement,
            entry.source.hint,
            entry.source.group,
            entry.source.version_added,
            entry.source.difficulty,
            entry.deprecated,
            source_json,
            curation_json,
        ],
    )?;

    conn.execute(
        "DELETE FROM achievement_tags WHERE achievement_id = ?1",
        params![entry.id],
    )?;
    for tag in &entry.curation.tags {
        conn.execute(
            "INSERT INTO achievement_tags (achievement_id, tag) VALUES (?1, ?2)",
            params![entry.id, tag],
        )?;
    }

    conn.execute(
        "DELETE FROM achievement_conditions WHERE achievement_id = ?1",
        params![entry.id],
    )?;
    for condition in &entry.curation.conditions {
        conn.execute(
            r#"
            INSERT INTO achievement_conditions (
              achievement_id, condition_type, dimension, operator, value_json,
              timing, mutability, severity, source, notes
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                entry.id,
                condition.condition_type,
                condition.dimension,
                condition.operator,
                serde_json::to_string(&condition.value)?,
                condition.timing,
                condition.mutability,
                condition.severity,
                condition.source,
                condition.notes,
            ],
        )?;
    }

    Ok(())
}

fn deprecate_missing_achievements(conn: &Connection, active_ids: &HashSet<&str>) -> Result<()> {
    let mut stmt = conn.prepare("SELECT id FROM achievements")?;
    let existing_ids = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    for existing_id in existing_ids {
        if !active_ids.contains(existing_id.as_str()) {
            conn.execute(
                "UPDATE achievements SET deprecated = 1, updated_at = datetime('now') WHERE id = ?1",
                params![existing_id],
            )?;
        }
    }

    Ok(())
}

pub fn load_completion_overrides(conn: &Connection) -> Result<Vec<AchievementOverride>> {
    initialize_catalog_schema(conn)?;

    let mut stmt = conn.prepare(
        r#"
        SELECT achievement_id, manual_override
        FROM player_achievements
        WHERE manual_override IS NOT NULL
        ORDER BY achievement_id COLLATE NOCASE ASC
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        let manual_override: String = row.get(1)?;
        Ok(AchievementOverride {
            achievement_id: row.get(0)?,
            completed: manual_override == "force_completed",
        })
    })?;

    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn set_completion_override(
    conn: &Connection,
    achievement_id: &str,
    completed: bool,
) -> Result<()> {
    initialize_catalog_schema(conn)?;
    ensure_achievement_exists(conn, achievement_id)?;

    let manual_override = if completed {
        "force_completed"
    } else {
        "force_incomplete"
    };
    let displayed_unlocked = i64::from(completed);

    conn.execute(
        r#"
        INSERT INTO player_achievements (
          achievement_id, manual_override, manual_override_updated_at, displayed_unlocked
        ) VALUES (?1, ?2, datetime('now'), ?3)
        ON CONFLICT(achievement_id) DO UPDATE SET
          manual_override = excluded.manual_override,
          manual_override_updated_at = excluded.manual_override_updated_at,
          displayed_unlocked = excluded.displayed_unlocked
        "#,
        params![achievement_id, manual_override, displayed_unlocked],
    )?;

    Ok(())
}

pub fn clear_completion_override(conn: &Connection, achievement_id: &str) -> Result<()> {
    initialize_catalog_schema(conn)?;
    ensure_achievement_exists(conn, achievement_id)?;

    conn.execute(
        r#"
        UPDATE player_achievements
        SET manual_override = NULL,
            manual_override_updated_at = NULL,
            displayed_unlocked = COALESCE(steam_unlocked, 0)
        WHERE achievement_id = ?1
        "#,
        params![achievement_id],
    )?;

    Ok(())
}

fn ensure_achievement_exists(conn: &Connection, achievement_id: &str) -> Result<()> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM achievements WHERE id = ?1)",
        params![achievement_id],
        |row| row.get(0),
    )?;

    if !exists {
        return Err(Error::Validation(format!(
            "unknown achievement id for completion override: {achievement_id}"
        )));
    }

    Ok(())
}

fn normalize_catalog(mut catalog: AchievementCatalog) -> Result<AchievementCatalog> {
    catalog.snapshot_kind = normalize_required_text("snapshot_kind", &catalog.snapshot_kind)?;
    for entry in &mut catalog.achievements {
        normalize_entry(entry)?;
    }

    Ok(catalog)
}

fn normalize_entry(entry: &mut AchievementCatalogEntry) -> Result<()> {
    entry.id = normalize_required_text("achievement.id", &entry.id)?;
    entry.steam_api_name = normalize_optional_identifier(entry.steam_api_name.take());
    entry.local_key = normalize_optional_identifier(entry.local_key.take());

    let mut seen_tags = HashSet::new();
    let mut normalized_tags = Vec::new();
    for tag in &entry.curation.tags {
        let normalized = normalize_required_slug("achievement.curation.tags[]", tag)?;
        if seen_tags.insert(normalized.clone()) {
            normalized_tags.push(normalized);
        }
    }
    entry.curation.tags = normalized_tags;

    for condition in &mut entry.curation.conditions {
        normalize_condition(condition)?;
    }
    Ok(())
}

fn normalize_condition(condition: &mut AchievementCondition) -> Result<()> {
    condition.condition_type =
        normalize_required_snake("condition.condition_type", &condition.condition_type)?;
    condition.dimension = normalize_required_snake("condition.dimension", &condition.dimension)?;
    condition.operator = normalize_required_snake("condition.operator", &condition.operator)?;
    condition.timing = normalize_required_snake("condition.timing", &condition.timing)?;
    condition.mutability = normalize_required_snake("condition.mutability", &condition.mutability)?;
    condition.severity = normalize_required_snake("condition.severity", &condition.severity)?;
    condition.source = normalize_optional_identifier(condition.source.take());
    condition.notes = normalize_optional_text(condition.notes.take());
    Ok(())
}

fn normalize_required_text(field: &str, value: &str) -> Result<String> {
    let trimmed = value.trim();
    require_non_empty(field, trimmed)?;
    Ok(trimmed.to_string())
}

fn normalize_required_slug(field: &str, value: &str) -> Result<String> {
    let normalized = value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|ch| match ch {
            'a'..='z' | '0'..='9' => ch,
            '-' | '_' | ' ' | '/' => '-',
            _ => ch,
        })
        .collect::<String>();
    let collapsed = collapse_hyphens(&normalized);
    require_non_empty(field, &collapsed)?;
    if !collapsed
        .chars()
        .all(|ch| matches!(ch, 'a'..='z' | '0'..='9' | '-'))
    {
        return Err(Error::Parse(format!(
            "{field} must contain only lowercase letters, digits, and hyphens after normalization"
        )));
    }
    Ok(collapsed)
}

fn normalize_required_snake(field: &str, value: &str) -> Result<String> {
    Ok(normalize_required_slug(field, value)?.replace('-', "_"))
}

fn normalize_optional_identifier(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn collapse_hyphens(value: &str) -> String {
    let mut collapsed = String::new();
    let mut previous_was_hyphen = false;
    for ch in value.chars() {
        if ch == '-' {
            if !previous_was_hyphen {
                collapsed.push(ch);
            }
            previous_was_hyphen = true;
        } else {
            collapsed.push(ch);
            previous_was_hyphen = false;
        }
    }
    collapsed.trim_matches('-').to_string()
}

fn validate_catalog(catalog: &AchievementCatalog) -> Result<()> {
    require_non_empty("catalog_version", &catalog.catalog_version)?;
    if !is_semver_catalog_version(&catalog.catalog_version) {
        return Err(Error::Parse(format!(
            "catalog_version must use MAJOR.MINOR.PATCH digits: {}",
            catalog.catalog_version
        )));
    }
    if catalog.snapshot_kind != "full" {
        return Err(Error::Parse(format!(
            "unsupported snapshot_kind {}; only full catalog snapshots can be imported",
            catalog.snapshot_kind
        )));
    }
    require_non_empty("updated_at", &catalog.updated_at)?;
    if catalog.achievements.is_empty() {
        return Err(Error::Parse(
            "catalog must contain at least one achievement".to_string(),
        ));
    }

    let mut ids = HashSet::new();
    for entry in &catalog.achievements {
        validate_entry(entry, &mut ids)?;
    }

    Ok(())
}

fn is_semver_catalog_version(value: &str) -> bool {
    let mut parts = value.split('.');
    let Some(major) = parts.next() else {
        return false;
    };
    let Some(minor) = parts.next() else {
        return false;
    };
    let Some(patch) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }
    [major, minor, patch]
        .iter()
        .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
}

fn validate_entry(entry: &AchievementCatalogEntry, ids: &mut HashSet<String>) -> Result<()> {
    require_non_empty("achievement.id", &entry.id)?;
    require_non_empty("achievement.source.name", &entry.source.name)?;
    if entry.steam_app_id != 281_990 {
        return Err(Error::Parse(format!(
            "achievement {} has unsupported steam_app_id {}",
            entry.id, entry.steam_app_id
        )));
    }
    if !ids.insert(entry.id.clone()) {
        return Err(Error::Parse(format!(
            "duplicate achievement id in catalog: {}",
            entry.id
        )));
    }
    if let Some(difficulty) = entry.source.difficulty.as_deref() {
        if !SUPPORTED_DIFFICULTIES.contains(&difficulty) {
            return Err(Error::Parse(format!(
                "achievement {} has unsupported difficulty {}",
                entry.id, difficulty
            )));
        }
    }
    for tag in &entry.curation.tags {
        require_non_empty("achievement.curation.tags[]", tag)?;
    }
    Ok(())
}

fn require_non_empty(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(Error::Parse(format!("{field} must not be empty")));
    }
    Ok(())
}
