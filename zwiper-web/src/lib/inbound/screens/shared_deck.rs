//! A shared deck at its public link, `/<owner>/deck/<slug>/<token>`, open to everyone: a signed-in
//! reader keeps their nav, a signed-out one gets the public nav. It is the deck page both sites
//! share, read only: List or Stacks, Group by, Show, Boards, the filter, the group and deck carts,
//! a card's image, printings (to look), details, flip and Meld, the Oracle tags' descriptions and
//! examples, the keys that read, then the stat panels. Only the share token names the deck; the
//! owner and slug move to the deck's current ones in place.

use std::collections::{HashMap, HashSet};

use components::hint::ContentHintDialog;
use components::keys::{ShortcutsList, use_offer_shortcuts, use_shortcuts_action};
use components::{
    DeckFacts, DeckHoverKeys, DeckRowHost, DockFilter, LoadFailed, Panel, StackDetails,
    use_deck_hover_keys,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::deck_metrics::{self, DeckMetrics};
use zwipe_core::domain::deck::{Board, DeckEntry, Format, deck_tag_label};
use zwipe_core::domain::share_link;
use zwipe_core::http::contracts::deck::HttpSharedDeck;
use zwipe_ui::deck::form::Slot;
use zwipe_ui::deck::profile::{DeckInfo, shared_warnings};
use zwipe_ui::deck::view::{DeckView, StatsProfile};
use zwipe_ui::deck_cards::keys::RowKind;
use zwipe_ui::deck_cards::listing::Listed;
use zwipe_ui::deck_cards::stacks::{CardsView, Reveal};
use zwipe_ui::deck_cards::{self, Boards, Zone, listing};
use zwipe_ui::filter::picker::Sources;
use zwipe_ui::hint::shared_deck as hint_copy;
use zwipe_ui::shortcuts::{self, CardKey, Editable};
use zwipe_ui::{buy, filter};

use crate::inbound::cards::card_row::{CardOverlays, use_card_overlays};
use crate::inbound::cards::otag::{OtagDescribe, OtagExamplesOpen, use_otag_reveals};
use crate::inbound::chrome::media::use_wide;
use crate::inbound::router::Route;
use crate::inbound::screens::deck::view::body::DeckPageBody;
use crate::inbound::screens::deck::view::cards_layer::use_cards_route;
use crate::inbound::screens::deck::view::panels::DataPanel;
use crate::inbound::screens::deck::view::profile::{DeckFoot, DeckProfilePhone};
use crate::inbound::session::return_to;
use crate::inbound::shell::ScreenFrame;

/// The tab's title while the deck loads or when it can't be shown.
const SHARED_DECK_TITLE: &str = "Shared deck";

/// How a shared deck's fetch can fail, from the reader's point of view.
#[derive(Debug, Clone, PartialEq)]
enum FetchError {
    /// 404: never shared, or sharing was stopped.
    NotShared,
    /// Anything else, worth a retry.
    Network,
}

/// The shared deck's command zone in its slots.
fn zone_of(deck: &HttpSharedDeck) -> Zone {
    Zone {
        commander: deck.commander.clone(),
        partner: deck.partner_commander.clone(),
        background: deck.background.clone(),
        signature: deck.signature_spell.clone(),
    }
}

/// Whether the shared deck has any card: an entry or a command-zone card.
fn has_cards(deck: &HttpSharedDeck) -> bool {
    !deck.entries.is_empty()
        || deck.commander.is_some()
        || deck.partner_commander.is_some()
        || deck.background.is_some()
        || deck.signature_spell.is_some()
}

/// The shared deck `token` names, at the link that carried `owner` and `slug`. Once it loads, the
/// link moves in place to the deck's current owner and slug when they differ; a server that sends
/// no owner leaves it as it is.
#[component]
pub fn SharedDeck(owner: String, slug: String, token: String, view: Option<DeckView>) -> Element {
    let navigator = use_navigator();
    let on_cards = {
        let (owner, slug, token) = (owner.clone(), slug.clone(), token.clone());
        use_cards_route(move |view| Route::SharedDeck {
            owner: owner.clone(),
            slug: slug.clone(),
            token: token.clone(),
            view,
        })
    };
    let cards_open = view.is_some();
    let export = Route::SharedDeckExport {
        owner: owner.clone(),
        slug: slug.clone(),
        token: token.clone(),
    };
    let client: Signal<ZwipeClient> = use_context();
    // A signed-out reader who signs in from here comes back to this deck.
    let session: Signal<Option<Session>> = use_context();
    let route = use_route::<Route>();
    use_hook(move || {
        let signed_in = session.peek().as_ref().is_some_and(|s| !s.is_expired());
        if !signed_in {
            return_to::remember(&route);
        }
    });
    let link_token = token.clone();
    let mut result: Resource<Result<HttpSharedDeck, FetchError>> =
        use_resource(use_reactive!(|token| async move {
            // A token that can't parse can't resolve, the same as the server's 404.
            let Ok(token) = token.parse::<Uuid>() else {
                return Err(FetchError::NotShared);
            };
            let http = client.peek().clone();
            http.fetch_shared_deck(token).await.map_err(|e| match e {
                ClientError::NotFound(_) => FetchError::NotShared,
                other => {
                    tracing::warn!("shared deck failed to load: {other}");
                    FetchError::Network
                }
            })
        }));

    // A stale owner or slug moves to the canonical link, in place.
    let at = (owner, slug);
    use_effect(move || {
        let canonical = match &*result.read() {
            Some(Ok(deck)) => deck
                .owner
                .clone()
                .map(|owner| (owner, share_link::deck_slug(&deck.name))),
            _ => None,
        };
        if let Some((owner, slug)) = canonical
            && at != (owner.clone(), slug.clone())
        {
            navigator.replace(Route::SharedDeck {
                owner,
                slug,
                token: link_token.clone(),
                view,
            });
        }
    });

    // `?` opens the page's hint, whose keyboard button opens the shortcuts sheet on a computer.
    let mut hint_open = use_signal(|| false);
    let on_hint = use_callback(move |()| hint_open.set(true));
    let shortcuts_action = use_shortcuts_action();
    let wide = use_wide();
    let loaded = matches!(&*result.read(), Some(Ok(_)));
    let title = match &*result.read() {
        Some(Ok(deck)) => deck.name.clone(),
        _ => SHARED_DECK_TITLE.to_owned(),
    };

    rsx! {
        // Reachable only by having the link; keep crawlers out of the page itself.
        document::Meta { name: "robots", content: "noindex" }
        ScreenFrame {
            title,
            on_hint: Some(on_hint),
            // The deck's head names it in its own h1 on a wide window and atop the Cards.
            titled: loaded && (wide() || cards_open),
            match &*result.read() {
                None => rsx! { SharedDeckView { deck: None, cards_open, on_cards, export, on_hint } },
                Some(Err(FetchError::NotShared)) => rsx! {
                    div { class: "content-enter web-page shared-deck-note",
                        Panel { title: "Deck not shared",
                            p { class: "subtitle",
                                "This deck is no longer shared. The owner may have stopped sharing it, or the link may be incomplete."
                            }
                        }
                    }
                },
                Some(Err(FetchError::Network)) => rsx! {
                    div { class: "content-enter web-page shared-deck-note",
                        LoadFailed {
                            message: "Couldn't load this deck",
                            on_retry: move |()| result.restart(),
                        }
                    }
                },
                Some(Ok(deck)) => rsx! {
                    SharedDeckView { deck: Some(deck.clone()), cards_open, on_cards, export, on_hint }
                },
            }
        }
        ContentHintDialog {
            open: hint_open,
            content: hint_copy::content(),
            actions: shortcuts_action.into_iter().collect::<Vec<_>>(),
        }
    }
}

/// The shared deck, read only: the owner's deck page body ([`DeckPageBody`]) with no edit action,
/// Quick add or foot, and a dock of the filter and the cart. Until the deck is in (`None`), the
/// body draws its ghosts.
#[component]
fn SharedDeckView(
    deck: Option<HttpSharedDeck>,
    cards_open: bool,
    on_cards: Callback<bool>,
    // The deck's Export screen.
    export: Route,
    // What the nav's `?` opens, over the page or its Cards.
    on_hint: Callback<()>,
) -> Element {
    // The image preview and the printing sheet, to look at only.
    let overlays = use_card_overlays();
    let CardOverlays {
        mut preview,
        mut printing,
    } = overlays;
    // The Oracle tags' descriptions and their example cards.
    let examples = use_otag_reveals();
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);

    // The controls' state, ephemeral per visit.
    let group_by = use_signal(|| GroupByOption::CardType);
    let mut card_filter: Signal<CardQueryBuilder> = use_signal(CardQueryBuilder::default);
    let mut filter_open = use_signal(|| false);
    // The read-only deck page's keys on the shortcuts sheet.
    use_offer_shortcuts(ShortcutsList {
        rows: shortcuts::deck_cards_rows(Editable::No)
            .into_iter()
            .chain(shortcuts::EVERYWHERE_ROWS)
            .collect(),
        note: Some(shortcuts::DECK_CARDS_NOTE),
    });
    let boards = use_signal(Boards::default);
    let show_zone = use_signal(|| true);
    let show_tokens = use_signal(|| false);
    let show_art = use_signal(|| true);
    let view = use_signal(|| CardsView::List);
    let collapsed: Signal<HashSet<String>> = use_signal(HashSet::new);
    let selected: Signal<Option<Uuid>> = use_signal(|| None);
    let face = use_signal(|| 0usize);
    let faces: Signal<HashMap<Uuid, usize>> = use_signal(HashMap::new);
    let currency = use_signal(PriceCurrency::default);
    // The stacks' reveal and whether the open card's drawer shows.
    let reveal = use_signal(Reveal::default);
    let details_open = use_signal(|| false);
    let details = use_context_provider(|| StackDetails {
        reveal,
        details_open,
    });

    // The rows as the owner's deck page lists them, from the same pass, with each card's
    // quantity, board and star.
    let listing_memo = use_memo(use_reactive!(|deck| {
        let Some(deck) = deck else {
            return (Listed::default(), HashMap::new());
        };
        let zone = zone_of(&deck);
        let mut entries: Vec<DeckEntry> = deck.entries.clone();
        zone.strip(&mut entries);
        let applied = card_filter();
        let meta = listing::meta_of(&entries);
        let display = deck_cards::pass(&entries, &zone, &applied, group_by());
        let tokens = if show_tokens() {
            deck.tokens
        } else {
            Vec::new()
        };
        let listed = listing::list_rows(&display, &meta, tokens, &applied, boards(), show_zone());
        (listed, meta)
    }));
    // `p`, `i`, `v` and `d` read the card under the cursor, else the open one; `f` opens the
    // filter.
    use_deck_hover_keys(DeckHoverKeys {
        row_of: Callback::new(move |id: Uuid| {
            let held = listing_memo.peek();
            let (listed, meta) = &*held;
            let row = listed
                .rows
                .iter()
                .find(|r| r.card.scryfall_data.id == id)
                .cloned()?;
            let board = match row.kind {
                RowKind::Board(board) => board,
                RowKind::Main { .. } | RowKind::Zone(_) | RowKind::Token => {
                    listing::board_of(meta, id)
                }
            };
            Some((row, board))
        }),
        faces,
        details: Some(details),
        actions: None,
        on_printing: Callback::new(move |(card, _): (Card, Option<Slot>)| {
            printing.set(Some(card));
        }),
        on_image: Callback::new(move |shown: (ScryfallData, usize)| preview.set(Some(shown))),
        other_keys: Some(Callback::new(move |key: String| {
            if shortcuts::card_key(&key) == Some(CardKey::Filter) {
                filter_open.set(true);
            }
        })),
    });

    let host = DeckRowHost {
        on_image: Some(Callback::new(move |shown: (ScryfallData, usize)| {
            preview.set(Some(shown));
        })),
        on_printing: None,
        on_view_printing: Some(Callback::new(move |card: Card| printing.set(Some(card)))),
        describe_tag,
        on_examples,
    };
    let (listed, meta) = listing_memo();
    let on_reset_filter = use_callback(move |()| card_filter.set(CardQueryBuilder::default()));
    let Some(deck) = deck else {
        return rsx! {
            DeckPageBody {
                loaded: false,
                name: None,
                facts: None,
                featured: Vec::new(),
                format: None,
                listing: listed,
                meta,
                group_by,
                show_tokens: None,
                show_zone: None,
                show_art,
                boards,
                has_maybe: false,
                has_side: false,
                view,
                collapsed,
                selected,
                face,
                faces,
                details,
                filter: card_filter,
                filter_open,
                on_reset_filter,
                sources: Sources::default(),
                row_currency: PriceCurrency::Usd,
                currency,
                stats: None,
                metrics: None,
                land_target: None,
                deck_buy: None,
                actions: None,
                host,
                drawer_host: host,
                overlays,
                examples,
                cards_open,
                on_cards,
                cards_hint: on_hint,
                phone_profile: rsx! {
                    DeckProfilePhone {
                        info: None,
                        stats: StatsProfile::default(),
                        metrics: None,
                        currency,
                        warnings: Vec::new(),
                        actions: None,
                        data: rsx! {},
                    }
                },
            }
        };
    };

    // The command zone in its slots, and every board's entries without it.
    let zone = zone_of(&deck);
    let mut entries: Vec<DeckEntry> = deck.entries.clone();
    zone.strip(&mut entries);
    let mainboard: Vec<DeckEntry> = entries
        .iter()
        .filter(|e| e.deck_card.board == Board::Deck)
        .cloned()
        .collect();
    let has_maybe = entries.iter().any(|e| e.deck_card.board.is_maybeboard());
    let has_side = entries.iter().any(|e| e.deck_card.board.is_sideboard());
    let is_oathbreaker = deck
        .format
        .as_ref()
        .is_some_and(Format::has_signature_spell);
    let featured = deck_cards::featured(&zone, &mainboard, is_oathbreaker);
    let sources = Sources::deck(&mainboard.iter().map(|e| e.card.clone()).collect::<Vec<_>>());

    // The head's facts: the main board plus each present command zone slot, priced at each
    // card's chosen printing.
    let zone_cards: Vec<Card> = zone.cards().into_iter().map(|(_, c)| c.clone()).collect();
    let price = deck_metrics::deck_price(&deck.entries, &zone_cards, PriceCurrency::Usd);
    let card_count: i64 = mainboard
        .iter()
        .map(|e| i64::from(*e.deck_card.quantity))
        .sum();
    let zone_slots = [
        deck.format.as_ref().is_some_and(Format::has_commander) && deck.commander.is_some(),
        deck.partner_commander.is_some(),
        deck.background.is_some(),
        deck.signature_spell.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    let display_count =
        i64::try_from(zone_slots).map_or(card_count, |slots| card_count.saturating_add(slots));
    let facts = DeckFacts {
        cards: display_count,
        format: deck.format,
        power_level: deck.power_level,
        zone_names: zone
            .cards()
            .into_iter()
            .map(|(_, c)| c.scryfall_data.name.clone())
            .collect(),
        tags: deck.tags.iter().map(|t| deck_tag_label(t)).collect(),
        other_tags: deck
            .other_tags
            .iter()
            .map(|t| t.display_name().to_owned())
            .collect(),
        price: (price > 0.0).then(|| format!("${price:.2}")),
    };
    let metrics = has_cards(&deck)
        .then(|| DeckMetrics::from_entries_and_command_zone(&deck.entries, &zone_cards));
    let deck_buy = has_cards(&deck).then(|| buy::shared_deck_links(&deck));
    // The Profile and the foot, as the owner's: Warnings read only, and Export alone in Data.
    let stats = StatsProfile::from(&deck);
    let warnings = shared_warnings(&deck);
    let info = DeckInfo::of_shared(
        &deck,
        metrics.as_ref().map_or_else(
            || usize::try_from(display_count).unwrap_or(0),
            |m| m.total_cards,
        ),
    );
    let data = rsx! {
        DataPanel { import: None, export, has_cards: metrics.is_some() }
    };

    rsx! {
        DeckPageBody {
            loaded: true,
            name: Some(deck.name.clone()),
            byline: deck.owner.clone(),
            facts: Some(facts),
            featured,
            format: deck.format,
            listing: listed,
            meta,
            group_by,
            show_tokens: (!deck.tokens.is_empty()).then_some(show_tokens),
            show_zone: zone.offers_chip(deck.format).then_some(show_zone),
            show_art,
            boards,
            has_maybe,
            has_side,
            view,
            collapsed,
            selected,
            face,
            faces,
            details,
            filter: card_filter,
            filter_open,
            on_reset_filter,
            sources,
            row_currency: PriceCurrency::Usd,
            currency,
            stats: Some(stats.clone()),
            metrics: metrics.clone(),
            land_target: deck.land_target,
            deck_buy,
            actions: None,
            host,
            drawer_host: host,
            overlays,
            examples,
            cards_open,
            on_cards,
            cards_hint: on_hint,
            phone_profile: rsx! {
                DeckProfilePhone {
                    info,
                    stats,
                    metrics,
                    currency,
                    warnings: warnings.clone(),
                    actions: None,
                    data: data.clone(),
                }
            },
            foot: rsx! {
                DeckFoot { loaded: true, warnings, actions: None, data }
            },
            // The Profile's filter, before the cart and Cards.
            profile_dock: rsx! {
                DockFilter {
                    label: "Filter",
                    dot: filter::shows_dot(&card_filter.read()),
                    active: !card_filter.read().is_empty(),
                    onclick: move |()| filter_open.set(true),
                    on_reset: move |()| on_reset_filter.call(()),
                    reset_label: "Reset filter",
                }
            },
        }
    }
}
