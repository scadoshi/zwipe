//! Everybody's counts as last fetched, kept on disk so the next launch's first screen shows them while it fetches again.

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use zwipe_core::http::contracts::metrics::HttpPublicMetrics;

use super::files;

/// `counts.json` beside the session.
#[derive(Debug, Clone)]
pub(crate) struct CountsFile {
    path: PathBuf,
}

impl CountsFile {
    pub(crate) fn at(path: PathBuf) -> Self {
        Self { path }
    }

    /// The counts last saved, or `None` when there are none or they do not parse.
    pub(crate) fn load(&self) -> Option<HttpPublicMetrics> {
        serde_json::from_str(&fs::read_to_string(&self.path).ok()?).ok()
    }

    /// Saves `metrics` for the next launch.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be written.
    pub(crate) fn save(&self, metrics: &HttpPublicMetrics) -> Result<()> {
        files::write_private(&self.path, &serde_json::to_string(metrics)?)
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn saved_counts_come_back_and_a_missing_file_is_none() {
        let dir = std::env::temp_dir().join(format!("zwiper-tui-counts-{}", std::process::id()));
        let file = CountsFile::at(dir.join("counts.json"));
        assert!(file.load().is_none());
        file.save(&HttpPublicMetrics {
            cards_swiped: 3,
            searches: 2,
            decks_created: 1,
        })
        .unwrap();
        assert_eq!(file.load().map(|m| m.cards_swiped), Some(3));
        fs::remove_dir_all(dir).unwrap();
    }
}
