fn main() {
    if let Err(err) = stellaris_hunter_scan::run_cli() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
