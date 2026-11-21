//! The commander swipe's card details: "Why this commander?" sits in the sheet's body as the
//! Suggested row's folded chip, so the footer keeps only its actions and the body isn't squeezed.

const SWIPE: &str = include_str!("../src/lib/inbound/screens/deck/components/swipe_select.rs");
const REASONS: &str = include_str!("../src/lib/inbound/screens/deck/suggested_commanders.rs");

#[test]
fn the_reasons_sit_in_the_body_not_the_footer() {
    let sheet = SWIPE
        .split("CardDetailsSheet {")
        .nth(1)
        .and_then(|rest| rest.split("PrintingSheet {").next())
        .expect("the swipe's details sheet");
    assert!(!sheet.contains("footer:"), "no reasons in the footer");
    let body_end = sheet
        .split("body_end:")
        .nth(1)
        .expect("the reasons in the body's end");
    assert!(body_end.contains("SuggestionReasons {"), "{body_end}");
}

#[test]
fn the_reasons_always_fold_under_their_chip() {
    let block = REASONS
        .split("pub(crate) fn SuggestionReasons(")
        .nth(1)
        .and_then(|rest| rest.split("\n}\n").next())
        .expect("the reasons block");
    assert!(
        block.contains("Chip {") && block.contains("\"collapsible open\""),
        "the row's chip and its fold"
    );
    assert!(!block.contains("chips-label"), "no open heading form");
}
