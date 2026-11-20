//! Playstyle's identity chips: an identity drawn as its name and pips.

use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::colors::{Color, identity_name};
use zwipe_ui::playstyle::identity_label;

/// A chip's stable key: the identity's codes, `C` for colorless.
pub(crate) fn identity_key(identity: &[Color]) -> String {
    if identity.is_empty() {
        "C".to_string()
    } else {
        identity.iter().map(Color::to_short_name).collect()
    }
}

/// An identity's name, then its pips in the deck list's group-header
/// grammar, then `count` when given. A single color is its pip alone;
/// colorless is the name alone. The pips are hidden from a screen reader,
/// which reads the name in their place when it isn't shown.
#[component]
pub fn IdentityPips(
    identity: Vec<Color>,
    #[props(default)] count: Option<String>,
    #[props(default = true)] show_name: bool,
    /// Gives screen readers the identity's name when none shows; off beside
    /// a name the host already shows, such as the Top color row's.
    #[props(default = true)]
    speak: bool,
) -> Element {
    let name = identity_label(&identity).filter(|_| show_name);
    let spoken = spoken(&identity, show_name, speak);
    rsx! {
        span { class: "identity-pips",
            if let Some(name) = name {
                span { "{name}" }
            } else if let Some(spoken) = spoken {
                span { class: "sr-only", "{spoken}" }
            }
            for color in identity.iter() {
                i {
                    key: "{color.to_short_name()}",
                    class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost",
                    "aria-hidden": "true",
                }
            }
            if let Some(count) = count {
                span { "{count}" }
            }
        }
    }
}

/// What screen readers hear for `identity` where no name shows: its name
/// (Mono-red for a single color), or nothing when `speak` is off or a name
/// shows.
fn spoken(identity: &[Color], show_name: bool, speak: bool) -> Option<String> {
    let shown = show_name && identity_label(identity).is_some();
    (speak && !shown && !identity.is_empty()).then(|| identity_name(identity).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_color_beside_its_shown_name_speaks_nothing_more() {
        let red = [Color::Red];
        assert_eq!(
            spoken(&red, true, true).as_deref(),
            Some("Mono-red"),
            "an identity chip"
        );
        assert_eq!(
            spoken(&red, true, false),
            None,
            "the Top color row names it already"
        );
        let boros = [Color::Red, Color::White];
        assert_eq!(
            spoken(&boros, true, true),
            None,
            "a named identity shows its name"
        );
    }
}
