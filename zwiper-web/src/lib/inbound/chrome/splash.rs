//! Whether this load opens on the splash: once a tab session, as on
//! zwipe.net. A fresh tab plays it. A reload, a later load in the same tab,
//! a landing from zwipe.net and a viewer who asked for less motion skip it.
//!
//! The answer is read once, before the first render, so the overlay never
//! flashes on a load that skips it. `sessionStorage` does not cross origins,
//! so a landing from zwipe.net shows in `document.referrer`, which both
//! sites send as the origin (`Referrer-Policy: strict-origin-when-cross-origin`).

/// The tab session's mark that the splash has played, the key zwipe.net
/// uses too.
#[cfg(target_arch = "wasm32")]
const SEEN_KEY: &str = "splash.seen";

/// The sister site's origins: a landing from either skips the splash.
const SITE_ORIGINS: [&str; 2] = ["https://zwipe.net", "https://www.zwipe.net"];

/// What a load knows when it decides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplashLoad<'a> {
    /// The splash already played in this tab session.
    pub seen_this_session: bool,
    /// `document.referrer`: the page this load came from, empty for none.
    pub referrer: &'a str,
    /// The viewer asked for less motion.
    pub reduced_motion: bool,
}

impl SplashLoad<'_> {
    /// Whether the splash plays: only on a tab session's first load, not
    /// arriving from zwipe.net, for a viewer who has not asked for less
    /// motion.
    #[must_use]
    pub fn plays(&self) -> bool {
        !self.seen_this_session && !self.reduced_motion && !from_site(self.referrer)
    }
}

/// Whether `referrer` is a page on zwipe.net.
fn from_site(referrer: &str) -> bool {
    SITE_ORIGINS.iter().any(|origin| {
        referrer
            .strip_prefix(origin)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    })
}

/// Decides this load and marks the tab session as having seen the splash.
/// Off the browser it always plays.
#[must_use]
pub fn splash_plays_this_load() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            return true;
        };
        let session = window.session_storage().ok().flatten();
        let seen_this_session = session
            .as_ref()
            .and_then(|s| s.get_item(SEEN_KEY).ok().flatten())
            .is_some_and(|v| v == "1");
        if let Some(session) = session
            && session.set_item(SEEN_KEY, "1").is_err()
        {
            tracing::debug!("could not mark the splash as seen");
        }
        let referrer = window.document().map(|d| d.referrer()).unwrap_or_default();
        let reduced_motion = window
            .match_media("(prefers-reduced-motion: reduce)")
            .ok()
            .flatten()
            .is_some_and(|query| query.matches());
        SplashLoad {
            seen_this_session,
            referrer: &referrer,
            reduced_motion,
        }
        .plays()
    }
    #[cfg(not(target_arch = "wasm32"))]
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRESH: SplashLoad<'static> = SplashLoad {
        seen_this_session: false,
        referrer: "",
        reduced_motion: false,
    };

    #[test]
    fn a_fresh_tab_plays_it() {
        assert!(FRESH.plays());
    }

    #[test]
    fn a_later_load_in_the_tab_skips_it() {
        assert!(
            !SplashLoad {
                seen_this_session: true,
                ..FRESH
            }
            .plays()
        );
    }

    #[test]
    fn less_motion_skips_it() {
        assert!(
            !SplashLoad {
                reduced_motion: true,
                ..FRESH
            }
            .plays()
        );
    }

    #[test]
    fn a_landing_from_zwipe_net_skips_it() {
        for referrer in [
            "https://zwipe.net/",
            "https://zwipe.net",
            "https://www.zwipe.net/",
            "https://zwipe.net/deck/abc",
        ] {
            assert!(!SplashLoad { referrer, ..FRESH }.plays(), "{referrer}");
        }
    }

    #[test]
    fn a_landing_from_elsewhere_plays_it() {
        for referrer in [
            "https://app.zwipe.net/",
            "https://zwipe.network/",
            "https://zwipe.net.example.com/",
            "http://zwipe.net/",
            "https://www.google.com/",
        ] {
            assert!(SplashLoad { referrer, ..FRESH }.plays(), "{referrer}");
        }
    }
}
