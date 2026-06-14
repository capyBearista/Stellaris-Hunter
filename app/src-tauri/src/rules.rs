use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::model::{
    AchievementCatalogEntry, AchievementCondition, ConditionEvaluation,
    PlannerAchievementEvaluation, RunAchievementUserStatus, RunFactSummary,
};

const STATUS_COMPLETED: &str = "Completed";
const STATUS_PLANNED: &str = "Planned";
const STATUS_POSSIBLE: &str = "Possible";
const STATUS_INCOMPATIBLE: &str = "Incompatible";
const STATUS_IMPOSSIBLE: &str = "Impossible";
const STATUS_UNKNOWN: &str = "Unknown";

#[derive(Debug, Clone)]
struct FactLookup {
    facts: HashMap<(String, String), Value>,
}

impl FactLookup {
    fn new(facts: &[RunFactSummary]) -> Self {
        let facts = facts
            .iter()
            .filter(|fact| fact.confidence == "high")
            .map(|fact| {
                (
                    (fact.dimension.clone(), fact.key.clone()),
                    fact.value.clone(),
                )
            })
            .collect();
        Self { facts }
    }

    fn find(&self, catalog_dimension: &str) -> Option<Value> {
        let (dimension, key) = fact_address(catalog_dimension)?;
        self.facts
            .get(&(dimension.to_string(), key.to_string()))
            .cloned()
    }
}

pub fn evaluate_planner_achievements(
    achievements: Vec<AchievementCatalogEntry>,
    facts: &[RunFactSummary],
    completed_achievement_ids: &HashSet<String>,
    user_statuses: &[RunAchievementUserStatus],
) -> Vec<PlannerAchievementEvaluation> {
    let fact_lookup = FactLookup::new(facts);
    let status_by_achievement: HashMap<&str, &str> = user_statuses
        .iter()
        .map(|status| (status.achievement_id.as_str(), status.user_status.as_str()))
        .collect();

    achievements
        .into_iter()
        .map(|achievement| {
            let completed = completed_achievement_ids.contains(&achievement.id);
            let user_status = status_by_achievement.get(achievement.id.as_str()).copied();
            evaluate_achievement(achievement, &fact_lookup, completed, user_status)
        })
        .collect()
}

fn evaluate_achievement(
    achievement: AchievementCatalogEntry,
    fact_lookup: &FactLookup,
    completed: bool,
    user_status: Option<&str>,
) -> PlannerAchievementEvaluation {
    let conditions = achievement
        .curation
        .conditions
        .iter()
        .map(|condition| evaluate_condition(condition, fact_lookup))
        .collect::<Vec<_>>();
    let warnings = achievement.curation.warnings.clone();
    let mut reasons = Vec::new();
    let computed_status = compute_status(&conditions, &mut reasons);
    let planned = user_status == Some("planned");
    let ignored = user_status == Some("ignored");

    let status = if completed {
        reasons.insert(
            0,
            "Achievement is marked completed by Steam state or local override.".to_string(),
        );
        STATUS_COMPLETED.to_string()
    } else if planned {
        reasons.insert(
            0,
            format!("Achievement is planned for this run; compatibility is {computed_status}."),
        );
        STATUS_PLANNED.to_string()
    } else {
        computed_status.clone()
    };

    PlannerAchievementEvaluation {
        achievement,
        status,
        computed_status,
        planned,
        ignored,
        reasons,
        warnings,
        conditions,
    }
}

fn evaluate_condition(
    condition: &AchievementCondition,
    fact_lookup: &FactLookup,
) -> ConditionEvaluation {
    let fact_value = fact_lookup.find(&condition.dimension);
    let (passed, reason) = match fact_value.as_ref() {
        Some(value) => evaluate_operator(&condition.operator, value, &condition.value),
        None => (
            None,
            format!(
                "No high-confidence parsed fact currently maps to catalog dimension '{}'.",
                condition.dimension
            ),
        ),
    };

    ConditionEvaluation {
        dimension: condition.dimension.clone(),
        operator: condition.operator.clone(),
        condition_value: condition.value.clone(),
        fact_value,
        passed,
        severity: condition.severity.clone(),
        timing: condition.timing.clone(),
        mutability: condition.mutability.clone(),
        reason,
    }
}

