#[test]
fn command_wrapper_matches_direct_auto_discovery_scan() {
    let install_root = tempfile::tempdir().expect("synthetic install root should be created");
    let documents_root = tempfile::tempdir().expect("synthetic documents root should be created");

    std::env::set_var("STELLARIS_INSTALL_PATH", install_root.path());
    std::env::set_var("STELLARIS_DOCUMENTS_PATH", documents_root.path());
    let _env_guard = EnvGuard::new(&["STELLARIS_INSTALL_PATH", "STELLARIS_DOCUMENTS_PATH"]);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    let direct = stellaris_hunter_scan::scan_all(None, None);
    let command = runtime
        .block_on(stellaris_hunter_scan::commands::scan_local_state())
        .unwrap();

    assert_eq!(
        serde_json::to_value(direct).unwrap(),
        serde_json::to_value(command).unwrap()
    );
}

struct EnvGuard<'a> {
    keys: &'a [&'a str],
}

impl<'a> EnvGuard<'a> {
    fn new(keys: &'a [&'a str]) -> Self {
        Self { keys }
    }
}

impl Drop for EnvGuard<'_> {
    fn drop(&mut self) {
        for key in self.keys {
            std::env::remove_var(key);
        }
    }
}
