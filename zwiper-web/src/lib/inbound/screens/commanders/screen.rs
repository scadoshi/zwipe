//! The Commanders screen: [`MaybeboardPanel`] and [`SuggestedPanel`] in the
//! page grammar's two columns on a wide window, Maybeboard on the left, and
//! one after the other below the breakpoint. Each list has its own filter,
//! the funnel in its console, over its own Show row. The card keys, the cursor, the hover image, the
//! image and Printings overlays and one open row are the screen's too, over
//! both lists' cards.

use components::deck_cursor::use_card_list_cursor;
use components::keys::focus_by_id;
use components::{HoverStackHost, use_hover_stack};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_ui::screens::playstyle as suggested_model;
use zwipe_ui::telemetry::vocabulary::Screen;

use super::list_filter::ListFilterSheet;
use super::maybeboard::{MaybeboardPanel, QUICK_ADD_ID};
use super::suggested::SuggestedPanel;
use crate::inbound::cards::card_row::{CardOverlaysHost, use_card_overlays};
use crate::inbound::cards::otag::{OtagExamplesHost, use_otag_reveals};
use crate::inbound::cards::row_keys::use_card_row_keys_and;
use crate::inbound::cards::stack_view::use_row_faces;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::chrome::shortcuts::{self, CardKey, ShortcutsScreen};
use crate::inbound::screens::playstyle::use_suggestions;
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;

/// The screen's title, which the nav's entry also reads.
pub const TITLE: &str = "Commanders";

/// What a key does on the Commanders screen, past the card keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScreenKey {
    /// Focuses the Maybeboard's Quick add.
    QuickAdd,
    /// Opens the filter of the list the user is in.
    Filter,
    /// Opens the filter of the list the user is in at its Name section, the name field focused.
    FilterByName,
}

/// The screen action `key` stands for: `q` Quick add, `f` the filter of the list the user is in
/// and `/` that filter at its name field ([`shortcuts::card_key`]).
fn screen_key(key: &str) -> Option<ScreenKey> {
    match shortcuts::card_key(key) {
        Some(CardKey::Filter) => Some(ScreenKey::Filter),
        Some(CardKey::FilterByName) => Some(ScreenKey::FilterByName),
        Some(CardKey::Flip | CardKey::Details) | None => {
            (key == shortcuts::QUICK_ADD_KEY).then_some(ScreenKey::QuickAdd)
        }
    }
}

/// The list the user is in: the one holding the keyboard cursor or the one last touched, the
/// Maybeboard before either. The filter keys act on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum ActiveList {
    /// The saved commanders, on the left.
    #[default]
    Maybeboard,
    /// The suggested commanders, on the right.
    Suggested,
}

/// A filter sheet a key opens: the list's, at its name field or not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FilterOpen {
    list: ActiveList,
    at_name: bool,
}

/// The filter sheet `key` opens while the user is in `active`, or `None` for a key that opens
/// no filter.
fn filter_opened(key: ScreenKey, active: ActiveList) -> Option<FilterOpen> {
    match key {
        ScreenKey::Filter => Some(FilterOpen {
            list: active,
            at_name: false,
        }),
        ScreenKey::FilterByName => Some(FilterOpen {
            list: active,
            at_name: true,
        }),
        ScreenKey::QuickAdd => None,
    }
}

