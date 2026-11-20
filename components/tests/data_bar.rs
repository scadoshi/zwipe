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
    let html =
        render(|| rsx! { DataBar { percent: 57, tone: "p3", value: "17".to_owned(), index: 2 } });
    assert_eq!(
        html,
        r#"<span class="data-bar data-bar-p3"><span class="data-bar-fill chart-bar" style="width: 57%; animation-delay:90ms;"></span><span class="data-bar-value chart-value" style="animation-delay:90ms;">17</span></span>"#,
        "the fill grows in and the value fades in as a chart's bar and value do, staggered by row"
    );
}

#[test]
fn a_ghost_bar_is_the_bar_in_its_own_color_with_no_value() {
    let html = render(|| rsx! { DataBarGhost { percent: 40, tone: "p2" } });
    assert_eq!(
        html,
        r#"<span class="skeleton-bar data-bar data-bar-p2 data-bar-ghost"><span class="data-bar-fill" style="width: 40%;"></span></span>"#
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

#[test]
fn a_bar_column_s_header_keeps_its_name_and_hint_on_one_line_where_its_panel_fits_it() {
    assert!(
        MAGIC.contains("@container (min-width: 28rem) {\n    .tag-data-table .data-table-bar {\n        width: 20%;\n    }\n\n    .tag-data-table thead th.data-table-bar {\n        white-space: nowrap;\n    }\n}"),
        "keyed to the panel, not the window: the bars take their share and the headers one line only in a wide panel"
    );
}

#[test]
fn a_tag_breaks_at_its_hyphens_first_and_mid_word_only_when_a_word_cannot_fit() {
    assert!(
        MAGIC.contains(":where(:has(> .tag-data-table)) {\n    container-type: inline-size;\n}"),
        "the tag table's panel is the container its rules read"
    );
    let floor = rule(".tag-data-table tbody th::before");
    assert!(
        floor.contains("width: 6.25rem;") && floor.contains("height: 0;"),
        "a narrow panel's floor: {floor}"
    );
    assert!(
        MAGIC.contains("@container (min-width: 18.5rem) {\n    .tag-data-table tbody th::before {\n        width: 8.35rem;\n    }\n}"),
        "from a 360 phone's panel up, room for the longest real word (planeswalkerfall, 16)"
    );
    let chip = rule(".tag-data-table tbody th .chip");
    assert!(
        chip.contains("overflow-wrap: anywhere;"),
        "only a word too long for the cell breaks: {chip}"
    );
    assert!(
        !chip.contains("min-width"),
        "a short chip hugs its text: {chip}"
    );
    assert!(
        !MAGIC.contains("@media (min-width: 400px)"),
        "no window breakpoint"
    );
}
