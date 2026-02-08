use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::Context;

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

/// Write content to a file atomically (write to .tmp then rename).
/// Sets file permissions to 0600 (owner-only read/write).
pub fn atomic_write(path: &Path, content: &str) -> anyhow::Result<()> {
    let tmp_path = path.with_extension("tmp");

    {
        let mut file = std::fs::File::create(&tmp_path)
            .with_context(|| format!("failed to create {}", tmp_path.display()))?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }

    std::fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "failed to rename {} to {}",
            tmp_path.display(),
            path.display()
        )
    })?;

    Ok(())
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
    fn test_atomic_write() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.conf");

        atomic_write(&path, "hello world").unwrap();

        assert!(path.exists());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello world");

        // Check permissions
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "file should have 0600 permissions");
    }

    #[test]
    fn test_atomic_write_overwrites() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.conf");

        atomic_write(&path, "first").unwrap();
        atomic_write(&path, "second").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
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