/// The Commanders screen, at `/commanders`.
#[component]
pub fn CommanderMaybeboard() -> Element {
    telemetry::use_telemetry_screen(Screen::CommanderMaybeboard);
    let hint_open = use_one_time_hint(OneTimeHint::Commanders);
    // Each list's own filter, and whether its sheet is open.
    let maybeboard_filter = use_signal(CardQueryBuilder::new);
    let maybeboard_filter_open = use_signal(|| false);
    let suggested_filter = use_signal(CardQueryBuilder::new);
    let suggested_filter_open = use_signal(|| false);
    let mut active_list = use_signal(ActiveList::default);
    let examples = use_otag_reveals();
    let overlays = use_card_overlays();
    let hover_stack = use_hover_stack();
    let mut expanded_card: Signal<Option<Uuid>> = use_signal(|| None);
    use_row_faces();

    // The saved commanders, which the Maybeboard list loads and edits, and the suggestions.
    let mut saved: Signal<Vec<Card>> = use_signal(Vec::new);
    let suggestions = use_suggestions(Some(suggested_filter));
    let mut held = suggestions.entries;

    // Every card on the screen, which the card keys act on.
    let listed_cards = use_memo(move || {
        saved()
            .into_iter()
            .chain(held.read().iter().map(|s| s.card.clone()))
            .collect::<Vec<Card>>()
    });
    let screen_keys = Callback::new(move |key: String| {
        let Some(key) = screen_key(&key) else {
            return;
        };
        let Some(FilterOpen { list, at_name }) = filter_opened(key, *active_list.peek()) else {
            focus_by_id(QUICK_ADD_ID);
            return;
        };
        let mut open = match list {
            ActiveList::Maybeboard => maybeboard_filter_open,
            ActiveList::Suggested => suggested_filter_open,
        };
        if at_name {
            components::open_at_name(open);
        } else {
            open.set(true);
        }
    });
    use_card_row_keys_and(
        listed_cards,
        expanded_card,
        ShortcutsScreen::CommanderMaybeboard,
        Some(screen_keys),
    );
    // The arrow keys and h, j, k and l move a cursor through the rows: h and l cross between
    // the two lists, each marked as a cursor list.
    use_card_list_cursor();

    // A saved commander's printing swap is display only: the maybeboard is keyed by oracle, and
    // a refetch returns the preferred printing. A suggestion's carries into Create deck. Either
    // way a Create deck after the swap starts with the picked printing; a card on neither list
    // only browses its printings.
    let picks = overlays.printing.read().as_ref().is_some_and(|card| {
        let oracle = card.scryfall_data.oracle_id;
        oracle.is_some()
            && saved
                .read()
                .iter()
                .any(|c| c.scryfall_data.oracle_id == oracle)
            || suggested_model::picks_printing(&held.read(), card)
    });
    let on_printing_save = move |picked: Card| {
        let target = picked.scryfall_data.oracle_id;
        if target.is_none() {
            return;
        }
        expanded_card.set(Some(picked.scryfall_data.id));
        if let Some(slot) = saved
            .write()
            .iter_mut()
            .find(|c| c.scryfall_data.oracle_id == target)
        {
            *slot = picked.clone();
        }
        if let Some(slot) = held
            .write()
            .iter_mut()
            .find(|s| s.card.scryfall_data.oracle_id == target)
        {
            slot.card = picked;
        }
    };

    rsx! {
        ScreenFrame {
            title: TITLE,
            on_hint: move |()| {
                let mut hint_open = hint_open;
                hint_open.set(true);
            },
            div {
                class: "screen-content",
                onmousemove: move |e| hover_stack.track(e.client_coordinates().x),
                div { class: "web-page web-commanders content-enter",
                    div { class: "web-columns",
                        MaybeboardPanel {
                            entries: saved,
                            filter: maybeboard_filter,
                            filter_open: maybeboard_filter_open,
                            expanded_card,
                            on_active: move |()| active_list.set(ActiveList::Maybeboard),
                        }
                        SuggestedPanel {
                            list: suggestions,
                            filter: suggested_filter,
                            filter_open: suggested_filter_open,
                            expanded_card,
                            on_active: move |()| active_list.set(ActiveList::Suggested),
                        }
                    }
                }
            }
        }
        OneTimeHintDialog { hint: OneTimeHint::Commanders, open: hint_open }
        ListFilterSheet { filter: maybeboard_filter, filter_open: maybeboard_filter_open }
        ListFilterSheet { filter: suggested_filter, filter_open: suggested_filter_open }
        HoverStackHost { stack: hover_stack }
        CardOverlaysHost {
            overlays,
            on_printing_save: picks.then_some(EventHandler::new(on_printing_save)),
        }
        OtagExamplesHost { examples }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn q_focuses_quick_add_and_f_and_slash_open_the_filter_as_everywhere() {
        assert_eq!(screen_key("q"), Some(ScreenKey::QuickAdd));
        assert_eq!(screen_key("/"), Some(ScreenKey::FilterByName));
        for byte in b' '..=b'~' {
            let key = char::from(byte).to_string();
            assert_eq!(
                screen_key(&key) == Some(ScreenKey::Filter),
                shortcuts::card_key(&key) == Some(CardKey::Filter),
                "{key}"
            );
            assert_eq!(
                screen_key(&key) == Some(ScreenKey::FilterByName),
                shortcuts::card_key(&key) == Some(CardKey::FilterByName),
                "{key}"
            );
        }
    }

    #[test]
    fn f_and_slash_open_the_filter_of_the_list_the_user_is_in() {
        assert_eq!(
            ActiveList::default(),
            ActiveList::Maybeboard,
            "before any touch"
        );
        for list in [ActiveList::Maybeboard, ActiveList::Suggested] {
            assert_eq!(
                filter_opened(ScreenKey::Filter, list),
                Some(FilterOpen {
                    list,
                    at_name: false
                })
            );
            assert_eq!(
                filter_opened(ScreenKey::FilterByName, list),
                Some(FilterOpen {
                    list,
                    at_name: true
                })
            );
            assert_eq!(
                filter_opened(ScreenKey::QuickAdd, list),
                None,
                "q is Maybeboard's"
            );
        }
    }

    #[test]
    fn shift_h_lists_the_filter_and_quick_add_keys_that_work_here() {
        let listed: Vec<&str> = shortcuts::sheet_rows(&ShortcutsScreen::CommanderMaybeboard)
            .into_iter()
            .flat_map(|(keys, _)| keys.iter().copied())
            .collect();
        for key in ["f", "/", "q"] {
            assert!(listed.contains(&key), "{key} is listed: {listed:?}");
            assert!(screen_key(key).is_some(), "{key} works here");
        }
        for key in ["p", "i", "v", "d", "Enter", "Shift+H"] {
            assert!(listed.contains(&key), "{key} is listed: {listed:?}");
        }
    }
}
