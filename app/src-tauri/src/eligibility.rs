use crate::model::{
    ChecksumScope, EligibilityConclusion, LauncherModSummary, ModChecksumRisk, SaveEligibility,
    SaveSummary,
};

pub fn compute_save_eligibility(
    save: &SaveSummary,
    launcher_enabled_mods: Option<&[LauncherModSummary]>,
    dlc_load_enabled_mods: Option<&[String]>,
    checksum_scopes: &[ChecksumScope],
) -> SaveEligibility {
    let mod_risk = classify_mod_risk(
        launcher_enabled_mods,
        dlc_load_enabled_mods,
        checksum_scopes,
    );
    let mut reasons = Vec::new();
    let mut warnings = Vec::new();

    record_cheat_signal(save.cheated_on_save, &mut reasons, &mut warnings);
    record_ironman_signal(save.ironman, &mut reasons, &mut warnings);
    record_mod_signal(
        &mod_risk,
        launcher_enabled_mods,
        dlc_load_enabled_mods,
        &mut reasons,
        &mut warnings,
    );

    let conclusion = conclude(save.cheated_on_save, &mod_risk);

    SaveEligibility {
        conclusion,
        cheated_on_save: save.cheated_on_save,
        ironman: save.ironman,
        mod_risk,
        reasons,
        warnings,
    }
}

fn classify_mod_risk(
    launcher_enabled_mods: Option<&[LauncherModSummary]>,
    dlc_load_enabled_mods: Option<&[String]>,
    _checksum_scopes: &[ChecksumScope],
) -> ModChecksumRisk {
    // The current launcher summary tells us which mods are enabled in the
    // active playset, not which files those mods contain. Until a later pass
    // inventories mod contents and compares them with checksum scopes, any
    // enabled mod remains an unknown checksum risk rather than a proven scoped
    // risk.
    if has_entries(launcher_enabled_mods) || has_entries(dlc_load_enabled_mods) {
        return ModChecksumRisk::Unknown;
    }

    match (launcher_enabled_mods, dlc_load_enabled_mods) {
        (Some([]), Some([])) => ModChecksumRisk::None,
        _ => ModChecksumRisk::Unknown,
    }
}

fn has_entries<T>(items: Option<&[T]>) -> bool {
    items.map(|items| !items.is_empty()).unwrap_or(false)
}

fn conclude(cheated_on_save: Option<bool>, mod_risk: &ModChecksumRisk) -> EligibilityConclusion {
    if cheated_on_save == Some(true) || matches!(mod_risk, ModChecksumRisk::ChecksumScoped) {
        return EligibilityConclusion::LikelyIneligible;
    }

    if cheated_on_save == Some(false) && matches!(mod_risk, ModChecksumRisk::None) {
        return EligibilityConclusion::LikelyEligible;
    }

    EligibilityConclusion::Unknown
}

fn record_cheat_signal(
    cheated_on_save: Option<bool>,
    reasons: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    match cheated_on_save {
        Some(true) => reasons.push("Save reports cheated_on_save=yes.".to_string()),
        Some(false) => reasons.push("Save reports cheated_on_save=no.".to_string()),
        None => warnings.push("Save did not expose a cheated_on_save flag.".to_string()),
    }
}

fn record_ironman_signal(
    ironman: Option<bool>,
    reasons: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    match ironman {
        Some(true) => reasons.push("Save reports ironman=yes.".to_string()),
        Some(false) => warnings.push(
            "Save reports ironman=no; this is metadata, not a global achievement blocker."
                .to_string(),
        ),
        None => warnings.push("Save did not expose an ironman flag.".to_string()),
    }
}

