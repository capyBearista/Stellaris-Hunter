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

fn fact_address(catalog_dimension: &str) -> Option<(&'static str, &'static str)> {
    // MVP supported dimension map. Unsupported catalog dimensions deliberately
    // evaluate to Unknown until the save parser starts emitting those facts.
    match catalog_dimension {
        "authority" => Some(("empire", "authority")),
        "civic" => Some(("empire", "civics")),
        "ethic" => Some(("empire", "ethics")),
        "origin" => Some(("empire", "origin")),
        "government_type" => Some(("empire", "government_type")),
        "species_class" => Some(("species", "founder_species_class")),
        "species_trait" => Some(("species", "founder_species_traits")),
        "portrait" | "species_portrait" => Some(("species", "founder_species_portrait")),
        "required_dlc" | "dlc" => Some(("save", "required_dlcs")),
        "ironman" => Some(("eligibility", "ironman")),
        "cheated_on_save" => Some(("eligibility", "cheated_on_save")),
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
        }
    }
}
