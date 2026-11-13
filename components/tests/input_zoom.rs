//! On a touch screen every text field reads at 16px or more, so focusing one never zooms the
//! page (iOS zooms into a field under 16px), while a mouse keeps the compact sizes.

const FILTER: &str = include_str!("../assets/filter.css");
const GUIDES: &str = include_str!("../assets/guides.css");

/// The body of the first rule in `css` from `selector` after `media`.
fn touch_rule<'css>(css: &'css str, selector: &str) -> Option<&'css str> {
    let media = css.find("@media (pointer: coarse) {")?;
    let at = css.get(media..)?.find(selector)?.checked_add(media)?;
    css.get(at..)?.split('}').next()
}

#[test]
fn a_compact_field_reads_at_16px_on_a_touch_screen() {
    let compact = touch_rule(FILTER, ".input-compact {").expect("the compact field's touch rule");
    assert!(compact.contains("font-size: 1rem;"), "{compact}");
    let guides =
        touch_rule(GUIDES, ".guide-search-input {").expect("the guides search's touch rule");
    assert!(guides.contains("font-size: 1rem;"), "{guides}");
}
