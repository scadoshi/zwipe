//! A data table's bar (`DataBar`): a thin bar in its column's color, its length the value's share
//! of the column's most, the value inside it in the text color. Its ghost is the same bar in the
//! same color, pulsing, with no value.

use components::{DataBar, DataBarGhost};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;

const MAGIC: &str = include_str!("../assets/magic.css");

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    dioxus_ssr::render(&dom)
}

fn rule(selector: &str) -> &'static str {
    MAGIC
        .split_once(&format!("\n{selector} {{"))
        .and_then(|(_, rest)| rest.split_once('}'))
        .map_or("", |(body, _)| body)
}

#[test]
fn a_bar_carries_its_tone_its_share_and_its_value() {
    let html = render(|| rsx! { DataBar { percent: 57, tone: "p3", value: "17".to_owned() } });
    assert_eq!(
        html,
        r#"<span class="data-bar data-bar-p3" style="--data-bar-fill: 57%;"><span class="data-bar-value">17</span></span>"#
    );
}

#[test]
fn a_ghost_bar_is_the_bar_in_its_own_color_with_no_value() {
    let html = render(|| rsx! { DataBarGhost { percent: 40, tone: "p2" } });
    assert_eq!(
        html,
        r#"<span class="skeleton-bar data-bar data-bar-p2 data-bar-ghost" style="--data-bar-fill: 40%;"></span>"#
    );
}

#[test]
fn bars_are_thin_with_4px_ends_and_the_value_in_the_text_color() {
    let bar = rule(".data-bar");
    assert!(bar.contains("border-radius: 4px;"), "{bar}");
    let value = rule(".data-bar-value");
    assert!(
        value.contains("color: var(--text-primary);"),
        "text color, not the series color: {value}"
    );
    let cell = rule(".data-table .data-table-bar");
    assert!(
        cell.contains("padding-left: 1px;") && cell.contains("padding-right: 1px;"),
        "a 2px gap between bars: {cell}"
    );
    for (tone, palette) in [
        ("p3", "--palette-3"),
        ("p5", "--palette-5"),
        ("p2", "--palette-2"),
    ] {
        let rule = rule(&format!(".data-bar-{tone}"));
        assert!(
            rule.contains(&format!("--data-bar-tone: var({palette});")),
            "{tone}: {rule}"
        );
    }
}
