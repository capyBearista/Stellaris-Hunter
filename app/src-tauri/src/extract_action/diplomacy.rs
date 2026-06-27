use super::*;

pub(super) fn count_player_active_wars(
    game_root: &ClausewitzValue,
    player_country_id: &str,
) -> (usize, Option<String>) {
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

pub(super) fn extract_subjects_data(
    country_value: &ClausewitzValue,
) -> (Option<usize>, Option<usize>) {
    let count = query_path(country_value, &["subjects"]).map(count_entries);
    (count, count)
}

pub(super) fn extract_federation_formed(country_value: &ClausewitzValue) -> Option<bool> {
    match query_path(country_value, &["federation"]) {
        Some(ClausewitzValue::Atom(s)) => Some(!s.is_empty()),
        Some(_) => Some(true),
        None => Some(false),
    }
}

pub(super) fn extract_gc_exists(game_root: &ClausewitzValue) -> Option<bool> {
    query_path(game_root, &["galactic_community", "community_formed"]).map(|_| true)
}

pub(super) fn extract_gc_custodian(
    game_root: &ClausewitzValue,
    player_country_id: &str,
) -> Option<bool> {
    let active = query_f64(game_root, &["galactic_community", "custodian_term"])
        .map(|t| t != -1.0)
        .unwrap_or(false);
    if !active {
        return Some(false);
    }
    Some(
        query_atom(game_root, &["galactic_community", "leader"])
            .map(|id| id == player_country_id)
            .unwrap_or(false),
    )
}

pub(super) fn extract_subject_type(country_value: &ClausewitzValue) -> Option<String> {
    let subjects = query_path(country_value, &["subjects"])?;
    match subjects {
        ClausewitzValue::Block(nodes) => {
            let mut types = Vec::new();
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

pub(super) fn extract_enclave_interaction_type(country_value: &ClausewitzValue) -> Option<String> {
    let patterns = &[
        "recruit_curator_scientist",
        "buy_trader_resource",
        "destroyed_enclave",
        "destroy_enclave",
        "traded_with_artisans",
        "recruited_curator",
    ];
    let mut found = Vec::new();
    for_each_flag_key(country_value, |key| {
        for pattern in patterns {
            let pattern = pattern.to_string();
            if key.contains(&pattern) && !found.contains(&pattern) {
                found.push(pattern);
            }
        }
    });
    if found.is_empty() {
        None
    } else {
        Some(found.join(", "))
    }
}

pub(super) fn get_federation_entry<'a>(
    game_root: &'a ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<&'a ClausewitzValue> {
    let fed_id = query_atom(country_value, &["federation"])?;
    query_path(game_root, &["federation", &fed_id])
}

pub(super) fn extract_federation_type(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<String> {
    let fed = get_federation_entry(game_root, country_value)?;
    query_atom(fed, &["type"])
}

pub(super) fn extract_federation_level(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<usize> {
    let fed = get_federation_entry(game_root, country_value)?;
    query_f64(fed, &["level"])
        .or_else(|| query_f64(fed, &["federation_level"]))
        .or_else(|| query_f64(fed, &["centralization_level"]))
        .map(|v| v as usize)
}

pub(super) fn extract_federation_member_ethics(
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

    let mut ethics = Vec::new();
    for node in members {
        let member_id = match node {
            ClausewitzNode::Pair(_, value) => {
                query_atom(value, &["country"]).or_else(|| match value {
                    ClausewitzValue::Atom(s) if !s.is_empty() => Some(s.clone()),
                    _ => None,
                })
            }
            ClausewitzNode::Value(ClausewitzValue::Block(fields)) => fields.iter().find_map(|f| {
                if let ClausewitzNode::Pair(k, ClausewitzValue::Atom(v)) = f {
                    if k == "country" {
                        return Some(v.clone());
                    }
                }
                None
            }),
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

pub(super) fn count_espionage_operations_completed(
    country_value: &ClausewitzValue,
) -> Option<usize> {
    let flags = query_path(country_value, &["flags"])?;
    match flags {
        ClausewitzValue::Block(nodes) => Some(
            nodes
                .iter()
                .filter(|node| matches!(node, ClausewitzNode::Pair(key, _) if key.contains("espionage_operation_completed")))
                .count(),
        ),
        ClausewitzValue::Atom(_) => Some(0),
    }
}

pub(super) fn count_migration_treaties(country_value: &ClausewitzValue) -> Option<usize> {
    let agreements = query_path(country_value, &["agreements"])?;
    match agreements {
        ClausewitzValue::Block(nodes) => Some(
            nodes
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
                .count(),
        ),
        _ => Some(0),
    }
}

pub(super) fn count_proxy_wars(country_value: &ClausewitzValue) -> Option<usize> {
    let flags = query_path(country_value, &["flags"])?;
    match flags {
        ClausewitzValue::Block(nodes) => Some(
            nodes
                .iter()
                .filter(|node| matches!(node, ClausewitzNode::Pair(key, _) if key.contains("proxy_war")))
                .count(),
        ),
        ClausewitzValue::Atom(_) => Some(0),
    }
}

#[cfg(test)]
mod tests {
    use super::super::{extract_action_facts, test_support::*};

    #[test]
    fn test_federation_membership() {
        let country_in_fed = make_block(vec![
            ("federation", make_atom("16777220")),
            ("flags", make_block(vec![])),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country_in_fed, "0");
        assert_eq!(facts.federation_formed, Some(true));

        let facts = extract_action_facts(&make_game_root(vec![]), &make_country(vec![]), "0");
        assert_eq!(facts.federation_formed, Some(false));
    }

    #[test]
    fn test_war_participation_and_multiple_wars() {
        let participant = make_block(vec![
            ("country", make_atom("0")),
            ("call_type", make_atom("primary")),
        ]);
        let war1 = make_block(vec![
            ("attackers", make_value_block(vec![participant.clone()])),
            ("defenders", make_value_block(vec![])),
            (
                "war_goal",
                make_block(vec![("type", make_atom("war_type_conquest"))]),
            ),
        ]);
        let war2 = make_block(vec![
            ("attackers", make_value_block(vec![])),
            ("defenders", make_value_block(vec![participant])),
            (
                "war_goal",
                make_block(vec![("type", make_atom("war_type_defense"))]),
            ),
        ]);
        let game = make_game_root(vec![(
            "war",
            make_block(vec![("war_1", war1), ("war_2", war2)]),
        )]);
        let facts = extract_action_facts(&game, &make_country(vec![]), "0");
        assert_eq!(facts.active_wars, Some(2));
        assert_eq!(facts.war_type, Some("war_type_conquest".to_string()));
    }

    #[test]
    fn test_galactic_community_and_federation_details() {
        let federation_entry = make_block(vec![
            ("type", make_atom("galactic_union")),
            ("level", make_atom("3")),
            (
                "members",
                make_value_block(vec![make_block(vec![("country", make_atom("1"))])]),
            ),
        ]);
        let game = make_block(vec![
            (
                "galactic_community",
                make_block(vec![
                    ("community_formed", make_atom("2200.01.01")),
                    ("custodian_term", make_atom("3")),
                    ("leader", make_atom("0")),
                ]),
            ),
            (
                "federation",
                make_block(vec![("16777220", federation_entry)]),
            ),
            (
                "country",
                make_block(vec![(
                    "1",
                    make_block(vec![(
                        "ethics",
                        make_value_block(vec![make_atom("ethic_militarist")]),
                    )]),
                )]),
            ),
        ]);
        let country = make_block(vec![
            ("federation", make_atom("16777220")),
            ("flags", make_block(vec![])),
        ]);
        let facts = extract_action_facts(&game, &country, "0");
        assert_eq!(facts.galactic_community_exists, Some(true));
        assert_eq!(facts.galactic_custodian, Some(true));
        assert_eq!(facts.federation_type, Some("galactic_union".to_string()));
        assert_eq!(facts.federation_level, Some(3));
        assert_eq!(
            facts.federation_member_ethics,
            vec!["ethic_militarist".to_string()]
        );
    }

    #[test]
    fn test_subject_and_diplomacy_extractors() {
        let country = make_block(vec![
            (
                "subjects",
                make_block(vec![
                    ("s1", make_block(vec![("type", make_atom("vassal"))])),
                    ("s2", make_block(vec![("type", make_atom("bulwark"))])),
                ]),
            ),
            (
                "agreements",
                make_block(vec![(
                    "a1",
                    make_block(vec![("type", make_atom("migration_treaty"))]),
                )]),
            ),
            (
                "flags",
                make_block(vec![
                    ("recruit_curator_scientist", make_atom("yes")),
                    ("proxy_war_won", make_atom("yes")),
                    ("espionage_operation_completed_01", make_atom("yes")),
                ]),
            ),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.subjects_acquired, Some(2));
        assert_eq!(facts.vassal_count, Some(2));
        assert!(facts.subject_type.unwrap().contains("vassal"));
        assert!(facts
            .enclave_interaction_type
            .unwrap()
            .contains("recruit_curator_scientist"));
        assert_eq!(facts.migration_treaty_count, Some(1));
        assert_eq!(facts.proxy_war_count, Some(1));
        assert_eq!(facts.espionage_operations_completed, Some(1));
    }

    #[test]
    fn test_secret_fealty_detection() {
        for flag in &["secret_fealty_pledged", "secret_fealty", "fealty_pledged"] {
            let facts = extract_action_facts(
                &make_game_root(vec![]),
                &make_country(vec![(flag, "yes")]),
                "0",
            );
            assert_eq!(facts.secret_fealty_pledged, Some(true));
        }
    }
}
