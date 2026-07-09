//! Client platform of a session.
//!
//! Which kind of client created (or last rotated) a session. Recorded per
//! session — a single user can hold sessions from several platforms at once —
//! for platform analytics and targeted comms. Serialized/stored as a lowercase
//! string (`"ios"`, `"android"`, `"desktop"`, `"web"`).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// The platform a client is running on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClientPlatform {
    Ios,
    Android,
    Desktop,
    Web,
}

impl ClientPlatform {
    /// Lowercase wire/storage token.
    pub fn as_str(&self) -> &'static str {
        match self {
            ClientPlatform::Ios => "ios",
            ClientPlatform::Android => "android",
            ClientPlatform::Desktop => "desktop",
            ClientPlatform::Web => "web",
        }
    }

    /// The platform this build is compiled for. On non-client targets (e.g. the
    /// server) this resolves to `Desktop` and is simply never read there.
    #[cfg(target_os = "ios")]
    pub const CURRENT: ClientPlatform = ClientPlatform::Ios;
    #[cfg(target_os = "android")]
    pub const CURRENT: ClientPlatform = ClientPlatform::Android;
    #[cfg(all(target_arch = "wasm32", not(any(target_os = "ios", target_os = "android"))))]
    pub const CURRENT: ClientPlatform = ClientPlatform::Web;
    #[cfg(all(
        not(target_os = "ios"),
        not(target_os = "android"),
        not(target_arch = "wasm32")
    ))]
    pub const CURRENT: ClientPlatform = ClientPlatform::Desktop;
}

impl fmt::Display for ClientPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The string did not name a known [`ClientPlatform`].
#[derive(Debug, thiserror::Error)]
#[error("unknown client platform: {0}")]
pub struct ParseClientPlatformError(pub String);

impl FromStr for ClientPlatform {
    type Err = ParseClientPlatformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ios" => Ok(ClientPlatform::Ios),
            "android" => Ok(ClientPlatform::Android),
            "desktop" => Ok(ClientPlatform::Desktop),
            "web" => Ok(ClientPlatform::Web),
            other => Err(ParseClientPlatformError(other.to_string())),
        }
    }
}
