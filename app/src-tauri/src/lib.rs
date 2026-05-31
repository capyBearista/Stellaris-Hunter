pub mod documents;
pub mod error;
pub mod install;
pub mod model;
pub mod paths;
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

    report
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
