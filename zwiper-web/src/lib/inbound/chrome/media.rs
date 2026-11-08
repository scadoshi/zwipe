//! Whether the window is at least a width, for the few places where a wide
//! window changes what a screen does and not only how it looks.

use dioxus::prelude::*;
use uuid::Uuid;

/// The width at which the layout leaves the phone's: the stylesheet's wide
/// breakpoint.
pub const WIDE_PX: u32 = 900;

/// The widest window whose header folds behind the hamburger. The full bar
/// needs about 1110px on one row, so below that it folds rather than wrap.
pub const NAV_FOLD_MAX_PX: u32 = 1109;

/// The kit's nav fold rules under [`NAV_FOLD_MAX_PX`], mounted after its own
/// sheet so the header folds up to that width and not only to 60rem.
pub fn nav_fold_css() -> String {
    format!(
        "@media (max-width: {NAV_FOLD_MAX_PX}px) {{\n{}}}",
        components::NAV_FOLD_CSS
    )
}

/// The width, 60rem, from which the kit draws a sheet form as a modal.
pub const KIT_MODAL_PX: u32 = 960;

/// True while the window is wide, as [`use_min_width`] with [`WIDE_PX`].
pub fn use_wide() -> Signal<bool> {
    use_min_width(WIDE_PX)
}

/// True while the window is at least `px` wide, kept current as it resizes.
/// Off the browser it stays false.
pub fn use_min_width(px: u32) -> Signal<bool> {
    let mut wide = use_signal(|| false);
    let id = use_hook(Uuid::new_v4);
    use_hook(move || {
        let script = MIN_WIDTH_JS
            .replace("__PX__", &px.to_string())
            .replace("__ID__", &id.to_string());
        spawn(async move {
            let mut changes = document::eval(&script);
            while let Ok(matches) = changes.recv::<bool>().await {
                wide.set(matches);
            }
        });
    });
    use_drop(move || {
        document::eval(&MIN_WIDTH_DROP_JS.replace("__ID__", &id.to_string()));
    });
    wide
}

/// Sends whether the window matches, then again on each change.
const MIN_WIDTH_JS: &str = r"
const query = window.matchMedia('(min-width: __PX__px)');
const onChange = (e) => dioxus.send(e.matches);
window['__zwipeWidth__ID__'] = [query, onChange];
dioxus.send(query.matches);
query.addEventListener('change', onChange);
";

/// Stops one listener.
const MIN_WIDTH_DROP_JS: &str = r"
const held = window['__zwipeWidth__ID__'];
if (held) held[0].removeEventListener('change', held[1]);
delete window['__zwipeWidth__ID__'];
";

#[cfg(test)]
mod tests {
    use super::*;

    const WEB_CSS: &str = include_str!("../../../../assets/web.css");
    const HEADER_HIDE_JS: &str = include_str!("../../../../assets/header_hide.js");

    /// The query right before the first rule for `selector` in web.css.
    fn query_for(selector: &str) -> &'static str {
        let at = WEB_CSS
            .find(&format!("    {selector} {{"))
            .unwrap_or_else(|| panic!("no rule for {selector}"));
        let head = WEB_CSS.get(..at).unwrap();
        let query = head.rfind("@media").unwrap();
        let line = head.get(query..).unwrap();
        line.get(..line.find('{').unwrap()).unwrap().trim()
    }

    #[test]
    fn the_header_folds_below_1110_and_never_wraps() {
        assert_eq!(NAV_FOLD_MAX_PX, 1109);
        assert!(nav_fold_css().starts_with("@media (max-width: 1109px) {"));
        assert!(nav_fold_css().contains(components::NAV_FOLD_CSS));
        for folded in [".web-nav-wide", ".web-nav-hint", ".web-shell"] {
            assert_eq!(query_for(folded), "@media (max-width: 1109px)", "{folded}");
        }
        assert_eq!(query_for(".web-nav-narrow"), "@media (min-width: 1110px)");
        assert!(HEADER_HIDE_JS.contains("matchMedia(\"(max-width: 1109px)\")"));
        assert!(
            !WEB_CSS.contains("and (max-width: 60rem)"),
            "no band where the bar is held open to wrap"
        );
    }
}
