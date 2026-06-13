// Allow dead_code: this module's functions are not yet wired into the save
// parse pipeline, but will be once fact extraction is integrated into
// save parsing. Tests below verify correctness.
#![allow(dead_code)]

/// Extract discovery facts (galaxy-generation-dependent) from a parsed
/// Stellaris gamestate AST.
///
/// Each function below targets a specific dimension documented in
/// [`DiscoveryFacts`] (see `model.rs`). The strategies use:
///
/// - **Country flags** (`country > flags`): Key-value pairs where keys are
///   flag names (e.g., `crisis_contingency`, `l_cluster_unlocked`) and values
///   are timestamps. Flag existence signals the event happened.
///
/// - **Archaeological sites** (`archaeological_sites > sites`): Site entries
///   keyed by numeric ID, each containing a `type` field (e.g.,
///   `site_zroni_01`) and `last_excavator_country`.
///
/// - **Ambient objects** (`ambient_object`): Map of entity ID to type (e.g.,
///   `ether_drake`, `stellarite`) for leviathan detection.
///
/// - **First contacts** (`first_contacts > contacts`): Contact entries with
///   pre-FTL civilization data.
///
/// - **Galactic objects** (`galactic_object`): Star system and entity entries,
///   used for enclave detection.
///
/// - **Bypasses** (`bypasses`, `usable_bypasses`): Gateway/wormhole IDs for
///   L-cluster detection.
use crate::model::DiscoveryFacts;
use crate::save::*;

