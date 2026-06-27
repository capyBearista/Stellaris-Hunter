use super::*;

pub(super) fn check_horizon_signal(country_value: &ClausewitzValue) -> Option<bool> {
    Some(has_any_flag(
        country_value,
        &["horizon_signal", "worm_in_rift", "horizon_signal_completed"],
    ))
}

pub(super) fn extract_covenant(country_value: &ClausewitzValue) -> Option<String> {
    for (flag, name) in &[
        ("covenant_whisperers", "whisperers"),
        ("covenant_instrument", "instrument"),
        ("covenant_composer", "composer"),
        ("covenant_eater", "eater"),
    ] {
        if has_flag(country_value, flag) {
            return Some((*name).to_string());
        }
    }
    for (perk, name) in &[
        ("ap_covenant_whisperers", "whisperers"),
        ("ap_covenant_instrument", "instrument"),
        ("ap_covenant_composer", "composer"),
        ("ap_covenant_eater", "eater"),
    ] {
        if field_contains(country_value, "ascension_perks", perk) {
            return Some((*name).to_string());
        }
    }
    None
}

pub(super) fn extract_special_project_completed_type(
    country_value: &ClausewitzValue,
) -> Option<String> {
    let known_projects = &[
        "breach_the_shroud",
        "limbo",
        "synthetic_infiltrator_detection",
        "infinity_sphere_peaceful",
        "unexpected_mineral_seams",
        "knights_toxic_god_final",
        "mysterious_chart",
    ];
    let mut found = Vec::new();
    for_each_flag_key(country_value, |key| {
        for project in known_projects {
            let project = project.to_string();
            if key.contains(&project) && !found.contains(&project) {
                found.push(project);
            }
        }
    });
    if let Some(ClausewitzValue::Block(nodes)) = query_path(country_value, &["special_projects"]) {
        for node in nodes {
            if let ClausewitzNode::Pair(key, _) = node {
                for project in known_projects {
                    let project = project.to_string();
                    if key.contains(&project) && !found.contains(&project) {
                        found.push(project);
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

pub(super) fn count_livestock_species(game_root: &ClausewitzValue) -> Option<usize> {
    count_distinct_species_in_category(game_root, "livestock")
}

pub(super) fn max_species_on_planet(
    game_root: &ClausewitzValue,
    player_country_id: &str,
) -> Option<usize> {
    let mut max_count = 0;
    let mut any_planet = false;
    for_each_owned_planet(game_root, player_country_id, |planet_value| {
        any_planet = true;
        let mut species_set = std::collections::HashSet::new();
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

pub(super) fn count_volcanic_planets(
    game_root: &ClausewitzValue,
    player_country_id: &str,
) -> Option<usize> {
    let mut count = 0;
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

pub(super) fn count_galactic_memorials_on_tomb_worlds(
    game_root: &ClausewitzValue,
    player_country_id: &str,
) -> Option<usize> {
    let mut count = 0;
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

pub(super) fn colony_count_without_hyperspace(country_value: &ClausewitzValue) -> Option<usize> {
    let tech_status = query_path(country_value, &["tech_status"])?;
    match tech_status {
        ClausewitzValue::Block(nodes) => {
            let has_hyperspace = nodes.iter().any(|node| {
                matches!(node, ClausewitzNode::Pair(key, ClausewitzValue::Atom(val)) if key == "technology" && val == "tech_hyperspace")
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

#[cfg(test)]
mod tests {
    use super::super::{extract_action_facts, test_support::*};

    #[test]
    fn test_covenant_horizon_and_projects() {
        let country = make_block(vec![
            (
                "ascension_perks",
                make_atom("ap_covenant_eater ap_mind_over_matter"),
            ),
            (
                "special_projects",
                make_block(vec![("limbo", make_atom("yes"))]),
            ),
            (
                "flags",
                make_block(vec![
                    ("horizon_signal", make_atom("yes")),
                    ("breach_the_shroud", make_atom("yes")),
                ]),
            ),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.covenant_type, Some("eater".to_string()));
        assert_eq!(facts.horizon_signal_completed, Some(true));
        assert!(facts
            .special_project_completed_type
            .unwrap()
            .contains("limbo"));
        assert_eq!(facts.psionic_techs_unlocked, Some(true));
    }

    #[test]
    fn test_species_and_planet_extractors() {
        let planet1 = make_block(vec![
            ("controller", make_atom("0")),
            ("planet_class", make_atom("pc_tomb")),
            (
                "buildings_cache",
                make_value_block(vec![make_atom("bld_galactic_memorial_01")]),
            ),
            (
                "pop_groups",
                make_block(vec![
                    (
                        "p1",
                        make_block(vec![("key", make_block(vec![("species", make_atom("0"))]))]),
                    ),
                    (
                        "p2",
                        make_block(vec![("key", make_block(vec![("species", make_atom("1"))]))]),
                    ),
                ]),
            ),
        ]);
        let planet2 = make_block(vec![
            ("controller", make_atom("0")),
            ("planet_class", make_atom("pc_volcanic")),
            (
                "pop_groups",
                make_block(vec![
                    (
                        "p1",
                        make_block(vec![("key", make_block(vec![("species", make_atom("0"))]))]),
                    ),
                    (
                        "p2",
                        make_block(vec![("key", make_block(vec![("species", make_atom("1"))]))]),
                    ),
                    (
                        "p3",
                        make_block(vec![("key", make_block(vec![("species", make_atom("2"))]))]),
                    ),
                ]),
            ),
        ]);
        let game = make_block(vec![
            (
                "planets",
                make_block(vec![(
                    "planet",
                    make_block(vec![("1", planet1), ("2", planet2)]),
                )]),
            ),
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
                ]),
            ),
        ]);
        let facts = extract_action_facts(&game, &make_country(vec![]), "0");
        assert_eq!(facts.species_on_planet_count, Some(3));
        assert_eq!(facts.livestock_species_count, Some(2));
        assert_eq!(facts.planets_terraform_to_volcanic, Some(1));
        assert_eq!(facts.galactic_memorials_on_tomb_worlds, Some(1));
    }

    #[test]
    fn test_colony_and_variable_backed_flags() {
        let country = make_block(vec![
            (
                "tech_status",
                make_block(vec![("technology", make_atom("tech_physics_1"))]),
            ),
            (
                "owned_planets",
                make_value_block(vec![make_atom("1"), make_atom("2"), make_atom("3")]),
            ),
            (
                "flags",
                make_block(vec![
                    ("INF_A_blazing_tomb_world", make_atom("yes")),
                    ("with_great_power_achievement_locked", make_atom("yes")),
                    ("built_artificial_ship", make_atom("yes")),
                ]),
            ),
            (
                "variables",
                make_block(vec![
                    ("smorgasblorg_phenotypes", make_atom("6")),
                    ("hyperthermia_giant_var", make_atom("50")),
                ]),
            ),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.colony_count_with_hyperspace_not_researched, Some(3));
        assert_eq!(facts.blazing_scourge_decisions, Some(true));
        assert_eq!(facts.pre_ftl_invasion_occurred, Some(true));
        assert_eq!(facts.artificial_military_ships_built, Some(true));
        assert_eq!(facts.species_dna_phenotypes_collected, Some(6));
        assert_eq!(facts.stars_terraform_to_red_giant, Some(50));
    }
}
