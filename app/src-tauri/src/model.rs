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
pub struct RunNote {
    pub run_folder_path: String,
    pub note_text: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunAchievementNote {
    pub run_folder_path: String,
    pub achievement_id: String,
    pub notes: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunAchievementUserStatus {
    pub run_folder_path: String,
    pub achievement_id: String,
    pub user_status: String,
    pub notes: Option<String>,
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
    #[serde(default)]
    pub children: Vec<ConditionEvaluation>,
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
    /// Whether this achievement is completed (from Steam sync or manual override).
    /// Populated at query time via LEFT JOIN with player_achievements.
    #[serde(default)]
    pub completed: bool,
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
    #[serde(default)]
    pub dimension: String,
    #[serde(default)]
    pub operator: String,
    #[serde(default)]
    pub value: Value,
    pub timing: String,
    pub mutability: String,
    pub severity: String,
    pub source: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub children: Vec<AchievementCondition>,
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
    /// Classified DLC state with enabled vs disabled DLCs identified by naming
    /// heuristics against the raw dlc_load.json fields.
    pub dlc_state: Option<DlcState>,
}

/// Classified DLC state from dlc_load.json.
/// This is a conservative view: we identify DLC entries from the enabled_mods
/// list by their naming pattern, but we do NOT infer ownership from
/// dlc_load.json.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DlcState {
    /// DLC identifiers extracted from enabled_mods list, normalized for
    /// matching against save-required DLCs.  Entries are normalized by
    /// stripping a leading `dlc/` or `dlc\` prefix and a trailing `.dlc`
    /// extension, then lowercased for comparison.
    pub enabled_dlcs: Vec<String>,
    /// DLC identifiers from the disabled_dlcs field, normalized the same way.
    /// An explicit disable is **not** treated as proof of non-ownership, but
    /// it is a useful signal: a DLC the user has deliberately switched off
    /// may indicate they do not want its content active.
    pub disabled_dlcs: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LauncherDlcSummary {
    pub id: Option<String>,
    pub name: Option<String>,
    pub registry_id: Option<String>,
    pub path: Option<String>,
    /// Whether this DLC is enabled in the active launcher playset.
    ///
    /// `Some(true)` means the active playset explicitly enables it.
    /// `Some(false)` means the active playset explicitly disables it.
    /// `None` means the launcher catalog knows about it, but the active
    /// playset did not expose a clear enablement row for it.
    pub enabled_in_active_playset: Option<bool>,
}

/// Per-run DLC information combining save-required DLCs with current local DLC
/// state from `dlc_load.json` and launcher tables. Conservative semantics:
///
///   - **enabled**  = present in `dlc_load.json` DLC signals or explicitly
///     enabled in the active launcher playset
///   - **disabled** = present in `dlc_load.json` disabled_dlcs or explicitly
///     disabled in the active launcher playset
///   - **unknown**  = neither enabled nor disabled — the app cannot determine
///     whether the account owns this DLC from local-only sources
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunDlcInfo {
    /// DLCs required by this save AND detected as enabled locally.
    pub enabled_and_required: Vec<String>,
    /// DLCs required by this save but explicitly disabled locally.
    pub disabled_but_required: Vec<String>,
    /// DLCs required by this save whose status is unknown (not found in either
    /// the enabled or disabled lists).
    pub unknown_status_required: Vec<String>,
    /// All DLCs detected as currently enabled across local sources.
    pub all_enabled_dlcs: Vec<String>,
    /// All DLCs currently disabled across local sources.
    pub all_disabled_dlcs: Vec<String>,
}

/// Generate all possible search keys for a DLC identifier, enabling matching
/// between canonical IDs (e.g. `"dlc014_leviathans"`) and short launcher IDs
/// (e.g. `"leviathans"`), including known aliases like `plantoid` ↔ `plantoids`.
///
/// The returned set always includes the normalized form of the input itself.
/// For canonical IDs (`dlcNNN_name`) the short `name` suffix is also included.
/// Known aliases (e.g. `plantoid` ↔ `plantoids`) are added so that either form
/// matches the other.
pub fn dlc_match_keys(id: &str) -> Vec<String> {
    let normalized = normalize_dlc_id(id);
    let mut keys = vec![normalized.clone()];

    // Short name from canonical ID pattern: "dlc014_leviathans" → "leviathans"
    if let Some((_prefix, suffix)) = normalized.split_once('_') {
        if !suffix.is_empty()
            && !suffix.chars().all(|c| c.is_ascii_digit())
            && !keys.contains(&suffix.to_string())
        {
            keys.push(suffix.to_string());
        }
    }

    // Known aliases: launcher short names that differ from the canonical suffix
    // by more than the prefix.  Key = canonical suffix, values = launcher forms.
    // Example: canonical saves use "plantoids" but the launcher stores "plantoid".
    static DLC_ALIASES: &[(&str, &[&str])] = &[
        ("plantoids", &["plantoid"]),
        ("anniversary_portraits", &["anniversary"]),
        ("first_contact", &["firstcontact"]),
        ("galactic_paragons", &["paragon"]),
        ("shadows_of_the_shroud", &["shadows_shroud"]),
        ("rick_the_cube_species_portrait", &["rick_the_cube"]),
        ("stargazer_species_portrait", &["stargazer"]),
    ];

    // Collect extra alias keys for any key we already have.
    let mut extra = Vec::new();
    for key in &keys {
        for &(canonical, aliases) in DLC_ALIASES {
            if key == canonical {
                extra.extend(aliases.iter().map(|s| s.to_string()));
            }
            // Also handle reverse: if key is an alias, add the canonical form.
            if aliases.iter().any(|a| a == key) {
                extra.push(canonical.to_string());
            }
        }
    }
    keys.extend(extra);

    keys.sort();
    keys.dedup();
    keys
}

/// Derive the best single match key from a `LauncherDlcSummary` for use in
/// enablement lookup and DLC state matching.
///
/// Field priority:
/// 1. `registry_id`  (e.g. `"dlc009_plantoids"`) — the internal game key
/// 2. `path` / `dirPath` — extract last path component
/// 3. `name` — human-readable (least reliable for matching)
/// 4. `id` — raw primary key from the `dlc` table
pub(crate) fn launcher_dlc_match_key(dlc: &LauncherDlcSummary) -> Option<String> {
    if let Some(registry_id) = dlc.registry_id.as_deref() {
        return Some(normalize_dlc_id(registry_id));
    }
    if let Some(path) = dlc.path.as_deref() {
        let filename = path
            .split(&['/', '\\'][..])
            .rfind(|s| !s.is_empty())
            .unwrap_or(path);
        return Some(normalize_dlc_id(filename));
    }
    if let Some(name) = dlc.name.as_deref() {
        return Some(normalize_dlc_id(name));
    }
    dlc.id.as_deref().map(normalize_dlc_id)
}

/// Normalize a raw DLC identifier from dlc_load.json or save metadata into a
/// canonical form for matching.
///
/// Strips a leading `dlc/` or `dlc\` path prefix and a trailing `.dlc`
/// extension, then lowercases.  For subdirectory paths like
/// `dlc/dlc031_astral_planes/dlc031.dlc`, only the DLC directory name
/// (`dlc031_astral_planes`) is kept.
///
/// # Examples
///
/// ```
/// # use stellaris_hunter_scan::model::normalize_dlc_id;
/// assert_eq!(normalize_dlc_id("dlc/dlc014_leviathans.dlc"), "dlc014_leviathans");
/// assert_eq!(normalize_dlc_id("dlc028_ancient_relics"), "dlc028_ancient_relics");
/// assert_eq!(normalize_dlc_id("DLC/DLC_ALPHA.DLC"), "dlc_alpha");
/// assert_eq!(normalize_dlc_id("dlc/dlc031_astral_planes/dlc031.dlc"), "dlc031_astral_planes");
/// ```
pub fn normalize_dlc_id(raw: &str) -> String {
    let lower = raw.to_lowercase();
    let s = lower
        .strip_prefix("dlc/")
        .or_else(|| lower.strip_prefix("dlc\\"))
        .unwrap_or(&lower);
    let s = s.strip_suffix(".dlc").unwrap_or(s);
    // For subdirectory paths like "dlc031_astral_planes/dlc031", keep only
    // the DLC directory name (the part before any remaining separator).
    let s = s.split(&['/', '\\'][..]).next().unwrap_or(s);
    s.trim_end_matches(['/', '\\']).to_string()
}

/// Normalize a human-readable save `required_dlcs` name into a stable
/// underscore key for matching against launcher/registry DLC identifiers.
///
/// Canonical/internal IDs (e.g. `"dlc014_leviathans"`) pass through unchanged.
/// Human-readable names are lowercased, spaces become underscores, and known
/// suffixes (`_story_pack`, `_species_pack`) are stripped so that
/// `"Ancient Relics Story Pack"` → `"ancient_relics"`.
pub fn normalize_readable_dlc_name(raw: &str) -> String {
    // Canonical/internal IDs pass through (they already contain "dlc" prefix
    // and use underscores, so the suffix-strip below won't touch them).
    let mut s = raw.to_lowercase();
    s = s.replace(' ', "_");

    // Strip common DLC name suffixes (order matters: longer first).
    let suffixes = ["_story_pack", "_species_pack", "_expansion"];
    for suffix in &suffixes {
        if let Some(stripped) = s.strip_suffix(suffix) {
            return stripped.to_string();
        }
    }

    s
}

/// Classify entries from dlc_load.json into DLC vs non-DLC mods using naming
/// heuristics.
///
/// Entries in `enabled_mods` whose last path component ends with `.dlc` are
/// classified as DLCs.  Everything else (mod paths, workshop items, etc.) is
/// treated as a non-DLC mod and discarded from the DLC state.
///
/// The `disabled_dlcs` field is already identified by the game as DLC, so
/// its entries are passed through and normalized for matching.
///
/// This is **conservative**: we do not infer ownership from either list.
pub fn classify_dlc_state(enabled_mods: &[String], disabled_dlcs: &[String]) -> DlcState {
    let mut enabled_dlcs = Vec::new();
    for entry in enabled_mods
        .iter()
        .filter(|entry| {
            let filename = entry.split(&['/', '\\'][..]).next_back().unwrap_or(entry);
            filename.to_lowercase().ends_with(".dlc")
        })
        .map(|entry| normalize_dlc_id(entry))
    {
        if !enabled_dlcs.contains(&entry) {
            enabled_dlcs.push(entry);
        }
    }

    let mut normalized_disabled_dlcs = Vec::new();
    for entry in disabled_dlcs.iter().map(|entry| normalize_dlc_id(entry)) {
        if !normalized_disabled_dlcs.contains(&entry) {
            normalized_disabled_dlcs.push(entry);
        }
    }

    DlcState {
        enabled_dlcs,
        disabled_dlcs: normalized_disabled_dlcs,
    }
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
    pub dlcs: Vec<LauncherDlcSummary>,
    pub issues: Vec<String>,
}

/// Discovery facts extracted from save (galaxy-gen dependent)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DiscoveryFacts {
    pub endgame_crisis: Option<String>,
    pub sol_system_era: Option<String>,
    pub primitive_earth_present: Option<bool>,
    pub pre_ftl_era_target: Option<String>,
    pub target_species_class: Option<String>,
    pub target_homeworld_class: Option<String>,
    pub precursor_type: Option<String>,
    pub precursor_chain_completed: Option<String>,
    pub l_cluster_unlocked: Option<bool>,
    pub shielded_world_unlocked: Option<bool>,
    pub ancient_leviathan: Option<String>,
    pub enclave_type_present: Option<String>,
    pub great_khan_spawned: Option<bool>,
}

/// Progression facts extracted from save (numeric thresholds)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProgressionFacts {
    pub owned_planets: Option<usize>,
    pub colonized_planets: Option<usize>,
    pub surveyed_planets: Option<usize>,
    pub total_pops: Option<usize>,
    pub enslaved_pops_count: Option<usize>,
    pub robot_pop_ratio: Option<f64>,
    pub energy_stored: Option<f64>,
    pub energy_monthly: Option<f64>,
    pub minerals_monthly: Option<f64>,
    pub alloys_monthly: Option<f64>,
    pub trade_value_monthly: Option<f64>,
    pub strategic_resources_types: Option<usize>,
    pub organic_empires_remaining: Option<usize>,
    pub fleet_power: Option<f64>,
    pub fleet_count: Option<usize>,
    pub starbase_count: Option<usize>,
    pub gateway_count: Option<usize>,
    pub hyper_relay_count: Option<usize>,
    pub rare_technologies_acquired: Option<usize>,
    pub traditions_adopted: Option<usize>,
    pub ascension_perks_unlocked: Option<usize>,
    pub ascension_path: Option<String>,
    pub years_played: Option<f64>,
    pub years_at_peace: Option<f64>,
    pub diplomatic_weight: Option<f64>,
    pub intel_level_count: Option<usize>,
    pub observation_station_count: Option<usize>,
    pub capital_building_level: Option<usize>,
    pub living_standard: Option<String>,
    pub mercenary_enclaves_patroned: Option<usize>,
    pub vivarium_capacity: Option<usize>,
    pub megastructure_types: Vec<String>,
}

