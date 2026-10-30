//! Domain types and HTTP contracts shared by zerver, zwiper and zite.
#![warn(missing_docs)]

pub mod content;
pub mod domain;
pub mod http;
pub mod legal;
pub mod serde_helpers;
pub mod version;

#[cfg(test)]
pub mod test_utils;
