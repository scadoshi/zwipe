//! Staples' Type and Color chip rows sit above its panel at every width, a panel gap over it, as
//! they do on a wide window and as Suggested's controls do: never inside the panel on a phone.

/// The Staples panel's source.
const ROWS: &str = include_str!("../src/lib/inbound/screens/playstyle/rows.rs");

/// The page's loading ghost.
const SKELETON: &str = include_str!("../src/lib/inbound/screens/playstyle/skeleton.rs");

/// The web app's own sheet.
const WEB_CSS: &str = include_str!("../assets/web.css");

#[test]
fn the_chip_rows_sit_above_the_panel_at_every_width() {
    for (name, src) in [("the panel", ROWS), ("its ghost", SKELETON)] {
        // From the Staples cell on.
        let (_, src) = src
            .split_once("web-playstyle-cell web-playstyle-staples")
            .unwrap_or_default();
        assert!(
            !src.contains("web-card-group-filters"),
            "{name} never puts the rows inside the panel"
        );
        assert!(
            src.contains(r#"div { class: "deck-controls""#),
            "{name} puts them in the controls over it"
        );
        assert!(!src.contains("if wide {"), "{name}: at every width");
    }
}

#[test]
fn the_panel_gap_over_the_panel_holds_at_every_width() {
    // Unindented: a top-level rule, not one inside the wide window's media query.
    let at = WEB_CSS
        .find("\n.web-playstyle-staples {")
        .unwrap_or_else(|| panic!("a rule outside the wide window's"));
    let rule = WEB_CSS.get(at..).unwrap_or_default();
    let rule = rule.split_once('}').map_or("", |(body, _)| body);
    for want in [
        "display: flex;",
        "flex-direction: column;",
        "gap: var(--panel-gap);",
    ] {
        assert!(rule.contains(want), "{want} in {rule}");
    }
}
