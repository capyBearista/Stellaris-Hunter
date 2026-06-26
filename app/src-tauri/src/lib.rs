pub mod catalog;
pub mod catalog_sync;
pub mod commands;
pub mod db;
pub mod documents;
pub mod eligibility;
pub mod error;
pub mod extract_action;
pub mod extract_discovery;
pub mod extract_progression;
pub mod icons;
pub mod install;
pub mod ipc_helpers;
pub mod model;
pub mod paths;
pub mod rules;
pub mod run_state;
pub mod save;
pub mod serve;
pub mod steam;

use std::path::PathBuf;

use clap::Parser;

pub use error::{Error, Result};
pub use model::*;

#[derive(Debug, Default, serde::Serialize)]
pub struct ScanReport {
    pub install: Option<InstallSummary>,
    pub documents: Option<DocumentsSummary>,
    pub errors: Vec<String>,
}

pub fn scan_all(install_path: Option<PathBuf>, documents_path: Option<PathBuf>) -> ScanReport {
    let mut report = ScanReport::default();

    match install::discover_install(install_path) {
        Ok(found) => report.install = found,
        Err(err) => report.errors.push(err.to_string()),
    }

    match documents::discover_documents(documents_path) {
        Ok(found) => report.documents = found,
        Err(err) => report.errors.push(err.to_string()),
    }

    add_dlc_info_summaries(&mut report);
    add_eligibility_summaries(&mut report);

    report
}

fn add_dlc_info_summaries(report: &mut ScanReport) {
    let Some(documents) = &mut report.documents else {
        return;
    };

    let mut enabled_dlcs = documents
        .dlc_load
        .as_ref()
        .and_then(|dlc_load| dlc_load.dlc_state.as_ref())
        .map(|dlc_state| dlc_state.enabled_dlcs.clone())
        .unwrap_or_default();
    let mut disabled_dlcs = documents
        .dlc_load
        .as_ref()
        .and_then(|dlc_load| dlc_load.dlc_state.as_ref())
        .map(|dlc_state| dlc_state.disabled_dlcs.clone())
        .unwrap_or_default();

    if let Some(launcher) = documents.launcher.as_ref() {
        for dlc in &launcher.dlcs {
            let Some(normalized) = model::launcher_dlc_match_key(dlc) else {
                continue;
            };

            match dlc.enabled_in_active_playset {
                Some(true) => push_unique(&mut enabled_dlcs, normalized),
                Some(false) => push_unique(&mut disabled_dlcs, normalized),
                None => {}
            }
        }
    }

    for run in &mut documents.save_runs {
        let Some(save) = &run.latest_save else {
            continue;
        };

        let mut enabled_and_required = Vec::new();
        let mut disabled_but_required = Vec::new();
        let mut unknown_status_required = Vec::new();

        for required_dlc in &save.required_dlcs {
            // Match by expanding both sides: a normalized save DLC like
            // "ancient_relics" must find overlap with a launcher entry like
            // "dlc028_ancient_relics" (whose own match keys include
            // "ancient_relics").  One-directional containment would miss it.
            let required_keys = model::dlc_match_keys(required_dlc);
            let is_enabled = enabled_dlcs.iter().any(|e| {
                model::dlc_match_keys(e)
                    .iter()
                    .any(|ek| required_keys.contains(ek))
            });
            let is_disabled = disabled_dlcs.iter().any(|d| {
                model::dlc_match_keys(d)
                    .iter()
                    .any(|dk| required_keys.contains(dk))
            });

            if is_enabled {
                enabled_and_required.push(required_dlc.clone());
            } else if is_disabled {
                disabled_but_required.push(required_dlc.clone());
            } else {
                unknown_status_required.push(required_dlc.clone());
            }
        }

        run.dlc_info = Some(model::RunDlcInfo {
            enabled_and_required,
            disabled_but_required,
            unknown_status_required,
            all_enabled_dlcs: enabled_dlcs.clone(),
            all_disabled_dlcs: disabled_dlcs.clone(),
        });
    }
}

fn push_unique(items: &mut Vec<String>, value: String) {
    if !items.contains(&value) {
        items.push(value);
    }
}

fn add_eligibility_summaries(report: &mut ScanReport) {
    let checksum_scopes = report
        .install
        .as_ref()
        .map(|install| install.checksum_manifest.as_slice())
        .unwrap_or(&[]);

    let Some(documents) = &mut report.documents else {
        return;
    };

    let enabled_mods = documents
        .launcher
        .as_ref()
        .filter(|launcher| launcher.issues.is_empty())
        .map(|launcher| launcher.enabled_mods.as_slice());
    let dlc_load_enabled_mods = documents
        .dlc_load
        .as_ref()
        .map(|dlc_load| dlc_load.enabled_mods.as_slice());

    for run in &mut documents.save_runs {
        run.eligibility = run.latest_save.as_ref().map(|save| {
            eligibility::compute_save_eligibility(
                save,
                enabled_mods,
                dlc_load_enabled_mods,
                checksum_scopes,
            )
        });
    }
}

#[derive(Debug, Parser)]
pub struct CliArgs {
    #[arg(long)]
    pub install_path: Option<PathBuf>,
    #[arg(long)]
    pub documents_path: Option<PathBuf>,
}

pub fn run_cli() -> Result<()> {
    let args = CliArgs::parse();
    let report = scan_all(args.install_path, args.documents_path);
    let json = serde_json::to_string_pretty(&report)?;
    println!("{json}");
    Ok(())
}

#[cfg(feature = "desktop")]
pub fn run_app() -> tauri::Result<()> {
    use tauri::Manager;

    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let db_file = data_dir.join("stellaris-hunter.db");

            let mut conn = db::open_app_db(&db_file).map_err(|e| {
                eprintln!("failed to open app db: {e}");
                tauri::Error::Anyhow(e.into())
            })?;

            match db::ensure_catalog_imported(&mut conn) {
                Ok(true) => eprintln!("imported bundled catalog into app db"),
                Ok(false) => eprintln!("catalog already imported"),
                Err(e) => eprintln!("warning: catalog import failed: {e}"),
            }

            // Drop connection before storing path — commands open their own.
            drop(conn);
            app.manage(db::AppDbPath(db_file));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::scan_local_state,
            commands::catalog_commands::load_achievements,
            commands::catalog_commands::load_catalog_info,
            commands::catalog_commands::load_completion_overrides,
            commands::catalog_commands::set_completion_override,
            commands::catalog_commands::clear_completion_override,
            commands::catalog_commands::load_runs,
            commands::catalog_commands::load_run_facts,
            commands::catalog_commands::rescan_saves,
            commands::catalog_commands::load_planner_evaluations,
            commands::catalog_commands::set_run_achievement_status,
            commands::catalog_commands::load_fact_overrides,
            commands::catalog_commands::set_fact_override,
            commands::catalog_commands::clear_fact_override,
            commands::catalog_commands::load_run_notes,
            commands::catalog_commands::set_run_note,
            commands::catalog_commands::clear_run_note,
            commands::catalog_commands::load_run_achievement_notes,
            commands::catalog_commands::set_run_achievement_note,
            commands::catalog_commands::clear_run_achievement_note,
            commands::catalog_commands::sync_catalog,
            commands::catalog_commands::get_achievement_icon,
            commands::catalog_commands::sync_icons,
            commands::catalog_commands::load_app_config,
            commands::catalog_commands::save_app_config,
            commands::catalog_commands::load_app_info,
            commands::sync_steam_achievements,
        ])
        .run(tauri::generate_context!())
}
