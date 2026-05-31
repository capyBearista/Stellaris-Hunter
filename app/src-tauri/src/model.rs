use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
