// Functions are pub(crate) and used by save.rs as part of the parse pipeline.

//! Extract action/event facts from a parsed Stellaris gamestate AST.

mod diplomacy;
mod empire;
mod exploration;
mod military;
mod relics;

use crate::model::ActionFacts;
use crate::save::*;

pub(super) fn has_flag(country_value: &ClausewitzValue, flag_name: &str) -> bool {
    query_path(country_value, &["flags", flag_name]).is_some()
}

pub(super) fn has_any_flag(country_value: &ClausewitzValue, flag_names: &[&str]) -> bool {
    flag_names.iter().any(|name| has_flag(country_value, name))
}

pub(super) fn collect_space_list(country_value: &ClausewitzValue, field: &str) -> Vec<String> {
    match query_path(country_value, &[field]) {
        Some(ClausewitzValue::Atom(s)) => s.split_whitespace().map(|s| s.to_string()).collect(),
        Some(_) => query_atoms(country_value, &[field]),
        None => vec![],
    }
}

pub(super) fn field_contains(country_value: &ClausewitzValue, field: &str, value: &str) -> bool {
    collect_space_list(country_value, field)
        .iter()
        .any(|s| s == value)
}

pub(super) fn for_each_flag_key<F>(country_value: &ClausewitzValue, mut f: F)
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

