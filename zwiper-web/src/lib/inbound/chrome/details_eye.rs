//! The swipe surfaces' eye (card details) button. Below the breakpoint it
//! opens the details sheet, as in the app. On a wide window the details sit
//! beside the card, and the eye hides and shows that panel instead, lit
//! while the panel shows. The panel starts shown when the window has room
//! for it beside the card and hidden when it would touch the card, decided
//! once as the screen opens.

use components::icons::EyeIcon;
use dioxus::prelude::*;

use crate::inbound::chrome::media::use_wide;
use crate::outbound::browser;

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

/// Where a swipe surface draws its card and docks its panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    /// The whole window: Add, Remove and swipe select.
    Window,
    /// The large modal the examples open in on a wide window.
    LargeModal,
}

/// The root font size the stylesheet's rems resolve against, in pixels.
const REM: f64 = 16.0;

impl Surface {
    /// The surface's width in a window `window_w` wide: the window, or the
    /// large modal's `max(2rem, 50vw - 38rem)` inset on each side.
    fn width(self, window_w: f64) -> f64 {
        match self {
            Self::Window => window_w,
            Self::LargeModal => window_w - 2.0 * (2.0 * REM).max(window_w / 2.0 - 38.0 * REM),
        }
    }
}

/// The card's drawn width in a window `window_h` tall, as web.css sizes it:
/// its height the stack's `min(68vh, 42rem)` less the stack card's 3.25rem,
/// its width following at a card's 63:88, at most the 26rem the stack's
/// 30rem column leaves it.
fn card_width(window_h: f64) -> f64 {
    let height = (0.68 * window_h).min(42.0 * REM) - 3.25 * REM;
    (height * 63.0 / 88.0).min(26.0 * REM)
}

/// Whether the details panel fits beside the card on `surface` in a window
/// `window_w` by `window_h`: the card centered in the surface, the panel
/// `min(24rem, 100vw - 2rem)` wide and 1rem in from the right, not
/// touching it.
#[must_use]
pub fn beside_fits(surface: Surface, window_w: f64, window_h: f64) -> bool {
    let width = surface.width(window_w);
    let panel = (24.0 * REM).min(window_w - 2.0 * REM);
    let card_right = width / 2.0 + card_width(window_h) / 2.0;
    card_right < width - REM - panel
}

/// Whether the panel starts hidden on `surface` in the current window, or
/// shown off the browser.
fn starts_hidden(surface: Surface) -> bool {
    browser::window_size().is_some_and(|(w, h)| !beside_fits(surface, w, h))
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

/// The eye for `surface`, whose details sheet opens on `sheet`. The beside
/// panel starts shown where it fits beside the card (see [`beside_fits`]);
/// later resizes leave the eye as it is.
pub fn use_details_eye(sheet: Signal<bool>, surface: Surface) -> DetailsEye {
    let wide = use_wide();
    let beside_hidden = use_signal(|| starts_hidden(surface));
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
    fn the_panel_starts_shown_where_it_clears_the_card() {
        assert!(beside_fits(Surface::Window, 1280.0, 900.0));
        assert!(beside_fits(Surface::Window, 1600.0, 900.0));
        assert!(!beside_fits(Surface::Window, 1000.0, 900.0));
        assert!(!beside_fits(Surface::Window, 1000.0, 700.0));
    }

    #[test]
    fn the_large_modal_measures_from_its_own_width() {
        assert!(beside_fits(Surface::LargeModal, 1600.0, 900.0));
        assert!(!beside_fits(Surface::LargeModal, 1000.0, 900.0));
        assert!(!beside_fits(Surface::LargeModal, 1600.0, 1200.0));
    }

    #[test]
    fn the_card_follows_the_window_height_up_to_its_column() {
        assert!((card_width(900.0) - 401.0).abs() < 1.0);
        assert!((card_width(2000.0) - 416.0).abs() < f64::EPSILON);
    }

    #[test]
    fn the_eye_never_lights_below_the_breakpoint() {
        assert!(!eye_lit(false, false));
        assert!(!eye_lit(false, true));
    }
}
