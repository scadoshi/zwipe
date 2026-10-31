//! The changelog: the copy fetched once at launch, and how the Changelog screen files its releases under years, months and minor versions.

use zwipe_core::http::contracts::changelog::HttpChangelog;

/// The changelog as fetched at launch, logged in or not. Fetched once per launch, with no retry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum ChangelogCache {
    #[default]
    Loading,
    Loaded(HttpChangelog),
    Failed,
}
