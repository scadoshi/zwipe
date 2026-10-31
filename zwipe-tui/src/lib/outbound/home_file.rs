//! What home keeps between launches, beside the session: everybody's counters as last fetched, and whose counters it shows.

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use zwipe_core::http::contracts::metrics::HttpPublicMetrics;

use super::files;
use crate::domain::home::Scope;

/// The file holding everybody's counters.
const COUNTS_FILE: &str = "public_metrics.json";

/// The file holding the scope as `all` or `mine`, the bytes the app stores.
const SCOPE_FILE: &str = "count_scope";

/// The folder both files live in.
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

    /// Everybody's counters from the last launch, if saved and readable.
    pub(crate) fn load_counts(&self) -> Option<HttpPublicMetrics> {
        let json = fs::read_to_string(self.dir.join(COUNTS_FILE)).ok()?;
        serde_json::from_str(&json).ok()
    }

    /// Keeps `metrics` for the next launch.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be written.
    pub(crate) fn save_counts(&self, metrics: &HttpPublicMetrics) -> Result<()> {
        files::write_private(
            &self.dir.join(COUNTS_FILE),
            &serde_json::to_string(metrics)?,
        )
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
    fn counts_and_scope_round_trip_and_default_when_missing() {
        let dir = std::env::temp_dir().join(format!("zwipe-tui-home-{}", uuid::Uuid::new_v4()));
        let file = HomeFile::at(dir.clone());
        assert!(file.load_counts().is_none());
        assert_eq!(file.load_scope(), Scope::Everybody);
        let metrics = HttpPublicMetrics {
            cards_swiped: 3,
            searches: 2,
            decks_created: 1,
        };
        file.save_counts(&metrics).unwrap();
        file.save_scope(Scope::OnlyMe).unwrap();
        assert_eq!(
            file.load_counts().map(|m| m.searches),
            Some(metrics.searches)
        );
        assert_eq!(file.load_scope(), Scope::OnlyMe);
        fs::remove_dir_all(dir).unwrap();
    }
}
