//! The Oracle tags dictionary answers `/` and its letter keys over the sheet that opened it (the
//! deck page's Filter, the deck form's Oracle tags picker), at every width.

/// The dictionary's source.
const SRC: &str = include_str!("../src/lib/inbound/screens/oracle_tag_dictionary.rs");

#[test]
fn the_dictionary_listens_while_it_is_the_topmost_modal() {
    let compact: String = SRC.split_whitespace().collect();
    assert!(
        compact.contains("use_modal_keys(DICTIONARY,"),
        "its keys pass while it is on top, not only while no sheet is open"
    );
    assert!(
        !SRC.contains("use_page_keys("),
        "the page guard holds every key under the sheet it sits over"
    );
}
