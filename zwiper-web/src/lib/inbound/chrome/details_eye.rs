//! The swipe surfaces' eye (card details) button. Below the breakpoint it
//! opens the details sheet, as in the app. On a wide window the details sit
//! beside the card, and the eye hides and shows that panel instead, lit
//! while the panel shows.

use components::icons::EyeIcon;
use dioxus::prelude::*;

use crate::inbound::chrome::media::use_wide;

/// The class a swipe surface's body takes while its beside-the-card
/// details are hidden.
pub const DETAILS_HIDDEN_CLASS: &str = "web-details-hidden";

/// What a press of the eye toggles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EyePress {
    /// The details sheet, below the breakpoint.
    Sheet,
    /// The details panel beside the card, on a wide window.
    Beside,
}

impl EyePress {
    /// What a press toggles at this width.
    #[must_use]
    pub fn at(wide: bool) -> Self {
        if wide { Self::Beside } else { Self::Sheet }
    }
}

/// Whether the eye shows as pressed: on a wide window while the panel
/// beside the card shows. Below the breakpoint it never lights, as in the
/// app, where the sheet covers it.
#[must_use]
pub fn eye_lit(wide: bool, beside_hidden: bool) -> bool {
    wide && !beside_hidden
}

/// One surface's eye: the window's width, the surface's details sheet and
/// whether the panel beside the card is hidden.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetailsEye {
    wide: Signal<bool>,
    sheet: Signal<bool>,
    beside_hidden: Signal<bool>,
}

impl DetailsEye {
    /// Toggles the sheet below the breakpoint and the beside panel on a
    /// wide window.
    pub fn press(self) {
        let Self {
            wide,
            mut sheet,
            mut beside_hidden,
        } = self;
        match EyePress::at(*wide.peek()) {
            EyePress::Sheet => sheet.toggle(),
            EyePress::Beside => beside_hidden.toggle(),
        }
    }

    /// Whether the eye shows as pressed (see [`eye_lit`]).
    #[must_use]
    pub fn lit(self) -> bool {
        eye_lit((self.wide)(), (self.beside_hidden)())
    }

    /// Whether the eye is on a wide window, where it is a toggle.
    #[must_use]
    pub fn wide(self) -> bool {
        (self.wide)()
    }

    /// The class for the surface's body: [`DETAILS_HIDDEN_CLASS`] while the
    /// beside panel is hidden, else empty.
    #[must_use]
    pub fn body_class(self) -> &'static str {
        if (self.beside_hidden)() {
            DETAILS_HIDDEN_CLASS
        } else {
            ""
        }
    }
}

/// The eye for a surface whose details sheet opens on `sheet`. The beside
/// panel starts shown.
pub fn use_details_eye(sheet: Signal<bool>) -> DetailsEye {
    let wide = use_wide();
    let beside_hidden = use_signal(|| false);
    DetailsEye {
        wide,
        sheet,
        beside_hidden,
    }
}

/// The eye as a dock button, disabled while there is no card. On a wide
/// window it carries its pressed state as the dock's lit toggle and
/// `aria-pressed`.
#[component]
pub fn DockEye(eye: DetailsEye, disabled: bool) -> Element {
    let lit = eye.lit();
    let pressed = eye.wide().then(|| lit.to_string());
    rsx! {
        button {
            class: if lit { "action-dock-btn web-dock-eye action-dock-on" } else { "action-dock-btn web-dock-eye" },
            "aria-label": "Card details",
            "aria-pressed": pressed,
            disabled,
            onclick: move |_| eye.press(),
            EyeIcon {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_eye_opens_the_sheet_below_the_breakpoint() {
        assert_eq!(EyePress::at(false), EyePress::Sheet);
    }

    #[test]
    fn the_eye_toggles_the_beside_panel_on_a_wide_window() {
        assert_eq!(EyePress::at(true), EyePress::Beside);
    }

    #[test]
    fn the_eye_lights_on_a_wide_window_while_the_panel_shows() {
        assert!(eye_lit(true, false));
        assert!(!eye_lit(true, true));
    }

    #[test]
    fn the_eye_never_lights_below_the_breakpoint() {
        assert!(!eye_lit(false, false));
        assert!(!eye_lit(false, true));
    }
}
