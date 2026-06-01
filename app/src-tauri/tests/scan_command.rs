#[test]
fn command_wrapper_matches_direct_auto_discovery_scan() {
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
