//! Playstyle's card rows: a suggestion with the reasons it fits, and the
//! Staples panel with its Type and Color filters.

use components::{CardListFold, CardRoleChips, Chip, StapleDecksChip};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_type::CardType;
use zwipe_core::http::contracts::suggestions::{
    HttpCommanderSuggestion, HttpStaple, HttpSuggestionReasons,
};
use zwipe_ui::deck_cards::stacks::CardsView;
use zwipe_ui::playstyle::{
    StapleColor, StapleDeck, color_lines, group_shared_tags, matching_staples, named_staple_decks,
    staple_colors_present, staple_decks, staple_identity, staple_percent, staple_type_line,
    staple_types_present,
};

use super::chips::{IdentityPips, identity_key};
use crate::inbound::cards::card_row::CardRow;
use crate::inbound::cards::otag::{OtagDescribe, OtagExamplesOpen};
use crate::inbound::cards::stack_view::{CardsStack, CardsViewSwitch};
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::router::Route;
use crate::outbound::cards_view_store::CardsViewPlace;

/// The Staples group's title.
const STAPLES_TITLE: &str = "Staples";

/// The "Why this commander?" block under a suggestion: folded under a chip
/// in a list row (`foldable`), or open under a `chips-label` heading in a
/// details sheet. It holds the shared oracle tags grouped by card role and
/// one bullet per shared color, each left out when empty, and renders
/// nothing when both are. Tag reveals use `describe_tag` and `on_examples`,
/// else the screen's contexts.
#[component]
pub fn SuggestionReasons(
    reasons: HttpSuggestionReasons,
    deck_count: u32,
    #[props(default)] describe_tag: Option<Callback<String, Option<String>>>,
    #[props(default)] on_examples: Option<Callback<String>>,
    #[props(default)] foldable: bool,
) -> Element {
    let mut reasons_open = use_signal(|| false);
    let context_describe = try_use_context::<OtagDescribe>().map(|d| d.0);
    let context_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let colors = color_lines(&reasons.colors, deck_count);
    if reasons.tags.is_empty() && colors.is_empty() {
        return rsx! {};
    }
    let shared = group_shared_tags(&reasons.tags);
    let describe_tag = describe_tag.or(context_describe);
    let on_examples = on_examples.or(context_examples);
    let bullets = rsx! {
        ul { class: "card-detail-bullets",
            if !reasons.tags.is_empty() {
                li {
                    "Shares Oracle tags with your decks"
                    CardRoleChips {
                        roles: shared.roles,
                        tags_by_role: shared.by_role,
                        other_tags: shared.other,
                        describe_tag,
                        on_examples,
                        show_label: false,
                    }
                }
            }
            for line in colors {
                li { key: "{line}", "{line}" }
            }
        }
    };
    rsx! {
        div { class: "card-detail-footer",
            if foldable {
                Chip {
                    selected: reasons_open(),
                    onclick: move |evt: MouseEvent| {
                        evt.stop_propagation();
                        reasons_open.toggle();
                    },
                    span {
                        class: if reasons_open() { "chip-fold-arrow expanded" } else { "chip-fold-arrow" },
                        "\u{25b8}"
                    }
                    "Why this commander?"
                }
                div { class: if reasons_open() { "collapsible open" } else { "collapsible" },
                    div { class: "collapsible-inner", {bullets} }
                }
            } else {
                span { class: "chips-label", "Why this commander?" }
                {bullets}
            }
        }
    }
}

/// One suggestion: the card row with its reasons in the footer and
/// `Create deck`, then `Maybeboard` and `Remove` when the card has an oracle
/// id.
#[component]
pub fn SuggestionRow(
    suggestion: HttpCommanderSuggestion,
    deck_count: u32,
    expanded_card: Signal<Option<Uuid>>,
    on_start: EventHandler<Card>,
    on_maybe: EventHandler<Uuid>,
    on_dismiss: EventHandler<Uuid>,
) -> Element {
    let card = suggestion.card;
    let actions = suggestion_actions(card.clone(), on_start, on_maybe, on_dismiss);
    rsx! {
        CardRow {
            card,
            qty: 1,
            expanded_card,
            footer: rsx! {
                SuggestionReasons { reasons: suggestion.reasons, deck_count, foldable: true }
            },
            extra_actions: actions,
        }
    }
}

/// A suggestion's actions: `Create deck`, then `Maybeboard` and `Remove`
/// when the card has an oracle id. Its row and its stack card's drawer both
/// show them.
pub(super) fn suggestion_actions(
    card: Card,
    on_start: EventHandler<Card>,
    on_maybe: EventHandler<Uuid>,
    on_dismiss: EventHandler<Uuid>,
) -> Element {
    let oracle_id = card.scryfall_data.oracle_id;
    rsx! {
        button {
            class: "card-action-btn",
            onclick: move |evt| {
                evt.stop_propagation();
                on_start.call(card.clone());
            },
            "Create deck"
        }
        if let Some(oid) = oracle_id {
            button {
                class: "card-action-btn",
                onclick: move |evt| {
                    evt.stop_propagation();
                    on_maybe.call(oid);
                },
                "Maybeboard"
            }
            button {
                class: "card-action-btn",
                onclick: move |evt| {
                    evt.stop_propagation();
                    on_dismiss.call(oid);
                },
                "Remove"
            }
        }
    }
}

