//! Whether the window matches a media query: at least a width, or a computer's wide layout, for
//! the places where the window changes what a screen does and not only how it looks.

use dioxus::prelude::*;
use uuid::Uuid;

/// The width, 60rem, from which the kit draws a sheet form as a modal.
pub const KIT_MODAL_PX: u32 = 960;

/// The query for a computer: a pointer that hovers and is precise, which a phone's touch is
/// not, with the window on the wide layout, from 900px.
pub const COMPUTER_QUERY: &str = "(hover: hover) and (pointer: fine) and (min-width: 900px)";

/// True on a computer's wide layout ([`COMPUTER_QUERY`]), kept current as the window resizes.
/// Off the browser it stays false.
#[must_use]
pub fn use_computer() -> Signal<bool> {
    use_media(COMPUTER_QUERY.to_owned())
}

/// True while the window is at least `px` wide, kept current as it resizes.
/// Off the browser it stays false.
#[must_use]
pub fn use_min_width(px: u32) -> Signal<bool> {
    use_media(format!("(min-width: {px}px)"))
}

/// True while the window matches the media `query`, kept current as it changes. Off the browser
/// it stays false.
fn use_media(query: String) -> Signal<bool> {
    let mut matches = use_signal(|| false);
    let id = use_hook(Uuid::new_v4);
    use_hook(move || {
        let script = MEDIA_JS
            .replace("__QUERY__", &query)
            .replace("__ID__", &id.to_string());
        spawn(async move {
            let mut changes = document::eval(&script);
            while let Ok(now) = changes.recv::<bool>().await {
                matches.set(now);
            }
        });
    });
    use_drop(move || {
        document::eval(&MEDIA_DROP_JS.replace("__ID__", &id.to_string()));
    });
    matches
}

/// Sends whether the window matches, then again on each change.
const MEDIA_JS: &str = r"
const query = window.matchMedia('__QUERY__');
const onChange = (e) => dioxus.send(e.matches);
window['__zwipeWidth__ID__'] = [query, onChange];
dioxus.send(query.matches);
query.addEventListener('change', onChange);
";

/// Stops one listener.
const MEDIA_DROP_JS: &str = r"
const held = window['__zwipeWidth__ID__'];
if (held) held[0].removeEventListener('change', held[1]);
delete window['__zwipeWidth__ID__'];
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_listener_matches_the_query_it_is_given() {
        assert!(
            MEDIA_JS.contains("window.matchMedia('__QUERY__')"),
            "{MEDIA_JS}"
        );
        assert!(COMPUTER_QUERY.starts_with("(hover: hover) and (pointer: fine)"));
    }
}
