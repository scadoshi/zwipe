//! A modal's own dock stays while a form drawer below it is open: the drawer
//! slides the page's docks away, never the dock of a modal over it (an Oracle
//! tag's examples opened from the filter's dictionary, or from the deck form's
//! Oracle tags picker, where no page screen holds it).

const MAGIC: &str = include_str!("../assets/magic.css");

/// The web's large modals: the Oracle tag dictionary and its examples.
const LARGE_MODAL: &str = ".web-large-modal *";

#[test]
fn the_drawer_hides_only_the_page_s_docks() {
    let rules: Vec<&str> = MAGIC
        .lines()
        .filter(|line| {
            line.contains("body:has(.sheet-form-drawer > .bottom-sheet.show) .action-dock")
        })
        .collect();
    assert!(!rules.is_empty(), "the drawer's dock rules");
    for rule in rules {
        assert!(
            rule.contains(".screen .screen *"),
            "a nested screen's dock is spared: {rule}"
        );
        assert!(
            rule.contains(LARGE_MODAL),
            "a large modal's dock is spared wherever it mounts: {rule}"
        );
    }
}

/// The Printings sheet's dock, Flip or Meld beside Select, which the sheet draws over a
/// details drawer it opened from.
const PRINTINGS_DOCK: &str = ".web-printing-save";

#[test]
fn the_drawer_spares_the_printings_sheet_s_dock() {
    let rules: Vec<&str> = MAGIC
        .lines()
        .filter(|line| {
            line.contains("body:has(.sheet-form-drawer > .bottom-sheet.show) .action-dock")
        })
        .collect();
    assert!(!rules.is_empty(), "the drawer's dock rules");
    for rule in rules {
        assert!(
            rule.contains(PRINTINGS_DOCK),
            "Printings opened from the drawer keeps Flip and Select: {rule}"
        );
    }
}
