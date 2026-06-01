fn main() {
    #[cfg(feature = "desktop")]
    if let Err(err) = stellaris_hunter_scan::run_app() {
        eprintln!("{err}");
        std::process::exit(1);
    }

    #[cfg(not(feature = "desktop"))]
    if let Err(err) = stellaris_hunter_scan::run_cli() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
