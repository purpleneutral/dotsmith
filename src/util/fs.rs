use std::path::{Path, PathBuf};

/// Check if a path is a symlink (without following it).
pub fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

/// Get the symlink target if path is a symlink, None otherwise.
pub fn symlink_target(path: &Path) -> Option<PathBuf> {
    if is_symlink(path) {
        std::fs::read_link(path).ok()
    } else {
        None
    }
}

/// Check if a resolved path is within the user's home directory.
/// Returns Ok(()) if safe, Err with a warning message if the path escapes $HOME.
pub fn check_path_safety(path: &Path) -> anyhow::Result<()> {
    let resolved = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => return Ok(()), // Can't resolve — file may not exist yet, not a safety issue
    };

    if let Some(home) = dirs::home_dir()
        && !resolved.starts_with(&home)
    {
        anyhow::bail!(
            "path '{}' resolves to '{}' which is outside your home directory",
            path.display(),
            resolved.display()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_is_symlink_true() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target");
        std::fs::write(&target, "content").unwrap();
        let link = tmp.path().join("link");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        assert!(is_symlink(&link));
    }

    #[test]
    fn test_is_symlink_false() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("regular");
        std::fs::write(&file, "content").unwrap();

        assert!(!is_symlink(&file));
    }

    #[test]
    fn test_is_symlink_nonexistent() {
        assert!(!is_symlink(Path::new("/nonexistent/path")));
    }

    #[test]
    fn test_symlink_target() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target");
        std::fs::write(&target, "content").unwrap();
        let link = tmp.path().join("link");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        assert_eq!(symlink_target(&link), Some(target));
    }

    #[test]
    fn test_symlink_target_regular_file() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("regular");
        std::fs::write(&file, "content").unwrap();

        assert_eq!(symlink_target(&file), None);
    }

    #[test]
    fn test_check_path_safety_within_home() {
        // A file within a temp dir can't be guaranteed inside $HOME,
        // so we test with an actual home-relative path if it exists
        if let Some(home) = dirs::home_dir() {
            let path = home.join(".bashrc");
            if path.exists() {
                assert!(check_path_safety(&path).is_ok());
            }
        }
    }

    #[test]
    fn test_check_path_safety_nonexistent() {
        // Nonexistent paths should not fail safety check
        let result = check_path_safety(Path::new("/nonexistent/path/file"));
        assert!(result.is_ok());
    }
}
