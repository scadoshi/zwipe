//! The `zwiper-tui` folder in the user's config directory, its move from the old `zwipe-tui` name, and owner-only writes to it.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// The folder inside the user's config directory.
const APP_DIR_NAME: &str = "zwiper-tui";

/// The folder's name before the crate was renamed to zwiper-tui.
const OLD_APP_DIR_NAME: &str = "zwipe-tui";

/// `zwiper-tui` under the user's config directory: `~/.config` on Linux, `~/Library/Application Support` on macOS.
///
/// # Errors
///
/// Fails when the system has no config directory.
pub(crate) fn app_dir() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("could not find the config directory")?
        .join(APP_DIR_NAME))
}

/// Moves the old `zwipe-tui` folder to `zwiper-tui` once, so a saved session and theme carry over. Any failure is ignored and the app starts fresh.
pub(crate) fn migrate_app_dir() {
    if let Some(config) = dirs::config_dir() {
        let _ = migrate(&config.join(OLD_APP_DIR_NAME), &config.join(APP_DIR_NAME));
    }
}

/// Renames `old` to `new` and makes it owner-only, when `old` is a folder and `new` does not exist yet.
///
/// # Errors
///
/// Fails when the folder cannot be renamed or its permissions cannot be set.
fn migrate(old: &Path, new: &Path) -> Result<()> {
    if new.exists() || !old.is_dir() {
        return Ok(());
    }
    fs::rename(old, new)?;
    restrict(new, 0o700)
}

/// Writes `contents` to `path`, creating its folder owner-only and the file readable by the owner only, on Unix.
///
/// # Errors
///
/// Fails when the folder or the file cannot be created or written.
pub(crate) fn write_private(path: &Path, contents: &str) -> Result<()> {
    if let Some(dir) = path.parent()
        && !dir.exists()
    {
        fs::create_dir_all(dir)?;
        restrict(dir, 0o700)?;
    }
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    if path.exists() {
        restrict(path, 0o600)?;
    }
    options.open(path)?.write_all(contents.as_bytes())?;
    Ok(())
}

/// Deletes the file at `path`. A file that is already gone is not an error.
///
/// # Errors
///
/// Fails when the file exists and cannot be deleted.
pub(crate) fn remove(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
        _ => Ok(()),
    }
}

/// Sets `path`'s permission bits on Unix; elsewhere does nothing.
#[cfg_attr(
    not(unix),
    expect(clippy::unnecessary_wraps, reason = "matches the Unix signature")
)]
fn restrict(path: &Path, mode: u32) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }
    #[cfg(not(unix))]
    let _ = (path, mode);
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn writes_are_owner_only_in_an_owner_only_folder() {
        let dir = std::env::temp_dir().join(format!("zwiper-tui-files-{}", std::process::id()));
        let path = dir.join("x.json");
        write_private(&path, "{}").unwrap();
        let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o777;
        let (dir_mode, file_mode) = (mode(&dir), mode(&path));
        remove(&path).unwrap();
        remove(&path).unwrap();
        fs::remove_dir(&dir).unwrap();
        assert_eq!((dir_mode, file_mode), (0o700, 0o600));
    }

    /// A fresh parent folder for one migration test.
    fn migration_root() -> PathBuf {
        std::env::temp_dir().join(format!("zwiper-tui-migrate-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn migrate_moves_the_old_folder_and_keeps_it_owner_only() {
        let root = migration_root();
        let (old, new) = (root.join("old"), root.join("new"));
        write_private(&old.join("session.json"), "{}").unwrap();
        fs::set_permissions(&old, fs::Permissions::from_mode(0o755)).unwrap();
        migrate(&old, &new).unwrap();
        let moved = fs::read_to_string(new.join("session.json")).unwrap();
        let mode = fs::metadata(&new).unwrap().permissions().mode() & 0o777;
        let old_gone = !old.exists();
        fs::remove_dir_all(&root).unwrap();
        assert_eq!((moved.as_str(), mode, old_gone), ("{}", 0o700, true));
    }

    #[test]
    fn migrate_leaves_both_folders_when_the_new_one_exists() {
        let root = migration_root();
        let (old, new) = (root.join("old"), root.join("new"));
        write_private(&old.join("session.json"), "old").unwrap();
        write_private(&new.join("session.json"), "new").unwrap();
        migrate(&old, &new).unwrap();
        let kept = fs::read_to_string(new.join("session.json")).unwrap();
        let old_kept = old.exists();
        fs::remove_dir_all(&root).unwrap();
        assert_eq!((kept.as_str(), old_kept), ("new", true));
    }

    #[test]
    fn migrate_does_nothing_without_an_old_folder() {
        let root = migration_root();
        migrate(&root.join("old"), &root.join("new")).unwrap();
        assert!(!root.exists());
    }

    #[test]
    fn migrate_failure_is_an_error_not_a_panic() {
        let root = migration_root();
        let old = root.join("old");
        write_private(&old.join("session.json"), "{}").unwrap();
        let result = migrate(&old, &root.join("missing").join("new"));
        fs::remove_dir_all(&root).unwrap();
        assert!(result.is_err());
    }
}
