//! A modal's own dock stays while a form drawer below it is open: the drawer
//! slides the page's docks away, never the dock of a modal over it (an Oracle
//! tag's examples opened from the filter's dictionary).

const MAGIC: &str = include_str!("../assets/magic.css");

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
    }
}
