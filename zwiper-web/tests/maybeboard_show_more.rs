//! The Commanders screen's Maybeboard shows ten rows at a time: Show more sits centered under its
//! panel and shows ten more with Show more's ghost rows, and a new sort or filter shows the first
//! ten again.

const MAYBEBOARD: &str = include_str!("../src/lib/inbound/screens/commanders/maybeboard.rs");

#[test]
fn the_list_shows_ten_at_a_time_and_a_new_sort_or_filter_starts_over() {
    assert!(MAYBEBOARD.contains("let mut more_page = use_signal(MorePage::new);"));
    assert!(MAYBEBOARD.contains("let view = use_memo(move || (sort(), filter()));"));
    assert!(MAYBEBOARD.contains("more_page.write().reset();"));
}

#[test]
fn show_more_sits_under_the_panel_with_its_ghost_rows_in_the_list() {
    let fold = MAYBEBOARD.find("title: LIST_TITLE,").unwrap();
    let ghosts = MAYBEBOARD.find("MoreGhostRows {").unwrap();
    let more = MAYBEBOARD.find("ListMore {").unwrap();
    let danger = MAYBEBOARD.find("DangerZoneAction {\n").unwrap();
    assert!(fold < ghosts && ghosts < more && more < danger);
    assert!(MAYBEBOARD.contains("leaves: page.next_shows_the_rest(filtered.len()),"));
}
