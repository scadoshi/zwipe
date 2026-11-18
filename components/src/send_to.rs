//! Send to: a card added to another of the user's decks from wherever it shows. The Send to
//! button sits in a card's details and the deck page's opened rows; the sheet it opens leads
//! with the boards as chips and lists the decks under them, and a tap on a deck sends the card.
//! The client provides [`SendTo`] above its signed-in screens and does the fetching and the
//! write; where it isn't provided, as on zwipe.net, the button draws nothing.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::Board;
use zwipe_ui::send_to::{
    self, BOARDS, LOADING, NO_DECKS, NO_MATCH, SEARCH_PLACEHOLDER, TITLE, Target,
};

use crate::{
    Chip, PickerChoice, PickerSearch, PickerSheet, use_overlay_back_action, use_picker_search,
};

/// The open Send to: whether the sheet is open, and the card it sends. Provided by the client
/// above its signed-in screens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SendTo {
    /// Whether the sheet is open.
    pub open: Signal<bool>,
    /// The card the sheet sends.
    pub card: Signal<Option<Card>>,
}

impl SendTo {
    /// Opens the sheet to send `card`, as its Send to button does.
    pub fn open_for(self, card: Card) {
        let Self {
            mut open,
            card: mut sending,
        } = self;
        sending.set(Some(card));
        open.set(true);
    }
}

/// The board the next send goes to: the one last picked, kept for the session.
static BOARD: GlobalSignal<Board> = Signal::global(|| Board::Deck);

/// The search field's id.
const SEARCH_ID: &str = "send-to-search";

/// The button's `data-action`.
pub const SEND_TO_ACTION: &str = "send-to";

/// The Send to button for `card`, in a row of the card's actions. Draws nothing where the
/// client has not provided [`SendTo`].
#[component]
pub fn SendToButton(card: Card) -> Element {
    let Some(send_to) = try_use_context::<SendTo>() else {
        return rsx! {};
    };
    rsx! {
        button {
            class: "card-action-btn",
            "data-action": SEND_TO_ACTION,
            onclick: move |evt| {
                evt.stop_propagation();
                send_to.open_for(card.clone());
            },
            {TITLE}
        }
    }
}

/// The Send to sheet, open while `open`: the board chips, Main by default and then the last one
/// picked, the search, and `decks` (`None` while they load) less `current`, the deck the user is
/// on. A tap on a deck calls `on_send` with it and the board.
#[component]
pub fn SendToSheet(
    open: Signal<bool>,
    decks: Option<Vec<Target>>,
    current: Option<Uuid>,
    on_send: EventHandler<(Target, Board)>,
) -> Element {
    let query = use_picker_search(open.into());
    let mut board = BOARD.signal();
    let layer = use_overlay_back_action(
        open.into(),
        Callback::new(move |()| {
            let mut open = open;
            open.set(false);
        }),
    );
    let shown = decks
        .as_deref()
        .map(|decks| send_to::targets(decks, current, &query()));
    let target = shown
        .as_deref()
        .and_then(|shown| crate::enter_target(&query(), shown))
        .cloned();
    let target_id = target.as_ref().map(|t| t.id);
    // Enter sends to the first deck for the search as typed, which can be ahead of the list.
    let all_decks = decks.unwrap_or_default();
    let send_target = move |typed: String| {
        if let Some(target) = send_to::target_on_enter(&all_decks, current, &typed) {
            on_send.call((target, board()));
        }
    };
    let empty_line = if query().trim().is_empty() {
        NO_DECKS
    } else {
        NO_MATCH
    };
    rsx! {
        PickerSheet {
            open,
            layer: layer(),
            title: TITLE,
            search_id: SEARCH_ID,
            search_key: false,
            on_back: move |()| {
                let mut open = open;
                open.set(false);
            },
            div { class: "tag-controls",
                div { class: "chip-row send-to-boards",
                    span { class: "stat-chip stat-chip-p1 chip-row-label", {TITLE} }
                    for choice in BOARDS {
                        Chip {
                            key: "{send_to::chip_label(choice)}",
                            selected: board() == choice,
                            onclick: move |_| board.set(choice),
                            {send_to::chip_label(choice)}
                        }
                    }
                }
                PickerSearch {
                    id: SEARCH_ID,
                    placeholder: SEARCH_PLACEHOLDER,
                    query,
                    on_enter: send_target,
                }
            }
            div { class: "tag-grid send-to-decks",
                match shown {
                    None => rsx! {
                        div { class: "chip-unselected", {LOADING} }
                    },
                    Some(shown) if shown.is_empty() => rsx! {
                        div { class: "chip-unselected", {empty_line} }
                    },
                    Some(shown) => rsx! {
                        for target in shown {
                            PickerChoice { key: "{target.id}", target: target_id == Some(target.id),
                                Chip {
                                    selected: false,
                                    onclick: {
                                        let target = target.clone();
                                        move |_| on_send.call((target.clone(), board()))
                                    },
                                    "{target.name}"
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}
