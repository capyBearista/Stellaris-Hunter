use super::*;

pub(super) fn extract_relics_owned(country_value: &ClausewitzValue) -> Option<String> {
    let relics = collect_space_list(country_value, "relics");
    if relics.is_empty() {
        None
    } else {
        Some(relics.join(", "))
    }
}

pub(super) fn extract_galatron(country_value: &ClausewitzValue) -> Option<bool> {
    Some(field_contains(country_value, "relics", "r_galatron"))
}

pub(super) fn extract_relic_active_effect_used(country_value: &ClausewitzValue) -> Option<String> {
    let flags = query_path(country_value, &["flags"])?;
    let has_relic_flag = match flags {
        ClausewitzValue::Block(nodes) => nodes.iter().any(|node| {
            matches!(node, ClausewitzNode::Pair(key, _) if key == "relic_active_effect_used" || key.contains("relic_active"))
        }),
        ClausewitzValue::Atom(_) => false,
    };
    if has_relic_flag {
        Some("true".to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::super::{extract_action_facts, test_support::*};

    #[test]
    fn test_relic_ownership() {
        let country = make_block(vec![
            (
                "relics",
                make_value_block(vec![make_atom("r_galatron"), make_atom("r_dragon_trophy")]),
            ),
            ("flags", make_block(vec![])),
        ]);
        let facts = extract_action_facts(&make_game_root(vec![]), &country, "0");
        assert!(facts.relic_owned.unwrap().contains("r_galatron"));
        assert_eq!(facts.galatron_acquired, Some(true));
    }

    #[test]
    fn test_relic_active_effect_used() {
        let facts = extract_action_facts(
            &make_game_root(vec![]),
            &make_country(vec![("relic_active_effect_used", "yes")]),
            "0",
        );
        assert_eq!(facts.relic_active_effect_used, Some("true".to_string()));
        let facts = extract_action_facts(&make_game_root(vec![]), &make_country(vec![]), "0");
        assert!(facts.relic_active_effect_used.is_none());
    }
}
