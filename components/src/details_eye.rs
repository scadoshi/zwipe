//! The swipe surfaces' eye (card details) button. Below the breakpoint it
//! opens the details sheet, as in the app. On a wide window the details sit
//! beside the card, and the eye hides and shows that panel instead, lit
//! while the panel shows. The panel starts hidden where it would touch the
//! card, and elsewhere as the eye last left it (kept by the host's
//! [`DetailsEyeHost`]), shown by default, decided once as the screen opens.

use dioxus::prelude::*;

use crate::icons::EyeIcon;
use crate::media::use_wide;

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

/// The large modal's least inset each side, web.css's `max(2rem, ..)`.
const MODAL_INSET_MIN_REM: f64 = 2.0;
/// Half the large modal's width, its `50vw - 38rem` inset.
const MODAL_HALF_REM: f64 = 38.0;
/// The swipe stack's height share of the window, `min(68vh, ..)`.
const STACK_HEIGHT_VH: f64 = 0.68;
/// The swipe stack's tallest, `min(.., 42rem)`.
const STACK_MAX_REM: f64 = 42.0;
/// What the stack card takes from the stack's height besides the card.
const STACK_CARD_CHROME_REM: f64 = 3.25;
/// A card's width to its height, 63:88.
const CARD_WIDTH: f64 = 63.0;
/// See [`CARD_WIDTH`].
const CARD_HEIGHT: f64 = 88.0;
/// The widest the card draws, what the stack's 30rem column leaves it.
const CARD_MAX_REM: f64 = 26.0;
/// The details panel's width, `min(24rem, ..)`.
const PANEL_REM: f64 = 24.0;
/// What the panel leaves of a narrow window, `min(.., 100vw - 2rem)`.
const PANEL_GUTTER_REM: f64 = 2.0;

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
            Self::LargeModal => {
                window_w
                    - 2.0 * (MODAL_INSET_MIN_REM * REM).max(window_w / 2.0 - MODAL_HALF_REM * REM)
            }
        }
    }
}

/// The card's drawn width in a window `window_h` tall, as web.css sizes it:
/// its height the stack's `min(68vh, 42rem)` less the stack card's 3.25rem,
/// its width following at a card's 63:88, at most the 26rem the stack's
/// 30rem column leaves it.
fn card_width(window_h: f64) -> f64 {
    let height =
        (STACK_HEIGHT_VH * window_h).min(STACK_MAX_REM * REM) - STACK_CARD_CHROME_REM * REM;
    (height * CARD_WIDTH / CARD_HEIGHT).min(CARD_MAX_REM * REM)
}

/// Whether the details panel fits beside the card on `surface` in a window
/// `window_w` by `window_h`: the card centered in the surface, the panel
/// `min(24rem, 100vw - 2rem)` wide and 1rem in from the right, not
/// touching it.
#[must_use]
pub fn beside_fits(surface: Surface, window_w: f64, window_h: f64) -> bool {
    let width = surface.width(window_w);
    let panel = (PANEL_REM * REM).min(window_w - PANEL_GUTTER_REM * REM);
    let card_right = width / 2.0 + card_width(window_h) / 2.0;
    card_right < width - REM - panel
}

/// Whether the panel starts hidden: always where it does not `fit` beside
/// the card, else as the eye last left it (`stored`, whether it was shown),
/// else shown.
#[must_use]
pub fn starts_hidden(fits: bool, stored: Option<bool>) -> bool {
    !fits || stored.is_some_and(|shown| !shown)
}

/// What the host keeps for the eye: the window's size and the eye's last choice, in the host's
/// storage. Without one the panel fits and starts shown, and a press is not kept.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetailsEyeHost {
    /// The window's inner width and height in CSS pixels, where the host can read them.
    pub window_size: Callback<(), Option<(f64, f64)>>,
    /// Whether the panel was last left shown, or `None` when the eye was never pressed.
    pub load: Callback<(), Option<bool>>,
    /// Keeps whether the panel is shown.
    pub save: Callback<bool>,
}

/// Whether the panel starts hidden on `surface` in the current window, with
/// the eye's stored choice; without a host it fits and starts shown.
fn starts_hidden_here(surface: Surface, host: Option<DetailsEyeHost>) -> bool {
    let window = host.and_then(|h| h.window_size.call(()));
    let fits = window.is_none_or(|(w, h)| beside_fits(surface, w, h));
    starts_hidden(fits, host.and_then(|h| h.load.call(())))
}

/// One surface's eye: the window's width, the surface's details sheet and
/// whether the panel beside the card is hidden.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetailsEye {
    wide: Signal<bool>,
    sheet: Signal<bool>,
    beside_hidden: Signal<bool>,
    host: Option<DetailsEyeHost>,
}

impl DetailsEye {
    /// Toggles the sheet below the breakpoint and the beside panel on a
    /// wide window.
    pub fn press(self) {
        let Self {
            wide,
            mut sheet,
            mut beside_hidden,
            host,
        } = self;
        match EyePress::at(*wide.peek()) {
            EyePress::Sheet => sheet.toggle(),
            EyePress::Beside => {
                beside_hidden.toggle();
                if let Some(host) = host {
                    host.save.call(!*beside_hidden.peek());
                }
            }
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
/// panel starts as [`starts_hidden`] says, from [`beside_fits`] and the
/// eye's stored choice in the [`DetailsEyeHost`] in context; later resizes leave the eye as it
/// is.
pub fn use_details_eye(sheet: Signal<bool>, surface: Surface) -> DetailsEye {
    let wide = use_wide();
    let host = try_use_context::<DetailsEyeHost>();
    let beside_hidden = use_signal(|| starts_hidden_here(surface, host));
    DetailsEye {
        wide,
        sheet,
        beside_hidden,
        host,
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
    fn the_panel_starts_as_the_eye_left_it_where_it_fits() {
        assert!(!starts_hidden(true, None));
        assert!(!starts_hidden(true, Some(true)));
        assert!(starts_hidden(true, Some(false)));
    }

    #[test]
    fn the_panel_starts_hidden_where_it_does_not_fit() {
        assert!(starts_hidden(false, None));
        assert!(starts_hidden(false, Some(true)));
        assert!(starts_hidden(false, Some(false)));
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
