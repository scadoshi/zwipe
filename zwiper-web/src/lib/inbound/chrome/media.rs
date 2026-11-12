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
    fn the_swipe_chips_clear_the_folded_nav() {
        // From the wide layout's 900px up to the fold, the folded nav
        // overlays the swipe screen's top, so its chips drop by its height.
        let band = "@media (min-width: 900px) and (max-width: 1109px) {";
        let at = WEB_CSS.find(band).expect("a rule for the folded wide band");
        let rules = WEB_CSS.get(at..).unwrap();
        let rules = rules.get(..rules.find("\n}\n").unwrap()).unwrap();
        let chips = rules
            .find("    .web-cards-swipe > .web-screen-chips {")
            .and_then(|i| rules.get(i..))
            .expect("the chips' rule in that band");
        let chips = chips.get(..chips.find('}').unwrap()).unwrap();
        assert!(chips.contains("var(--web-nav-h"), "{chips}");
    }

    /// The body of `selector`'s rule inside the block that opens with `query`.
    fn rule_in(query: &str, selector: &str) -> &'static str {
        let at = WEB_CSS.find(query).unwrap_or_else(|| panic!("no {query}"));
        let block = WEB_CSS.get(at..).unwrap();
        let block = block.get(..block.find("\n}\n").unwrap()).unwrap();
        let rule = block
            .find(&format!("    {selector} {{"))
            .and_then(|i| block.get(i..))
            .unwrap_or_else(|| panic!("no {selector} in {query}"));
        rule.get(..rule.find('}').unwrap()).unwrap()
    }

    #[test]
    fn the_deck_page_runs_the_window_s_width_and_pairs_its_groups_from_1400() {
        let page = rule_in(
            "@media (min-width: 900px) {\n    .web-deck-page {",
            ".web-page.web-deck-page",
        );
        assert!(page.contains("max-width: none"), "{page}");
        let columns = rule_in(
            "@media (min-width: 1400px) {\n    .web-deck-page .deck-list-columns",
            ".web-deck-page .deck-list-columns",
        );
        assert!(
            columns.contains("grid-template-columns: repeat(2, minmax(0, 1fr))"),
            "{columns}"
        );
        assert!(
            !WEB_CSS.contains("column-count"),
            "no CSS multi-column, which Firefox lays out again on every change"
        );
    }

    #[test]
    fn the_few_panel_pages_share_zwipe_net_s_1400px_column_and_text_fills_it() {
        const ZITE_CSS: &str = include_str!("../../../../../zite/assets/style.css");
        let root = WEB_CSS.get(..WEB_CSS.find("\n}\n").unwrap()).unwrap();
        assert!(root.contains("--web-page-max: 87.5rem;"), "{root}");
        assert!(
            ZITE_CSS.contains("--zite-page-max: 87.5rem;"),
            "zite's column moved"
        );
        let page = rule_in(
            "@media (min-width: 900px) {\n    /* Pinned to the top",
            ".web-page",
        );
        assert!(page.contains("max-width: var(--web-page-max)"), "{page}");
        // Home takes the shared column rather than a width of its own, and
        // no text in it or elsewhere stops at a character measure.
        assert!(!WEB_CSS.contains("70ch"), "text fills its panel");
        assert!(
            !WEB_CSS.contains(".web-page.web-home-page {"),
            "Home has no width of its own"
        );
        // Decks and Colors share a row at every wide width, with Featured
        // flavor on its own row under them.
        assert!(
            !WEB_CSS.contains(".web-home .web-home-page {\n        display: grid;"),
            "Home never puts its three panels in one row"
        );
    }

    #[test]
    fn import_fills_its_column_as_export_does() {
        let import = rule_in(
            "@media (min-width: 900px) {\n    /* Import and Export",
            ".web-import.screen-content",
        );
        assert!(
            import.contains("grid-template-columns: minmax(0, 1fr)"),
            "{import}"
        );
        let results = rule_in(
            "@media (min-width: 900px) {\n    /* Import and Export",
            ".web-import-results",
        );
        assert!(
            !results.contains("grid-column: 2"),
            "results sit under the input: {results}"
        );
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
