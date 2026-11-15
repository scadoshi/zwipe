//! The Colors bars round their corners in the deck charts' bars' proportion
//! at any thickness: a host that grows `--color-bar-thickness` grows the
//! radius with it, and the compact default lands on the deck bars' radius.

const MAGIC: &str = include_str!("../assets/magic.css");

/// The declarations of the rule whose selector is exactly `selector`.
fn rule(selector: &str) -> Option<&'static str> {
    let start = MAGIC.find(&format!("\n{selector} {{"))?;
    let body = MAGIC.get(start..)?;
    let open = body.find('{')?;
    let close = body.find('}')?;
    body.get(open.checked_add(1)?..close)
}

/// The value of `property` in `body`.
fn value<'css>(body: &'css str, property: &str) -> Option<&'css str> {
    body.lines()
        .find_map(|line| line.trim().strip_prefix(&format!("{property}:")))
        .map(|rest| rest.trim().trim_end_matches(';'))
}

/// A length in rem, as a number.
fn rem(length: &str) -> Option<f64> {
    length.trim().trim_end_matches("rem").parse().ok()
}

#[test]
fn the_colors_bars_keep_the_deck_bars_corner_proportion_as_they_grow() {
    let deck = rule(".chart-track").unwrap();
    let deck_height = rem(value(deck, "height").unwrap()).unwrap();
    let deck_radius = rem(value(deck, "border-radius").unwrap()).unwrap();
    let ratio = deck_height / deck_radius;

    for selector in [".color-bar-track", ".skeleton-bar.color-bar-track"] {
        let body = rule(selector).unwrap();
        let radius = value(body, "border-radius").unwrap();
        let Some(inner) = radius
            .strip_prefix("calc(var(--color-bar-thickness, ")
            .and_then(|rest| rest.split_once(") / "))
        else {
            panic!("{selector}'s radius doesn't follow the thickness: {radius}");
        };
        let (default, divisor) = inner;
        let divisor: f64 = divisor.trim_end_matches(')').parse().unwrap();
        assert!(
            (divisor - ratio).abs() < 1e-9,
            "{selector} divides by {divisor}, the deck bars' height over radius is {ratio}"
        );
        assert_eq!(
            value(body, "height"),
            Some(format!("var(--color-bar-thickness, {default})").as_str()),
            "{selector}'s radius and height read the same default"
        );
        assert!(
            (rem(default).unwrap() / divisor - deck_radius).abs() < 1e-9,
            "{selector}'s compact bar keeps the deck bars' radius"
        );
    }
}
