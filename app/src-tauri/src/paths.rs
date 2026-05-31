use std::path::{Path, PathBuf};

pub fn candidate_paths(
    explicit: Option<PathBuf>,
    env_vars: &[&str],
    defaults: &[PathBuf],
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(path) = explicit {
        candidates.push(path);
    }

    for name in env_vars {
        if let Ok(value) = std::env::var(name) {
            if !value.trim().is_empty() {
                candidates.push(PathBuf::from(value));
            }
        }
    }

    candidates.extend(defaults.iter().cloned());
    dedupe_paths(candidates)
}

pub fn install_candidates(explicit: Option<PathBuf>) -> Vec<PathBuf> {
    let defaults = vec![
        PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\Stellaris"),
        PathBuf::from("/mnt/c/Program Files (x86)/Steam/steamapps/common/Stellaris"),
        PathBuf::from(r"C:\Games\Steam\steamapps\common\Stellaris"),
        PathBuf::from("/mnt/c/Games/Steam/steamapps/common/Stellaris"),
        PathBuf::from(r"C:\[GAMES]\[Steam]\steamapps\common\Stellaris"),
        PathBuf::from("/mnt/c/[GAMES]/[Steam]/steamapps/common/Stellaris"),
    ];

    candidate_paths(
        explicit,
        &["STELLARIS_INSTALL_PATH", "PARADOX_STELLARIS_INSTALL_PATH"],
        &defaults,
    )
}

pub fn documents_candidates(explicit: Option<PathBuf>) -> Vec<PathBuf> {
    let username = std::env::var("USERNAME")
        .ok()
        .or_else(|| std::env::var("USER").ok());
    let mut defaults = Vec::new();

    if let Some(user) = username {
        defaults.push(PathBuf::from(format!(
            r"C:\Users\{user}\Documents\Paradox Interactive\Stellaris"
        )));
        defaults.push(PathBuf::from(format!(
            "/mnt/c/Users/{user}/Documents/Paradox Interactive/Stellaris"
        )));
    }

    candidate_paths(
        explicit,
        &[
            "STELLARIS_DOCUMENTS_PATH",
            "PARADOX_STELLARIS_DOCUMENTS_PATH",
        ],
        &defaults,
    )
}

pub fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unique: Vec<PathBuf> = Vec::new();
    for path in paths {
        if !unique.iter().any(|existing| existing == &path) {
            unique.push(path);
        }
    }
    unique
}

pub fn first_existing(paths: &[PathBuf]) -> Option<PathBuf> {
    paths.iter().find(|path| Path::new(path).exists()).cloned()
}