pub(super) fn sum_pop_groups_by_category(
    game_root: &ClausewitzValue,
    category: &str,
) -> Option<usize> {
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

pub(super) fn count_distinct_species_in_category(
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

pub(super) fn for_each_owned_planet<'a>(
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

pub(super) fn block_entries_contain_country(block: &ClausewitzValue, target_id: &str) -> bool {
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

pub(crate) fn extract_action_facts(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
    player_country_id: &str,
) -> ActionFacts {
    let (active_wars, war_type) = diplomacy::count_player_active_wars(game_root, player_country_id);
    let (subjects_acquired, vassal_count) = diplomacy::extract_subjects_data(country_value);

    ActionFacts {
        active_wars: if active_wars > 0 {
            Some(active_wars)
        } else {
            None
        },
        war_type,
        subjects_acquired,
        vassal_count,
        subject_type: diplomacy::extract_subject_type(country_value),
        subject_contract_modified: Some(has_flag(country_value, "subject_contract_modified")),
        secret_fealty_pledged: Some(has_any_flag(
            country_value,
            &["secret_fealty_pledged", "secret_fealty", "fealty_pledged"],
        )),
        proxy_war_count: diplomacy::count_proxy_wars(country_value),
        federation_formed: diplomacy::extract_federation_formed(country_value),
        federation_type: diplomacy::extract_federation_type(game_root, country_value),
        federation_level: diplomacy::extract_federation_level(game_root, country_value),
        federation_member_ethics: diplomacy::extract_federation_member_ethics(
            game_root,
            country_value,
            player_country_id,
        ),
        galactic_community_exists: diplomacy::extract_gc_exists(game_root),
        galactic_community_founding_member: Some(has_flag(
            country_value,
            "galactic_community_founding_member",
        )),
        galactic_custodian: diplomacy::extract_gc_custodian(game_root, player_country_id),
        galactic_custodian_action: query_atom(country_value, &["flags", "custodian_action"]),
        galactic_emperor: Some(has_flag(country_value, "galactic_emperor")),
        galactic_emperor_rebellion: Some(has_flag(country_value, "galactic_emperor_rebellion")),
        colossus_built: Some(field_contains(
            country_value,
            "ascension_perks",
            "ap_colossus",
        )),
        colossus_weapon_type: military::extract_colossus_weapon_type(country_value),
        colossus_destroyed_while_firing: Some(has_flag(country_value, "colossus_destroyed")),
        species_genetically_modified: Some(has_flag(country_value, "genetically_modified")),
        species_uplifted: Some(has_flag(country_value, "species_uplifted")),
        species_on_planet_count: empire::max_species_on_planet(game_root, player_country_id),
        species_dna_phenotypes_collected: query_f64(
            country_value,
            &["variables", "smorgasblorg_phenotypes"],
        )
        .map(|v| v as usize),
        slavery_type: query_atom(country_value, &["flags", "slavery_type"]),
        livestock_species_count: empire::count_livestock_species(game_root),
        purged_pops: military::count_purged_pops(game_root),
        species_enslaved: Some(has_flag(country_value, "species_enslaved")),
        relic_owned: relics::extract_relics_owned(country_value),
        relic_active_effect_used: relics::extract_relic_active_effect_used(country_value),
        galatron_acquired: relics::extract_galatron(country_value),
        galatron_captured: Some(has_flag(country_value, "galatron_captured")),
        archaeological_site_completed: exploration::extract_archaeology_sites(
            game_root,
            player_country_id,
        ),
        wormhole_travel_completed: Some(has_flag(country_value, "wormhole_travel_completed")),
        pre_ftl_infiltration_completed: Some(has_flag(country_value, "pre_ftl_infiltrated")),
        first_contact_result: exploration::extract_first_contact_result(game_root),
        espionage_operations_completed: diplomacy::count_espionage_operations_completed(
            country_value,
        ),
        astral_rifts_explored: exploration::count_astral_rifts_explored(game_root),
        crisis_defeated: Some(has_flag(country_value, "crisis_defeated")),
        captured_prethoryn_scourge_queen: Some(has_flag(country_value, "prethoryn_queen_captured")),
        crisis_path_nemesis: military::extract_crisis_path_nemesis(country_value),
        crisis_path_cosmogenesis: military::extract_crisis_path_cosmogenesis(country_value),
        crisis_path_hyperthermia: Some(has_flag(country_value, "hyperthermia")),
        crisis_path_behemoth_fury: Some(has_flag(country_value, "behemoth_fury")),
        machine_uprising_victory: Some(has_flag(country_value, "machine_uprising_survived")),
        amoeba_companion_found: Some(has_flag(country_value, "space_amoeba_companion")),
        amoeba_companion_killed: Some(has_flag(country_value, "space_amoeba_killed")),
        artisan_enclave_patron: Some(has_flag(country_value, "artisan_enclave_patron")),
        enclave_interaction_type: diplomacy::extract_enclave_interaction_type(country_value),
        migration_treaty_count: diplomacy::count_migration_treaties(country_value),
        legendary_paragon_recruited: Some(has_flag(country_value, "legendary_paragon_recruited")),
        robot_pop_built: Some(has_flag(country_value, "robot_pop_built")),
        horizon_signal_completed: empire::check_horizon_signal(country_value),
        civil_war_completed: Some(has_flag(country_value, "civil_war_completed")),
        special_project_completed_type: empire::extract_special_project_completed_type(
            country_value,
        ),
        covenant_type: empire::extract_covenant(country_value),
        psionic_techs_unlocked: Some(
            field_contains(country_value, "ascension_perks", "ap_mind_over_matter")
                || has_flag(country_value, "psionic_theory_researched"),
        ),
        quantum_catapult_used: Some(has_flag(country_value, "quantum_catapult_used")),
        blazing_scourge_decisions: Some(has_flag(country_value, "INF_A_blazing_tomb_world")),
        stars_terraform_to_red_giant: query_f64(
            country_value,
            &["variables", "hyperthermia_giant_var"],
        )
        .map(|v| v as usize),
        planets_terraform_to_volcanic: empire::count_volcanic_planets(game_root, player_country_id),
        volcanic_holy_world_created: Some(has_flag(country_value, "volcanic_holy_world")),
        galactic_memorials_on_tomb_worlds: empire::count_galactic_memorials_on_tomb_worlds(
            game_root,
            player_country_id,
        ),
        space_fauna_type_captured: exploration::extract_space_fauna_type_captured(game_root),
        colony_count_with_hyperspace_not_researched: empire::colony_count_without_hyperspace(
            country_value,
        ),
        pre_ftl_invasion_occurred: Some(has_flag(
            country_value,
            "with_great_power_achievement_locked",
        )),
        artificial_military_ships_built: Some(has_flag(country_value, "built_artificial_ship")),
        invaded_primitive_earth: Some(has_flag(country_value, "invaded_earth")),
    }
}

#[cfg(test)]
pub(super) mod test_support {
    use crate::save::{ClausewitzNode, ClausewitzValue};

    pub fn make_atom(s: &str) -> ClausewitzValue {
        ClausewitzValue::Atom(s.to_string())
    }

    pub fn make_block(pairs: Vec<(&str, ClausewitzValue)>) -> ClausewitzValue {
        ClausewitzValue::Block(
            pairs
                .into_iter()
                .map(|(k, v)| ClausewitzNode::Pair(k.to_string(), v))
                .collect(),
        )
    }

    pub fn make_value_block(values: Vec<ClausewitzValue>) -> ClausewitzValue {
        ClausewitzValue::Block(values.into_iter().map(ClausewitzNode::Value).collect())
    }

    pub fn make_country(flags: Vec<(&str, &str)>) -> ClausewitzValue {
        let flag_pairs: Vec<(&str, ClausewitzValue)> =
            flags.iter().map(|(k, v)| (*k, make_atom(v))).collect();
        make_block(vec![("flags", make_block(flag_pairs))])
    }

    pub fn make_game_root(entries: Vec<(&str, ClausewitzValue)>) -> ClausewitzValue {
        make_block(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_action_facts, test_support::*};

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
        assert_eq!(facts.federation_formed, Some(false));
        assert!(facts.federation_type.is_none());
        assert!(facts.galactic_community_exists.is_none());
        assert_eq!(facts.colossus_built, Some(false));
        assert!(facts.colossus_weapon_type.is_none());
        assert!(facts.species_on_planet_count.is_none());
        assert!(facts.livestock_species_count.is_none());
        assert!(facts.purged_pops.is_none());
        assert!(facts.relic_owned.is_none());
        assert!(facts.relic_active_effect_used.is_none());
        assert_eq!(facts.galatron_acquired, Some(false));
        assert!(facts.archaeological_site_completed.is_none());
        assert!(facts.first_contact_result.is_none());
        assert_eq!(facts.espionage_operations_completed, Some(0));
        assert!(facts.astral_rifts_explored.is_none());
        assert_eq!(facts.horizon_signal_completed, Some(false));
        assert!(facts.enclave_interaction_type.is_none());
        assert!(facts.migration_treaty_count.is_none());
        assert!(facts.special_project_completed_type.is_none());
        assert!(facts.covenant_type.is_none());
        assert!(facts.planets_terraform_to_volcanic.is_none());
        assert!(facts.galactic_memorials_on_tomb_worlds.is_none());
        assert!(facts.space_fauna_type_captured.is_none());
        assert!(facts.colony_count_with_hyperspace_not_researched.is_none());
    }

    #[test]
    fn test_flag_extraction() {
        let country = make_country(vec![
            ("subject_contract_modified", "yes"),
            ("galactic_emperor", "yes"),
            ("crisis_defeated", "yes"),
            ("invaded_earth", "yes"),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");

        assert_eq!(facts.subject_contract_modified, Some(true));
        assert_eq!(facts.galactic_emperor, Some(true));
        assert_eq!(facts.crisis_defeated, Some(true));
        assert_eq!(facts.invaded_primitive_earth, Some(true));
        assert_eq!(facts.crisis_path_cosmogenesis, Some(false));
        assert_eq!(facts.robot_pop_built, Some(false));
        assert_eq!(facts.wormhole_travel_completed, Some(false));
    }
}
