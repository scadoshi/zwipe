//! Domain types and HTTP contracts shared by zerver, zwiper and zite.
#![warn(missing_docs)]

/// Static content every edge renders, such as the changelog.
pub mod content;
/// Entities, value objects and requests, grouped by business area.
pub mod domain;
pub mod http;
pub mod legal;
pub mod serde_helpers;
pub mod version;

#[cfg(test)]
pub mod test_utils;
