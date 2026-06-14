// Functions are pub(crate) and used by save.rs as part of the parse pipeline.

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

/// Find a flag whose key contains any of the given substrings.
/// Returns the matching flag key name.
#[allow(dead_code)]
fn find_flag_containing(country_value: &ClausewitzValue, substrings: &[&str]) -> Option<String> {
    let flags = query_path(country_value, &["flags"])?;
    match flags {
        ClausewitzValue::Block(nodes) => {
            for node in nodes {
                if let ClausewitzNode::Pair(key, _) = node {
                    for substr in substrings {
                        if key.contains(substr) {
                            return Some(key.clone());
                        }
                    }
                }
            }
            None
        }
        ClausewitzValue::Atom(_) => None,
    }
}

/// Iterate over all flag keys in the flags block.
fn for_each_flag_key<F>(country_value: &ClausewitzValue, mut f: F)
where
    F: FnMut(&str),
{
    if let Some(ClausewitzValue::Block(nodes)) = query_path(country_value, &["flags"]) {
        for node in nodes {
            if let ClausewitzNode::Pair(key, _) = node {
                f(key);
            }
        }
    }
}

/// Iterate over pop_groups entries matching a given category and sum their sizes.
fn sum_pop_groups_by_category(game_root: &ClausewitzValue, category: &str) -> Option<usize> {
    let pop_groups = query_path(game_root, &["pop_groups"])?;
    match pop_groups {
        ClausewitzValue::Block(nodes) => {
            let total: usize = nodes
                .iter()
                .filter_map(|node| {
                    if let ClausewitzNode::Pair(_, entry) = node {
                        if query_atom(entry, &["key", "category"]).as_deref() == Some(category) {
                            return Some(query_f64(entry, &["size"]).unwrap_or(0.0) as usize);
                        }
                    }
                    None
                })
                .sum();
            Some(total)
        }
        _ => Some(0),
    }
}

/// Count distinct species in pop_groups matching a given category.
fn count_distinct_species_in_category(
    game_root: &ClausewitzValue,
    category: &str,
) -> Option<usize> {
    let pop_groups = query_path(game_root, &["pop_groups"])?;
    let pop_groups = match pop_groups {
        ClausewitzValue::Block(nodes) => nodes,
        _ => return Some(0),
    };

    let mut species = std::collections::HashSet::new();
    for node in pop_groups {
        if let ClausewitzNode::Pair(_, entry) = node {
            if query_atom(entry, &["key", "category"]).as_deref() == Some(category) {
                if let Some(s) = query_atom(entry, &["key", "species"]) {
                    species.insert(s);
                }
            }
        }
    }
    Some(species.len())
}

