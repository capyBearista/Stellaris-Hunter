use super::*;

pub(super) fn extract_crisis_path_nemesis(country_value: &ClausewitzValue) -> Option<bool> {
    let via_flag = has_any_flag(country_value, &["became_crisis", "crisis_path_nemesis"]);
    let via_perk = field_contains(country_value, "ascension_perks", "ap_crisis_acolyte");
    Some(via_flag || via_perk)
}

pub(super) fn extract_crisis_path_cosmogenesis(country_value: &ClausewitzValue) -> Option<bool> {
    Some(has_any_flag(
        country_value,
        &["cosmogenesis", "crisis_path_cosmogenesis"],
    ))
}

pub(super) fn extract_colossus_weapon_type(country_value: &ClausewitzValue) -> Option<String> {
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

pub(super) fn count_purged_pops(game_root: &ClausewitzValue) -> Option<usize> {
    sum_pop_groups_by_category(game_root, "purge")
}

#[cfg(test)]
mod tests {
    use super::super::{extract_action_facts, test_support::*};

    #[test]
    fn test_colossus_and_crisis_extractors() {
        let country = make_block(vec![
            (
                "ascension_perks",
                make_atom("ap_colossus ap_crisis_acolyte"),
            ),
            (
                "flags",
                make_block(vec![
                    ("colossus_weapon_world_cracker", make_atom("yes")),
                    ("cosmogenesis", make_atom("yes")),
                ]),
            ),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert_eq!(facts.colossus_built, Some(true));
        assert_eq!(
            facts.colossus_weapon_type,
            Some("world_cracker".to_string())
        );
        assert_eq!(facts.crisis_path_nemesis, Some(true));
        assert_eq!(facts.crisis_path_cosmogenesis, Some(true));
    }

    #[test]
    fn test_purged_pops_count() {
        let game = make_block(vec![(
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
                                ("category", make_atom("purge")),
                            ]),
                        ),
                        ("size", make_atom("3")),
                    ]),
                ),
            ]),
        )]);
        let facts = extract_action_facts(&game, &make_country(vec![]), "0");
        assert_eq!(facts.purged_pops, Some(8));
    }
}
