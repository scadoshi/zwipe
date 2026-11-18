//! Every server write on the deck and commander screens (Add, Remove, the commander swipes,
//! the deck editor, Quick add, the deck view, Playstyle, the suggested commanders both Playstyle and
//! Commanders show, and Commanders' Maybeboard) runs in
//! `spawn_write`, never in a task that ends with the screen, so leaving mid-write or
//! mid-undo never drops one.

/// The screens' sources.
const SCREENS: [(&str, &str); 9] = [
    (
        "Add",
        include_str!("../src/lib/inbound/screens/deck/cards/add.rs"),
    ),
    (
        "Remove",
        include_str!("../src/lib/inbound/screens/deck/cards/remove.rs"),
    ),
    (
        "commander swipes",
        include_str!("../src/lib/inbound/cards/swipe_select.rs"),
    ),
    (
        "deck editor",
        include_str!("../src/lib/inbound/screens/deck/cards/editor.rs"),
    ),
    (
        "Quick add",
        include_str!("../src/lib/inbound/screens/deck/cards/quick_add.rs"),
    ),
    (
        "deck view",
        include_str!("../src/lib/inbound/screens/deck/view/mod.rs"),
    ),
    (
        "Playstyle",
        include_str!("../src/lib/inbound/screens/playstyle/mod.rs"),
    ),
    (
        "Commanders' Maybeboard",
        include_str!("../src/lib/inbound/screens/commanders/maybeboard.rs"),
    ),
    (
        "suggested commanders",
        include_str!("../src/lib/inbound/screens/playstyle/suggestions.rs"),
    ),
];

/// The client calls that write to the server.
const WRITES: [&str; 13] = [
    ".create_deck_card(",
    ".delete_deck_card(",
    ".delete_undone_deck_card(",
    ".update_deck_card(",
    ".skip_deck_card(",
    ".unskip_deck_card(",
    ".dismiss_commander(",
    ".undismiss_commander(",
    ".add_commander_maybeboard_card(",
    ".remove_commander_maybeboard_card(",
    ".clear_commander_maybeboard(",
    ".update_deck_profile(",
    ".clear_commander_dismissals(",
];

/// Each `spawn(async move { .. })` block in `src`: a task that ends with its screen.
fn screen_tasks(src: &str) -> Vec<&str> {
    const OPEN: &str = "spawn(async move {";
    let mut found = Vec::new();
    let mut rest = src;
    while let Some(at) = rest.find(OPEN) {
        let before = rest.get(..at).and_then(|head| head.chars().next_back());
        let tail = rest.get(at..).unwrap_or_default();
        let mut depth = 0_u32;
        let mut end = tail.len();
        for (i, ch) in tail.char_indices() {
            match ch {
                '{' => depth = depth.saturating_add(1),
                '}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        end = i;
                        break;
                    }
                }
                _ => {}
            }
        }
        if !before.is_some_and(|c| c == '_' || c.is_alphanumeric()) {
            found.push(tail.get(..end).unwrap_or(tail));
        }
        rest = tail.get(OPEN.len()..).unwrap_or_default();
    }
    found
}

#[test]
fn no_screen_writes_from_a_task_that_ends_with_it() {
    for (screen, src) in SCREENS {
        for task in screen_tasks(src) {
            for write in WRITES {
                assert!(
                    !task.contains(write),
                    "{screen} calls {write} in a screen task; use spawn_write:\n{task}"
                );
            }
        }
    }
}

#[test]
fn the_check_finds_a_write_in_a_screen_task() {
    let src = "fn undo() { spawn(async move { c.delete_deck_card(id).await; }); spawn_write(async move { c.create_deck_card(id).await; }); }";
    let tasks = screen_tasks(src);
    assert_eq!(tasks.len(), 1, "{tasks:?}");
    assert!(tasks.iter().any(|t| t.contains(".delete_deck_card(")));
}
