//! Extract Progression facts from a parsed Stellaris gamestate AST.
//!
//! This module receives the full gamestate root (`game_root`) and the
//! player's country block (`country_value`) and extracts all progression-
//! relevant dimensions using the query helpers from [`crate::save`].
//!
//! # Extraction Strategy
//!
//! Many dimensions come directly from country fields (`military_power`,
//! `num_sapient_pops`). Block-based dimensions (traditions, owned_planets)
//! use `query_count` or `query_atoms`. Cross-referencing with `game_root`
//! is needed for date parsing, megastructure identification, and survey
//! data.

use std::collections::HashSet;

use crate::model::ProgressionFacts;
use crate::save::*;

/// Extract all progression facts from a parsed Stellaris gamestate.
///
/// # Arguments
///
/// * `game_root` — The root of the parsed gamestate AST.
/// * `country_value` — The player country block (looked up from `country` by player ID).
pub(crate) fn extract_progression_facts(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> ProgressionFacts {
    let player_country_id = query_atom(game_root, &["player", "country"]);

    ProgressionFacts {
        owned_planets: query_count(country_value, &["owned_planets"]),
        colonized_planets: query_count(country_value, &["controlled_planets"]),
        surveyed_planets: count_surveyed_planets(game_root),
        total_pops: query_f64(country_value, &["num_sapient_pops"]).map(|v| v as usize),
        enslaved_pops_count: count_enslaved_pops(game_root),
        robot_pop_ratio: calc_robot_pop_ratio(game_root),
        energy_stored: try_resource_stored(country_value, "energy"),
        energy_monthly: try_monthly_income(country_value, "energy"),
        minerals_monthly: try_monthly_income(country_value, "minerals"),
        alloys_monthly: try_monthly_income(country_value, "alloys"),
        trade_value_monthly: try_monthly_income(country_value, "trade"),
        strategic_resources_types: count_strategic_resource_types(country_value),
        organic_empires_remaining: count_organic_empires_remaining(
            game_root,
            player_country_id.as_deref(),
        ),
        fleet_power: query_f64(country_value, &["military_power"]),
        fleet_count: count_owned_fleet_entries(country_value),
        starbase_count: query_f64(country_value, &["num_upgraded_starbase"]).map(|v| v as usize),
        gateway_count: count_gateways(game_root),
        hyper_relay_count: count_hyper_relays(game_root),
        rare_technologies_acquired: count_rare_technologies(country_value),
        traditions_adopted: query_count(country_value, &["traditions"]),
        ascension_perks_unlocked: query_count(country_value, &["ascension_perks"]),
        ascension_path: detect_ascension_path(country_value),
        years_played: compute_years_played(game_root),
        years_at_peace: compute_years_at_peace(country_value, game_root),
        diplomatic_weight: query_f64(country_value, &["diplomatic_weight"]),
        intel_level_count: count_intel_levels(country_value),
        observation_station_count: count_country_flags(country_value, "built_observation_post"),
        capital_building_level: find_capital_building_level(game_root, country_value),
        living_standard: extract_living_standard(country_value),
        mercenary_enclaves_patroned: count_country_flags(country_value, "10yr_patronage"),
        vivarium_capacity: compute_vivarium_capacity(game_root),
        megastructure_types: extract_megastructure_types(game_root, player_country_id.as_deref()),
    }
}

/// Iterate over planets owned by a given country from `game_root > "planets" > "planet"`.
///
/// For each planet where `controller` equals `player_country_id`, the closure `f`
/// is called with that planet's value block.
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

/// Count planets that have a `survey` field (indicating they've been surveyed).
fn count_surveyed_planets(game_root: &ClausewitzValue) -> Option<usize> {
    let planets_block = query_path(game_root, &["planets", "planet"])?;
    match planets_block {
        ClausewitzValue::Block(nodes) => {
            let count = nodes
                .iter()
                .filter(|node| {
                    if let ClausewitzNode::Pair(_, planet_value) = node {
                        query_path(planet_value, &["survey"]).is_some()
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

/// Count the number of fleet entries in `fleets_manager > owned_fleets`.
///
/// The `owned_fleets` block contains entries like `{ fleet = <id> }`, each
/// stored as a `Value(Block(...))` node. `query_count` counts all nodes in
/// the block, which gives the fleet count.
fn count_owned_fleet_entries(country_value: &ClausewitzValue) -> Option<usize> {
    query_count(country_value, &["fleets_manager", "owned_fleets"])
}

/// Internal technology IDs for technologies with the `rare_technology` category.
/// These are the techs that count toward achievement requirements involving
/// rare technology acquisition.
const RARE_TECHNOLOGIES: &[&str] = &[
    "tech_arcane_deciphering",
    "tech_archaeostudies",
    "tech_bio_reactor",
    "tech_climate_restoration",
    "tech_colossus",
    "tech_commercialization",
    "tech_cryostasis_2",
    "tech_dangerous_initiatives",
    "tech_decryption_1",
    "tech_diplomatic_networking",
    "tech_encryption_1",
    "tech_genome_mapping",
    "tech_global_production_strategy",
    "tech_living_matter",
    "tech_mine_rare_crystals",
    "tech_mine_rare_gases",
    "tech_mine_satramene",
    "tech_mine_zro",
    "tech_nanite_transmutation",
    "tech_neural_implants",
    "tech_paradise_dome",
    "tech_psionic_theory",
    "tech_repeatable_improved_tile_energy",
    "tech_repeatable_improved_tile_minerals",
    "tech_repeatable_improved_tile_food",
    "tech_repeatable_improved_tile_physics",
    "tech_repeatable_improved_tile_society",
    "tech_repeatable_improved_tile_engineering",
    "tech_repeatable_naval_cap",
    "tech_repeatable_command_limit",
    "tech_sapient_ai",
    "tech_sentient_ai",
    "tech_society_engineering",
    "tech_subdermal_stimulation",
    "tech_symbolism",
    "tech_teratogenic_society",
    "tech_transcendent_thought",
    "tech_utopian_abundance",
    "tech_wilderness_preservation",
    "tech_xeno_diplomacy",
    "tech_zro_distillation",
];

/// Count `technology` entries in `tech_status` that match the
/// `RARE_TECHNOLOGIES` list.
fn count_rare_technologies(country_value: &ClausewitzValue) -> Option<usize> {
    let tech_status = query_path(country_value, &["tech_status"])?;
    match tech_status {
        ClausewitzValue::Block(nodes) => {
            let count = nodes
                .iter()
                .filter(|node| {
                    matches!(node, ClausewitzNode::Pair(key, ClausewitzValue::Atom(val))
                        if key == "technology" && RARE_TECHNOLOGIES.contains(&val.as_str()))
                })
                .count();
            Some(count)
        }
        _ => None,
    }
}

/// Count all `technology` entries in `tech_status` as a proxy for acquired
/// technologies. This counts every technology pair, not just rare ones.
#[allow(dead_code)]
fn count_technologies(country_value: &ClausewitzValue) -> Option<usize> {
    let tech_status = query_path(country_value, &["tech_status"])?;
    match tech_status {
        ClausewitzValue::Block(nodes) => {
            let count = nodes
                .iter()
                .filter(|node| matches!(node, ClausewitzNode::Pair(key, _) if key == "technology"))
                .count();
            Some(count)
        }
        _ => None,
    }
}

/// Detect the ascension path by scanning traditions and ascension perks for
/// path-indicating keywords ("psionic", "genetic", "cybernetic", "synthetic").
fn detect_ascension_path(country_value: &ClausewitzValue) -> Option<String> {
    let sources = [
        query_atoms(country_value, &["traditions"]),
        query_atoms(country_value, &["ascension_perks"]),
    ];
    for source in &sources {
        for entry in source {
            if entry.contains("psionic") {
                return Some("psionic".to_string());
            }
            if entry.contains("genetic") {
                return Some("genetic".to_string());
            }
            if entry.contains("cybernetic") {
                return Some("cybernetic".to_string());
            }
            if entry.contains("synthetic") {
                return Some("synthetic".to_string());
            }
        }
    }
    None
}

/// Parse the game date (format `"YYYY.MM.DD"`) and compute years played
/// from the default start year 2200.
fn compute_years_played(game_root: &ClausewitzValue) -> Option<f64> {
    let date_str = query_atom(game_root, &["date"])?;
    let year_str = date_str.split('.').next()?;
    let year: f64 = year_str.parse().ok()?;
    Some(year - 2200.0)
}

/// Try to extract a resource's stored amount from the budget structure.
///
/// Attempts paths in order of specificity:
/// 1. `budget > current_month > stored > <resource>`
/// 2. `budget > stored > <resource>`
fn try_resource_stored(country_value: &ClausewitzValue, resource: &str) -> Option<f64> {
    let path = ["budget", "current_month", "stored", resource];
    query_f64(country_value, &path)
        .or_else(|| query_f64(country_value, &["budget", "stored", resource]))
}

/// Try to extract monthly income for a resource from the budget structure.
///
/// Attempts paths in order:
/// 1. `budget > current_month > income > <resource>`
/// 2. `budget > last_month > income > <resource>`
/// 3. `budget > income > <resource>`
fn try_monthly_income(country_value: &ClausewitzValue, resource: &str) -> Option<f64> {
    let path_cur = ["budget", "current_month", "income", resource];
    let path_last = ["budget", "last_month", "income", resource];
    query_f64(country_value, &path_cur)
        .or_else(|| query_f64(country_value, &path_last))
        .or_else(|| query_f64(country_value, &["budget", "income", resource]))
}

/// Collect unique megastructure type strings owned by the player.
///
/// Iterates owned planets and collects `planet_class` values for those that
/// have a `megastructure` field set.
fn extract_megastructure_types(
    game_root: &ClausewitzValue,
    player_country_id: Option<&str>,
) -> Vec<String> {
    let Some(player_id) = player_country_id else {
        return Vec::new();
    };
    let mut types = Vec::new();
    for_each_owned_planet(game_root, player_id, |planet_value| {
        if query_path(planet_value, &["megastructure"]).is_some() {
            if let Some(class) = query_atom(planet_value, &["planet_class"]) {
                types.push(class);
            }
        }
    });
    types.sort();
    types.dedup();
    types
}

// ── Enslaved Pops ──────────────────────────────────────────────

/// Sum the size of all pop groups with `key > category = "slave"`.
fn count_enslaved_pops(game_root: &ClausewitzValue) -> Option<usize> {
    let pop_groups = query_path(game_root, &["pop_groups"])?;
    match pop_groups {
        ClausewitzValue::Block(nodes) => {
            let total: usize = nodes
                .iter()
                .filter_map(|node| {
                    if let ClausewitzNode::Pair(_, entry) = node {
                        if query_atom(entry, &["key", "category"]).as_deref() == Some("slave") {
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

// ── Robot Pop Ratio ──────────────────────────────────────────

/// Compute the ratio of machine pops to total pops by cross-referencing
/// `species_db` with `pop_groups`.
fn calc_robot_pop_ratio(game_root: &ClausewitzValue) -> Option<f64> {
    // Build set of machine species IDs
    let species_db = query_path(game_root, &["species_db"])?;
    let machine_species: HashSet<String> = match species_db {
        ClausewitzValue::Block(nodes) => nodes
            .iter()
            .filter_map(|node| {
                if let ClausewitzNode::Pair(id, entry) = node {
                    if query_atom(entry, &["class"]).as_deref() == Some("MACHINE") {
                        return Some(id.clone());
                    }
                }
                None
            })
            .collect(),
        _ => return None,
    };

    // Accumulate pop sizes
    let pop_groups = query_path(game_root, &["pop_groups"])?;
    let mut machine_total = 0.0f64;
    let mut grand_total = 0.0f64;

    match pop_groups {
        ClausewitzValue::Block(nodes) => {
            for node in nodes {
                if let ClausewitzNode::Pair(_, entry) = node {
                    let size = query_f64(entry, &["size"]).unwrap_or(0.0);
                    if size == 0.0 {
                        continue;
                    }
                    let species = query_atom(entry, &["key", "species"]);
                    let is_machine = species.is_some_and(|s| machine_species.contains(&s));
                    grand_total += size;
                    if is_machine {
                        machine_total += size;
                    }
                }
            }
        }
        _ => return None,
    }

    if grand_total > 0.0 {
        Some(machine_total / grand_total)
    } else {
        None
    }
}

// ── Strategic Resources ──────────────────────────────────────

/// Count distinct strategic resource types with positive monthly income.
fn count_strategic_resource_types(country_value: &ClausewitzValue) -> Option<usize> {
    const KNOWN_STRATEGIC: &[&str] = &[
        "sr_zro",
        "sr_living_metal",
        "sr_dark_matter",
        "nanites",
        "volatile_motes",
        "exotic_gases",
        "rare_crystals",
        "minor_artifacts",
    ];

    let income = query_path(country_value, &["budget", "current_month", "income"])?;
    let income_block = match income {
        ClausewitzValue::Block(nodes) => nodes,
        _ => return Some(0),
    };

    let mut found = HashSet::new();

    for node in income_block {
        if let ClausewitzNode::Pair(key, value) = node {
            // Check direct entries
            if KNOWN_STRATEGIC.contains(&key.as_str()) {
                if let Some(val) = parse_f64(value) {
                    if val > 0.0 {
                        found.insert(key.clone());
                    }
                }
            }
            // Check sub-blocks (planet_buildings, starbase_buildings, etc.)
            if let ClausewitzValue::Block(entries) = value {
                for entry in entries {
                    if let ClausewitzNode::Pair(sub_key, sub_value) = entry {
                        if KNOWN_STRATEGIC.contains(&sub_key.as_str()) {
                            if let Some(val) = parse_f64(sub_value) {
                                if val > 0.0 {
                                    found.insert(sub_key.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Some(found.len())
}

// ── Organic Empires Remaining ───────────────────────────────

/// Count other organic playable empires (excluding player and gestalt/FE types).
fn count_organic_empires_remaining(
    game_root: &ClausewitzValue,
    player_country_id: Option<&str>,
) -> Option<usize> {
    let player_id = player_country_id?;
    let country_block = query_path(game_root, &["country"])?;
    match country_block {
        ClausewitzValue::Block(nodes) => {
            let count = nodes
                .iter()
                .filter(|node| {
                    if let ClausewitzNode::Pair(id, entry) = node {
                        // Skip player
                        if id == player_id {
                            return false;
                        }
                        // Skip non-numeric IDs (pseudo-entries)
                        if !id.chars().all(|c| c.is_ascii_digit()) {
                            return false;
                        }
                        // Skip non-playable types
                        if let Some(t) = query_atom(entry, &["type"]) {
                            if matches!(
                                t.as_str(),
                                "fallen_empire"
                                    | "primitive"
                                    | "marauder"
                                    | "nomad"
                                    | "crisis"
                                    | "guardian"
                            ) {
                                return false;
                            }
                        }
                        // Skip gestalt authorities
                        if let Some(auth) = query_atom(entry, &["government", "authority"]) {
                            if matches!(
                                auth.as_str(),
                                "auth_hive_mind"
                                    | "auth_machine_intelligence"
                                    | "auth_ancient_machine_intelligence"
                            ) {
                                return false;
                            }
                        }
                        true
                    } else {
                        false
                    }
                })
                .count();
            Some(count)
        }
        _ => None,
    }
}

// ── Gateways ─────────────────────────────────────────────────

/// Count gateway-type bypasses (gateway, gateway_restored, gateway_final).
fn count_gateways(game_root: &ClausewitzValue) -> Option<usize> {
    let bypasses = query_path(game_root, &["bypasses"])?;
    match bypasses {
        ClausewitzValue::Block(nodes) => {
            // Space-delimited ID lists have only Value nodes — skip those
            let has_any_pair = nodes
                .iter()
                .any(|n| matches!(n, ClausewitzNode::Pair(_, _)));
            if !has_any_pair {
                return None;
            }
            let count = nodes
                .iter()
                .filter(|n| {
                    if let ClausewitzNode::Pair(_, entry) = n {
                        matches!(
                            query_atom(entry, &["type"]).as_deref(),
                            Some("gateway" | "gateway_restored" | "gateway_final")
                        )
                    } else {
                        false
                    }
                })
                .count();
            Some(count)
        }
        _ => None,
    }
}

// ── Hyper Relays ────────────────────────────────────────────

/// Count hyper_relay megastructures.
fn count_hyper_relays(game_root: &ClausewitzValue) -> Option<usize> {
    let megastructures = query_path(game_root, &["megastructures"])?;
    match megastructures {
        ClausewitzValue::Block(nodes) => {
            // Space-delimited ID lists have only Value nodes — skip those
            let has_any_pair = nodes
                .iter()
                .any(|n| matches!(n, ClausewitzNode::Pair(_, _)));
            if !has_any_pair {
                return None;
            }
            let count = nodes
                .iter()
                .filter(|n| {
                    if let ClausewitzNode::Pair(_, entry) = n {
                        query_atom(entry, &["type"]).as_deref() == Some("hyper_relay")
                    } else {
                        false
                    }
                })
                .count();
            Some(count)
        }
        _ => None,
    }
}

// ── Years at Peace ──────────────────────────────────────────

/// Compute years since the last war by parsing `last_date_at_war`.
///
/// If the empire was never at war (`last_date_at_war == "2200.01.01"`),
/// returns the total years played.
fn compute_years_at_peace(
    country_value: &ClausewitzValue,
    game_root: &ClausewitzValue,
) -> Option<f64> {
    let last_war_str = query_atom(country_value, &["last_date_at_war"])?;
    let current_year =
        query_atom(game_root, &["date"]).and_then(|d| d.split('.').next()?.parse::<f64>().ok())?;
    let war_year = last_war_str.split('.').next()?.parse::<f64>().ok()?;

    if last_war_str == "2200.01.01" {
        // Game start — never been at war; return years played
        Some(current_year - 2200.0)
    } else {
        Some(current_year - war_year)
    }
}

// ── Intel Level Count ───────────────────────────────────────

/// Return the number of entries in the `intel_level` block.
///
/// Counts nodes directly rather than using `query_atoms` (which deduplicates),
/// since we want the count of all intel level entries, not unique values.
fn count_intel_levels(country_value: &ClausewitzValue) -> Option<usize> {
    let il_path = query_path(country_value, &["intel_level"])?;
    match il_path {
        ClausewitzValue::Block(nodes) => Some(nodes.len()),
        _ => Some(0),
    }
}

// ── Country Flag Counter ────────────────────────────────────

/// Count direct `Pair(key, _)` entries in the country block with the given key.
/// This handles top-level repeating keys like `built_observation_post` and
/// `10yr_patronage`.
fn count_country_flags(country_value: &ClausewitzValue, key: &str) -> Option<usize> {
    match country_value {
        ClausewitzValue::Block(nodes) => {
            let count = nodes
                .iter()
                .filter(|n| matches!(n, ClausewitzNode::Pair(k, _) if k == key))
                .count();
            Some(count)
        }
        _ => None,
    }
}

// ── Capital Building Level ──────────────────────────────────

/// Determine the highest capital building level on the capital planet.
///
/// Cross-references the country's capital planet ID through
/// `planets > planet > <id> > buildings_cache` into `buildings > <id> > type`.
fn find_capital_building_level(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<usize> {
    let capital_id = query_atom(country_value, &["capital"])?;
    let planet = query_path(game_root, &["planets", "planet", &capital_id])?;

    let building_ids = query_atoms(planet, &["buildings_cache"]);
    if building_ids.is_empty() {
        return None;
    }

    let mut max_level = 0usize;
    for bid in &building_ids {
        let Some(building) = query_path(game_root, &["buildings", bid]) else {
            continue;
        };
        let Some(btype) = query_atom(building, &["type"]) else {
            continue;
        };
        let level = match btype.as_str() {
            "building_capital" => 1,
            "building_system_capital" => 2,
            "building_hab_capital" => 1,
            "building_hab_major_capital" => 2,
            _ => continue,
        };
        max_level = max_level.max(level);
    }

    if max_level > 0 {
        Some(max_level)
    } else {
        None
    }
}

// ── Living Standard ─────────────────────────────────────────

/// Extract the primary living standard from species rights.
fn extract_living_standard(country_value: &ClausewitzValue) -> Option<String> {
    query_atom(
        country_value,
        &[
            "standard_species_rights_module",
            "primary",
            "living_standard",
        ],
    )
    .or_else(|| {
        query_atom(
            country_value,
            &[
                "standard_species_rights_module",
                "default",
                "living_standard",
            ],
        )
    })
}

// ── Vivarium Capacity ───────────────────────────────────────

/// Count total entries in `vivarium_critters` + `exhibits` blocks.
fn compute_vivarium_capacity(game_root: &ClausewitzValue) -> Option<usize> {
    let critters = query_path(game_root, &["vivarium_critters"]);
    let exhibits = query_path(game_root, &["exhibits"]);

    match (critters, exhibits) {
        (None, None) => None,
        _ => {
            let c = critters.map_or(0, count_entries);
            let e = exhibits.map_or(0, count_entries);
            Some(c + e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── test AST constructors ──────────────────────────────────

    fn a(s: &str) -> ClausewitzValue {
        ClausewitzValue::Atom(s.to_string())
    }

    fn block(pairs: Vec<(&str, ClausewitzValue)>) -> ClausewitzValue {
        ClausewitzValue::Block(
            pairs
                .into_iter()
                .map(|(k, v)| ClausewitzNode::Pair(k.to_string(), v))
                .collect(),
        )
    }

    fn values(items: Vec<ClausewitzValue>) -> ClausewitzValue {
        ClausewitzValue::Block(items.into_iter().map(ClausewitzNode::Value).collect())
    }

    // ── tests ──────────────────────────────────────────────────

    #[test]
    fn test_basic_fields() {
        let game = block(vec![
            ("date", a("2230.06.15")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![
            ("owned_planets", values(vec![a("1"), a("2"), a("3")])),
            ("controlled_planets", values(vec![a("1"), a("2")])),
            ("num_sapient_pops", a("500")),
            ("military_power", a("15000.5")),
            ("num_upgraded_starbase", a("10")),
            (
                "traditions",
                values(vec![
                    a("tr_diplomacy_adopt"),
                    a("tr_diplomacy_the_federation"),
                ]),
            ),
            ("ascension_perks", values(vec![])),
        ]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.owned_planets, Some(3));
        assert_eq!(facts.colonized_planets, Some(2));
        assert_eq!(facts.total_pops, Some(500));
        assert_eq!(facts.fleet_power, Some(15000.5));
        assert_eq!(facts.starbase_count, Some(10));
        assert_eq!(facts.traditions_adopted, Some(2));
        assert_eq!(facts.ascension_perks_unlocked, Some(0));
        assert_eq!(facts.years_played, Some(30.0));
    }

    #[test]
    fn test_fleet_count() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![(
            "fleets_manager",
            block(vec![(
                "owned_fleets",
                values(vec![
                    block(vec![("fleet", a("100"))]),
                    block(vec![("fleet", a("101"))]),
                    block(vec![("fleet", a("102"))]),
                ]),
            )]),
        )]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.fleet_count, Some(3));
    }

    #[test]
    fn test_empty_fleet() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![(
            "fleets_manager",
            block(vec![("owned_fleets", values(vec![]))]),
        )]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.fleet_count, Some(0));
    }

    #[test]
    fn test_ascension_path_psionic() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![
            (
                "traditions",
                values(vec![
                    a("tr_diplomacy_adopt"),
                    a("tr_psionic_adopt"),
                    a("tr_psionic_the_shroud"),
                ]),
            ),
            ("ascension_perks", values(vec![a("ap_psionic_the_shroud")])),
        ]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.ascension_path, Some("psionic".to_string()));
        assert_eq!(facts.traditions_adopted, Some(3));
        assert_eq!(facts.ascension_perks_unlocked, Some(1));
    }

    #[test]
    fn test_ascension_path_synthetic_via_perks() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![
            ("traditions", values(vec![a("tr_diplomacy_adopt")])),
            ("ascension_perks", values(vec![a("ap_synthetic_evolution")])),
        ]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.ascension_path, Some("synthetic".to_string()));
    }

    #[test]
    fn test_no_ascension_path() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![
            ("traditions", values(vec![a("tr_diplomacy_adopt")])),
            ("ascension_perks", values(vec![])),
        ]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.ascension_path, None);
    }

    #[test]
    fn test_years_played() {
        let game = block(vec![
            ("date", a("2500.07.30")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert!((facts.years_played.unwrap() - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_megastructure_types() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            (
                "planets",
                block(vec![(
                    "planet",
                    block(vec![
                        (
                            "1",
                            block(vec![
                                ("controller", a("0")),
                                ("planet_class", a("pc_continental")),
                            ]),
                        ),
                        (
                            "100",
                            block(vec![
                                ("controller", a("0")),
                                ("megastructure", a("100")),
                                ("planet_class", a("pc_dyson_sphere")),
                            ]),
                        ),
                        (
                            "101",
                            block(vec![
                                ("controller", a("0")),
                                ("megastructure", a("101")),
                                ("planet_class", a("pc_science_nexus")),
                            ]),
                        ),
                        (
                            "200",
                            block(vec![
                                ("controller", a("1")), // other empire
                                ("megastructure", a("200")),
                                ("planet_class", a("pc_dyson_sphere")),
                            ]),
                        ),
                    ]),
                )]),
            ),
        ]);
        let country = block(vec![(
            "owned_planets",
            values(vec![a("1"), a("100"), a("101")]),
        )]);

        let facts = extract_progression_facts(&game, &country);

        let mut expected = vec![
            "pc_dyson_sphere".to_string(),
            "pc_science_nexus".to_string(),
        ];
        expected.sort();
        assert_eq!(facts.megastructure_types, expected);
    }

    #[test]
    fn test_surveyed_planets() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            (
                "planets",
                block(vec![(
                    "planet",
                    block(vec![
                        (
                            "1",
                            block(vec![("controller", a("0")), ("survey", a("yes"))]),
                        ),
                        (
                            "2",
                            block(vec![("controller", a("1")), ("survey", a("yes"))]),
                        ),
                        (
                            "3",
                            block(vec![("controller", a("1"))]), // no survey
                        ),
                    ]),
                )]),
            ),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        // Two planets have a survey field (IDs 1 and 2)
        assert_eq!(facts.surveyed_planets, Some(2));
    }

    #[test]
    fn test_technologies_count() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![(
            "tech_status",
            block(vec![
                ("technology", a("tech_physics_1")),
                ("level", a("1")),
                ("technology", a("tech_society_2")),
                ("level", a("1")),
                ("technology", a("tech_engineering_3")),
                ("level", a("2")),
            ]),
        )]);

        let facts = extract_progression_facts(&game, &country);

        // None of the test techs are in the rare list
        assert_eq!(facts.rare_technologies_acquired, Some(0));
    }

    #[test]
    fn test_rare_technologies_count() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![(
            "tech_status",
            block(vec![
                // Rare techs
                ("technology", a("tech_psionic_theory")),
                ("level", a("1")),
                ("technology", a("tech_paradise_dome")),
                ("level", a("1")),
                ("technology", a("tech_colossus")),
                ("level", a("1")),
                // Non-rare techs (should not be counted)
                ("technology", a("tech_physics_1")),
                ("level", a("2")),
                ("technology", a("tech_society_2")),
                ("level", a("1")),
            ]),
        )]);

        let facts = extract_progression_facts(&game, &country);

        // Should count only the 3 rare techs
        assert_eq!(facts.rare_technologies_acquired, Some(3));
    }

    #[test]
    fn test_missing_blocks_return_none() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.owned_planets, None);
        assert_eq!(facts.total_pops, None);
        assert_eq!(facts.fleet_power, None);
        assert_eq!(facts.fleet_count, None);
        assert_eq!(facts.starbase_count, None);
        assert_eq!(facts.traditions_adopted, None);
        assert_eq!(facts.ascension_perks_unlocked, None);
    }

    #[test]
    fn test_non_numeric_fields_graceful() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![
            ("num_sapient_pops", a("not_a_number")),
            ("military_power", a("also_bad")),
        ]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.total_pops, None);
        assert_eq!(facts.fleet_power, None);
    }

    // ── Enslaved Pops ──────────────────────────────────────────

    #[test]
    fn test_enslaved_pops_count() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            (
                "pop_groups",
                block(vec![
                    (
                        "0",
                        block(vec![
                            (
                                "key",
                                block(vec![("species", a("0")), ("category", a("slave"))]),
                            ),
                            ("size", a("5")),
                        ]),
                    ),
                    (
                        "1",
                        block(vec![
                            (
                                "key",
                                block(vec![("species", a("1")), ("category", a("normal"))]),
                            ),
                            ("size", a("10")),
                        ]),
                    ),
                    (
                        "2",
                        block(vec![
                            (
                                "key",
                                block(vec![("species", a("2")), ("category", a("slave"))]),
                            ),
                            ("size", a("3")),
                        ]),
                    ),
                ]),
            ),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.enslaved_pops_count, Some(8)); // 5 + 3
    }

    #[test]
    fn test_enslaved_pops_missing_block() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.enslaved_pops_count, None);
    }

    #[test]
    fn test_enslaved_pops_no_slaves() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            (
                "pop_groups",
                block(vec![(
                    "0",
                    block(vec![
                        (
                            "key",
                            block(vec![("species", a("0")), ("category", a("normal"))]),
                        ),
                        ("size", a("10")),
                    ]),
                )]),
            ),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.enslaved_pops_count, Some(0));
    }

    // ── Robot Pop Ratio ────────────────────────────────────────

    #[test]
    fn test_robot_pop_ratio() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            (
                "species_db",
                block(vec![
                    ("0", block(vec![("class", a("MACHINE"))])),
                    ("1", block(vec![("class", a("BIO"))])),
                ]),
            ),
            (
                "pop_groups",
                block(vec![
                    (
                        "0",
                        block(vec![
                            ("key", block(vec![("species", a("0"))])),
                            ("size", a("3")),
                        ]),
                    ),
                    (
                        "1",
                        block(vec![
                            ("key", block(vec![("species", a("1"))])),
                            ("size", a("7")),
                        ]),
                    ),
                    (
                        "2",
                        block(vec![
                            ("key", block(vec![("species", a("0"))])),
                            ("size", a("5")),
                        ]),
                    ),
                ]),
            ),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        let ratio = facts.robot_pop_ratio.unwrap();
        assert!((ratio - 8.0 / 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_robot_pop_ratio_no_machines() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            (
                "species_db",
                block(vec![("0", block(vec![("class", a("BIO"))]))]),
            ),
            (
                "pop_groups",
                block(vec![(
                    "0",
                    block(vec![
                        ("key", block(vec![("species", a("0"))])),
                        ("size", a("10")),
                    ]),
                )]),
            ),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.robot_pop_ratio, Some(0.0));
    }

    #[test]
    fn test_robot_pop_ratio_missing_blocks() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.robot_pop_ratio, None);
    }

    // ── Strategic Resources ────────────────────────────────────

    #[test]
    fn test_strategic_resources_types() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![(
            "budget",
            block(vec![(
                "current_month",
                block(vec![(
                    "income",
                    block(vec![
                        (
                            "planet_buildings",
                            block(vec![
                                ("energy", a("100")),
                                ("sr_zro", a("2.5")),
                                ("volatile_motes", a("1.0")),
                            ]),
                        ),
                        (
                            "starbase_buildings",
                            block(vec![("sr_dark_matter", a("3.0"))]),
                        ),
                        ("minerals", a("50")),
                    ]),
                )]),
            )]),
        )]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.strategic_resources_types, Some(3)); // sr_zro, volatile_motes, sr_dark_matter
    }

    #[test]
    fn test_strategic_resources_none() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![(
            "budget",
            block(vec![(
                "current_month",
                block(vec![(
                    "income",
                    block(vec![(
                        "planet_buildings",
                        block(vec![("energy", a("100")), ("minerals", a("50"))]),
                    )]),
                )]),
            )]),
        )]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.strategic_resources_types, Some(0));
    }

    #[test]
    fn test_strategic_resources_missing_budget() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.strategic_resources_types, None);
    }

    // ── Organic Empires Remaining ──────────────────────────────

    #[test]
    fn test_organic_empires_remaining() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            (
                "country",
                block(vec![
                    // Player — should be skipped
                    (
                        "0",
                        block(vec![(
                            "government",
                            block(vec![("authority", a("auth_democratic"))]),
                        )]),
                    ),
                    // Organic democratic empire — counted
                    (
                        "1",
                        block(vec![(
                            "government",
                            block(vec![("authority", a("auth_democratic"))]),
                        )]),
                    ),
                    // Hive mind — skipped
                    (
                        "2",
                        block(vec![(
                            "government",
                            block(vec![("authority", a("auth_hive_mind"))]),
                        )]),
                    ),
                    // Machine intelligence — skipped
                    (
                        "3",
                        block(vec![(
                            "government",
                            block(vec![("authority", a("auth_machine_intelligence"))]),
                        )]),
                    ),
                    // Fallen empire — skipped
                    (
                        "4",
                        block(vec![
                            ("type", a("fallen_empire")),
                            (
                                "government",
                                block(vec![("authority", a("auth_democratic"))]),
                            ),
                        ]),
                    ),
                    // Organic authoritarian empire — counted
                    (
                        "5",
                        block(vec![(
                            "government",
                            block(vec![("authority", a("auth_authocratic"))]),
                        )]),
                    ),
                    // Primitive — skipped
                    (
                        "6",
                        block(vec![
                            ("type", a("primitive")),
                            (
                                "government",
                                block(vec![("authority", a("auth_democratic"))]),
                            ),
                        ]),
                    ),
                ]),
            ),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.organic_empires_remaining, Some(2)); // IDs 1 and 5
    }

    // ── Gateways ───────────────────────────────────────────────

    #[test]
    fn test_gateway_count() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            (
                "bypasses",
                block(vec![
                    (
                        "0",
                        block(vec![("type", a("gateway")), ("active", a("yes"))]),
                    ),
                    (
                        "1",
                        block(vec![("type", a("gateway_restored")), ("active", a("yes"))]),
                    ),
                    ("2", block(vec![("type", a("wormhole"))])),
                    (
                        "3",
                        block(vec![("type", a("gateway_final")), ("active", a("yes"))]),
                    ),
                ]),
            ),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.gateway_count, Some(3)); // gateway, gateway_restored, gateway_final
    }

    #[test]
    fn test_gateway_count_no_bypasses() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.gateway_count, None);
    }

    #[test]
    fn test_gateway_count_id_list_format() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            ("bypasses", values(vec![a("100"), a("101"), a("102")])),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        // ID list format — not structured data, return None
        assert_eq!(facts.gateway_count, None);
    }

    // ── Hyper Relays ───────────────────────────────────────────

    #[test]
    fn test_hyper_relay_count() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            (
                "megastructures",
                block(vec![
                    (
                        "0",
                        block(vec![("type", a("hyper_relay")), ("owner", a("0"))]),
                    ),
                    (
                        "1",
                        block(vec![("type", a("science_nexus")), ("owner", a("0"))]),
                    ),
                    (
                        "2",
                        block(vec![("type", a("hyper_relay")), ("owner", a("0"))]),
                    ),
                ]),
            ),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.hyper_relay_count, Some(2));
    }

    #[test]
    fn test_hyper_relay_count_id_list_format() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            ("megastructures", values(vec![a("100"), a("101")])),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.hyper_relay_count, None);
    }

    // ── Years at Peace ─────────────────────────────────────────

    #[test]
    fn test_years_at_peace() {
        let game = block(vec![
            ("date", a("2230.06.15")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![("last_date_at_war", a("2225.03.10"))]);

        let facts = extract_progression_facts(&game, &country);

        assert!((facts.years_at_peace.unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_years_at_peace_never_war() {
        let game = block(vec![
            ("date", a("2230.06.15")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![("last_date_at_war", a("2200.01.01"))]);

        let facts = extract_progression_facts(&game, &country);

        // Never been at war — return years played (2230 - 2200 = 30)
        assert!((facts.years_at_peace.unwrap() - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_years_at_peace_missing_field() {
        let game = block(vec![
            ("date", a("2230.06.15")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.years_at_peace, None);
    }

    // ── Intel Level Count ──────────────────────────────────────

    #[test]
    fn test_intel_level_count() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![(
            "intel_level",
            values(vec![a("3"), a("2"), a("1"), a("0"), a("3")]),
        )]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.intel_level_count, Some(5));
    }

    #[test]
    fn test_intel_level_empty() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![("intel_level", values(vec![]))]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.intel_level_count, Some(0));
    }

    #[test]
    fn test_intel_level_missing() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.intel_level_count, None);
    }

    // ── Observation Stations ───────────────────────────────────

    #[test]
    fn test_observation_station_count() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![
            ("built_observation_post", a("100")),
            ("built_observation_post", a("101")),
            ("built_observation_post", a("102")),
        ]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.observation_station_count, Some(3));
    }

    #[test]
    fn test_observation_station_none() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.observation_station_count, Some(0));
    }

    // ── Capital Building Level ─────────────────────────────────

    #[test]
    fn test_capital_building_level() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            (
                "planets",
                block(vec![(
                    "planet",
                    block(vec![(
                        "42",
                        block(vec![
                            ("controller", a("0")),
                            ("buildings_cache", values(vec![a("100"), a("200")])),
                        ]),
                    )]),
                )]),
            ),
            (
                "buildings",
                block(vec![
                    ("100", block(vec![("type", a("building_system_capital"))])),
                    ("200", block(vec![("type", a("building_lab"))])),
                ]),
            ),
        ]);
        let country = block(vec![("capital", a("42"))]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.capital_building_level, Some(2));
    }

    #[test]
    fn test_capital_building_level_basic() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            (
                "planets",
                block(vec![(
                    "planet",
                    block(vec![(
                        "10",
                        block(vec![
                            ("controller", a("0")),
                            ("buildings_cache", values(vec![a("50")])),
                        ]),
                    )]),
                )]),
            ),
            (
                "buildings",
                block(vec![("50", block(vec![("type", a("building_capital"))]))]),
            ),
        ]);
        let country = block(vec![("capital", a("10"))]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.capital_building_level, Some(1));
    }

    #[test]
    fn test_capital_building_level_no_capital() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            (
                "planets",
                block(vec![(
                    "planet",
                    block(vec![(
                        "10",
                        block(vec![
                            ("controller", a("0")),
                            ("buildings_cache", values(vec![a("50")])),
                        ]),
                    )]),
                )]),
            ),
            (
                "buildings",
                block(vec![("50", block(vec![("type", a("building_lab"))]))]),
            ),
        ]);
        let country = block(vec![("capital", a("10"))]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.capital_building_level, None);
    }

    // ── Living Standard ────────────────────────────────────────

    #[test]
    fn test_living_standard_primary() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![(
            "standard_species_rights_module",
            block(vec![(
                "primary",
                block(vec![(
                    "living_standard",
                    a("living_standard_utopian_abundance"),
                )]),
            )]),
        )]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(
            facts.living_standard,
            Some("living_standard_utopian_abundance".to_string())
        );
    }

    #[test]
    fn test_living_standard_fallback_default() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![(
            "standard_species_rights_module",
            block(vec![(
                "default",
                block(vec![(
                    "living_standard",
                    a("living_standard_decent_conditions"),
                )]),
            )]),
        )]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(
            facts.living_standard,
            Some("living_standard_decent_conditions".to_string())
        );
    }

    // ── Mercenary Enclaves Patroned ────────────────────────────

    #[test]
    fn test_mercenary_enclaves_patroned() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![
            ("10yr_patronage", a("100")),
            ("10yr_patronage", a("101")),
        ]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.mercenary_enclaves_patroned, Some(2));
    }

    #[test]
    fn test_mercenary_enclaves_none() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.mercenary_enclaves_patroned, Some(0));
    }

    // ── Vivarium Capacity ──────────────────────────────────────

    #[test]
    fn test_vivarium_capacity() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            (
                "vivarium_critters",
                block(vec![
                    ("0", block(vec![("species", a("0"))])),
                    ("1", block(vec![("species", a("1"))])),
                    ("2", block(vec![("species", a("2"))])),
                ]),
            ),
            (
                "exhibits",
                block(vec![("0", block(vec![("name", a("exhibit_a"))]))]),
            ),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.vivarium_capacity, Some(4)); // 3 critters + 1 exhibit
    }

    #[test]
    fn test_vivarium_capacity_empty() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            ("vivarium_critters", block(vec![])),
            ("exhibits", block(vec![])),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.vivarium_capacity, Some(0));
    }

    #[test]
    fn test_vivarium_capacity_missing() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.vivarium_capacity, None);
    }

    #[test]
    fn test_vivarium_capacity_only_critters() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
            (
                "vivarium_critters",
                block(vec![("0", block(vec![("species", a("0"))]))]),
            ),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.vivarium_capacity, Some(1));
    }

    // ── New dimension defaults ───────────────────────────────

    #[test]
    fn test_new_dimensions_all_none_when_missing() {
        let game = block(vec![
            ("date", a("2205.01.01")),
            ("player", block(vec![("country", a("0"))])),
        ]);
        let country = block(vec![]);

        let facts = extract_progression_facts(&game, &country);

        assert_eq!(facts.enslaved_pops_count, None);
        assert_eq!(facts.robot_pop_ratio, None);
        assert_eq!(facts.strategic_resources_types, None);
        assert_eq!(facts.organic_empires_remaining, None);
        assert_eq!(facts.gateway_count, None);
        assert_eq!(facts.hyper_relay_count, None);
        assert_eq!(facts.years_at_peace, None);
        assert_eq!(facts.intel_level_count, None);
        assert_eq!(facts.observation_station_count, Some(0));
        assert_eq!(facts.capital_building_level, None);
        assert_eq!(facts.living_standard, None);
        assert_eq!(facts.mercenary_enclaves_patroned, Some(0));
        assert_eq!(facts.vivarium_capacity, None);
    }
}