fn record_mod_signal(
    mod_risk: &ModChecksumRisk,
    launcher_enabled_mods: Option<&[LauncherModSummary]>,
    dlc_load_enabled_mods: Option<&[String]>,
    reasons: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    match mod_risk {
        ModChecksumRisk::None => {
            reasons.push("No enabled launcher mods were detected.".to_string())
        }
        ModChecksumRisk::Unknown => {
            let launcher_count = launcher_enabled_mods.map(|mods| mods.len());
            let dlc_load_count = dlc_load_enabled_mods.map(|mods| mods.len());
            let known_enabled_count = launcher_count.unwrap_or(0).max(dlc_load_count.unwrap_or(0));
            match known_enabled_count {
                count if count > 0 => warnings.push(format!(
                    "{count} enabled launcher mod(s) detected; checksum impact is unvalidated."
                )),
                _ if launcher_count.is_none() && dlc_load_count.is_none() => {
                    warnings.push("Launcher mod state is unavailable.".to_string())
                }
                _ => warnings.push(
                    "Launcher mod state is incomplete or ambiguous; checksum impact is unknown."
                        .to_string(),
                ),
            }
        }
        ModChecksumRisk::ChecksumScoped => warnings.push(
            "At least one enabled mod appears to overlap checksum-scoped game paths.".to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn save(ironman: Option<bool>, cheated_on_save: Option<bool>) -> SaveSummary {
        SaveSummary {
            ironman,
            cheated_on_save,
            ..Default::default()
        }
    }

    #[test]
    fn uncheated_save_without_mods_is_likely_eligible() {
        let result =
            compute_save_eligibility(&save(Some(true), Some(false)), Some(&[]), Some(&[]), &[]);

        assert_eq!(result.conclusion, EligibilityConclusion::LikelyEligible);
        assert_eq!(result.mod_risk, ModChecksumRisk::None);
    }

    #[test]
    fn cheated_save_is_likely_ineligible() {
        let result =
            compute_save_eligibility(&save(Some(true), Some(true)), Some(&[]), Some(&[]), &[]);

        assert_eq!(result.conclusion, EligibilityConclusion::LikelyIneligible);
    }

    #[test]
    fn non_ironman_save_is_not_globally_ineligible() {
        let result =
            compute_save_eligibility(&save(Some(false), Some(false)), Some(&[]), Some(&[]), &[]);

        assert_eq!(result.conclusion, EligibilityConclusion::LikelyEligible);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("not a global achievement blocker")));
    }

    #[test]
    fn enabled_mods_make_eligibility_unknown_until_validated() {
        let enabled_mods = vec![LauncherModSummary {
            name: Some("UI Mod".to_string()),
            ..Default::default()
        }];

        let result = compute_save_eligibility(
            &save(Some(true), Some(false)),
            Some(&enabled_mods),
            Some(&[]),
            &[],
        );

        assert_eq!(result.conclusion, EligibilityConclusion::Unknown);
        assert_eq!(result.mod_risk, ModChecksumRisk::Unknown);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("checksum impact is unvalidated")));
    }

    #[test]
    fn missing_cheat_flag_keeps_eligibility_unknown() {
        let result = compute_save_eligibility(&save(Some(true), None), Some(&[]), Some(&[]), &[]);

        assert_eq!(result.conclusion, EligibilityConclusion::Unknown);
    }

    #[test]
    fn unavailable_mod_state_keeps_eligibility_unknown() {
        let result = compute_save_eligibility(&save(Some(true), Some(false)), None, None, &[]);

        assert_eq!(result.conclusion, EligibilityConclusion::Unknown);
        assert_eq!(result.mod_risk, ModChecksumRisk::Unknown);
    }

    #[test]
    fn dlc_load_mods_make_eligibility_unknown_even_without_launcher_mods() {
        let dlc_mods = vec!["mod/alpha.mod".to_string()];

        let result = compute_save_eligibility(
            &save(Some(true), Some(false)),
            Some(&[]),
            Some(&dlc_mods),
            &[],
        );

        assert_eq!(result.conclusion, EligibilityConclusion::Unknown);
        assert_eq!(result.mod_risk, ModChecksumRisk::Unknown);
    }

    #[test]
    fn checksum_scopes_do_not_prove_mod_overlap_without_mod_file_inventory() {
        let checksum_scopes = vec![ChecksumScope {
            directory: Some("common".to_string()),
            patterns: vec!["*.txt".to_string()],
            recursive: true,
            ..Default::default()
        }];

        let result = compute_save_eligibility(
            &save(Some(true), Some(false)),
            Some(&[]),
            Some(&[]),
            &checksum_scopes,
        );

        assert_eq!(result.conclusion, EligibilityConclusion::LikelyEligible);
        assert_eq!(result.mod_risk, ModChecksumRisk::None);
    }
}
