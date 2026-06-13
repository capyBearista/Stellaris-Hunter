use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AchievementCatalog {
    pub catalog_version: String,
    pub snapshot_kind: String,
    pub stellaris_version: Option<String>,
    pub source_url: Option<String>,
    pub source_hash: Option<String>,
    pub updated_at: String,
    pub achievements: Vec<AchievementCatalogEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CatalogVersionMetadata {
    pub catalog_version: String,
    pub stellaris_version: Option<String>,
    pub source_url: Option<String>,
    pub source_hash: Option<String>,
    pub updated_at: String,
    pub imported_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CatalogEntriesLoad {
    pub entries: Vec<AchievementCatalogEntry>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AchievementOverride {
    pub achievement_id: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunAchievementUserStatus {
    pub run_folder_path: String,
    pub achievement_id: String,
    pub user_status: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlannerAchievementEvaluation {
    pub achievement: AchievementCatalogEntry,
    pub status: String,
    pub computed_status: String,
    pub planned: bool,
    pub ignored: bool,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
    pub conditions: Vec<ConditionEvaluation>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConditionEvaluation {
    pub dimension: String,
    pub operator: String,
    pub condition_value: Value,
    pub fact_value: Option<Value>,
    pub passed: Option<bool>,
    pub severity: String,
    pub timing: String,
    pub mutability: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AchievementCatalogEntry {
    pub id: String,
    #[serde(default = "default_steam_app_id")]
    pub steam_app_id: u32,
    pub steam_api_name: Option<String>,
    pub local_key: Option<String>,
    #[serde(default)]
    pub deprecated: bool,
    pub source: AchievementSourceFields,
    #[serde(default)]
    pub curation: AchievementCurationFields,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AchievementSourceFields {
    pub name: String,
    pub description: Option<String>,
    pub requirement: Option<String>,
    pub hint: Option<String>,
    pub group: Option<String>,
    pub version_added: Option<String>,
    pub difficulty: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AchievementCurationFields {
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub conditions: Vec<AchievementCondition>,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub planner_notes: Option<String>,
    #[serde(default)]
    pub known_limitations: Vec<String>,
    pub rule_confidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AchievementCondition {
    pub condition_type: String,
    pub dimension: String,
    pub operator: String,
    pub value: Value,
    pub timing: String,
    pub mutability: String,
    pub severity: String,
    pub source: Option<String>,
    pub notes: Option<String>,
}

fn default_steam_app_id() -> u32 {
    281_990
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChecksumScope {
    pub raw: String,
    pub directory: Option<String>,
    pub patterns: Vec<String>,
    pub recursive: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstallSummary {
    pub root: PathBuf,
    pub steam_appid: Option<u32>,
    pub version: Option<String>,
    pub raw_version: Option<String>,
    pub checksum_manifest: Vec<ChecksumScope>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContinueGameTarget {
    pub title: Option<String>,
    pub desc: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DlcLoadSummary {
    pub enabled_mods: Vec<String>,
    pub disabled_dlcs: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LauncherPlaysetSummary {
    pub uuid: Option<String>,
    pub name: Option<String>,
    pub sync_state: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LauncherModSummary {
    pub position: Option<i64>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub registry_id: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LauncherStateSummary {
    pub active_playset: Option<LauncherPlaysetSummary>,
    pub enabled_mods: Vec<LauncherModSummary>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaveSummary {
    pub path: PathBuf,
    pub version: Option<String>,
    pub date: Option<String>,
    pub name: Option<String>,
    pub required_dlcs: Vec<String>,
    pub ironman: Option<bool>,
    pub cheated_on_save: Option<bool>,
    pub player_country: Option<String>,
    pub authority: Option<String>,
    pub government_type: Option<String>,
    pub origin: Option<String>,
    pub ethics: Vec<String>,
    pub civics: Vec<String>,
    pub founder_species_ref: Option<String>,
    pub founder_species_class: Option<String>,
    pub founder_species_portrait: Option<String>,
    pub founder_species_traits: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaveRunSummary {
    pub run_folder: String,
    pub save_count: usize,
    pub latest_save: Option<SaveSummary>,
    /// Conservative eligibility computed from the parsed latest save plus the
    /// current launcher active-playset snapshot. This is not historical per-run
    /// mod state.
    pub eligibility: Option<SaveEligibility>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistedRunSummary {
    pub folder_path: String,
    pub run_folder: String,
    pub display_name: Option<String>,
    pub latest_save_path: Option<String>,
    pub latest_save_file_name: Option<String>,
    pub latest_ingame_date: Option<String>,
    pub game_version: Option<String>,
    pub parse_status: Option<String>,
    pub parse_error: Option<String>,
    pub fact_count: usize,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunFactSummary {
    pub run_folder_path: String,
    pub dimension: String,
    pub key: String,
    pub value: Value,
    pub source: String,
    pub confidence: String,
    pub updated_from_save_path: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaveEligibility {
    pub conclusion: EligibilityConclusion,
    pub cheated_on_save: Option<bool>,
    pub ironman: Option<bool>,
    pub mod_risk: ModChecksumRisk,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EligibilityConclusion {
    LikelyEligible,
    LikelyIneligible,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModChecksumRisk {
    None,
    #[default]
    Unknown,
    /// Reserved for a later mod-file overlap pass. The current MVP classifier
    /// does not emit this until enabled mod contents are mapped to checksum
    /// manifest scopes.
    ChecksumScoped,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentsSummary {
    pub root: PathBuf,
    pub continue_game: Option<ContinueGameTarget>,
    pub dlc_load: Option<DlcLoadSummary>,
    pub save_runs: Vec<SaveRunSummary>,
    pub launcher: Option<LauncherStateSummary>,
    pub issues: Vec<String>,
}
