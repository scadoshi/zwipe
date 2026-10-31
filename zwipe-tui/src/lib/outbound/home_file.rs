//! What home keeps between launches, beside the session: whose counters it shows. Everybody's counters are kept by `counts_file`.

use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use super::files;
use crate::domain::home::Scope;

/// The file holding the scope as `all` or `mine`, the bytes the app stores.
const SCOPE_FILE: &str = "count_scope";

/// The folder the scope file lives in.
#[derive(Debug, Clone)]
pub(crate) struct HomeFile {
    dir: PathBuf,
}

impl HomeFile {
    /// The app's config folder.
    ///
    /// # Errors
    ///
    /// Fails when the system has no config directory.
    pub(crate) fn in_config_dir() -> Result<Self> {
        Ok(Self::at(files::app_dir()?))
    }

    pub(crate) fn at(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// The saved scope, everybody when none is saved.
    pub(crate) fn load_scope(&self) -> Scope {
        fs::read(self.dir.join(SCOPE_FILE)).map_or(Scope::Everybody, |b| Scope::from_bytes(&b))
    }

    /// Keeps `scope` for the next launch.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be written.
    pub(crate) fn save_scope(&self, scope: Scope) -> Result<()> {
        let text = String::from_utf8_lossy(scope.as_bytes());
        files::write_private(&self.dir.join(SCOPE_FILE), &text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scope_round_trips_and_defaults_when_missing() {
        let dir = std::env::temp_dir().join(format!("zwipe-tui-home-{}", uuid::Uuid::new_v4()));
        let file = HomeFile::at(dir.clone());
        assert_eq!(file.load_scope(), Scope::Everybody);
        file.save_scope(Scope::OnlyMe).unwrap();
        assert_eq!(file.load_scope(), Scope::OnlyMe);
        fs::remove_dir_all(dir).unwrap();
    }
}
