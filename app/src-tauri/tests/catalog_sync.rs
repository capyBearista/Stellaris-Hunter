use stellaris_hunter_scan::catalog_sync::is_newer_version;

#[test]
fn test_newer_version() {
    assert!(is_newer_version("1.0.0", "1.1.0"));
    assert!(is_newer_version("1.0.0", "2.0.0"));
    assert!(is_newer_version("1.1.0", "1.1.1"));
    assert!(!is_newer_version("1.1.0", "1.1.0"));
    assert!(!is_newer_version("1.1.0", "1.0.0"));
    assert!(!is_newer_version("2.0.0", "1.9.9"));
}

#[test]
fn test_invalid_version() {
    assert!(!is_newer_version("invalid", "1.0.0"));
    assert!(!is_newer_version("1.0.0", "invalid"));
    assert!(!is_newer_version("1.0", "1.0.0"));
}

#[test]
fn test_edge_cases() {
    // Zero padding
    assert!(!is_newer_version("01.0.0", "1.0.0"));
    // Large numbers
    assert!(is_newer_version("0.0.1", "999.999.999"));
    assert!(!is_newer_version("999.999.999", "0.0.1"));
    // Empty strings
    assert!(!is_newer_version("", "1.0.0"));
    assert!(!is_newer_version("1.0.0", ""));
    // Extra segments (4-part version)
    assert!(!is_newer_version("1.0.0", "1.0.0.1"));
    assert!(!is_newer_version("1.0.0.1", "1.0.0"));
}