/// The cards in the most of the user's decks, under their own Type and
/// Color rows; the first ten that match both show. The rows sit above the
/// panel when `wide`, and inside it under the header otherwise. The filters
/// start over when the screen mounts again.
#[component]
pub fn StaplesPanel(
    staples: Vec<HttpStaple>,
    deck_count: u32,
    /// The user's decks, which name each staple's deck chips.
    decks: Vec<StapleDeck>,
    expanded_card: Signal<Option<Uuid>>,
    wide: bool,
    /// List rows, or one stack in Stacks.
    view: Signal<CardsView>,
) -> Element {
    let active_type: Signal<Option<CardType>> = use_signal(|| None);
    let active_color: Signal<Option<StapleColor>> = use_signal(|| None);
    let types = staple_types_present(&staples, staple_type_line);
    let colors = staple_colors_present(&staples, staple_identity);
    let rows = matching_staples(
        &staples,
        staple_type_line,
        staple_identity,
        active_type(),
        active_color(),
    );
    let filters = rsx! {
        StapleTypeChips { active_type, types }
        StapleColorChips { active_color, colors }
        CardsViewSwitch {
            view,
            place: CardsViewPlace::Staples,
            on_change: move |()| {
                let mut expanded = expanded_card;
                expanded.set(None);
            },
        }
    };
    // In Stacks the staples float on the page under their group's header, as
    // the deck page's stacks do, the filters above.
    let stacked = view() == CardsView::Stacks && !rows.is_empty();
    rsx! {
        div { class: "web-playstyle-cell web-playstyle-staples",
            if wide || stacked {
                div { class: "deck-controls", {filters.clone()} }
            }
            if stacked {
                CardsStack {
                    cards: rows.iter().map(|s| s.card.clone()).collect::<Vec<Card>>(),
                    title: STAPLES_TITLE.to_owned(),
                    trailing: rsx! {
                        InfoButton { topic: HintTopic::SuggestionStaples }
                    },
                }
            } else {
                CardListFold {
                    title: STAPLES_TITLE,
                    actions: rsx! {
                        InfoButton { topic: HintTopic::SuggestionStaples }
                    },
                    if !wide {
                        div { class: "web-card-group-filters", {filters} }
                    }
                    if rows.is_empty() {
                        div { class: "activity-total activity-total-group",
                            span { class: "chip-note", "No staples match" }
                        }
                    }
                    for staple in rows {
                        StapleRow {
                            key: "{staple.card.scryfall_data.id}",
                            in_decks: named_staple_decks(&staple.deck_ids, &decks),
                            staple,
                            deck_count,
                            expanded_card,
                        }
                    }
                }
            }
        }
    }
}

/// A staple: its share of the user's decks in place of the quantity, and
/// how many decks run it in the footer, a chip that lists `in_decks`, each
/// opening its deck.
#[component]
fn StapleRow(
    staple: HttpStaple,
    deck_count: u32,
    in_decks: Vec<StapleDeck>,
    expanded_card: Signal<Option<Uuid>>,
) -> Element {
    let footer = staple_footer(&staple, deck_count, in_decks);
    rsx! {
        CardRow {
            card: staple.card,
            qty: i32::try_from(staple.decks).unwrap_or(i32::MAX),
            qty_tags: staple_percent(staple.decks, deck_count).into_iter().collect::<Vec<String>>(),
            expanded_card,
            footer,
        }
    }
}

/// How many of the user's decks run `staple`, as a chip that lists `in_decks`,
/// each opening its deck. Its row and its stack card's drawer both show it.
pub(super) fn staple_footer(
    staple: &HttpStaple,
    deck_count: u32,
    in_decks: Vec<StapleDeck>,
) -> Element {
    let decks_label = staple_decks(staple.decks, deck_count);
    rsx! {
        StapleDecksChip {
            label: decks_label,
            decks: in_decks,
            on_open: move |deck_id: Uuid| {
                navigator().push(Route::ViewDeck { deck_id });
            },
        }
    }
}

/// The Type row: `All`, then each card type some staple has.
#[component]
fn StapleTypeChips(mut active_type: Signal<Option<CardType>>, types: Vec<CardType>) -> Element {
    rsx! {
        div { class: "chip-row", "data-settle": "",
            span { class: "stat-chip stat-chip-p1 chip-row-label", "Type" }
            Chip {
                selected: active_type().is_none(),
                onclick: move |_| active_type.set(None),
                "All"
            }
            for card_type in types {
                Chip {
                    key: "{card_type}",
                    selected: active_type() == Some(card_type),
                    onclick: move |_| {
                        if active_type() == Some(card_type) {
                            active_type.set(None);
                        } else {
                            active_type.set(Some(card_type));
                        }
                    },
                    "{card_type}"
                }
            }
        }
    }
}

/// The Color row: `All`, then each color (and colorless) some staple falls
/// under.
#[component]
fn StapleColorChips(
    mut active_color: Signal<Option<StapleColor>>,
    colors: Vec<StapleColor>,
) -> Element {
    rsx! {
        div { class: "chip-row", "data-settle": "",
            span { class: "stat-chip stat-chip-p1 chip-row-label", "Color" }
            Chip {
                selected: active_color().is_none(),
                onclick: move |_| active_color.set(None),
                "All"
            }
            for chip in colors {
                Chip {
                    key: "{identity_key(&chip.pips())}",
                    selected: active_color() == Some(chip),
                    onclick: move |_| {
                        if active_color() == Some(chip) {
                            active_color.set(None);
                        } else {
                            active_color.set(Some(chip));
                        }
                    },
                    IdentityPips { identity: chip.pips() }
                }
            }
        }
    }
}
