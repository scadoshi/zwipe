//! Client platform of a session.
//!
//! Which kind of client created (or last rotated) a session. Recorded per
//! session (a single user can hold sessions from several platforms at once)
//! for platform analytics and targeted comms. Serialized/stored as a lowercase
//! string (`"ios"`, `"android"`, `"desktop"`, `"web"`, `"terminal"`).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// The platform a client is running on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClientPlatform {
    /// The iOS app.
    Ios,
    /// The Android app.
    Android,
    /// The desktop app.
    Desktop,
    /// The browser client.
    Web,
    /// The terminal client (`zwiper-tui`). No build target resolves to it, so
    /// the terminal client sets it explicitly instead of using [`Self::CURRENT`].
    Terminal,
}

impl ClientPlatform {
    /// Lowercase wire/storage token.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            ClientPlatform::Ios => "ios",
            ClientPlatform::Android => "android",
            ClientPlatform::Desktop => "desktop",
            ClientPlatform::Web => "web",
            ClientPlatform::Terminal => "terminal",
        }
    }

    /// The platform this build is compiled for. On non-client targets (e.g. the
    /// server) this resolves to `Desktop` and is simply never read there.
    #[cfg(target_os = "ios")]
    pub const CURRENT: ClientPlatform = ClientPlatform::Ios;
    /// The platform this build is compiled for. On non-client targets (e.g. the
    /// server) this resolves to `Desktop` and is simply never read there.
    #[cfg(target_os = "android")]
    pub const CURRENT: ClientPlatform = ClientPlatform::Android;
    /// The platform this build is compiled for. On non-client targets (e.g. the
    /// server) this resolves to `Desktop` and is simply never read there.
    #[cfg(all(
        target_arch = "wasm32",
        not(any(target_os = "ios", target_os = "android"))
    ))]
    pub const CURRENT: ClientPlatform = ClientPlatform::Web;
    /// The platform this build is compiled for. On non-client targets (e.g. the
    /// server) this resolves to `Desktop` and is simply never read there.
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
            "terminal" => Ok(ClientPlatform::Terminal),
            other => Err(ParseClientPlatformError(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_round_trips_through_serde_and_from_str() {
        let json = serde_json::to_string(&ClientPlatform::Terminal).unwrap();
        assert_eq!(json, r#""terminal""#);
        let back: ClientPlatform = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ClientPlatform::Terminal);
        assert_eq!(ClientPlatform::Terminal.as_str(), "terminal");
        assert_eq!(
            "terminal".parse::<ClientPlatform>().unwrap(),
            ClientPlatform::Terminal
        );
    }

    #[test]
    fn every_variant_round_trips_through_as_str() {
        for platform in [
            ClientPlatform::Ios,
            ClientPlatform::Android,
            ClientPlatform::Desktop,
            ClientPlatform::Web,
            ClientPlatform::Terminal,
        ] {
            assert_eq!(
                platform.as_str().parse::<ClientPlatform>().unwrap(),
                platform,
                "{platform}"
            );
        }
    }
}
