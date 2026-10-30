//! The `zwipe-tui` folder in the user's config directory, and owner-only writes to it.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// The folder inside the user's config directory.
const APP_DIR_NAME: &str = "zwipe-tui";

/// `zwipe-tui` under the user's config directory: `~/.config` on Linux, `~/Library/Application Support` on macOS.
///
/// # Errors
///
/// Fails when the system has no config directory.
pub(crate) fn app_dir() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("could not find the config directory")?
        .join(APP_DIR_NAME))
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
        let dir = std::env::temp_dir().join(format!("zwipe-tui-files-{}", std::process::id()));
        let path = dir.join("x.json");
        write_private(&path, "{}").unwrap();
        let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o777;
        let (dir_mode, file_mode) = (mode(&dir), mode(&path));
        remove(&path).unwrap();
        remove(&path).unwrap();
        fs::remove_dir(&dir).unwrap();
        assert_eq!((dir_mode, file_mode), (0o700, 0o600));
    }
}
