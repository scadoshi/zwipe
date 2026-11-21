//! The commander swipe's card details: "Why this commander?" folds under the Suggested row's
//! chip in both the wide panel and the phone's sheet, so the swipe matches the card row.

const SWIPE: &str = include_str!("../src/lib/inbound/cards/swipe_select.rs");
const REASONS: &str = include_str!("../src/lib/inbound/screens/playstyle/rows.rs");

#[test]
fn the_panel_and_the_sheet_both_carry_the_reasons_in_the_body() {
    let body = SWIPE.split("#[cfg(test)]").next().unwrap_or_default();
    let blocks = body.split("body_end:").skip(1).filter(|rest| {
        rest.split("}),")
            .next()
            .is_some_and(|slot| slot.contains("SuggestionReasons {"))
    });
    assert_eq!(blocks.count(), 2, "the wide panel's and the sheet's");
}

#[test]
fn the_reasons_always_fold_under_their_chip() {
    let block = REASONS
        .split("pub fn SuggestionReasons(")
        .nth(1)
        .and_then(|rest| rest.split("\n}\n").next())
        .expect("the reasons block");
    assert!(
        block.contains("Chip {") && block.contains("\"collapsible open\""),
        "the row's chip and its fold"
    );
    assert!(!block.contains("chips-label"), "no open heading form");
}