fn evaluate_operator(
    operator: &str,
    fact_value: &Value,
    condition_value: &Value,
) -> (Option<bool>, String) {
    match operator {
        "equals" => (
            Some(values_equal(fact_value, condition_value)),
            format!("Parsed value is {}.", display_value(fact_value)),
        ),
        "contains" => match fact_value.as_array() {
            Some(values) => (
                Some(
                    values
                        .iter()
                        .any(|value| values_equal(value, condition_value)),
                ),
                format!("Parsed values are {}.", display_value(fact_value)),
            ),
            None => (
                None,
                format!(
                    "Parsed value {} is not a list, so 'contains' cannot be evaluated.",
                    display_value(fact_value)
                ),
            ),
        },
        "at_least" => compare_numbers(fact_value, condition_value, |fact, required| {
            fact >= required
        }),
        "greater_than" => compare_numbers(fact_value, condition_value, |fact, required| {
            fact > required
        }),
        "less_than" => compare_numbers(fact_value, condition_value, |fact, required| {
            fact < required
        }),
        "not_equals" => {
            let eq = values_equal(fact_value, condition_value);
            (
                Some(!eq),
                if eq {
                    format!(
                        "Parsed value {} unexpectedly equals condition.",
                        display_value(fact_value)
                    )
                } else {
                    format!("Parsed value is {}.", display_value(fact_value))
                },
            )
        }
        other => (None, format!("Unsupported condition operator '{other}'.")),
    }
}

fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::String(left), Value::String(right)) => left.eq_ignore_ascii_case(right),
        _ => left == right,
    }
}

fn compare_numbers(
    fact_value: &Value,
    condition_value: &Value,
    compare: impl FnOnce(f64, f64) -> bool,
) -> (Option<bool>, String) {
    match (fact_value.as_f64(), condition_value.as_f64()) {
        (Some(fact), Some(required)) => (
            Some(compare(fact, required)),
            format!("Parsed numeric value is {fact}."),
        ),
        _ => (
            None,
            format!(
                "Cannot compare non-numeric values: parsed {}, required {}.",
                display_value(fact_value),
                display_value(condition_value)
            ),
        ),
    }
}

fn compute_status(conditions: &[ConditionEvaluation], reasons: &mut Vec<String>) -> String {
    if conditions.is_empty() {
        reasons.push(
            "Catalog entry has no curated conditions yet, so no hard blocker is known.".to_string(),
        );
        return STATUS_POSSIBLE.to_string();
    }

    let mut has_unknown = false;
    let mut has_incompatibility = false;

    for condition in conditions {
        match condition.passed {
            Some(true) => {}
            Some(false) if is_impossible_blocker(condition) => {
                reasons.push(format!(
                    "{} requirement failed and is treated as immutable hard blocker: {}",
                    condition.dimension, condition.reason
                ));
                return STATUS_IMPOSSIBLE.to_string();
            }
            Some(false) if condition.severity == "hard" => {
                has_incompatibility = true;
                reasons.push(format!(
                    "{} hard requirement is not currently satisfied: {}",
                    condition.dimension, condition.reason
                ));
            }
            Some(false) => {
                has_incompatibility = true;
                reasons.push(format!(
                    "{} requirement is not currently satisfied: {}",
                    condition.dimension, condition.reason
                ));
            }
            None => {
                has_unknown = true;
                reasons.push(format!(
                    "{} is unknown: {}",
                    condition.dimension, condition.reason
                ));
            }
        }
    }

    if has_unknown {
        STATUS_UNKNOWN.to_string()
    } else if has_incompatibility {
        STATUS_INCOMPATIBLE.to_string()
    } else {
        reasons.push("All currently supported curated conditions are satisfied.".to_string());
        STATUS_POSSIBLE.to_string()
    }
}

fn is_impossible_blocker(condition: &ConditionEvaluation) -> bool {
    condition.severity == "hard"
        && matches!(
            condition.mutability.as_str(),
            "immutable" | "rng_locked" | "event_limited"
        )
        && matches!(
            condition.timing.as_str(),
            "setup" | "terminal" | "event_limited"
        )
}

