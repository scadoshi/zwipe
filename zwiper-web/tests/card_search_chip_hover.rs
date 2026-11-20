//! The card search's result chips light under a mouse as every other chip a click acts on does:
//! they sit in the one hover rule the web app's clickable chips share.

const WEB: &str = include_str!("../assets/web.css");

#[test]
fn a_result_chip_lights_with_the_other_clickable_chips() {
    let (_, rest) = WEB
        .split_once("/* Every chip a click acts on, the same way:")
        .unwrap_or_else(|| panic!("the clickable chips' hover"));
    let (selectors, body) = rest
        .split_once("*/")
        .and_then(|(_, rule)| rule.split_once('{'))
        .unwrap_or_else(|| panic!("its rule"));
    assert!(
        selectors.contains(".card-search-chip"),
        "the result chips share the rule: {selectors}"
    );
    assert!(
        body.contains("border-color: var(--accent-primary);"),
        "{body}"
    );
}
