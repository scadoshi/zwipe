//! The theme to draw before sign-in: the last account's, remembered on this machine.

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::files;

/// An account's theme preference: an orangui slug and dark mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RememberedTheme {
    pub(crate) theme: String,
    pub(crate) dark_mode: bool,
}

impl Default for RememberedTheme {
    /// Gruvbox dark, the app's theme before anyone signs in.
    fn default() -> Self {
        Self {
            theme: "gruvbox".to_owned(),
            dark_mode: true,
        }
    }
}

/// `theme.json` beside the session.
#[derive(Debug, Clone)]
pub(crate) struct ThemeFile {
    path: PathBuf,
}

impl ThemeFile {
    /// `theme.json` in the app's config folder.
    ///
    /// # Errors
    ///
    /// Fails when the system has no config directory.
    pub(crate) fn in_config_dir() -> Result<Self> {
        Ok(Self::at(files::app_dir()?.join("theme.json")))
    }

    pub(crate) fn at(path: PathBuf) -> Self {
        Self { path }
    }

    /// The remembered theme, or the app's default when there is none or it does not parse.
    pub(crate) fn load(&self) -> RememberedTheme {
        fs::read_to_string(&self.path)
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }

    /// Remembers `theme` for the next launch.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be written.
    pub(crate) fn save(&self, theme: &RememberedTheme) -> Result<()> {
        files::write_private(&self.path, &serde_json::to_string(theme)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_gives_the_apps_default_and_a_saved_theme_comes_back() {
        let path = std::env::temp_dir().join(format!(
            "zwiper-tui-theme-{}/theme.json",
            std::process::id()
        ));
        let file = ThemeFile::at(path.clone());
        assert_eq!(file.load(), RememberedTheme::default());

        let nord = RememberedTheme {
            theme: "nord".to_owned(),
            dark_mode: false,
        };
        file.save(&nord).unwrap();
        assert_eq!(file.load(), nord);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