#[rustfmt::skip]
fn fact_address(catalog_dimension: &str) -> Option<(&'static str, &'static str)> {
    match catalog_dimension {
        // ── Existing setup dimensions (6) ──
        "authority" => Some(("empire", "authority")),
        "civic" => Some(("empire", "civics")),
        "ethic" => Some(("empire", "ethics")),
        "origin" => Some(("empire", "origin")),
        "government_type" => Some(("empire", "government_type")),
        "species_class" => Some(("species", "founder_species_class")),
        "species_trait" => Some(("species", "founder_species_traits")),
        "portrait" | "species_portrait" => Some(("species", "founder_species_portrait")),
        "required_dlc" | "dlc" | "dlc_required" => Some(("save", "required_dlcs")),
        "ironman" => Some(("eligibility", "ironman")),
        "cheated_on_save" => Some(("eligibility", "cheated_on_save")),

        // ── Discovery dimensions (13) ──
        "endgame_crisis" => Some(("discovery", "endgame_crisis")),
        "sol_system_era" => Some(("discovery", "sol_system_era")),
        "primitive_earth_present" => Some(("discovery", "primitive_earth_present")),
        "pre_ftl_era_target" => Some(("discovery", "pre_ftl_era_target")),
        "target_species_class" => Some(("discovery", "target_species_class")),
        "target_homeworld_class" => Some(("discovery", "target_homeworld_class")),
        "precursor_type" => Some(("discovery", "precursor_type")),
        "precursor_chain_completed" => Some(("discovery", "precursor_chain_completed")),
        "l_cluster_unlocked" => Some(("discovery", "l_cluster_unlocked")),
        "shielded_world_unlocked" => Some(("discovery", "shielded_world_unlocked")),
        "ancient_leviathan" => Some(("discovery", "ancient_leviathan")),
        "enclave_type_present" => Some(("discovery", "enclave_type_present")),
        "great_khan_spawned" => Some(("discovery", "great_khan_spawned")),

        // ── Progression dimensions (32) ──
        "owned_planets" => Some(("progression", "owned_planets")),
        "colonized_planets" => Some(("progression", "colonized_planets")),
        "surveyed_planets" => Some(("progression", "surveyed_planets")),
        "total_pops" => Some(("progression", "total_pops")),
        "enslaved_pops_count" => Some(("progression", "enslaved_pops_count")),
        "robot_pop_ratio" => Some(("progression", "robot_pop_ratio")),
        "energy_stored" => Some(("progression", "energy_stored")),
        "energy_monthly" => Some(("progression", "energy_monthly")),
        "minerals_monthly" => Some(("progression", "minerals_monthly")),
        "alloys_monthly" => Some(("progression", "alloys_monthly")),
        "trade_value_monthly" => Some(("progression", "trade_value_monthly")),
        "strategic_resources_types" => Some(("progression", "strategic_resources_types")),
        "organic_empires_remaining" => Some(("progression", "organic_empires_remaining")),
        "fleet_power" => Some(("progression", "fleet_power")),
        "fleet_count" => Some(("progression", "fleet_count")),
        "starbase_count" => Some(("progression", "starbase_count")),
        "gateway_count" => Some(("progression", "gateway_count")),
        "hyper_relay_count" => Some(("progression", "hyper_relay_count")),
        "rare_technologies_acquired" => Some(("progression", "rare_technologies_acquired")),
        "traditions_adopted" => Some(("progression", "traditions_adopted")),
        "ascension_perks_unlocked" => Some(("progression", "ascension_perks_unlocked")),
        "ascension_path" => Some(("progression", "ascension_path")),
        "years_played" => Some(("progression", "years_played")),
        "years_at_peace" => Some(("progression", "years_at_peace")),
        "diplomatic_weight" => Some(("progression", "diplomatic_weight")),
        "intel_level_count" => Some(("progression", "intel_level_count")),
        "observation_station_count" => Some(("progression", "observation_station_count")),
        "capital_building_level" => Some(("progression", "capital_building_level")),
        "living_standard" => Some(("progression", "living_standard")),
        "mercenary_enclaves_patroned" => Some(("progression", "mercenary_enclaves_patroned")),
        "vivarium_capacity" => Some(("progression", "vivarium_capacity")),
        "megastructure_types" => Some(("progression", "megastructure_types")),

        // ── Action dimensions (67) ──
        "active_wars" => Some(("action", "active_wars")),
        "war_type" => Some(("action", "war_type")),
        "subjects_acquired" => Some(("action", "subjects_acquired")),
        "vassal_count" => Some(("action", "vassal_count")),
        "subject_type" => Some(("action", "subject_type")),
        "subject_contract_modified" => Some(("action", "subject_contract_modified")),
        "secret_fealty_pledged" => Some(("action", "secret_fealty_pledged")),
        "proxy_war_count" => Some(("action", "proxy_war_count")),
        "federation_formed" => Some(("action", "federation_formed")),
        "federation_type" => Some(("action", "federation_type")),
        "federation_level" => Some(("action", "federation_level")),
        "federation_member_ethics" => Some(("action", "federation_member_ethics")),
        "galactic_community_exists" => Some(("action", "galactic_community_exists")),
        "galactic_community_founding_member" => Some(("action", "galactic_community_founding_member")),
        "galactic_custodian" => Some(("action", "galactic_custodian")),
        "galactic_custodian_action" => Some(("action", "galactic_custodian_action")),
        "galactic_emperor" => Some(("action", "galactic_emperor")),
        "galactic_emperor_rebellion" => Some(("action", "galactic_emperor_rebellion")),
        "colossus_built" => Some(("action", "colossus_built")),
        "colossus_weapon_type" => Some(("action", "colossus_weapon_type")),
        "colossus_destroyed_while_firing" => Some(("action", "colossus_destroyed_while_firing")),
        "species_genetically_modified" => Some(("action", "species_genetically_modified")),
        "species_uplifted" => Some(("action", "species_uplifted")),
        "species_on_planet_count" => Some(("action", "species_on_planet_count")),
        "species_dna_phenotypes_collected" => Some(("action", "species_dna_phenotypes_collected")),
        "slavery_type" => Some(("action", "slavery_type")),
        "livestock_species_count" => Some(("action", "livestock_species_count")),
        "purged_pops" => Some(("action", "purged_pops")),
        "species_enslaved" => Some(("action", "species_enslaved")),
        "relic_owned" => Some(("action", "relic_owned")),
        "relic_active_effect_used" => Some(("action", "relic_active_effect_used")),
        "galatron_acquired" => Some(("action", "galatron_acquired")),
        "galatron_captured" => Some(("action", "galatron_captured")),
        "archaeological_site_completed" => Some(("action", "archaeological_site_completed")),
        "wormhole_travel_completed" => Some(("action", "wormhole_travel_completed")),
        "pre_ftl_infiltration_completed" => Some(("action", "pre_ftl_infiltration_completed")),
        "first_contact_result" => Some(("action", "first_contact_result")),
        "espionage_operations_completed" => Some(("action", "espionage_operations_completed")),
        "astral_rifts_explored" => Some(("action", "astral_rifts_explored")),
        "crisis_defeated" => Some(("action", "crisis_defeated")),
        "captured_prethoryn_scourge_queen" => Some(("action", "captured_prethoryn_scourge_queen")),
        "crisis_path_nemesis" => Some(("action", "crisis_path_nemesis")),
        "crisis_path_cosmogenesis" => Some(("action", "crisis_path_cosmogenesis")),
        "crisis_path_hyperthermia" => Some(("action", "crisis_path_hyperthermia")),
        "crisis_path_behemoth_fury" => Some(("action", "crisis_path_behemoth_fury")),
        "machine_uprising_victory" => Some(("action", "machine_uprising_victory")),
        "amoeba_companion_found" => Some(("action", "amoeba_companion_found")),
        "amoeba_companion_killed" => Some(("action", "amoeba_companion_killed")),
        "artisan_enclave_patron" => Some(("action", "artisan_enclave_patron")),
        "enclave_interaction_type" => Some(("action", "enclave_interaction_type")),
        "migration_treaty_count" => Some(("action", "migration_treaty_count")),
        "legendary_paragon_recruited" => Some(("action", "legendary_paragon_recruited")),
        "robot_pop_built" => Some(("action", "robot_pop_built")),
        "horizon_signal_completed" => Some(("action", "horizon_signal_completed")),
        "civil_war_completed" => Some(("action", "civil_war_completed")),
        "special_project_completed_type" => Some(("action", "special_project_completed_type")),
        "covenant_type" => Some(("action", "covenant_type")),
        "psionic_techs_unlocked" => Some(("action", "psionic_techs_unlocked")),
        "quantum_catapult_used" => Some(("action", "quantum_catapult_used")),
        "blazing_scourge_decisions" => Some(("action", "blazing_scourge_decisions")),
        "stars_terraform_to_red_giant" => Some(("action", "stars_terraform_to_red_giant")),
        "planets_terraform_to_volcanic" => Some(("action", "planets_terraform_to_volcanic")),
        "volcanic_holy_world_created" => Some(("action", "volcanic_holy_world_created")),
        "galactic_memorials_on_tomb_worlds" => Some(("action", "galactic_memorials_on_tomb_worlds")),
        "space_fauna_type_captured" => Some(("action", "space_fauna_type_captured")),
        "colony_count_with_hyperspace_not_researched" => Some(("action", "colony_count_with_hyperspace_not_researched")),
        "invaded_primitive_earth" => Some(("action", "invaded_primitive_earth")),

        _ => None,
    }
}

