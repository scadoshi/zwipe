//! Which copy of the theme wins when the server's settings arrive.
//!
//! A client keeps two things on the device: the theme it last showed,
//! written on every change, and the server's theme as it last saw it. When
//! the server's settings land, the one that changed more recently wins: the
//! server's copy, when it moved since this device last saw it (another device
//! chose it); else this device's, which then goes up to the server.

use orangui_core::theme::ThemeConfig;

/// What to do with the server's theme once it is fetched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sync {
    /// The client already shows it; nothing moves.
    Keep,
    /// The server's copy is the newer choice: show it.
    Apply(ThemeConfig),
    /// This device's choice is the newer one: send it to the server.
    Push(ThemeConfig),
}

/// Which theme wins between the one `shown`, the server's `fetched` copy and
/// the server's copy as this device `last_seen` it. With no record of the
/// server's copy the server wins, as at a first load.
#[must_use]
pub fn reconcile(
    shown: &ThemeConfig,
    last_seen: Option<&ThemeConfig>,
    fetched: &ThemeConfig,
) -> Sync {
    if shown == fetched {
        return Sync::Keep;
    }
    match last_seen {
        Some(seen) if seen == fetched => Sync::Push(shown.clone()),
        _ => Sync::Apply(fetched.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme(name: &str, is_dark: bool) -> ThemeConfig {
        ThemeConfig {
            name: name.to_string(),
            is_dark,
        }
    }

    #[test]
    fn the_server_agreeing_moves_nothing() {
        let light = theme("gruvbox", false);
        assert_eq!(reconcile(&light, Some(&light), &light), Sync::Keep);
        assert_eq!(reconcile(&light, None, &light), Sync::Keep);
    }

    #[test]
    fn a_toggle_the_server_never_got_goes_up_instead_of_flipping_back() {
        // Dark was saved and seen; the client went light and relaunched
        // before the save landed. The server still has dark, as last seen.
        let dark = theme("gruvbox", true);
        let light = theme("gruvbox", false);
        assert_eq!(reconcile(&light, Some(&dark), &dark), Sync::Push(light));
    }

    #[test]
    fn a_choice_made_on_another_device_shows_here() {
        // This device last saw dark and shows dark; the server now has
        // light, chosen elsewhere since.
        let dark = theme("gruvbox", true);
        let light = theme("gruvbox", false);
        assert_eq!(reconcile(&dark, Some(&dark), &light), Sync::Apply(light));
    }

    #[test]
    fn both_changed_the_saved_one_wins() {
        // A local toggle never reached the server, and another device saved
        // a theme: the server's choice is the one that was saved.
        let shown = theme("gruvbox", false);
        let seen = theme("gruvbox", true);
        let fetched = theme("nord", true);
        assert_eq!(
            reconcile(&shown, Some(&seen), &fetched),
            Sync::Apply(fetched)
        );
    }

    #[test]
    fn with_no_record_of_the_server_its_copy_wins() {
        let shown = theme("flexoki", false);
        let fetched = theme("nord", true);
        assert_eq!(reconcile(&shown, None, &fetched), Sync::Apply(fetched));
    }
}