/// Extract all discovery facts from the gamestate.
///
/// `game_root` is the full parsed gamestate (top-level block).
/// `country_value` is the parsed block for the player country (country ID 0).
pub(crate) fn extract_discovery_facts(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> DiscoveryFacts {
    DiscoveryFacts {
        endgame_crisis: extract_endgame_crisis(game_root, country_value),
        sol_system_era: extract_sol_system_era(game_root, country_value),
        primitive_earth_present: extract_primitive_earth_present(game_root, country_value),
        pre_ftl_era_target: extract_pre_ftl_era_target(game_root, country_value),
        target_species_class: extract_target_species_class(game_root, country_value),
        target_homeworld_class: extract_target_homeworld_class(game_root, country_value),
        precursor_type: extract_precursor_type(game_root, country_value),
        precursor_chain_completed: extract_precursor_chain_completed(game_root, country_value),
        l_cluster_unlocked: extract_l_cluster_unlocked(game_root, country_value),
        shielded_world_unlocked: extract_shielded_world_unlocked(game_root, country_value),
        ancient_leviathan: extract_ancient_leviathan(game_root, country_value),
        enclave_type_present: extract_enclave_type_present(game_root, country_value),
        great_khan_spawned: extract_great_khan_spawned(game_root, country_value),
    }
}

// ── Flag helpers ────────────────────────────────────────────────────────────

/// Get the flags block from a country value, if present.
fn country_flags(country_value: &ClausewitzValue) -> Option<&ClausewitzValue> {
    query_path(country_value, &["flags"])
}

/// Check if a specific flag exists by exact name.
#[allow(dead_code)]
fn has_flag(country_value: &ClausewitzValue, name: &str) -> bool {
    query_path(country_value, &["flags", name]).is_some()
}

/// Find the first flag whose key is one of the given exact names.
/// Returns the matching flag key name.
fn find_any_flag(country_value: &ClausewitzValue, names: &[&str]) -> Option<String> {
    let flags = country_flags(country_value)?;
    match flags {
        ClausewitzValue::Block(nodes) => {
            for node in nodes {
                if let ClausewitzNode::Pair(key, _) = node {
                    if names.contains(&key.as_str()) {
                        return Some(key.clone());
                    }
                }
            }
            None
        }
        ClausewitzValue::Atom(_) => None,
    }
}

/// Find the first flag whose key contains any of the given substrings.
/// Returns the matching flag key name.
fn find_flag_containing(country_value: &ClausewitzValue, substrings: &[&str]) -> Option<String> {
    let flags = country_flags(country_value)?;
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

// ── Dimension extractors ───────────────────────────────────────────────────

/// Extract the endgame crisis type from country flags.
///
/// Strategy: Check country flags for known crisis-type flag names such as
/// `crisis_contingency`, `crisis_prethoryn`, `crisis_unbidden`, or
/// `crisis_cetana`. Falls back to broader substring matching for any flag
/// containing "crisis".
fn extract_endgame_crisis(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<String> {
    let _ = game_root;

    // Exact flag names for known endgame crises
    if let Some(name) = find_any_flag(
        country_value,
        &[
            "crisis_contingency",
            "crisis_prethoryn",
            "crisis_unbidden",
            "crisis_cetana",
            "endgame_crisis",
            "contingency_spawned",
            "prethoryn_spawned",
            "unbidden_spawned",
        ],
    ) {
        return Some(name);
    }

    // Broader substring fallback
    find_flag_containing(country_value, &["crisis"])
}

/// Extract the Sol system era from country flags.
///
/// Strategy: Check for flags containing "sol_era" such as
/// `sol_era_industrial`, `sol_era_early_space`, etc., indicating what stage
/// of development the Sol system has reached.
fn extract_sol_system_era(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<String> {
    let _ = game_root;
    find_flag_containing(country_value, &["sol_era"])
}

/// Check whether primitive Earth is present in the galaxy.
///
/// Strategy: Check country flags for indicators of a primitive Earth
/// civilization, such as `primitive_earth`, `earth_primitive`, or
/// `sol_primitive`.
fn extract_primitive_earth_present(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<bool> {
    let _ = game_root;
    find_flag_containing(
        country_value,
        &["primitive_earth", "earth_primitive", "sol_primitive"],
    )
    .map(|_| true)
}

/// Extract the current era of a targeted pre-FTL civilization.
///
/// Strategy: Scan `first_contacts > contacts` for pre-FTL / primitive type
/// contacts and extract their `stage` or `era` field. This is in support of
/// achievements that require observing pre-FTL civilizations at a specific
/// technological stage.
fn extract_pre_ftl_era_target(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<String> {
    let _ = country_value;
    let contacts = query_path(game_root, &["first_contacts", "contacts"])?;
    match contacts {
        ClausewitzValue::Block(nodes) => {
            for node in nodes {
                if let ClausewitzNode::Pair(_, contact_block) = node {
                    // Check if this is a pre-FTL contact type
                    let contact_type = query_atom(contact_block, &["type"])?;
                    if contact_type.contains("pre_ftl") || contact_type.contains("primitive") {
                        // Return the stage/era if present
                        let stage = query_atom(contact_block, &["stage"])
                            .or_else(|| query_atom(contact_block, &["era"]));
                        if stage.is_some() {
                            return stage;
                        }
                    }
                }
            }
            None
        }
        ClausewitzValue::Atom(_) => None,
    }
}

/// Extract the species class of a targeted first-contact civilization.
///
/// Strategy: Scan `first_contacts > contacts` for species with a `class`
/// field inside their `species` sub-block.
fn extract_target_species_class(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<String> {
    let _ = country_value;
    let contacts = query_path(game_root, &["first_contacts", "contacts"])?;
    match contacts {
        ClausewitzValue::Block(nodes) => {
            for node in nodes {
                if let ClausewitzNode::Pair(_, contact_block) = node {
                    if let Some(species) = query_path(contact_block, &["species"]) {
                        if let Some(class) = query_atom(species, &["class"]) {
                            return Some(class);
                        }
                    }
                }
            }
            None
        }
        ClausewitzValue::Atom(_) => None,
    }
}

/// Extract the homeworld planet class of a targeted first-contact civilization.
///
/// Strategy: Scan `first_contacts > contacts` and look for a `homeworld`
/// sub-block with a `class` field.
fn extract_target_homeworld_class(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<String> {
    let _ = country_value;
    let contacts = query_path(game_root, &["first_contacts", "contacts"])?;
    match contacts {
        ClausewitzValue::Block(nodes) => {
            for node in nodes {
                if let ClausewitzNode::Pair(_, contact_block) = node {
                    if let Some(homeworld) = query_path(contact_block, &["homeworld"]) {
                        if let Some(class) = query_atom(homeworld, &["class"]) {
                            return Some(class);
                        }
                    }
                }
            }
            None
        }
        ClausewitzValue::Atom(_) => None,
    }
}

/// Determine which precursor civilization is active in the galaxy.
///
/// Strategy: Scan `archaeological_sites > sites` and check the site `type`
/// for known precursor prefixes (`site_zroni`, `site_vultaum`, etc.). Return
/// the site type prefix (e.g., `site_zroni`) for any site that has
/// `last_excavator_country` matching the player country ID.
fn extract_precursor_type(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<String> {
    let _ = country_value;
    let player_id = query_atom(game_root, &["player", "country"])?;
    let sites = query_path(game_root, &["archaeological_sites", "sites"])?;
    let precursors = &[
        "site_zroni",
        "site_vultaum",
        "site_yuht",
        "site_league",
        "site_irassian",
        "site_cybrex",
    ];

    match sites {
        ClausewitzValue::Block(nodes) => {
            for node in nodes {
                if let ClausewitzNode::Pair(_, site_value) = node {
                    let site_type = query_atom(site_value, &["type"])?;

                    // Only consider sites the player has worked on
                    let excavator = query_atom(site_value, &["last_excavator_country"]);
                    if excavator.as_deref() != Some(player_id.as_str()) {
                        continue;
                    }

                    for prefix in precursors {
                        if site_type.starts_with(prefix) {
                            return Some(prefix.to_string());
                        }
                    }
                }
            }
            None
        }
        ClausewitzValue::Atom(_) => None,
    }
}

/// Check whether a full precursor chain has been completed by the player.
///
/// Strategy: Scan `archaeological_sites > sites` and collect all unique
/// precursor prefixes whose sites have `last_excavator_country` matching the
/// player. Return the first such precursor prefix. A more precise check
/// (validating all chain sites are completed) would need a lookup table.
fn extract_precursor_chain_completed(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<String> {
    let _ = country_value;
    let player_id = query_atom(game_root, &["player", "country"])?;
    let sites = query_path(game_root, &["archaeological_sites", "sites"])?;
    let precursors = &[
        "site_zroni",
        "site_vultaum",
        "site_yuht",
        "site_league",
        "site_irassian",
        "site_cybrex",
    ];

    let mut completed: Vec<String> = Vec::new();

    if let ClausewitzValue::Block(nodes) = sites {
        for node in nodes {
            if let ClausewitzNode::Pair(_, site_value) = node {
                let site_type = query_atom(site_value, &["type"])?;

                let mut chain = None;
                for prefix in precursors {
                    if site_type.starts_with(prefix) {
                        chain = Some(prefix.to_string());
                        break;
                    }
                }
                let chain = chain?;

                let excavator = query_atom(site_value, &["last_excavator_country"]);
                if excavator.as_deref() == Some(player_id.as_str()) && !completed.contains(&chain) {
                    completed.push(chain);
                }
            }
        }
    }

    completed.into_iter().next()
}

/// Check whether the L-cluster has been unlocked.
///
/// Strategy: Check country flags for known L-cluster indicators
/// (`l_cluster_unlocked`, `terminal_egress`) or flags containing
/// "l_cluster", "terminal_egress", or "l_gate".
fn extract_l_cluster_unlocked(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<bool> {
    let _ = game_root;
    find_any_flag(
        country_value,
        &["l_cluster_unlocked", "terminal_egress_unlocked"],
    )
    .or_else(|| find_flag_containing(country_value, &["l_cluster", "terminal_egress", "l_gate"]))
    .map(|_| true)
}

/// Check whether a shielded world has been unlocked/encountered.
///
/// Strategy: Check country flags for indicators such as `shielded_world`,
/// `shield_world`, or any flag containing "shielded".
fn extract_shielded_world_unlocked(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<bool> {
    let _ = game_root;
    find_flag_containing(
        country_value,
        &["shielded_world", "shield_world", "shielded"],
    )
    .map(|_| true)
}

/// Check which ancient leviathan type is present (or was encountered).
///
/// Strategy: Scan `ambient_object` for entity types matching known leviathan
/// names (ether_drake, stellarite, dimensional_horror, void_spawn,
/// infinity_machine, spectre, scavenger_bot, automated_dreadnought,
/// enigmatic_fortress, wraith, tiyanki).
fn extract_ancient_leviathan(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<String> {
    let _ = country_value;

    let ambient = query_path(game_root, &["ambient_object"])?;
    let leviathan_patterns = &[
        "ether_drake",
        "stellarite",
        "dimensional_horror",
        "void_spawn",
        "infinity_machine",
        "spectre",
        "scavenger_bot",
        "automated_dreadnought",
        "enigmatic_fortress",
        "wraith",
        "tiyanki",
    ];

    match ambient {
        ClausewitzValue::Block(nodes) => {
            for node in nodes {
                if let ClausewitzNode::Pair(_, obj_value) = node {
                    if let Some(obj_type) = query_atom(obj_value, &["type"]) {
                        for pattern in leviathan_patterns {
                            if obj_type.contains(pattern) {
                                return Some(obj_type);
                            }
                        }
                    }
                }
            }
            None
        }
        ClausewitzValue::Atom(_) => None,
    }
}

/// Check which enclave type is present in the galaxy.
///
/// Strategy: Scan `galactic_object` for entries whose `type` field contains
/// an enclave identifier (`curator`, `trader`, `artisan`, `mercenary`).
/// Also checks country flags as a fallback.
fn extract_enclave_type_present(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<String> {
    let enclave_keywords = &["curator", "trader", "artisan", "mercenary"];

    // Check galactic objects for enclave types
    if let Some(ClausewitzValue::Block(nodes)) = query_path(game_root, &["galactic_object"]) {
        for node in nodes {
            if let ClausewitzNode::Pair(_, obj_value) = node {
                if let Some(obj_type) = query_atom(obj_value, &["type"]) {
                    for enclave in enclave_keywords {
                        if obj_type.contains(enclave) {
                            return Some(enclave.to_string());
                        }
                    }
                }
            }
        }
    }

    // Fallback: check country flags
    find_flag_containing(country_value, enclave_keywords)
}

/// Check whether the Great Khan has spawned.
///
/// Strategy: Check country flags for `great_khan_spawned`, `great_khan`,
/// `khan_spawned`, or any flag containing "great_khan" or "khan".
fn extract_great_khan_spawned(
    game_root: &ClausewitzValue,
    country_value: &ClausewitzValue,
) -> Option<bool> {
    let _ = game_root;
    find_any_flag(
        country_value,
        &["great_khan_spawned", "great_khan", "khan_spawned"],
    )
    .or_else(|| find_flag_containing(country_value, &["great_khan", "khan"]))
    .map(|_| true)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::{ClausewitzNode, ClausewitzValue};

    /// Convenience: create a ClausewitzValue::Atom from a &str.
    fn atom(s: &str) -> ClausewitzValue {
        ClausewitzValue::Atom(s.to_string())
    }

    /// Convenience: create a ClausewitzValue::Block from key-value pairs.
    fn block(pairs: Vec<(&str, ClausewitzValue)>) -> ClausewitzValue {
        ClausewitzValue::Block(
            pairs
                .into_iter()
                .map(|(k, v)| ClausewitzNode::Pair(k.to_string(), v))
                .collect(),
        )
    }

    /// Convenience: create a ClausewitzValue::Block from value-only nodes.
    fn value_block(values: Vec<ClausewitzValue>) -> ClausewitzValue {
        ClausewitzValue::Block(values.into_iter().map(ClausewitzNode::Value).collect())
    }

    // ── empty_state: all None ──────────────────────────────────────────────

    #[test]
    fn test_all_none_when_no_relevant_data() {
        let country = block(vec![]);
        let game = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.endgame_crisis, None);
        assert_eq!(facts.sol_system_era, None);
        assert_eq!(facts.primitive_earth_present, None);
        assert_eq!(facts.pre_ftl_era_target, None);
        assert_eq!(facts.target_species_class, None);
        assert_eq!(facts.target_homeworld_class, None);
        assert_eq!(facts.precursor_type, None);
        assert_eq!(facts.precursor_chain_completed, None);
        assert_eq!(facts.l_cluster_unlocked, None);
        assert_eq!(facts.shielded_world_unlocked, None);
        assert_eq!(facts.ancient_leviathan, None);
        assert_eq!(facts.enclave_type_present, None);
        assert_eq!(facts.great_khan_spawned, None);
    }

    // ── endgame_crisis from flags ──────────────────────────────────────────

    #[test]
    fn test_endgame_crisis_from_flags() {
        let country = block(vec![(
            "flags",
            block(vec![("crisis_contingency", atom("70000000"))]),
        )]);
        let game = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.endgame_crisis, Some("crisis_contingency".to_string()));
    }

    #[test]
    fn test_endgame_crisis_substring_fallback() {
        let country = block(vec![(
            "flags",
            block(vec![("some_crisis_flag", atom("70000000"))]),
        )]);
        let game = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.endgame_crisis, Some("some_crisis_flag".to_string()));
    }

    // ── great_khan_spawned from flags ──────────────────────────────────────

    #[test]
    fn test_great_khan_spawned_from_flags() {
        let country = block(vec![(
            "flags",
            block(vec![("great_khan_spawned", atom("70000000"))]),
        )]);
        let game = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.great_khan_spawned, Some(true));
    }

    #[test]
    fn test_great_khan_not_spawned() {
        let country = block(vec![(
            "flags",
            block(vec![("first_colony", atom("70000000"))]),
        )]);
        let game = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.great_khan_spawned, None);
    }

    // ── l_cluster_unlocked from flags ──────────────────────────────────────

    #[test]
    fn test_l_cluster_unlocked() {
        let country = block(vec![(
            "flags",
            block(vec![("l_cluster_unlocked", atom("70000000"))]),
        )]);
        let game = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.l_cluster_unlocked, Some(true));
    }

    #[test]
    fn test_l_cluster_terminal_egress() {
        let country = block(vec![(
            "flags",
            block(vec![("terminal_egress_unlocked", atom("70000000"))]),
        )]);
        let game = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.l_cluster_unlocked, Some(true));
    }

    // ── shielded_world from flags ──────────────────────────────────────────

    #[test]
    fn test_shielded_world_unlocked() {
        let country = block(vec![(
            "flags",
            block(vec![("shielded_world_unlocked", atom("70000000"))]),
        )]);
        let game = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.shielded_world_unlocked, Some(true));
    }

    // ── primitive_earth from flags ─────────────────────────────────────────

    #[test]
    fn test_primitive_earth_present() {
        let country = block(vec![(
            "flags",
            block(vec![("primitive_earth", atom("70000000"))]),
        )]);
        let game = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.primitive_earth_present, Some(true));
    }

    // ── sol_system_era from flags ──────────────────────────────────────────

    #[test]
    fn test_sol_system_era() {
        let country = block(vec![(
            "flags",
            block(vec![("sol_era_industrial", atom("70000000"))]),
        )]);
        let game = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.sol_system_era, Some("sol_era_industrial".to_string()));
    }

    // ── precursor_type from archaeological sites ───────────────────────────

    #[test]
    fn test_precursor_type_from_archaeological_sites() {
        // Build: archaeological_sites = { sites = { 001 = { type = "site_zroni_01"
        //   last_excavator_country = 0 } } }
        // And: player = { country = 0 }
        let site = block(vec![
            ("type", atom("site_zroni_01")),
            ("last_excavator_country", atom("0")),
        ]);
        let sites = block(vec![("001", site)]);
        let arch_sites = block(vec![("sites", sites)]);

        let game = block(vec![
            ("archaeological_sites", arch_sites),
            ("player", block(vec![("country", atom("0"))])),
        ]);
        let country = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.precursor_type, Some("site_zroni".to_string()));
    }

    #[test]
    fn test_precursor_type_not_mine() {
        // Site exists but player hasn't excavated it
        let site = block(vec![
            ("type", atom("site_vultaum_01")),
            ("last_excavator_country", atom("1")), // not player
        ]);
        let sites = block(vec![("001", site)]);
        let arch_sites = block(vec![("sites", sites)]);

        let game = block(vec![
            ("archaeological_sites", arch_sites),
            ("player", block(vec![("country", atom("0"))])),
        ]);
        let country = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.precursor_type, None);
    }

    // ── precursor_chain_completed ──────────────────────────────────────────

    #[test]
    fn test_precursor_chain_completed() {
        // One Zroni site completed by player
        let site = block(vec![
            ("type", atom("site_zroni_01")),
            ("last_excavator_country", atom("0")),
        ]);
        let sites = block(vec![("001", site)]);
        let arch_sites = block(vec![("sites", sites)]);

        let game = block(vec![
            ("archaeological_sites", arch_sites),
            ("player", block(vec![("country", atom("0"))])),
        ]);
        let country = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        // Returns the first chain with a completed site
        assert_eq!(
            facts.precursor_chain_completed,
            Some("site_zroni".to_string())
        );
    }

    // ── ancient_leviathan from ambient_object ──────────────────────────────

    #[test]
    fn test_ancient_leviathan_from_ambient() {
        let drake = block(vec![("type", atom("ether_drake"))]);
        let ambient = block(vec![("entity_001", drake)]);

        let game = block(vec![("ambient_object", ambient)]);
        let country = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.ancient_leviathan, Some("ether_drake".to_string()));
    }

    #[test]
    fn test_ancient_leviathan_when_empty() {
        let game = block(vec![("ambient_object", block(vec![]))]);
        let country = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.ancient_leviathan, None);
    }

    // ── enclave_type_present from galactic_object ──────────────────────────

    #[test]
    fn test_enclave_type_present() {
        let curator = block(vec![("type", atom("curator_enclave"))]);
        let gal_objects = block(vec![("entity_001", curator)]);

        let game = block(vec![("galactic_object", gal_objects)]);
        let country = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.enclave_type_present, Some("curator".to_string()));
    }

    #[test]
    fn test_enclave_type_present_from_flags() {
        let country = block(vec![(
            "flags",
            block(vec![("curator_enclave_discovered", atom("70000000"))]),
        )]);
        let game = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(
            facts.enclave_type_present,
            Some("curator_enclave_discovered".to_string())
        );
    }

    // ── first_contacts extraction ──────────────────────────────────────────

    #[test]
    fn test_pre_ftl_era_from_contacts() {
        let contact = block(vec![
            ("type", atom("pre_ftl_civilization")),
            ("stage", atom("early_space_age")),
        ]);
        let contacts = block(vec![("contact_001", contact)]);
        let first_contacts = block(vec![("contacts", contacts)]);

        let game = block(vec![("first_contacts", first_contacts)]);
        let country = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(
            facts.pre_ftl_era_target,
            Some("early_space_age".to_string())
        );
    }

    #[test]
    fn test_target_species_class_from_contacts() {
        let species = block(vec![("class", atom("MAM"))]);
        let contact = block(vec![
            ("type", atom("pre_ftl_civilization")),
            ("species", species),
        ]);
        let contacts = block(vec![("contact_001", contact)]);
        let first_contacts = block(vec![("contacts", contacts)]);

        let game = block(vec![("first_contacts", first_contacts)]);
        let country = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(facts.target_species_class, Some("MAM".to_string()));
    }

    #[test]
    fn test_target_homeworld_class_from_contacts() {
        let homeworld = block(vec![("class", atom("pc_continental"))]);
        let contact = block(vec![
            ("type", atom("pre_ftl_civilization")),
            ("homeworld", homeworld),
        ]);
        let contacts = block(vec![("contact_001", contact)]);
        let first_contacts = block(vec![("contacts", contacts)]);

        let game = block(vec![("first_contacts", first_contacts)]);
        let country = block(vec![]);
        let facts = extract_discovery_facts(&game, &country);

        assert_eq!(
            facts.target_homeworld_class,
            Some("pc_continental".to_string())
        );
    }
}
