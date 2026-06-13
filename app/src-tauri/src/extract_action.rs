// Functions are pub(crate) and intended for use by save.rs or commands — dead until
// integrated into the parse pipeline.
#![allow(dead_code)]

//! Extract action/event facts from a parsed Stellaris gamestate AST.
//!
//! These facts represent milestones, one-time events, and state transitions
//! that the player has triggered during a playthrough. Many are tracked via
//! country-level flags (`country.flags.<flag_name>`).
//!
//! # Organization
//! Each category of facts has a dedicated extraction function:
//!
//! * **War / Diplomacy** — Active war participation, subject/vassal counts, flag-based events
//! * **Federation** — Federation membership inferred from country-level field
//! * **Galactic Community** — GC existence, custodian, emperor from top-level + flags
//! * **Megastructures / Colossus** — Ascension perks, flags  
//! * **Species Actions** — Flag-based modifiers and events
//! * **Relics** — List of relics, galatron detection
//! * **Archaeology / Exploration** — Completed archaeological sites, wormhole travel
//! * **Crisis / Nemesis** — Crisis defeat, crisis paths from flags and perks  
//! * **Enclaves / Interactions** — Flag-based enclave interactions
//! * **Misc Events** — Horizon signal, covenant, psionics from flags/perks
//! * **Terraforming / Decisions** — Flag-based terraforming events
//! * **Legacy** — Invaded primitive Earth flag

use crate::model::ActionFacts;
use crate::save::*;

// ---------------------------------------------------------------------------
// Flag helpers
// ---------------------------------------------------------------------------

/// Check whether a boolean flag key exists in the country flags block.
///
/// Stellaris stores many achievement-relevant event triggers as country-level
/// flags. The mere presence of the flag key is sufficient to indicate the
/// event occurred; the value (typically `"yes"`) is secondary.
fn has_flag(country_value: &ClausewitzValue, flag_name: &str) -> bool {
    query_path(country_value, &["flags", flag_name]).is_some()
}

/// Return `true` if *any* of the given flag names is present.
fn has_any_flag(country_value: &ClausewitzValue, flag_names: &[&str]) -> bool {
    flag_names.iter().any(|name| has_flag(country_value, name))
}

// ---------------------------------------------------------------------------
// Country field helpers
// ---------------------------------------------------------------------------

/// Collect atom strings from a country field that may be either a single
/// space-delimited atom or a block of atom values.
fn collect_space_list(country_value: &ClausewitzValue, field: &str) -> Vec<String> {
    match query_path(country_value, &[field]) {
        Some(ClausewitzValue::Atom(s)) => s.split_whitespace().map(|s| s.to_string()).collect(),
        Some(_) => query_atoms(country_value, &[field]),
        None => vec![],
    }
}

/// Check if `field` contains an atom equal to the given value.
fn field_contains(country_value: &ClausewitzValue, field: &str, value: &str) -> bool {
    collect_space_list(country_value, field)
        .iter()
        .any(|s| s == value)
}

// ---------------------------------------------------------------------------
// Block iteration helpers
// ---------------------------------------------------------------------------

/// Check if a block of `{ country = <id> ... }` sub-blocks (Value nodes)
/// contains an entry whose `country` field matches `target_id`.
///
/// Used for war participant lists, archaeological site completions, etc.
fn block_entries_contain_country(block: &ClausewitzValue, target_id: &str) -> bool {
    match block {
        ClausewitzValue::Block(entries) => entries.iter().any(|entry| {
            if let ClausewitzNode::Value(ClausewitzValue::Block(fields)) = entry {
                fields.iter().any(|field| {
                    matches!(
                        field,
                        ClausewitzNode::Pair(key, ClausewitzValue::Atom(val))
                            if key == "country" && val == target_id
                    )
                })
            } else {
                false
            }
        }),
        _ => false,
    }
}

