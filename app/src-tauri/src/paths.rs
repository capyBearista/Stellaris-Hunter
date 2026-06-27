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
    let userprofile = std::env::var("USERPROFILE").ok();
    let onedrive = std::env::var("OneDrive").ok();
    let defaults = documents_default_candidates(
        username.as_deref(),
        userprofile.as_deref(),
        onedrive.as_deref(),
    );

    candidate_paths(
        explicit,
        &[
            "STELLARIS_DOCUMENTS_PATH",
            "PARADOX_STELLARIS_DOCUMENTS_PATH",
        ],
        &defaults,
    )
}

fn documents_default_candidates(
    username: Option<&str>,
    userprofile: Option<&str>,
    onedrive: Option<&str>,
) -> Vec<PathBuf> {
    let mut defaults = Vec::new();

    if let Some(profile) = userprofile {
        push_documents_root(&mut defaults, PathBuf::from(profile));
        if let Some(wsl_profile) = windows_path_to_wsl_mount(profile) {
            push_documents_root(&mut defaults, PathBuf::from(wsl_profile));
        }
    }

    if let Some(onedrive_root) = onedrive {
        push_documents_root(&mut defaults, PathBuf::from(onedrive_root));
        if let Some(wsl_onedrive_root) = windows_path_to_wsl_mount(onedrive_root) {
            push_documents_root(&mut defaults, PathBuf::from(wsl_onedrive_root));
        }
    }

    if let Some(user) = username {
        push_documents_root(&mut defaults, PathBuf::from(format!(r"C:\Users\{user}")));
        push_documents_root(&mut defaults, PathBuf::from(format!("/mnt/c/Users/{user}")));
        push_documents_root(
            &mut defaults,
            PathBuf::from(format!(r"C:\Users\{user}\OneDrive")),
        );
        push_documents_root(
            &mut defaults,
            PathBuf::from(format!("/mnt/c/Users/{user}/OneDrive")),
        );
    }

    dedupe_paths(defaults)
}

fn push_documents_root(defaults: &mut Vec<PathBuf>, root: PathBuf) {
    defaults.push(
        root.join("Documents")
            .join("Paradox Interactive")
            .join("Stellaris"),
    );
}

fn windows_path_to_wsl_mount(path: &str) -> Option<String> {
    let bytes = path.as_bytes();
    if bytes.len() < 3 || bytes[1] != b':' || bytes[2] != b'\\' {
        return None;
    }

    let drive = (bytes[0] as char).to_ascii_lowercase();
    let rest = path[3..].replace('\\', "/");
    Some(format!("/mnt/{drive}/{rest}"))
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

#[cfg(test)]
mod tests {
    use super::{
        candidate_paths, documents_candidates, documents_default_candidates, install_candidates,
        windows_path_to_wsl_mount,
    };
    use std::{path::PathBuf, sync::Mutex};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env_vars(vars: &[(&str, Option<&str>)], f: impl FnOnce()) {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let previous: Vec<(String, Option<String>)> = vars
            .iter()
            .map(|(name, _)| ((*name).to_string(), std::env::var(name).ok()))
            .collect();

        for (name, value) in vars {
            match value {
                Some(value) => unsafe { std::env::set_var(name, value) },
                None => unsafe { std::env::remove_var(name) },
            }
        }

        f();

        for (name, value) in previous {
            match value {
                Some(value) => unsafe { std::env::set_var(name, value) },
                None => unsafe { std::env::remove_var(name) },
            }
        }
    }

    #[test]
    fn converts_windows_path_to_wsl_mount() {
        assert_eq!(
            windows_path_to_wsl_mount(r"C:\Users\Arjun\OneDrive"),
            Some("/mnt/c/Users/Arjun/OneDrive".to_string())
        );
        assert_eq!(
            windows_path_to_wsl_mount(r"D:\Docs"),
            Some("/mnt/d/Docs".to_string())
        );
        assert_eq!(windows_path_to_wsl_mount("/mnt/c/Users/Arjun"), None);
    }

    #[test]
    fn includes_userprofile_and_onedrive_documents_candidates() {
        let candidates = documents_default_candidates(
            Some("Arjun"),
            Some(r"C:\Users\Arjun"),
            Some(r"C:\Users\Arjun\OneDrive"),
        );

        let expected_userprofile = PathBuf::from(r"C:\Users\Arjun")
            .join("Documents")
            .join("Paradox Interactive")
            .join("Stellaris");
        assert!(candidates.contains(&expected_userprofile));
        assert!(candidates.contains(&PathBuf::from(
            "/mnt/c/Users/Arjun/Documents/Paradox Interactive/Stellaris"
        )));
        let expected_onedrive = PathBuf::from(r"C:\Users\Arjun\OneDrive")
            .join("Documents")
            .join("Paradox Interactive")
            .join("Stellaris");
        assert!(candidates.contains(&expected_onedrive));
        assert!(candidates.contains(&PathBuf::from(
            "/mnt/c/Users/Arjun/OneDrive/Documents/Paradox Interactive/Stellaris"
        )));
    }

    #[test]
    fn candidate_paths_prioritizes_explicit_then_env_then_defaults() {
        with_env_vars(&[("STELLARIS_INSTALL_PATH", Some("/env/install"))], || {
            let paths = candidate_paths(
                Some(PathBuf::from("/explicit/install")),
                &["STELLARIS_INSTALL_PATH"],
                &[PathBuf::from("/default/install")],
            );

            assert_eq!(
                paths,
                vec![
                    PathBuf::from("/explicit/install"),
                    PathBuf::from("/env/install"),
                    PathBuf::from("/default/install"),
                ]
            );
        });
    }

    #[test]
    fn install_candidates_dedupe_explicit_and_env_duplicates() {
        let duplicate = "/mnt/c/Games/Steam/steamapps/common/Stellaris";
        with_env_vars(&[("STELLARIS_INSTALL_PATH", Some(duplicate))], || {
            let paths = install_candidates(Some(PathBuf::from(duplicate)));
            assert_eq!(paths.first(), Some(&PathBuf::from(duplicate)));
            assert_eq!(
                paths
                    .iter()
                    .filter(|path| **path == PathBuf::from(duplicate))
                    .count(),
                1
            );
        });
    }

    #[test]
    fn documents_candidates_include_explicit_env_and_generated_defaults() {
        with_env_vars(
            &[
                ("USERNAME", Some("Arjun")),
                ("USER", None),
                ("USERPROFILE", Some(r"C:\Users\Arjun")),
                ("OneDrive", Some(r"C:\Users\Arjun\OneDrive")),
                ("STELLARIS_DOCUMENTS_PATH", Some("/custom/documents")),
                ("PARADOX_STELLARIS_DOCUMENTS_PATH", None),
            ],
            || {
                let paths = documents_candidates(Some(PathBuf::from("/explicit/documents")));

                assert_eq!(paths.first(), Some(&PathBuf::from("/explicit/documents")));
                assert!(paths.contains(&PathBuf::from("/custom/documents")));
                assert!(paths.contains(
                    &PathBuf::from(r"C:\Users\Arjun")
                        .join("Documents")
                        .join("Paradox Interactive")
                        .join("Stellaris")
                ));
                assert!(paths.contains(&PathBuf::from(
                    "/mnt/c/Users/Arjun/OneDrive/Documents/Paradox Interactive/Stellaris"
                )));
            },
        );
    }
}
