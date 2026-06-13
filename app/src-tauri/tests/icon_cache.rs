use stellaris_hunter_scan::icons::IconCache;
use tempfile::tempdir;

#[test]
fn test_cache_store_and_get() {
    let dir = tempdir().unwrap();
    let cache = IconCache::new(dir.path());

    assert!(cache.get("test_ach").is_none());

    let png_bytes = vec![0x89, 0x50, 0x4E, 0x47]; // PNG magic bytes (stub)
    cache.store("test_ach", &png_bytes).unwrap();

    let path = cache.get("test_ach").unwrap();
    assert!(path.exists());

    let read_bytes = cache.read("test_ach").unwrap().unwrap();
    assert_eq!(read_bytes, png_bytes);
}

#[test]
fn test_cache_count_and_clear() {
    let dir = tempdir().unwrap();
    let cache = IconCache::new(dir.path());

    cache.store("ach1", &[1, 2, 3]).unwrap();
    cache.store("ach2", &[4, 5, 6]).unwrap();
    assert_eq!(cache.count(), 2);

    cache.clear().unwrap();
    assert_eq!(cache.count(), 0);
}

#[test]
fn test_cache_read_missing() {
    let dir = tempdir().unwrap();
    let cache = IconCache::new(dir.path());

    let result = cache.read("nonexistent").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_cache_store_then_clear_then_empty() {
    let dir = tempdir().unwrap();
    let cache = IconCache::new(dir.path());

    cache.store("ach1", &[1, 2, 3]).unwrap();
    assert_eq!(cache.count(), 1);

    cache.clear().unwrap();
    assert_eq!(cache.count(), 0);
    assert!(cache.get("ach1").is_none());
}

#[test]
fn test_cache_ensure_dir_creates_dir() {
    let dir = tempdir().unwrap();
    let cache = IconCache::new(dir.path());

    let icon_dir = dir.path().join("icons");
    assert!(!icon_dir.exists());

    cache.ensure_dir().unwrap();
    assert!(icon_dir.exists());
}

#[test]
fn test_cache_store_multiple_round_trip() {
    let dir = tempdir().unwrap();
    let cache = IconCache::new(dir.path());

    let icons: Vec<(&str, Vec<u8>)> = vec![
        ("star", vec![0x89, 0x50, 0x4E, 0x47, 0x01]),
        ("planet", vec![0x89, 0x50, 0x4E, 0x47, 0x02]),
        ("galaxy", vec![0x89, 0x50, 0x4E, 0x47, 0x03]),
    ];

    for (id, data) in &icons {
        cache.store(id, data).unwrap();
    }

    assert_eq!(cache.count(), 3);

    for (id, expected) in &icons {
        let got = cache.read(id).unwrap().expect("should exist");
        assert_eq!(&got, expected, "mismatch for {id}");
    }
}