/// Action and event facts extracted from save (milestones, one-offs)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ActionFacts {
    // War/diplomacy
    pub active_wars: Option<usize>,
    pub war_type: Option<String>,
    pub subjects_acquired: Option<usize>,
    pub vassal_count: Option<usize>,
    pub subject_type: Option<String>,
    pub subject_contract_modified: Option<bool>,
    pub secret_fealty_pledged: Option<bool>,
    pub proxy_war_count: Option<usize>,

    // Federation
    pub federation_formed: Option<bool>,
    pub federation_type: Option<String>,
    pub federation_level: Option<usize>,
    pub federation_member_ethics: Vec<String>,

    // Galactic Community
    pub galactic_community_exists: Option<bool>,
    pub galactic_community_founding_member: Option<bool>,
    pub galactic_custodian: Option<bool>,
    pub galactic_custodian_action: Option<String>,
    pub galactic_emperor: Option<bool>,
    pub galactic_emperor_rebellion: Option<bool>,

    // Megastructures/Colossus
    pub colossus_built: Option<bool>,
    pub colossus_weapon_type: Option<String>,
    pub colossus_destroyed_while_firing: Option<bool>,

    // Species actions
    pub species_genetically_modified: Option<bool>,
    pub species_uplifted: Option<bool>,
    pub species_on_planet_count: Option<usize>,
    pub species_dna_phenotypes_collected: Option<usize>,
    pub slavery_type: Option<String>,
    pub livestock_species_count: Option<usize>,
    pub purged_pops: Option<usize>,
    pub species_enslaved: Option<bool>,

    // Relics
    pub relic_owned: Option<String>,
    pub relic_active_effect_used: Option<String>,
    pub galatron_acquired: Option<bool>,
    pub galatron_captured: Option<bool>,

    // Archaeology/exploration
    pub archaeological_site_completed: Option<String>,
    pub wormhole_travel_completed: Option<bool>,
    pub pre_ftl_infiltration_completed: Option<bool>,
    pub first_contact_result: Option<String>,
    pub espionage_operations_completed: Option<usize>,
    pub astral_rifts_explored: Option<usize>,

    // Crisis/Nemesis
    pub crisis_defeated: Option<bool>,
    pub captured_prethoryn_scourge_queen: Option<bool>,
    pub crisis_path_nemesis: Option<bool>,
    pub crisis_path_cosmogenesis: Option<bool>,
    pub crisis_path_hyperthermia: Option<bool>,
    pub crisis_path_behemoth_fury: Option<bool>,
    pub machine_uprising_victory: Option<bool>,

    // Enclaves/interactions
    pub amoeba_companion_found: Option<bool>,
    pub amoeba_companion_killed: Option<bool>,
    pub artisan_enclave_patron: Option<bool>,
    pub enclave_interaction_type: Option<String>,
    pub migration_treaty_count: Option<usize>,
    pub legendary_paragon_recruited: Option<bool>,

    // Misc events
    pub robot_pop_built: Option<bool>,
    pub horizon_signal_completed: Option<bool>,
    pub civil_war_completed: Option<bool>,
    pub special_project_completed_type: Option<String>,
    pub covenant_type: Option<String>,
    pub psionic_techs_unlocked: Option<bool>,
    pub quantum_catapult_used: Option<bool>,

    // Terraforming/decisions
    pub blazing_scourge_decisions: Option<bool>,
    pub stars_terraform_to_red_giant: Option<usize>,
    pub planets_terraform_to_volcanic: Option<usize>,
    pub volcanic_holy_world_created: Option<bool>,
    pub galactic_memorials_on_tomb_worlds: Option<usize>,
    pub space_fauna_type_captured: Option<String>,
    pub colony_count_with_hyperspace_not_researched: Option<usize>,

    // Event tracking
    pub pre_ftl_invasion_occurred: Option<bool>,
    pub artificial_military_ships_built: Option<bool>,

    // Legacy
    pub invaded_primitive_earth: Option<bool>,
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
    pub ruler_traits: Vec<String>,
    pub discovery: Option<DiscoveryFacts>,
    pub progression: Option<ProgressionFacts>,
    pub actions: Option<ActionFacts>,
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
    /// Conservative DLC info for this run, combining save-required DLCs with
    /// current local DLC state from `dlc_load.json` and launcher tables.
    pub dlc_info: Option<RunDlcInfo>,
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
pub struct FactOverride {
    pub run_folder_path: String,
    pub dimension: String,
    pub key: String,
    pub value: Value,
    pub reason: Option<String>,
    pub created_at: String,
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
    pub is_override: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSyncResult {
    pub updated: bool,
    pub old_version: Option<String>,
    pub new_version: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamSyncResult {
    pub synced: u32,
    pub skipped: u32,
    pub unmatched: u32,
    pub total_steam_achievements: u32,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub install_path_override: Option<String>,
    pub documents_path_override: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub app_version: String,
    pub catalog_version: Option<String>,
    pub stellaris_version: Option<String>,
    pub last_catalog_sync: Option<String>,
    pub last_steam_sync: Option<String>,
    pub last_save_scan: Option<String>,
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
