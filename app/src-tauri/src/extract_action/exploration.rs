use super::*;

pub(super) fn extract_archaeology_sites(
    game_root: &ClausewitzValue,
    player_country_id: &str,
) -> Option<String> {
    let sites_block = query_path(game_root, &["archaeological_sites", "sites"])?;
    let mut completed_types = Vec::new();
    if let ClausewitzValue::Block(nodes) = sites_block {
        for node in nodes {
            if let ClausewitzNode::Pair(_, site_value) = node {
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

pub(super) fn extract_space_fauna_type_captured(game_root: &ClausewitzValue) -> Option<String> {
    let mut types = Vec::new();
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

pub(super) fn extract_first_contact_result(game_root: &ClausewitzValue) -> Option<String> {
    let contacts = query_path(game_root, &["first_contacts", "contacts"])?;
    let mut results = Vec::new();
    if let ClausewitzValue::Block(nodes) = contacts {
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
    if results.is_empty() {
        None
    } else {
        Some(results.join(", "))
    }
}

pub(super) fn count_astral_rifts_explored(game_root: &ClausewitzValue) -> Option<usize> {
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

#[cfg(test)]
mod tests {
    use super::super::{extract_action_facts, test_support::*};

    #[test]
    fn test_archaeology_and_first_contact() {
        let game = make_block(vec![
            (
                "archaeological_sites",
                make_block(vec![(
                    "sites",
                    make_block(vec![(
                        "1",
                        make_block(vec![
                            ("type", make_atom("ancient_ruins")),
                            (
                                "completed",
                                make_value_block(vec![make_block(vec![(
                                    "country",
                                    make_atom("0"),
                                )])]),
                            ),
                        ]),
                    )]),
                )]),
            ),
            (
                "first_contacts",
                make_block(vec![(
                    "contacts",
                    make_block(vec![
                        ("c1", make_block(vec![("result", make_atom("peaceful"))])),
                        ("c2", make_block(vec![("result", make_atom("aggressive"))])),
                    ]),
                )]),
            ),
        ]);
        let facts = extract_action_facts(&game, &make_country(vec![]), "0");
        assert_eq!(
            facts.archaeological_site_completed,
            Some("ancient_ruins".to_string())
        );
        let contact = facts.first_contact_result.unwrap();
        assert!(contact.contains("peaceful"));
        assert!(contact.contains("aggressive"));
    }

    #[test]
    fn test_space_fauna_and_astral_rifts() {
        let game = make_block(vec![
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
            (
                "astral_rifts",
                make_block(vec![
                    ("0", make_block(vec![("explored", make_atom("yes"))])),
                    ("1", make_block(vec![("explored", make_atom("no"))])),
                    ("2", make_block(vec![("explored", make_atom("yes"))])),
                ]),
            ),
        ]);
        let facts = extract_action_facts(&game, &make_country(vec![]), "0");
        let fauna = facts.space_fauna_type_captured.unwrap();
        assert!(fauna.contains("space_dragon"));
        assert!(fauna.contains("cutholoid"));
        assert_eq!(facts.astral_rifts_explored, Some(2));
    }
}
