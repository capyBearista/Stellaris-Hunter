pub mod catalog;
pub mod commands;
pub mod db;
pub mod documents;
pub mod eligibility;
pub mod error;
pub mod install;
pub mod model;
pub mod paths;
pub mod rules;
pub mod run_state;
pub mod save;
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

    add_eligibility_summaries(&mut report);

    report
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
        ])
        .run(tauri::generate_context!())
}