fn display_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Array(values) => values
            .iter()
            .map(display_value)
            .collect::<Vec<_>>()
            .join(", "),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::model::{AchievementCurationFields, AchievementSourceFields};

    #[test]
    fn immutable_setup_blocker_is_impossible() {
        let result = evaluate_achievement(
            achievement_with_condition(AchievementCondition {
                condition_type: "required".to_string(),
                dimension: "species_class".to_string(),
                operator: "equals".to_string(),
                value: json!("LITH"),
                timing: "setup".to_string(),
                mutability: "immutable".to_string(),
                severity: "hard".to_string(),
                source: None,
                notes: None,
            }),
            &FactLookup::new(&[fact("species", "founder_species_class", json!("HUM"))]),
            false,
            None,
        );

        assert_eq!(result.status, STATUS_IMPOSSIBLE);
        assert_eq!(result.computed_status, STATUS_IMPOSSIBLE);
        assert_eq!(result.conditions[0].passed, Some(false));
        assert!(result.reasons[0].contains("immutable hard blocker"));
    }

    #[test]
    fn missing_supported_fact_is_unknown() {
        let result = evaluate_achievement(
            achievement_with_condition(AchievementCondition {
                condition_type: "required".to_string(),
                dimension: "origin".to_string(),
                operator: "equals".to_string(),
                value: json!("origin_shattered_ring"),
                timing: "setup".to_string(),
                mutability: "immutable".to_string(),
                severity: "hard".to_string(),
                source: None,
                notes: None,
            }),
            &FactLookup::new(&[]),
            false,
            None,
        );

        assert_eq!(result.status, STATUS_UNKNOWN);
        assert_eq!(result.conditions[0].passed, None);
        assert!(result.reasons[0].contains("origin is unknown"));
    }

    #[test]
    fn completed_status_wins_over_planned() {
        let result = evaluate_achievement(
            achievement_with_condition(AchievementCondition {
                condition_type: "required".to_string(),
                dimension: "ethic".to_string(),
                operator: "contains".to_string(),
                value: json!("ethic_xenophile"),
                timing: "setup".to_string(),
                mutability: "slow_change".to_string(),
                severity: "hard".to_string(),
                source: None,
                notes: None,
            }),
            &FactLookup::new(&[fact("empire", "ethics", json!(["ethic_xenophobe"]))]),
            true,
            Some("planned"),
        );

        assert_eq!(result.status, STATUS_COMPLETED);
        assert_eq!(result.computed_status, STATUS_INCOMPATIBLE);
        assert!(result.planned);
    }

    #[test]
    fn supported_conditions_can_be_possible() {
        let result = evaluate_achievement(
            achievement_with_condition(AchievementCondition {
                condition_type: "required".to_string(),
                dimension: "ethic".to_string(),
                operator: "contains".to_string(),
                value: json!("ethic_xenophile"),
                timing: "setup".to_string(),
                mutability: "slow_change".to_string(),
                severity: "hard".to_string(),
                source: None,
                notes: None,
            }),
            &FactLookup::new(&[fact("empire", "ethics", json!(["ethic_xenophile"]))]),
            false,
            None,
        );

        assert_eq!(result.status, STATUS_POSSIBLE);
        assert_eq!(result.conditions[0].passed, Some(true));
    }

    #[test]
    fn mutable_hard_failure_is_incompatible_not_impossible() {
        let result = evaluate_achievement(
            achievement_with_condition(AchievementCondition {
                condition_type: "required".to_string(),
                dimension: "ethic".to_string(),
                operator: "contains".to_string(),
                value: json!("ethic_xenophile"),
                timing: "current".to_string(),
                mutability: "normal_change".to_string(),
                severity: "hard".to_string(),
                source: None,
                notes: None,
            }),
            &FactLookup::new(&[fact("empire", "ethics", json!(["ethic_xenophobe"]))]),
            false,
            None,
        );

        assert_eq!(result.status, STATUS_INCOMPATIBLE);
        assert_eq!(result.conditions[0].passed, Some(false));
    }

    #[test]
    fn numeric_operators_are_supported() {
        let at_least = evaluate_operator("at_least", &json!(10), &json!(10));
        let greater_than = evaluate_operator("greater_than", &json!(11), &json!(10));
        let equal_is_not_greater_than = evaluate_operator("greater_than", &json!(10), &json!(10));

        assert_eq!(at_least.0, Some(true));
        assert_eq!(greater_than.0, Some(true));
        assert_eq!(equal_is_not_greater_than.0, Some(false));
    }

    #[test]
    fn less_than_operator() {
        let less = evaluate_operator("less_than", &json!(5), &json!(10));
        let equal = evaluate_operator("less_than", &json!(10), &json!(10));
        let greater = evaluate_operator("less_than", &json!(15), &json!(10));

        assert_eq!(less.0, Some(true));
        assert_eq!(equal.0, Some(false));
        assert_eq!(greater.0, Some(false));
    }

    #[test]
    fn not_equals_operator() {
        let different = evaluate_operator(
            "not_equals",
            &json!("ethic_xenophile"),
            &json!("ethic_xenophobe"),
        );
        let same = evaluate_operator(
            "not_equals",
            &json!("ethic_xenophile"),
            &json!("ethic_xenophile"),
        );

        assert_eq!(different.0, Some(true));
        assert_eq!(same.0, Some(false));
    }

    #[test]
    fn not_equals_is_case_insensitive() {
        let result = evaluate_operator(
            "not_equals",
            &json!("ETHIC_XENOPHILE"),
            &json!("ethic_xenophile"),
        );
        assert_eq!(result.0, Some(false));
    }

    #[test]
    fn less_than_with_non_numeric_values() {
        let result = evaluate_operator("less_than", &json!("abc"), &json!(10));
        assert_eq!(result.0, None);
    }

    #[test]
    fn unsupported_operator_and_dimension_are_unknown() {
        let unsupported_operator = evaluate_operator("exists", &json!(true), &json!(true));
        assert_eq!(unsupported_operator.0, None);

        let result = evaluate_achievement(
            achievement_with_condition(AchievementCondition {
                condition_type: "required".to_string(),
                dimension: "fleet_power".to_string(),
                operator: "greater_than".to_string(),
                value: json!(100000),
                timing: "current".to_string(),
                mutability: "normal_change".to_string(),
                severity: "hard".to_string(),
                source: None,
                notes: None,
            }),
            &FactLookup::new(&[]),
            false,
            None,
        );

        assert_eq!(result.status, STATUS_UNKNOWN);
        assert_eq!(result.conditions[0].passed, None);
    }

    fn achievement_with_condition(condition: AchievementCondition) -> AchievementCatalogEntry {
        AchievementCatalogEntry {
            id: "test_achievement".to_string(),
            steam_app_id: 281_990,
            steam_api_name: Some("TEST".to_string()),
            local_key: None,
            deprecated: false,
            source: AchievementSourceFields {
                name: "Test Achievement".to_string(),
                ..AchievementSourceFields::default()
            },
            curation: AchievementCurationFields {
                conditions: vec![condition],
                ..AchievementCurationFields::default()
            },
            completed: false,
        }
    }

    fn fact(dimension: &str, key: &str, value: Value) -> RunFactSummary {
        RunFactSummary {
            run_folder_path: "/run".to_string(),
            dimension: dimension.to_string(),
            key: key.to_string(),
            value,
            source: "parsed_save".to_string(),
            confidence: "high".to_string(),
            updated_from_save_path: None,
            updated_at: "now".to_string(),
            is_override: false,
        }
    }
}