/// Check if the player's country participates in a war entry.
fn player_in_war(war_value: &ClausewitzValue, player_country_id: &str) -> bool {
    for side in &["attackers", "defenders"] {
        if let Some(side_block) = query_path(war_value, &[side]) {
            if block_entries_contain_country(side_block, player_country_id) {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// War / Diplomacy
// ---------------------------------------------------------------------------

/// Count active wars the player is involved in and capture the first war type.
fn count_player_active_wars(
    game_root: &ClausewitzValue,
    player_country_id: &str,
) -> (usize, Option<String>) {
    let war_block = match query_path(game_root, &["war"]) {
        Some(ClausewitzValue::Block(nodes)) => nodes,
        _ => return (0, None),
    };

    let mut count = 0;
    let mut war_type = None;

    for node in war_block {
        if let ClausewitzNode::Pair(_, war_value) = node {
            if player_in_war(war_value, player_country_id) {
                count += 1;
                if war_type.is_none() {
                    war_type = query_atom(war_value, &["war_goal", "type"]);
                }
            }
        }
    }

    (count, war_type)
}

fn extract_subjects_data(country_value: &ClausewitzValue) -> (Option<usize>, Option<usize>) {
    let count = query_path(country_value, &["subjects"]).map(count_entries);
    (count, count)
}

// ---------------------------------------------------------------------------
// Federation
// ---------------------------------------------------------------------------

/// Federation membership is tracked via the country's `federation` field:
/// a numeric ID atom if the player belongs to a federation, absent otherwise.
fn extract_federation_formed(country_value: &ClausewitzValue) -> Option<bool> {
    match query_path(country_value, &["federation"]) {
        Some(ClausewitzValue::Atom(s)) => Some(!s.is_empty()),
        Some(_) => Some(true),
        None => Some(false),
    }
}

// ---------------------------------------------------------------------------
// Galactic Community
// ---------------------------------------------------------------------------

fn extract_gc_exists(game_root: &ClausewitzValue) -> Option<bool> {
    query_path(game_root, &["galactic_community", "community_formed"]).map(|_| true)
}

fn extract_gc_custodian(game_root: &ClausewitzValue, player_country_id: &str) -> Option<bool> {
    // custodian_term != -1 indicates there is an active custodian
    let active = query_f64(game_root, &["galactic_community", "custodian_term"])
        .map(|t| t != -1.0)
        .unwrap_or(false);

    if !active {
        return Some(false);
    }

    // The GC leader is the custodian
    Some(
        query_atom(game_root, &["galactic_community", "leader"])
            .map(|id| id == player_country_id)
            .unwrap_or(false),
    )
}

// ---------------------------------------------------------------------------
// Relics
// ---------------------------------------------------------------------------

fn extract_relics_owned(country_value: &ClausewitzValue) -> Option<String> {
    let relics = collect_space_list(country_value, "relics");
    if relics.is_empty() {
        None
    } else {
        Some(relics.join(", "))
    }
}

fn extract_galatron(country_value: &ClausewitzValue) -> Option<bool> {
    Some(field_contains(country_value, "relics", "r_galatron"))
}

// ---------------------------------------------------------------------------
// Archaeology / Exploration
// ---------------------------------------------------------------------------

fn extract_archaeology_sites(
    game_root: &ClausewitzValue,
    player_country_id: &str,
) -> Option<String> {
    let sites_block = query_path(game_root, &["archaeological_sites", "sites"])?;

    let mut completed_types = Vec::new();

    if let ClausewitzValue::Block(nodes) = sites_block {
        for node in nodes {
            if let ClausewitzNode::Pair(_site_id, site_value) = node {
                let player_completed = query_path(site_value, &["completed"])
                    .map(|c| block_entries_contain_country(c, player_country_id))
                    .unwrap_or(false);

                if player_completed {
                    if let Some(site_type) = query_atom(site_value, &["type"]) {
                        completed_types.push(site_type);
                    }
                }
            }
        }
    }

    if completed_types.is_empty() {
        None
    } else {
        Some(completed_types.join(", "))
    }
}

// ---------------------------------------------------------------------------
// Crisis / Nemesis
// ---------------------------------------------------------------------------

fn extract_crisis_path_nemesis(country_value: &ClausewitzValue) -> Option<bool> {
    let via_flag = has_any_flag(country_value, &["became_crisis", "crisis_path_nemesis"]);
    let via_perk = field_contains(country_value, "ascension_perks", "ap_crisis_acolyte");
    Some(via_flag || via_perk)
}

fn extract_crisis_path_cosmogenesis(country_value: &ClausewitzValue) -> Option<bool> {
    Some(has_any_flag(
        country_value,
        &["cosmogenesis", "crisis_path_cosmogenesis"],
    ))
}

// ---------------------------------------------------------------------------
// Misc Events
// ---------------------------------------------------------------------------

fn check_horizon_signal(country_value: &ClausewitzValue) -> Option<bool> {
    Some(has_any_flag(
        country_value,
        &["horizon_signal", "worm_in_rift", "horizon_signal_completed"],
    ))
}

fn extract_covenant(country_value: &ClausewitzValue) -> Option<String> {
    // Check flag-based covenant tracking
    let flag_covenants = [
        ("covenant_whisperers", "whisperers"),
        ("covenant_instrument", "instrument"),
        ("covenant_composer", "composer"),
        ("covenant_eater", "eater"),
    ];

    for (flag, name) in &flag_covenants {
        if has_flag(country_value, flag) {
            return Some(name.to_string());
        }
    }

    // Check ascension perk-based covenant tracking
    let perk_covenants = [
        ("ap_covenant_whisperers", "whisperers"),
        ("ap_covenant_instrument", "instrument"),
        ("ap_covenant_composer", "composer"),
        ("ap_covenant_eater", "eater"),
    ];

    for (perk, name) in &perk_covenants {
        if field_contains(country_value, "ascension_perks", perk) {
            return Some(name.to_string());
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Extract all action/event facts from the parsed gamestate.
///
/// # Arguments
/// * `game_root` — The root of the parsed gamestate AST (from `gamestate` ZIP entry)
/// * `country_value` — The player country's block (country ID 0)
/// * `player_country_id` — The player's country ID string (typically `"0"`)
///
/// # Graceful Degradation
/// Fields that cannot be determined from available data are left as `None`.
/// No panics occur in non-test code (no `unwrap()`).
pub(crate) fn extract_action_facts(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
    player_country_id: &str,
) -> ActionFacts {
    let (active_wars, war_type) = count_player_active_wars(game_root, player_country_id);
    let (subjects_acquired, vassal_count) = extract_subjects_data(country_value);

    ActionFacts {
        // -- War / Diplomacy --
        active_wars: if active_wars > 0 {
            Some(active_wars)
        } else {
            None
        },
        war_type,
        subjects_acquired,
        vassal_count,
        subject_type: None, // Requires parsing agreement types -- uncertain
        subject_contract_modified: Some(has_flag(country_value, "subject_contract_modified")),
        secret_fealty_pledged: Some(has_any_flag(
            country_value,
            &["secret_fealty_pledged", "secret_fealty", "fealty_pledged"],
        )),
        proxy_war_count: None, // Proxy war tracking is flag-based and uncertain

        // -- Federation --
        federation_formed: extract_federation_formed(country_value),
        federation_type: None, // Federation type in agreements or flags -- uncertain
        federation_level: None, // Federation level tracking uncertain
        federation_member_ethics: vec![], // Requires lookup of member countries -- future work

        // -- Galactic Community --
        galactic_community_exists: extract_gc_exists(game_root),
        galactic_community_founding_member: Some(has_flag(
            country_value,
            "galactic_community_founding_member",
        )),
        galactic_custodian: extract_gc_custodian(game_root, player_country_id),
        galactic_custodian_action: query_atom(country_value, &["flags", "custodian_action"]),
        galactic_emperor: Some(has_flag(country_value, "galactic_emperor")),
        galactic_emperor_rebellion: Some(has_flag(country_value, "galactic_emperor_rebellion")),

        // -- Megastructures / Colossus --
        colossus_built: Some(field_contains(
            country_value,
            "ascension_perks",
            "ap_colossus",
        )),
        colossus_weapon_type: None, // Colossus weapon type is flag-based -- uncertain
        colossus_destroyed_while_firing: Some(has_flag(country_value, "colossus_destroyed")),

        // -- Species Actions --
        species_genetically_modified: Some(has_flag(country_value, "genetically_modified")),
        species_uplifted: Some(has_flag(country_value, "species_uplifted")),
        species_on_planet_count: None, // Requires iterating all planets -- complex
        species_dna_phenotypes_collected: None, // Requires species_db analysis -- complex
        slavery_type: query_atom(country_value, &["flags", "slavery_type"]),
        livestock_species_count: None, // Requires species_db analysis -- complex
        purged_pops: None,             // Requires pop iteration -- complex
        species_enslaved: Some(has_flag(country_value, "species_enslaved")),

        // -- Relics --
        relic_owned: extract_relics_owned(country_value),
        relic_active_effect_used: None, // Flag-based and uncertain
        galatron_acquired: extract_galatron(country_value),
        galatron_captured: Some(has_flag(country_value, "galatron_captured")),

        // -- Archaeology / Exploration --
        archaeological_site_completed: extract_archaeology_sites(game_root, player_country_id),
        wormhole_travel_completed: Some(has_flag(country_value, "wormhole_travel_completed")),
        pre_ftl_infiltration_completed: Some(has_flag(country_value, "pre_ftl_infiltrated")),
        first_contact_result: None, // First contact data format uncertain
        espionage_operations_completed: None, // Flag-based and uncertain
        astral_rifts_explored: None, // Requires astral_rifts block iteration -- uncertain

        // -- Crisis / Nemesis --
        crisis_defeated: Some(has_flag(country_value, "crisis_defeated")),
        captured_prethoryn_scourge_queen: Some(has_flag(country_value, "prethoryn_queen_captured")),
        crisis_path_nemesis: extract_crisis_path_nemesis(country_value),
        crisis_path_cosmogenesis: extract_crisis_path_cosmogenesis(country_value),
        crisis_path_hyperthermia: Some(has_flag(country_value, "hyperthermia")),
        crisis_path_behemoth_fury: Some(has_flag(country_value, "behemoth_fury")),
        machine_uprising_victory: Some(has_flag(country_value, "machine_uprising_survived")),

        // -- Enclaves / Interactions --
        amoeba_companion_found: Some(has_flag(country_value, "space_amoeba_companion")),
        amoeba_companion_killed: Some(has_flag(country_value, "space_amoeba_killed")),
        artisan_enclave_patron: Some(has_flag(country_value, "artisan_enclave_patron")),
        enclave_interaction_type: None, // Enclave interactions vary too much
        migration_treaty_count: None,   // Would need agreement iteration
        legendary_paragon_recruited: Some(has_flag(country_value, "legendary_paragon_recruited")),

        // -- Misc Events --
        robot_pop_built: Some(has_flag(country_value, "robot_pop_built")),
        horizon_signal_completed: check_horizon_signal(country_value),
        civil_war_completed: Some(has_flag(country_value, "civil_war_completed")),
        special_project_completed_type: None, // Too many special project types
        covenant_type: extract_covenant(country_value),
        psionic_techs_unlocked: Some(
            field_contains(country_value, "ascension_perks", "ap_mind_over_matter")
                || has_flag(country_value, "psionic_theory_researched"),
        ),
        quantum_catapult_used: Some(has_flag(country_value, "quantum_catapult_used")),

        // -- Terraforming / Decisions --
        blazing_scourge_decisions: None, // Complex -- requires decision tracking
        stars_terraform_to_red_giant: None, // Complex -- requires star type tracking
        planets_terraform_to_volcanic: None, // Would need planet iteration
        volcanic_holy_world_created: Some(has_flag(country_value, "volcanic_holy_world")),
        galactic_memorials_on_tomb_worlds: None, // Would need planet iteration
        space_fauna_type_captured: None,         // Would need fleet analysis
        colony_count_with_hyperspace_not_researched: None, // Complex -- future work

        // -- Legacy --
        invaded_primitive_earth: Some(has_flag(country_value, "invaded_earth")),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Test fixture builders ---

    fn make_atom(s: &str) -> ClausewitzValue {
        ClausewitzValue::Atom(s.to_string())
    }

    fn make_block(pairs: Vec<(&str, ClausewitzValue)>) -> ClausewitzValue {
        ClausewitzValue::Block(
            pairs
                .into_iter()
                .map(|(k, v)| ClausewitzNode::Pair(k.to_string(), v))
                .collect(),
        )
    }

    fn make_value_block(values: Vec<ClausewitzValue>) -> ClausewitzValue {
        ClausewitzValue::Block(values.into_iter().map(ClausewitzNode::Value).collect())
    }

    /// Build a minimal country block with optional flags.
    fn make_country(flags: Vec<(&str, &str)>) -> ClausewitzValue {
        let flag_pairs: Vec<(&str, ClausewitzValue)> =
            flags.iter().map(|(k, v)| (*k, make_atom(v))).collect();
        make_block(vec![("flags", make_block(flag_pairs))])
    }

    /// Build a minimal game root (empty except for provided entries).
    fn make_game_root(entries: Vec<(&str, ClausewitzValue)>) -> ClausewitzValue {
        make_block(entries)
    }

    // -----------------------------------------------------------------------
    // Test: Empty state -- all fields should be None/false/default
    // -----------------------------------------------------------------------

    #[test]
    fn test_empty_state_returns_defaults() {
        let country = make_country(vec![]);
        let game_root = make_game_root(vec![]);

        let facts = extract_action_facts(&game_root, &country, "0");

        assert!(facts.active_wars.is_none());
        assert!(facts.war_type.is_none());
        assert!(facts.subjects_acquired.is_none());
        assert!(facts.federation_formed == Some(false));
        assert!(facts.galactic_community_exists.is_none());
        assert!(facts.colossus_built == Some(false));
        assert!(facts.relic_owned.is_none());
        assert!(facts.galatron_acquired == Some(false));
        assert!(facts.archaeological_site_completed.is_none());
        assert!(facts.horizon_signal_completed == Some(false));
        assert!(facts.covenant_type.is_none());
        assert!(facts.colony_count_with_hyperspace_not_researched.is_none());
    }

    // -----------------------------------------------------------------------
    // Test: Flag extraction
    // -----------------------------------------------------------------------

    #[test]
    fn test_flag_extraction() {
        let country = make_country(vec![
            ("subject_contract_modified", "yes"),
            ("galactic_emperor", "yes"),
            ("crisis_defeated", "yes"),
            ("invaded_earth", "yes"),
        ]);
        let game_root = make_game_root(vec![]);

        let facts = extract_action_facts(&game_root, &country, "0");

        assert_eq!(facts.subject_contract_modified, Some(true));
        assert_eq!(facts.galactic_emperor, Some(true));
        assert_eq!(facts.crisis_defeated, Some(true));
        assert_eq!(facts.invaded_primitive_earth, Some(true));

        // Not-set flags should be false
        assert_eq!(facts.crisis_path_cosmogenesis, Some(false));
        assert_eq!(facts.robot_pop_built, Some(false));
        assert_eq!(facts.wormhole_travel_completed, Some(false));
    }

    // -----------------------------------------------------------------------
    // Test: Federation membership
    // -----------------------------------------------------------------------

    #[test]
    fn test_federation_membership() {
        // Country with federation ID
        let country_in_fed = make_block(vec![
            ("federation", make_atom("16777220")),
            ("flags", make_block(vec![])),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country_in_fed, "0");
        assert_eq!(facts.federation_formed, Some(true));

        // Country without federation
        let country_no_fed = make_country(vec![]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country_no_fed, "0");
        assert_eq!(facts.federation_formed, Some(false));
    }

    // -----------------------------------------------------------------------
    // Test: War participation
    // -----------------------------------------------------------------------

    #[test]
    fn test_war_participation() {
        // Build a war where player (country 0) is an attacker
        let participant = make_block(vec![
            ("country", make_atom("0")),
            ("call_type", make_atom("primary")),
        ]);

        let war_goal = make_block(vec![("type", make_atom("war_type_conquest"))]);

        let war_entry = make_block(vec![
            ("attackers", make_value_block(vec![participant])),
            ("defenders", make_value_block(vec![])),
            ("war_goal", war_goal),
        ]);

        let game_root = make_game_root(vec![("war", make_block(vec![("war_1", war_entry)]))]);
        let country = make_country(vec![]);

        let facts = extract_action_facts(&game_root, &country, "0");

        assert_eq!(facts.active_wars, Some(1));
        assert_eq!(facts.war_type, Some("war_type_conquest".to_string()));
    }

    // -----------------------------------------------------------------------
    // Test: War participation in defenders
    // -----------------------------------------------------------------------

    #[test]
    fn test_war_defender_side() {
        let defender = make_block(vec![
            ("country", make_atom("0")),
            ("call_type", make_atom("primary")),
        ]);

        let war_goal = make_block(vec![("type", make_atom("war_type_defense"))]);

        let war_entry = make_block(vec![
            ("attackers", make_value_block(vec![])),
            ("defenders", make_value_block(vec![defender])),
            ("war_goal", war_goal),
        ]);

        let game_root = make_game_root(vec![("war", make_block(vec![("war_1", war_entry)]))]);
        let country = make_country(vec![]);

        let facts = extract_action_facts(&game_root, &country, "0");

        assert_eq!(facts.active_wars, Some(1));
        assert_eq!(facts.war_type, Some("war_type_defense".to_string()));
    }

    // -----------------------------------------------------------------------
    // Test: Relic ownership and galatron
    // -----------------------------------------------------------------------

    #[test]
    fn test_relic_ownership() {
        // Relics as block of atoms
        let relics = make_block(vec![
            (
                "relics",
                make_value_block(vec![make_atom("r_galatron"), make_atom("r_dragon_trophy")]),
            ),
            ("flags", make_block(vec![])),
        ]);

        let facts = extract_action_facts(&make_game_root(vec![]), &relics, "0");

        assert!(facts
            .relic_owned
            .as_deref()
            .unwrap_or("")
            .contains("r_galatron"));
        assert!(facts
            .relic_owned
            .as_deref()
            .unwrap_or("")
            .contains("r_dragon_trophy"));
        assert_eq!(facts.galatron_acquired, Some(true));

        // Empty relics
        let country = make_country(vec![]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert!(facts.relic_owned.is_none());
        assert_eq!(facts.galatron_acquired, Some(false));
    }

    // -----------------------------------------------------------------------
    // Test: Multiple wars
    // -----------------------------------------------------------------------

    #[test]
    fn test_multiple_wars() {
        let make_participant = || {
            make_block(vec![
                ("country", make_atom("0")),
                ("call_type", make_atom("primary")),
            ])
        };

        let war1 = make_block(vec![
            ("attackers", make_value_block(vec![make_participant()])),
            ("defenders", make_value_block(vec![])),
            (
                "war_goal",
                make_block(vec![("type", make_atom("war_type_conquest"))]),
            ),
        ]);

        let war2 = make_block(vec![
            ("attackers", make_value_block(vec![make_participant()])),
            ("defenders", make_value_block(vec![])),
            (
                "war_goal",
                make_block(vec![("type", make_atom("war_type_status_quo"))]),
            ),
        ]);

        let game_root = make_game_root(vec![(
            "war",
            make_block(vec![("war_1", war1), ("war_2", war2)]),
        )]);
        let country = make_country(vec![]);

        let facts = extract_action_facts(&game_root, &country, "0");

        assert_eq!(facts.active_wars, Some(2));
        // First war type found
        assert_eq!(facts.war_type, Some("war_type_conquest".to_string()));
    }

    // -----------------------------------------------------------------------
    // Test: Galactic Community
    // -----------------------------------------------------------------------

    #[test]
    fn test_galactic_community() {
        // GC exists, player is NOT leader
        let gc_no_leader = make_block(vec![
            ("community_formed", make_atom("2200.01.01")),
            ("custodian_term", make_atom("-1")),
            ("leader", make_atom("1")),
        ]);
        let game_root = make_game_root(vec![("galactic_community", gc_no_leader)]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game_root, &country, "0");

        assert_eq!(facts.galactic_community_exists, Some(true));
        assert_eq!(facts.galactic_custodian, Some(false));

        // GC exists, player IS leader and custodian
        let gc_with_custodian = make_block(vec![
            ("community_formed", make_atom("2200.01.01")),
            ("custodian_term", make_atom("3")),
            ("leader", make_atom("0")),
        ]);
        let game_root = make_game_root(vec![("galactic_community", gc_with_custodian)]);
        let facts = extract_action_facts(&game_root, &country, "0");

        assert_eq!(facts.galactic_custodian, Some(true));
    }

    // -----------------------------------------------------------------------
    // Test: Ascension perks
    // -----------------------------------------------------------------------

    #[test]
    fn test_ascension_perks() {
        // Space-delimited atom
        let country = make_block(vec![
            (
                "ascension_perks",
                make_atom("ap_colossus ap_mind_over_matter ap_crisis_acolyte"),
            ),
            ("flags", make_block(vec![])),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");

        assert_eq!(facts.colossus_built, Some(true));
        assert_eq!(facts.psionic_techs_unlocked, Some(true));
        assert_eq!(facts.crisis_path_nemesis, Some(true));

        // No perks
        let country = make_country(vec![]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.colossus_built, Some(false));
        assert_eq!(facts.psionic_techs_unlocked, Some(false));
    }

    // -----------------------------------------------------------------------
    // Test: Ascension perks as block of atoms
    // -----------------------------------------------------------------------

    #[test]
    fn test_ascension_perks_as_block() {
        let perks_block = make_value_block(vec![
            make_atom("ap_colossus"),
            make_atom("ap_mind_over_matter"),
        ]);

        let country = make_block(vec![
            ("ascension_perks", perks_block),
            ("flags", make_block(vec![])),
        ]);

        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");

        assert_eq!(facts.colossus_built, Some(true));
        assert_eq!(facts.psionic_techs_unlocked, Some(true));
    }

    // -----------------------------------------------------------------------
    // Test: Horizon signal
    // -----------------------------------------------------------------------

    #[test]
    fn test_horizon_signal_detection() {
        // Various flag names that indicate completion
        for flag in &["horizon_signal", "worm_in_rift", "horizon_signal_completed"] {
            let country = make_country(vec![(flag, "yes")]);
            let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
            assert_eq!(
                facts.horizon_signal_completed,
                Some(true),
                "flag '{flag}' should trigger horizon signal"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Test: Subjects / Vassals
    // -----------------------------------------------------------------------

    #[test]
    fn test_subjects_counting() {
        // Country with 2 subjects
        let subject1 = make_block(vec![("type", make_atom("vassal"))]);
        let subject2 = make_block(vec![("type", make_atom("tributary"))]);

        let country = make_block(vec![
            (
                "subjects",
                make_block(vec![("subject_1", subject1), ("subject_2", subject2)]),
            ),
            ("flags", make_block(vec![])),
        ]);

        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");

        assert_eq!(facts.subjects_acquired, Some(2));
        assert_eq!(facts.vassal_count, Some(2));
    }

    // -----------------------------------------------------------------------
    // Test: Covenant extraction
    // -----------------------------------------------------------------------

    #[test]
    fn test_covenant_detection() {
        // Flag-based
        let country = make_country(vec![("covenant_whisperers", "yes")]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.covenant_type, Some("whisperers".to_string()));

        // Perk-based
        let country = make_block(vec![
            ("ascension_perks", make_atom("ap_covenant_eater")),
            ("flags", make_block(vec![])),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.covenant_type, Some("eater".to_string()));

        // No covenant
        let country = make_country(vec![]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert!(facts.covenant_type.is_none());
    }

    // -----------------------------------------------------------------------
    // Test: Secret fealty
    // -----------------------------------------------------------------------

    #[test]
    fn test_secret_fealty_detection() {
        for flag in &["secret_fealty_pledged", "secret_fealty", "fealty_pledged"] {
            let country = make_country(vec![(flag, "yes")]);
            let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
            assert_eq!(
                facts.secret_fealty_pledged,
                Some(true),
                "flag '{flag}' should trigger secret fealty"
            );
        }

        let country = make_country(vec![]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.secret_fealty_pledged, Some(false));
    }
}