/// Iterate over planets owned by a given country from `planets > planet`.
fn for_each_owned_planet<'a>(
    game_root: &'a ClausewitzValue,
    player_country_id: &str,
    mut f: impl FnMut(&'a ClausewitzValue),
) {
    let Some(ClausewitzValue::Block(nodes)) = query_path(game_root, &["planets", "planet"]) else {
        return;
    };
    for node in nodes {
        if let ClausewitzNode::Pair(_, planet_value) = node {
            if query_atom(planet_value, &["controller"]).as_deref() == Some(player_country_id) {
                f(planet_value);
            }
        }
    }
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
// Subject Type
// ---------------------------------------------------------------------------

/// Collect unique subject type atoms from `country > subjects > * > type`.
fn extract_subject_type(country_value: &ClausewitzValue) -> Option<String> {
    let subjects = query_path(country_value, &["subjects"])?;
    match subjects {
        ClausewitzValue::Block(nodes) => {
            let mut types: Vec<String> = Vec::new();
            for node in nodes {
                if let ClausewitzNode::Pair(_, entry) = node {
                    if let Some(t) = query_atom(entry, &["type"]) {
                        if !types.contains(&t) {
                            types.push(t);
                        }
                    }
                }
            }
            if types.is_empty() {
                None
            } else {
                Some(types.join(", "))
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Colossus Weapon Type
// ---------------------------------------------------------------------------

/// Extract the colossus weapon type from a flag like `colossus_weapon_world_cracker`.
fn extract_colossus_weapon_type(country_value: &ClausewitzValue) -> Option<String> {
    let flags = query_path(country_value, &["flags"])?;
    match flags {
        ClausewitzValue::Block(nodes) => {
            for node in nodes {
                if let ClausewitzNode::Pair(key, _) = node {
                    if let Some(suffix) = key.strip_prefix("colossus_weapon_") {
                        if !suffix.is_empty() {
                            return Some(suffix.to_string());
                        }
                    }
                }
            }
            None
        }
        ClausewitzValue::Atom(_) => None,
    }
}

// ---------------------------------------------------------------------------
// Enclave Interaction Type
// ---------------------------------------------------------------------------

/// Collect known enclave interaction flags from country flags.
fn extract_enclave_interaction_type(country_value: &ClausewitzValue) -> Option<String> {
    let patterns = &[
        "recruit_curator_scientist",
        "buy_trader_resource",
        "destroyed_enclave",
        "destroy_enclave",
        "traded_with_artisans",
        "recruited_curator",
    ];
    let mut found: Vec<String> = Vec::new();
    for_each_flag_key(country_value, |key| {
        for pattern in patterns {
            if key.contains(pattern) && !found.contains(&pattern.to_string()) {
                found.push(pattern.to_string());
            }
        }
    });
    if found.is_empty() {
        None
    } else {
        Some(found.join(", "))
    }
}

// ---------------------------------------------------------------------------
// Special Project Completed Type
// ---------------------------------------------------------------------------

/// Collect known special project completion types from flags and special_projects block.
fn extract_special_project_completed_type(country_value: &ClausewitzValue) -> Option<String> {
    let known_projects = &[
        "breach_the_shroud",
        "limbo",
        "synthetic_infiltrator_detection",
        "infinity_sphere_peaceful",
        "unexpected_mineral_seams",
        "knights_toxic_god_final",
        "mysterious_chart",
    ];
    let mut found: Vec<String> = Vec::new();

    // Check flags
    for_each_flag_key(country_value, |key| {
        for project in known_projects {
            if key.contains(project) && !found.contains(&project.to_string()) {
                found.push(project.to_string());
            }
        }
    });

    // Check special_projects block
    if let Some(ClausewitzValue::Block(nodes)) = query_path(country_value, &["special_projects"]) {
        for node in nodes {
            if let ClausewitzNode::Pair(key, _) = node {
                for project in known_projects {
                    if key.contains(project) && !found.contains(&project.to_string()) {
                        found.push(project.to_string());
                    }
                }
            }
        }
    }

    if found.is_empty() {
        None
    } else {
        Some(found.join(", "))
    }
}

// ---------------------------------------------------------------------------
// Purged Pops
// ---------------------------------------------------------------------------

/// Sum the size of all pop groups with `key > category = "purge"`.
fn count_purged_pops(game_root: &ClausewitzValue) -> Option<usize> {
    sum_pop_groups_by_category(game_root, "purge")
}

// ---------------------------------------------------------------------------
// Space Fauna Type Captured
// ---------------------------------------------------------------------------

/// Collect unique space fauna types from vivarium_critters and exhibits.
fn extract_space_fauna_type_captured(game_root: &ClausewitzValue) -> Option<String> {
    let mut types: Vec<String> = Vec::new();

    for block_name in &["vivarium_critters", "exhibits"] {
        if let Some(ClausewitzValue::Block(nodes)) = query_path(game_root, &[block_name]) {
            for node in nodes {
                if let ClausewitzNode::Pair(_, entry) = node {
                    if let Some(t) =
                        query_atom(entry, &["type"]).or_else(|| query_atom(entry, &["species"]))
                    {
                        if !types.contains(&t) {
                            types.push(t);
                        }
                    }
                }
            }
        }
    }

    if types.is_empty() {
        None
    } else {
        Some(types.join(", "))
    }
}

// ---------------------------------------------------------------------------
// First Contact Result
// ---------------------------------------------------------------------------

/// Collect unique first contact results from `first_contacts > contacts > *`.
fn extract_first_contact_result(game_root: &ClausewitzValue) -> Option<String> {
    let contacts = query_path(game_root, &["first_contacts", "contacts"])?;
    let mut results: Vec<String> = Vec::new();
    match contacts {
        ClausewitzValue::Block(nodes) => {
            for node in nodes {
                if let ClausewitzNode::Pair(_, contact_block) = node {
                    let result = query_atom(contact_block, &["result"])
                        .or_else(|| query_atom(contact_block, &["outcome"]))
                        .or_else(|| query_atom(contact_block, &["type"]));
                    if let Some(r) = result {
                        if !results.contains(&r) {
                            results.push(r);
                        }
                    }
                }
            }
        }
        ClausewitzValue::Atom(_) => {}
    }
    if results.is_empty() {
        None
    } else {
        Some(results.join(", "))
    }
}

// ---------------------------------------------------------------------------
// Federation helpers
// ---------------------------------------------------------------------------

/// Look up the federation entry from the save root using the federation ID
/// stored in the country's `federation` field.
fn get_federation_entry<'a>(
    game_root: &'a ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<&'a ClausewitzValue> {
    let fed_id = query_atom(country_value, &["federation"])?;
    query_path(game_root, &["federation", &fed_id])
}

/// Extract the federation type from the federation object.
fn extract_federation_type(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<String> {
    let fed = get_federation_entry(game_root, country_value)?;
    query_atom(fed, &["type"])
}

/// Extract the federation level from the federation object.
fn extract_federation_level(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<usize> {
    let fed = get_federation_entry(game_root, country_value)?;
    query_f64(fed, &["level"])
        .or_else(|| query_f64(fed, &["federation_level"]))
        .or_else(|| query_f64(fed, &["centralization_level"]))
        .map(|v| v as usize)
}

/// Collect unique ethics from all federation members (excluding the player).
fn extract_federation_member_ethics(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
    player_country_id: &str,
) -> Vec<String> {
    let fed = match get_federation_entry(game_root, country_value) {
        Some(f) => f,
        None => return vec![],
    };

    let members = match query_path(fed, &["members"]) {
        Some(ClausewitzValue::Block(nodes)) => nodes,
        _ => return vec![],
    };

    let mut ethics: Vec<String> = Vec::new();
    for node in members {
        let member_id = match node {
            ClausewitzNode::Pair(_, value) => {
                query_atom(value, &["country"]).or_else(|| match value {
                    ClausewitzValue::Atom(s) if !s.is_empty() => Some(s.clone()),
                    _ => None,
                })
            }
            ClausewitzNode::Value(ClausewitzValue::Block(fields)) => {
                // Value blocks like { country = <id> ... }
                fields.iter().find_map(|f| {
                    if let ClausewitzNode::Pair(k, ClausewitzValue::Atom(v)) = f {
                        if k == "country" {
                            return Some(v.clone());
                        }
                    }
                    None
                })
            }
            ClausewitzNode::Value(ClausewitzValue::Atom(id)) => Some(id.clone()),
        };

        if let Some(id) = member_id {
            if id == player_country_id {
                continue;
            }
            if let Some(member_country) = query_path(game_root, &["country", &id]) {
                for ethic in query_atoms(member_country, &["ethics"]) {
                    if !ethics.contains(&ethic) {
                        ethics.push(ethic);
                    }
                }
            }
        }
    }
    ethics
}

// ---------------------------------------------------------------------------
// Astral Rifts Explored
// ---------------------------------------------------------------------------

/// Count entries in the top-level `astral_rifts` block where `explored = yes`.
fn count_astral_rifts_explored(game_root: &ClausewitzValue) -> Option<usize> {
    let rifts = query_path(game_root, &["astral_rifts"])?;
    match rifts {
        ClausewitzValue::Block(nodes) => {
            let count = nodes
                .iter()
                .filter(|node| {
                    if let ClausewitzNode::Pair(_, entry) = node {
                        query_atom(entry, &["explored"])
                            .map(|v| v == "yes")
                            .unwrap_or(false)
                    } else {
                        false
                    }
                })
                .count();
            if count > 0 {
                Some(count)
            } else {
                None
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Espionage Operations Completed
// ---------------------------------------------------------------------------

/// Count country flags whose key contains "espionage_operation_completed".
fn count_espionage_operations_completed(country_value: &ClausewitzValue) -> Option<usize> {
    let flags = query_path(country_value, &["flags"])?;
    match flags {
        ClausewitzValue::Block(nodes) => {
            let count = nodes
                .iter()
                .filter(|node| {
                    if let ClausewitzNode::Pair(key, _) = node {
                        key.contains("espionage_operation_completed")
                    } else {
                        false
                    }
                })
                .count();
            Some(count)
        }
        ClausewitzValue::Atom(_) => Some(0),
    }
}

// ---------------------------------------------------------------------------
// Migration Treaty Count
// ---------------------------------------------------------------------------

/// Count migration-type agreements in the country's agreements block.
fn count_migration_treaties(country_value: &ClausewitzValue) -> Option<usize> {
    let agreements = query_path(country_value, &["agreements"])?;
    match agreements {
        ClausewitzValue::Block(nodes) => {
            let count = nodes
                .iter()
                .filter(|node| {
                    if let ClausewitzNode::Pair(_, entry) = node {
                        query_atom(entry, &["type"])
                            .map(|t| t.contains("migration"))
                            .unwrap_or(false)
                    } else {
                        false
                    }
                })
                .count();
            Some(count)
        }
        _ => Some(0),
    }
}

// ---------------------------------------------------------------------------
// Proxy War Count
// ---------------------------------------------------------------------------

/// Count proxy-war-related flags in the country's flags block.
fn count_proxy_wars(country_value: &ClausewitzValue) -> Option<usize> {
    let flags = query_path(country_value, &["flags"])?;
    match flags {
        ClausewitzValue::Block(nodes) => {
            let count = nodes
                .iter()
                .filter(|node| {
                    if let ClausewitzNode::Pair(key, _) = node {
                        key.contains("proxy_war")
                    } else {
                        false
                    }
                })
                .count();
            Some(count)
        }
        ClausewitzValue::Atom(_) => Some(0),
    }
}

// ---------------------------------------------------------------------------
// Livestock Species Count
// ---------------------------------------------------------------------------

/// Count distinct species in pop_groups with `key > category = "livestock"`.
fn count_livestock_species(game_root: &ClausewitzValue) -> Option<usize> {
    count_distinct_species_in_category(game_root, "livestock")
}

// ---------------------------------------------------------------------------
// Relic Active Effect Used
// ---------------------------------------------------------------------------

/// Check if any relic activation flag exists in country flags.
/// Returns `Some("true")` if found, `None` otherwise (bridges bool→String).
fn extract_relic_active_effect_used(country_value: &ClausewitzValue) -> Option<String> {
    let flags = query_path(country_value, &["flags"])?;
    let has_relic_flag = match flags {
        ClausewitzValue::Block(nodes) => nodes.iter().any(|node| {
            if let ClausewitzNode::Pair(key, _) = node {
                key == "relic_active_effect_used" || key.contains("relic_active")
            } else {
                false
            }
        }),
        ClausewitzValue::Atom(_) => false,
    };
    if has_relic_flag {
        Some("true".to_string())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Species On Planet Count (max distinct species on any owned planet)
// ---------------------------------------------------------------------------

/// Find the maximum number of distinct species on any single owned planet.
fn max_species_on_planet(game_root: &ClausewitzValue, player_country_id: &str) -> Option<usize> {
    let mut max_count = 0usize;
    let mut any_planet = false;

    for_each_owned_planet(game_root, player_country_id, |planet_value| {
        any_planet = true;
        let mut species_set = std::collections::HashSet::new();

        // Check pop_groups on the planet
        if let Some(ClausewitzValue::Block(nodes)) = query_path(planet_value, &["pop_groups"]) {
            for node in nodes {
                if let ClausewitzNode::Pair(_, entry) = node {
                    if let Some(species_ref) = query_atom(entry, &["key", "species"]) {
                        species_set.insert(species_ref);
                    }
                }
            }
        }

        max_count = max_count.max(species_set.len());
    });

    if any_planet {
        Some(max_count)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Planets Terraformed To Volcanic
// ---------------------------------------------------------------------------

/// Count owned planets with `planet_class = pc_volcanic`.
fn count_volcanic_planets(game_root: &ClausewitzValue, player_country_id: &str) -> Option<usize> {
    let mut count = 0usize;
    let mut any_planet = false;

    for_each_owned_planet(game_root, player_country_id, |planet_value| {
        any_planet = true;
        if query_atom(planet_value, &["planet_class"]).as_deref() == Some("pc_volcanic") {
            count += 1;
        }
    });

    if any_planet {
        Some(count)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Galactic Memorials On Tomb Worlds
// ---------------------------------------------------------------------------

/// Count owned tomb worlds with a galactic memorial building.
fn count_galactic_memorials_on_tomb_worlds(
    game_root: &ClausewitzValue,
    player_country_id: &str,
) -> Option<usize> {
    let mut count = 0usize;
    let mut any_planet = false;

    for_each_owned_planet(game_root, player_country_id, |planet_value| {
        any_planet = true;
        if query_atom(planet_value, &["planet_class"]).as_deref() == Some("pc_tomb") {
            let has_memorial = query_atoms(planet_value, &["buildings_cache"])
                .iter()
                .any(|b| b.contains("galactic_memorial"));
            if has_memorial {
                count += 1;
            }
        }
    });

    if any_planet {
        Some(count)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Colony Count With Hyperspace Not Researched
// ---------------------------------------------------------------------------

/// If hyperdrive tech is not researched, return owned_planets count.
/// If researched, return Some(0). If tech_status is missing, return None.
fn colony_count_without_hyperspace(country_value: &ClausewitzValue) -> Option<usize> {
    let tech_status = query_path(country_value, &["tech_status"])?;
    match tech_status {
        ClausewitzValue::Block(nodes) => {
            let has_hyperspace = nodes.iter().any(|node| {
                matches!(node, ClausewitzNode::Pair(key, ClausewitzValue::Atom(val))
                    if key == "technology" && val == "tech_hyperspace")
            });
            if has_hyperspace {
                Some(0)
            } else {
                query_count(country_value, &["owned_planets"])
            }
        }
        _ => None,
    }
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
        subject_type: extract_subject_type(country_value),
        subject_contract_modified: Some(has_flag(country_value, "subject_contract_modified")),
        secret_fealty_pledged: Some(has_any_flag(
            country_value,
            &["secret_fealty_pledged", "secret_fealty", "fealty_pledged"],
        )),
        proxy_war_count: count_proxy_wars(country_value),

        // -- Federation --
        federation_formed: extract_federation_formed(country_value),
        federation_type: extract_federation_type(game_root, country_value),
        federation_level: extract_federation_level(game_root, country_value),
        federation_member_ethics: extract_federation_member_ethics(
            game_root,
            country_value,
            player_country_id,
        ),

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
        colossus_weapon_type: extract_colossus_weapon_type(country_value),
        colossus_destroyed_while_firing: Some(has_flag(country_value, "colossus_destroyed")),

        // -- Species Actions --
        species_genetically_modified: Some(has_flag(country_value, "genetically_modified")),
        species_uplifted: Some(has_flag(country_value, "species_uplifted")),
        species_on_planet_count: max_species_on_planet(game_root, player_country_id),
        species_dna_phenotypes_collected: None, // Requires species_db analysis -- complex
        slavery_type: query_atom(country_value, &["flags", "slavery_type"]),
        livestock_species_count: count_livestock_species(game_root),
        purged_pops: count_purged_pops(game_root),
        species_enslaved: Some(has_flag(country_value, "species_enslaved")),

        // -- Relics --
        relic_owned: extract_relics_owned(country_value),
        relic_active_effect_used: extract_relic_active_effect_used(country_value),
        galatron_acquired: extract_galatron(country_value),
        galatron_captured: Some(has_flag(country_value, "galatron_captured")),

        // -- Archaeology / Exploration --
        archaeological_site_completed: extract_archaeology_sites(game_root, player_country_id),
        wormhole_travel_completed: Some(has_flag(country_value, "wormhole_travel_completed")),
        pre_ftl_infiltration_completed: Some(has_flag(country_value, "pre_ftl_infiltrated")),
        first_contact_result: extract_first_contact_result(game_root),
        espionage_operations_completed: count_espionage_operations_completed(country_value),
        astral_rifts_explored: count_astral_rifts_explored(game_root),

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
        enclave_interaction_type: extract_enclave_interaction_type(country_value),
        migration_treaty_count: count_migration_treaties(country_value),
        legendary_paragon_recruited: Some(has_flag(country_value, "legendary_paragon_recruited")),

        // -- Misc Events --
        robot_pop_built: Some(has_flag(country_value, "robot_pop_built")),
        horizon_signal_completed: check_horizon_signal(country_value),
        civil_war_completed: Some(has_flag(country_value, "civil_war_completed")),
        special_project_completed_type: extract_special_project_completed_type(country_value),
        covenant_type: extract_covenant(country_value),
        psionic_techs_unlocked: Some(
            field_contains(country_value, "ascension_perks", "ap_mind_over_matter")
                || has_flag(country_value, "psionic_theory_researched"),
        ),
        quantum_catapult_used: Some(has_flag(country_value, "quantum_catapult_used")),

        // -- Terraforming / Decisions --
        blazing_scourge_decisions: None, // Complex -- requires decision tracking
        stars_terraform_to_red_giant: None, // Complex -- requires star type tracking
        planets_terraform_to_volcanic: count_volcanic_planets(game_root, player_country_id),
        volcanic_holy_world_created: Some(has_flag(country_value, "volcanic_holy_world")),
        galactic_memorials_on_tomb_worlds: count_galactic_memorials_on_tomb_worlds(
            game_root,
            player_country_id,
        ),
        space_fauna_type_captured: extract_space_fauna_type_captured(game_root),
        colony_count_with_hyperspace_not_researched: colony_count_without_hyperspace(country_value),

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
        assert!(facts.vassal_count.is_none());
        assert!(facts.subject_type.is_none());
        assert_eq!(facts.proxy_war_count, Some(0));
        assert!(facts.federation_formed == Some(false));
        assert!(facts.federation_type.is_none());
        assert!(facts.galactic_community_exists.is_none());
        assert!(facts.colossus_built == Some(false));
        assert!(facts.colossus_weapon_type.is_none());
        assert!(facts.species_on_planet_count.is_none());
        assert!(facts.livestock_species_count.is_none());
        assert!(facts.purged_pops.is_none());
        assert!(facts.relic_owned.is_none());
        assert!(facts.relic_active_effect_used.is_none());
        assert!(facts.galatron_acquired == Some(false));
        assert!(facts.archaeological_site_completed.is_none());
        assert!(facts.first_contact_result.is_none());
        assert_eq!(facts.espionage_operations_completed, Some(0));
        assert!(facts.astral_rifts_explored.is_none());
        assert!(facts.horizon_signal_completed == Some(false));
        assert!(facts.enclave_interaction_type.is_none());
        assert!(facts.migration_treaty_count.is_none());
        assert!(facts.special_project_completed_type.is_none());
        assert!(facts.covenant_type.is_none());
        assert!(facts.planets_terraform_to_volcanic.is_none());
        assert!(facts.galactic_memorials_on_tomb_worlds.is_none());
        assert!(facts.space_fauna_type_captured.is_none());
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

    // -----------------------------------------------------------------------
    // Test: Subject type extraction
    // -----------------------------------------------------------------------

    #[test]
    fn test_subject_type_extraction() {
        let subject1 = make_block(vec![("type", make_atom("vassal"))]);
        let subject2 = make_block(vec![("type", make_atom("bulwark"))]);
        let subject3 = make_block(vec![("type", make_atom("vassal"))]);

        let country = make_block(vec![
            (
                "subjects",
                make_block(vec![
                    ("subj_1", subject1),
                    ("subj_2", subject2),
                    ("subj_3", subject3),
                ]),
            ),
            ("flags", make_block(vec![])),
        ]);

        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        let st = facts.subject_type.unwrap();
        assert!(st.contains("vassal"), "expected vassal in '{st}'");
        assert!(st.contains("bulwark"), "expected bulwark in '{st}'");
    }

    #[test]
    fn test_subject_type_missing() {
        let country = make_country(vec![]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert!(facts.subject_type.is_none());
    }

    // -----------------------------------------------------------------------
    // Test: Colossus weapon type
    // -----------------------------------------------------------------------

    #[test]
    fn test_colossus_weapon_type() {
        let country = make_country(vec![("colossus_weapon_world_cracker", "yes")]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(
            facts.colossus_weapon_type,
            Some("world_cracker".to_string())
        );
    }

    #[test]
    fn test_colossus_weapon_type_none() {
        let country = make_country(vec![]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert!(facts.colossus_weapon_type.is_none());
    }

    // -----------------------------------------------------------------------
    // Test: Enclave interaction type
    // -----------------------------------------------------------------------

    #[test]
    fn test_enclave_interaction_type() {
        let country = make_country(vec![
            ("recruit_curator_scientist", "yes"),
            ("traded_with_artisans", "yes"),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        let ei = facts.enclave_interaction_type.unwrap();
        assert!(ei.contains("recruit_curator_scientist"));
        assert!(ei.contains("traded_with_artisans"));
    }

    #[test]
    fn test_enclave_interaction_type_none() {
        let country = make_country(vec![]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert!(facts.enclave_interaction_type.is_none());
    }

    // -----------------------------------------------------------------------
    // Test: Special project completed type
    // -----------------------------------------------------------------------

    #[test]
    fn test_special_project_completed_type_via_flags() {
        let country = make_country(vec![("breach_the_shroud", "yes")]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        let sp = facts.special_project_completed_type.unwrap();
        assert!(sp.contains("breach_the_shroud"));
    }

    #[test]
    fn test_special_project_completed_type_via_block() {
        // Projects can also be stored in a special_projects block
        let country = make_block(vec![
            (
                "special_projects",
                make_block(vec![("limbo", make_atom("yes"))]),
            ),
            ("flags", make_block(vec![])),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        let sp = facts.special_project_completed_type.unwrap();
        assert!(sp.contains("limbo"));
    }

    #[test]
    fn test_special_project_completed_type_none() {
        let country = make_country(vec![]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert!(facts.special_project_completed_type.is_none());
    }

    // -----------------------------------------------------------------------
    // Test: Purged pops
    // -----------------------------------------------------------------------

    #[test]
    fn test_purged_pops_count() {
        let game = make_block(vec![
            ("date", make_atom("2205.01.01")),
            (
                "pop_groups",
                make_block(vec![
                    (
                        "0",
                        make_block(vec![
                            (
                                "key",
                                make_block(vec![
                                    ("species", make_atom("0")),
                                    ("category", make_atom("purge")),
                                ]),
                            ),
                            ("size", make_atom("5")),
                        ]),
                    ),
                    (
                        "1",
                        make_block(vec![
                            (
                                "key",
                                make_block(vec![
                                    ("species", make_atom("1")),
                                    ("category", make_atom("normal")),
                                ]),
                            ),
                            ("size", make_atom("10")),
                        ]),
                    ),
                    (
                        "2",
                        make_block(vec![
                            (
                                "key",
                                make_block(vec![
                                    ("species", make_atom("2")),
                                    ("category", make_atom("purge")),
                                ]),
                            ),
                            ("size", make_atom("3")),
                        ]),
                    ),
                ]),
            ),
        ]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        assert_eq!(facts.purged_pops, Some(8)); // 5 + 3
    }

    #[test]
    fn test_purged_pops_missing_block() {
        let game = make_game_root(vec![]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        assert!(facts.purged_pops.is_none());
    }

    #[test]
    fn test_purged_pops_none() {
        let game = make_block(vec![
            ("date", make_atom("2205.01.01")),
            (
                "pop_groups",
                make_block(vec![(
                    "0",
                    make_block(vec![
                        (
                            "key",
                            make_block(vec![
                                ("species", make_atom("0")),
                                ("category", make_atom("normal")),
                            ]),
                        ),
                        ("size", make_atom("10")),
                    ]),
                )]),
            ),
        ]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        assert_eq!(facts.purged_pops, Some(0));
    }

    // -----------------------------------------------------------------------
    // Test: Space fauna type captured
    // -----------------------------------------------------------------------

    #[test]
    fn test_space_fauna_type_captured() {
        let game = make_block(vec![
            ("date", make_atom("2205.01.01")),
            (
                "vivarium_critters",
                make_block(vec![(
                    "0",
                    make_block(vec![("type", make_atom("space_dragon"))]),
                )]),
            ),
            (
                "exhibits",
                make_block(vec![(
                    "0",
                    make_block(vec![("type", make_atom("cutholoid"))]),
                )]),
            ),
        ]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        let sft = facts.space_fauna_type_captured.unwrap();
        assert!(
            sft.contains("space_dragon"),
            "expected space_dragon in '{sft}'"
        );
        assert!(sft.contains("cutholoid"), "expected cutholoid in '{sft}'");
    }

    #[test]
    fn test_space_fauna_type_captured_none() {
        let game = make_game_root(vec![]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        assert!(facts.space_fauna_type_captured.is_none());
    }

    // -----------------------------------------------------------------------
    // Test: First contact result
    // -----------------------------------------------------------------------

    #[test]
    fn test_first_contact_result() {
        let contact1 = make_block(vec![
            ("type", make_atom("pre_ftl_civilization")),
            ("result", make_atom("peaceful")),
        ]);
        let contact2 = make_block(vec![
            ("type", make_atom("pre_ftl_civilization")),
            ("result", make_atom("aggressive")),
        ]);
        let contacts = make_block(vec![("c1", contact1), ("c2", contact2)]);
        let fcs = make_block(vec![("contacts", contacts)]);
        let game = make_block(vec![("first_contacts", fcs)]);

        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        let fcr = facts.first_contact_result.unwrap();
        assert!(fcr.contains("peaceful"), "expected peaceful in '{fcr}'");
        assert!(fcr.contains("aggressive"), "expected aggressive in '{fcr}'");
    }

    #[test]
    fn test_first_contact_result_none() {
        let game = make_game_root(vec![]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        assert!(facts.first_contact_result.is_none());
    }

    // -----------------------------------------------------------------------
    // Test: Federation type, level, and member ethics
    // -----------------------------------------------------------------------

    #[test]
    fn test_federation_type_level_and_ethics() {
        // Federation entry at top level
        let federation_entry = make_block(vec![
            ("type", make_atom("galactic_union")),
            ("level", make_atom("3")),
            (
                "members",
                make_value_block(vec![
                    // Member 1: another organic empire
                    make_block(vec![("country", make_atom("1"))]),
                    // Member 2: another empire
                    make_block(vec![("country", make_atom("2"))]),
                ]),
            ),
        ]);

        let game = make_block(vec![
            ("date", make_atom("2205.01.01")),
            (
                "federation",
                make_block(vec![("16777220", federation_entry)]),
            ),
            (
                "country",
                make_block(vec![
                    // Player — should be skipped
                    (
                        "0",
                        make_block(vec![
                            ("federation", make_atom("16777220")),
                            ("flags", make_block(vec![])),
                            (
                                "ethics",
                                make_value_block(vec![make_atom("ethic_egalitarian")]),
                            ),
                        ]),
                    ),
                    // Member 1
                    (
                        "1",
                        make_block(vec![
                            ("flags", make_block(vec![])),
                            (
                                "ethics",
                                make_value_block(vec![
                                    make_atom("ethic_militarist"),
                                    make_atom("ethic_spiritualist"),
                                ]),
                            ),
                        ]),
                    ),
                    // Member 2
                    (
                        "2",
                        make_block(vec![
                            ("flags", make_block(vec![])),
                            (
                                "ethics",
                                make_value_block(vec![make_atom("ethic_militarist")]),
                            ),
                        ]),
                    ),
                ]),
            ),
        ]);
        let country = make_block(vec![
            ("federation", make_atom("16777220")),
            ("flags", make_block(vec![])),
        ]);

        let facts = extract_action_facts(&game, &country, "0");

        assert_eq!(facts.federation_type, Some("galactic_union".to_string()));
        assert_eq!(facts.federation_level, Some(3));
        // Should have unique ethics from members excluding player
        assert!(facts
            .federation_member_ethics
            .contains(&"ethic_militarist".to_string()));
        assert!(facts
            .federation_member_ethics
            .contains(&"ethic_spiritualist".to_string()));
        assert_eq!(facts.federation_member_ethics.len(), 2);
    }

    #[test]
    fn test_federation_type_level_none() {
        let game = make_game_root(vec![]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        assert!(facts.federation_type.is_none());
        assert!(facts.federation_level.is_none());
        assert!(facts.federation_member_ethics.is_empty());
    }

    // -----------------------------------------------------------------------
    // Test: Astral rifts explored
    // -----------------------------------------------------------------------

    #[test]
    fn test_astral_rifts_explored() {
        let game = make_block(vec![
            ("date", make_atom("2205.01.01")),
            (
                "astral_rifts",
                make_block(vec![
                    ("0", make_block(vec![("explored", make_atom("yes"))])),
                    ("1", make_block(vec![("explored", make_atom("yes"))])),
                ]),
            ),
        ]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        assert_eq!(facts.astral_rifts_explored, Some(2));
    }

    #[test]
    fn test_astral_rifts_none() {
        let game = make_game_root(vec![]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        assert!(facts.astral_rifts_explored.is_none());
    }

    // -----------------------------------------------------------------------
    // Test: Espionage operations completed
    // -----------------------------------------------------------------------

    #[test]
    fn test_espionage_operations_completed() {
        let country = make_country(vec![
            ("espionage_operation_completed_01", "yes"),
            ("espionage_operation_completed_02", "yes"),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.espionage_operations_completed, Some(2));
    }

    #[test]
    fn test_espionage_operations_none() {
        let country = make_country(vec![]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.espionage_operations_completed, Some(0));
    }

    // -----------------------------------------------------------------------
    // Test: Migration treaty count
    // -----------------------------------------------------------------------

    #[test]
    fn test_migration_treaty_count() {
        let agreement1 = make_block(vec![("type", make_atom("migration_treaty"))]);
        let agreement2 = make_block(vec![("type", make_atom("migration_treaty"))]);
        let country = make_block(vec![
            (
                "agreements",
                make_block(vec![("a1", agreement1), ("a2", agreement2)]),
            ),
            ("flags", make_block(vec![])),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.migration_treaty_count, Some(2));
    }

    #[test]
    fn test_migration_treaty_count_none() {
        let country = make_country(vec![]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert!(facts.migration_treaty_count.is_none());
    }

    // -----------------------------------------------------------------------
    // Test: Proxy war count
    // -----------------------------------------------------------------------

    #[test]
    fn test_proxy_war_count() {
        let country = make_country(vec![
            ("proxy_war_participated", "yes"),
            ("proxy_war_won", "yes"),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.proxy_war_count, Some(2));
    }

    #[test]
    fn test_proxy_war_count_none() {
        let country = make_country(vec![]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.proxy_war_count, Some(0));
    }

    // -----------------------------------------------------------------------
    // Test: Livestock species count
    // -----------------------------------------------------------------------

    #[test]
    fn test_livestock_species_count() {
        let game = make_block(vec![
            ("date", make_atom("2205.01.01")),
            (
                "pop_groups",
                make_block(vec![
                    (
                        "0",
                        make_block(vec![
                            (
                                "key",
                                make_block(vec![
                                    ("species", make_atom("0")),
                                    ("category", make_atom("livestock")),
                                ]),
                            ),
                            ("size", make_atom("5")),
                        ]),
                    ),
                    (
                        "1",
                        make_block(vec![
                            (
                                "key",
                                make_block(vec![
                                    ("species", make_atom("1")),
                                    ("category", make_atom("livestock")),
                                ]),
                            ),
                            ("size", make_atom("3")),
                        ]),
                    ),
                    (
                        "2",
                        make_block(vec![
                            (
                                "key",
                                make_block(vec![
                                    ("species", make_atom("0")),
                                    ("category", make_atom("livestock")),
                                ]),
                            ),
                            ("size", make_atom("2")),
                        ]),
                    ),
                ]),
            ),
        ]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        assert_eq!(facts.livestock_species_count, Some(2)); // species 0 and 1
    }

    #[test]
    fn test_livestock_species_count_none() {
        let game = make_block(vec![
            ("date", make_atom("2205.01.01")),
            (
                "pop_groups",
                make_block(vec![(
                    "0",
                    make_block(vec![
                        (
                            "key",
                            make_block(vec![
                                ("species", make_atom("0")),
                                ("category", make_atom("normal")),
                            ]),
                        ),
                        ("size", make_atom("5")),
                    ]),
                )]),
            ),
        ]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        assert_eq!(facts.livestock_species_count, Some(0));
    }

    // -----------------------------------------------------------------------
    // Test: Relic active effect used
    // -----------------------------------------------------------------------

    #[test]
    fn test_relic_active_effect_used() {
        let country = make_country(vec![("relic_active_effect_used", "yes")]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.relic_active_effect_used, Some("true".to_string()));
    }

    #[test]
    fn test_relic_active_effect_used_none() {
        let country = make_country(vec![]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert!(facts.relic_active_effect_used.is_none());
    }

    // -----------------------------------------------------------------------
    // Test: Species on planet count (max distinct on any owned planet)
    // -----------------------------------------------------------------------

    #[test]
    fn test_species_on_planet_count() {
        // Planet 1 (owned): species 0, 0, 1 => 2 distinct
        // Planet 2 (owned): species 0, 1, 2 => 3 distinct (max)
        // Planet 3 (not owned): skip
        let planet1 = make_block(vec![
            ("controller", make_atom("0")),
            (
                "pop_groups",
                make_block(vec![
                    (
                        "p1",
                        make_block(vec![(
                            "key",
                            make_block(vec![
                                ("species", make_atom("0")),
                                ("category", make_atom("normal")),
                            ]),
                        )]),
                    ),
                    (
                        "p2",
                        make_block(vec![(
                            "key",
                            make_block(vec![
                                ("species", make_atom("0")),
                                ("category", make_atom("normal")),
                            ]),
                        )]),
                    ),
                    (
                        "p3",
                        make_block(vec![(
                            "key",
                            make_block(vec![
                                ("species", make_atom("1")),
                                ("category", make_atom("normal")),
                            ]),
                        )]),
                    ),
                ]),
            ),
        ]);
        let planet2 = make_block(vec![
            ("controller", make_atom("0")),
            (
                "pop_groups",
                make_block(vec![
                    (
                        "p1",
                        make_block(vec![(
                            "key",
                            make_block(vec![
                                ("species", make_atom("0")),
                                ("category", make_atom("normal")),
                            ]),
                        )]),
                    ),
                    (
                        "p2",
                        make_block(vec![(
                            "key",
                            make_block(vec![
                                ("species", make_atom("1")),
                                ("category", make_atom("normal")),
                            ]),
                        )]),
                    ),
                    (
                        "p3",
                        make_block(vec![(
                            "key",
                            make_block(vec![
                                ("species", make_atom("2")),
                                ("category", make_atom("normal")),
                            ]),
                        )]),
                    ),
                ]),
            ),
        ]);
        let planet3 = make_block(vec![
            ("controller", make_atom("1")), // not owned
            (
                "pop_groups",
                make_block(vec![(
                    "p1",
                    make_block(vec![(
                        "key",
                        make_block(vec![
                            ("species", make_atom("99")),
                            ("category", make_atom("normal")),
                        ]),
                    )]),
                )]),
            ),
        ]);

        let game = make_block(vec![
            ("date", make_atom("2205.01.01")),
            (
                "planets",
                make_block(vec![(
                    "planet",
                    make_block(vec![("1", planet1), ("2", planet2), ("3", planet3)]),
                )]),
            ),
        ]);
        let country = make_block(vec![
            (
                "owned_planets",
                make_value_block(vec![make_atom("1"), make_atom("2")]),
            ),
            ("flags", make_block(vec![])),
        ]);

        let facts = extract_action_facts(&game, &country, "0");
        assert_eq!(facts.species_on_planet_count, Some(3)); // max is planet 2 with 3 species
    }

    #[test]
    fn test_species_on_planet_count_no_planets() {
        let game = make_block(vec![("date", make_atom("2205.01.01"))]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        assert!(facts.species_on_planet_count.is_none());
    }

    // -----------------------------------------------------------------------
    // Test: Planets terraformed to volcanic (count owned volcanic planets)
    // -----------------------------------------------------------------------

    #[test]
    fn test_planets_terraform_to_volcanic() {
        let planet1 = make_block(vec![
            ("controller", make_atom("0")),
            ("planet_class", make_atom("pc_continental")),
        ]);
        let planet2 = make_block(vec![
            ("controller", make_atom("0")),
            ("planet_class", make_atom("pc_volcanic")),
        ]);
        let planet3 = make_block(vec![
            ("controller", make_atom("0")),
            ("planet_class", make_atom("pc_volcanic")),
        ]);
        let planet4 = make_block(vec![
            ("controller", make_atom("1")), // not owned
            ("planet_class", make_atom("pc_volcanic")),
        ]);

        let game = make_block(vec![
            ("date", make_atom("2205.01.01")),
            (
                "planets",
                make_block(vec![(
                    "planet",
                    make_block(vec![
                        ("1", planet1),
                        ("2", planet2),
                        ("3", planet3),
                        ("4", planet4),
                    ]),
                )]),
            ),
        ]);
        let country = make_block(vec![
            (
                "owned_planets",
                make_value_block(vec![make_atom("1"), make_atom("2"), make_atom("3")]),
            ),
            ("flags", make_block(vec![])),
        ]);

        let facts = extract_action_facts(&game, &country, "0");
        assert_eq!(facts.planets_terraform_to_volcanic, Some(2)); // planets 2 and 3
    }

    #[test]
    fn test_planets_terraform_to_volcanic_none() {
        let game = make_block(vec![("date", make_atom("2205.01.01"))]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        assert!(facts.planets_terraform_to_volcanic.is_none());
    }

    // -----------------------------------------------------------------------
    // Test: Galactic memorials on tomb worlds
    // -----------------------------------------------------------------------

    #[test]
    fn test_galactic_memorials_on_tomb_worlds() {
        // Planet 1: tomb world with galactic memorial
        // Planet 2: tomb world without memorial
        // Planet 3: continental with memorial (shouldn't count)
        let planet1 = make_block(vec![
            ("controller", make_atom("0")),
            ("planet_class", make_atom("pc_tomb")),
            (
                "buildings_cache",
                make_value_block(vec![make_atom("bld_galactic_memorial_01")]),
            ),
        ]);
        let planet2 = make_block(vec![
            ("controller", make_atom("0")),
            ("planet_class", make_atom("pc_tomb")),
            ("buildings_cache", make_value_block(vec![])),
        ]);
        let planet3 = make_block(vec![
            ("controller", make_atom("0")),
            ("planet_class", make_atom("pc_continental")),
            (
                "buildings_cache",
                make_value_block(vec![make_atom("bld_galactic_memorial_01")]),
            ),
        ]);

        let game = make_block(vec![
            ("date", make_atom("2205.01.01")),
            (
                "planets",
                make_block(vec![(
                    "planet",
                    make_block(vec![("1", planet1), ("2", planet2), ("3", planet3)]),
                )]),
            ),
        ]);
        let country = make_country(vec![]);

        let facts = extract_action_facts(&game, &country, "0");
        assert_eq!(facts.galactic_memorials_on_tomb_worlds, Some(1)); // only planet 1
    }

    #[test]
    fn test_galactic_memorials_on_tomb_worlds_none() {
        let game = make_block(vec![("date", make_atom("2205.01.01"))]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        assert!(facts.galactic_memorials_on_tomb_worlds.is_none());
    }

    // -----------------------------------------------------------------------
    // Test: Colony count with hyperspace not researched
    // -----------------------------------------------------------------------

    #[test]
    fn test_colony_count_with_hyperspace_researched() {
        let country = make_block(vec![
            (
                "tech_status",
                make_block(vec![
                    ("technology", make_atom("tech_hyperspace")),
                    ("level", make_atom("1")),
                ]),
            ),
            (
                "owned_planets",
                make_value_block(vec![make_atom("1"), make_atom("2")]),
            ),
            ("flags", make_block(vec![])),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.colony_count_with_hyperspace_not_researched, Some(0));
    }

    #[test]
    fn test_colony_count_without_hyperspace() {
        let country = make_block(vec![
            (
                "tech_status",
                make_block(vec![
                    ("technology", make_atom("tech_physics_1")),
                    ("level", make_atom("1")),
                ]),
            ),
            (
                "owned_planets",
                make_value_block(vec![make_atom("1"), make_atom("2"), make_atom("3")]),
            ),
            ("flags", make_block(vec![])),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.colony_count_with_hyperspace_not_researched, Some(3));
    }

    #[test]
    fn test_colony_count_without_tech_status() {
        let country = make_country(vec![]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert!(facts.colony_count_with_hyperspace_not_researched.is_none());
    }

    // -----------------------------------------------------------------------
    // Adversarial tests: mixed-state fixtures to validate filtering
    // -----------------------------------------------------------------------

    #[test]
    fn test_astral_rifts_filters_unexplored() {
        let game = make_block(vec![
            ("date", make_atom("2205.01.01")),
            (
                "astral_rifts",
                make_block(vec![
                    ("0", make_block(vec![("explored", make_atom("yes"))])),
                    ("1", make_block(vec![("explored", make_atom("no"))])),
                    ("2", make_block(vec![("explored", make_atom("yes"))])),
                    ("3", make_block(vec![("other_field", make_atom("value"))])),
                ]),
            ),
        ]);
        let country = make_country(vec![]);
        let facts = extract_action_facts(&game, &country, "0");
        assert_eq!(facts.astral_rifts_explored, Some(2));
    }

    #[test]
    fn test_espionage_ignores_non_operation_flags() {
        let country = make_country(vec![
            ("espionage_operation_completed_01", "yes"),
            ("espionage_network_established", "yes"),
            ("espionage_asset_recruited", "yes"),
            ("espionage_operation_completed_02", "yes"),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.espionage_operations_completed, Some(2));
    }
}
