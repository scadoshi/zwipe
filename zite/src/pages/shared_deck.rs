use std::collections::{HashMap, HashSet};

use chrono::Utc;
use components::icons::DockIcon;
use components::keys::{ShortcutsList, use_offer_shortcuts, use_page_keys};
use components::{
    BudgetBody, CART, CardFilterPanel, CardsViewSwitch, Chip, CurrencyChips, DeckCardsView,
    DeckControls, DeckFacts, DeckFold, DeckHead, DeckPage, DeckRowHost, DeckStackDrawer,
    DistributionsBody, DockFilter, DrawOddsStepper, FeaturedCard, FlippableCardImage,
    GroupBuyDialog, HoverStackHost, ManaBody, Panel, PrintingSheet, StackDetails, StoreDialog,
    TagsBody, charted, deck_sections, use_group_buy, use_hover_stack, use_stack_details_sync,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::ClientError;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::deck_metrics::{self, DeckMetrics};
use zwipe_core::domain::deck::{Board, DeckEntry, Format, deck_tag_label};
use zwipe_core::domain::share_link;
use zwipe_core::http::contracts::deck::HttpSharedDeck;
use zwipe_ui::deck::view::StatsProfile;
use zwipe_ui::deck_cards::stacks::{
    CardsView, DEAL_DURATION, DEAL_STAGGER_CAP, DealCue, Reveal, deal_delay,
};
use zwipe_ui::deck_cards::{self, Boards, Zone, listing};
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::filter::picker::Sources;
use zwipe_ui::shortcuts::{self, CardKey, Editable};
use zwipe_ui::stores::catalog::{Catalog, Kind};
use zwipe_ui::{buy, filter};

use crate::components::{
    OVERLAY_EXIT_MS, PageMeta, sleep_ms, use_meld_source, use_printings_source,
};
use crate::{Footer, Nav, Route, ShowsPips, api};

/// Shortest the retry spinner shows, so an instant failure still reads as a click.
const RETRY_SPINNER_MIN_MS: u32 = 500;

/// Longest the featured row waits for its images before it opens anyway.
const FEATURED_WAIT_MS: u32 = 4000;

/// How a shared-deck fetch can fail, from the reader's point of view.
#[derive(Clone, PartialEq)]
enum FetchError {
    /// 404: never shared, or sharing was stopped.
    NotShared,
    /// Anything else, worth a retry.
    Network(String),
}

/// Loading placeholder in the shape of the loaded deck: the head's title,
/// chips and one featured card, the three control rows, and three card
/// bands, in the deck page's own layout. Pulses while the fetch runs.
#[component]
fn SharedDeckSkeleton() -> Element {
    rsx! {
        div { class: "shared-deck content-enter",
            div { class: "deck-page",
                div { class: "deck-head",
                    div { class: "deck-head-text",
                        div { class: "deck-head-title-row",
                            h1 { class: "deck-head-title",
                                span { class: "sk sk-title" }
                            }
                        }
                        div { class: "deck-head-meta",
                            for i in 0..4 {
                                div { key: "{i}", class: "sk sk-chip" }
                            }
                        }
                    }
                    div { class: "deck-head-featured",
                        div { class: "deck-head-card",
                            div { class: "sk sk-card" }
                            div { class: "deck-head-card-chips",
                                div { class: "sk sk-chip" }
                                div { class: "sk sk-chip" }
                            }
                        }
                    }
                }
                // Group by with its four groupings, then Show with its two
                // chips and the fold-all button.
                div { class: "deck-controls",
                    div { class: "chip-row",
                        div { class: "sk sk-chip sk-chip-label" }
                        for i in 0..4 {
                            div { key: "{i}", class: "sk sk-chip" }
                        }
                    }
                    div { class: "chip-row",
                        div { class: "sk sk-chip sk-chip-label sk-chip-label-short" }
                        for i in 0..2 {
                            div { key: "{i}", class: "sk sk-chip" }
                        }
                        div { class: "sk sk-chip sk-chip-icon" }
                    }
                }
                div { class: "deck-bands",
                    // Ghost card bands: like every other skeleton piece, big
                    // translucent boxes with no chrome or inner structure.
                    for g in 0..3 {
                        div { key: "{g}", class: "sk sk-group" }
                    }
                }
            }
        }
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

/// A shared deck at its own link, `/<owner>/deck/<slug>/<token>`.
#[component]
pub fn SharedDeckPage(owner: String, slug: String, token: String) -> Element {
    rsx! { SharedDeck { token, at: Some((owner, slug)) } }
}

/// The shared deck `token` names. `at` is the owner and slug the link
/// carried (none for a `/deck/<token>` link); once the deck loads, the page
/// moves to its canonical link when they differ from its current owner and
/// slug. A server that sends no owner leaves the link as it is.
#[component]
pub fn SharedDeck(token: String, #[props(default)] at: Option<(String, String)>) -> Element {
    let navigator = use_navigator();
    let link_token = token.clone();
    let mut result: Resource<Result<HttpSharedDeck, FetchError>> = use_resource(move || {
        let token = token.clone();
        async move {
            // Share tokens are uuids; a token that can't parse can't ever
            // resolve, same outcome as the server's 404.
            let Ok(token) = token.parse::<Uuid>() else {
                return Err(FetchError::NotShared);
            };
            api::client()
                .fetch_shared_deck(token)
                .await
                .map_err(|e| match e {
                    // A revoked or unknown token is not an outage.
                    ClientError::NotFound(_) => FetchError::NotShared,
                    other => FetchError::Network(other.to_string()),
                })
        }
    });

    // A stale or old-style link moves to the canonical one, in place.
    use_effect(move || {
        let canonical = match &*result.read() {
            Some(Ok(deck)) => deck
                .owner
                .clone()
                .map(|owner| (owner, share_link::deck_slug(&deck.name))),
            _ => None,
        };
        if let Some((owner, slug)) = canonical
            && at.as_ref() != Some(&(owner.clone(), slug.clone()))
        {
            navigator.replace(Route::SharedDeckPage {
                owner,
                slug,
                token: link_token.clone(),
            });
        }
    });

    // The nav's pip toggle shows while the deck on screen has any card.
    let mut shows_pips = use_context::<ShowsPips>().0;
    use_effect(move || {
        let any = matches!(&*result.read(), Some(Ok(deck)) if has_cards(deck));
        shows_pips.set(any);
    });
    use_drop(move || shows_pips.set(false));

    // Drives the Retry button's transient "Retrying…" state so a click is
    // legible even when the refetch fails again instantly (held for a minimum
    // beat on click, below; the request itself fires without delay).
    let mut retrying = use_signal(|| false);

    rsx! {
        // Reachable only by having the link; keep crawlers out.
        document::Meta { name: "robots", content: "noindex" }
        Nav {}
        main { id: "content", class: "site-main",
            match &*result.read() {
                None => rsx! {
                    PageMeta {
                        title: "Shared deck".to_string(),
                        description: "A Magic: The Gathering deck shared from Zwipe.".to_string(),
                        path: "/deck".to_string(),
                    }
                    SharedDeckSkeleton {}
                },
                Some(Err(FetchError::NotShared)) => rsx! {
                    PageMeta {
                        title: "Deck not shared".to_string(),
                        description: "A Magic: The Gathering deck shared from Zwipe.".to_string(),
                        path: "/deck".to_string(),
                    }
                    // Inform-and-stop alert, formatted like the app's
                    // update-required screen; deliberately a dead end (the nav is
                    // right there for anyone who wants to explore).
                    div { class: "dead-end content-enter",
                        Panel {
                            title: "Deck not shared",
                            title_h1: true,
                            p { class: "subtitle",
                                "This deck is no longer shared. The owner may have stopped sharing it, or the link may be incomplete."
                            }
                        }
                    }
                },
                Some(Err(FetchError::Network(e))) => rsx! {
                    PageMeta {
                        title: "Shared deck".to_string(),
                        description: "A Magic: The Gathering deck shared from Zwipe.".to_string(),
                        path: "/deck".to_string(),
                    }
                    div { class: "dead-end content-enter",
                        Panel {
                            title: "Couldn't load this deck",
                            title_h1: true,
                            // Retry rides the panel's own action row.
                            actions: rsx! {
                                button {
                                    class: "sd-retry",
                                    disabled: retrying(),
                                    onclick: move |_| {
                                        retrying.set(true);
                                        result.restart();
                                        spawn(async move {
                                            sleep_ms(RETRY_SPINNER_MIN_MS).await;
                                            retrying.set(false);
                                        });
                                    },
                                    if retrying() { "Retrying\u{2026}" } else { "Retry" }
                                }
                            },
                            p { class: "subtitle", "Error: {e}" }
                        }
                    }
                },
                Some(Ok(deck)) => rsx! {
                    SharedDeckView { deck: deck.clone() }
                },
            }
        }
        Footer {}
    }
}

/// The stat panels' fold keys, shut when the page opens.
const FOLDED_STATS: [&str; 5] = ["Budget", "Tags", "Distributions", "Mana", "Draw odds"];

/// The shared deck, read only: the deck page both sites share, as the owner's deck page draws it
/// with no edit control. List or Stacks, Group by, Show, Boards, the filter, the group and deck
/// carts, a card's image, printings (to look), details, flip and Meld, then the stat panels.
#[component]
fn SharedDeckView(deck: HttpSharedDeck) -> Element {
    // The meld halves and the printing sheet read their cards through these.
    use_meld_source();
    use_printings_source();
    // Oracle-tag catalog for the card-role tag definitions, fetched once and held
    // in browser memory (the endpoint is public, no auth). On failure it stays
    // empty and exposed tags read "No description yet".
    let otags: Resource<Vec<OracleTag>> =
        use_resource(|| async move { api::client().fetch_oracle_tags().await.unwrap_or_default() });
    let describe_tag = use_callback(move |slug: String| {
        otags.read().as_ref().and_then(|tags| {
            tags.iter()
                .find(|t| t.slug == slug)
                .and_then(|t| t.description.clone())
        })
    });

    // The controls' state, ephemeral per visit.
    let group_by = use_signal(|| GroupByOption::CardType);
    let mut card_filter: Signal<CardQueryBuilder> = use_signal(CardQueryBuilder::default);
    let mut filter_open = use_signal(|| false);
    // The read-only deck page's keys on the shortcuts sheet, and `f` for the filter.
    use_offer_shortcuts(ShortcutsList {
        rows: shortcuts::deck_cards_rows(Editable::No)
            .into_iter()
            .chain(shortcuts::EVERYWHERE_ROWS)
            .collect(),
        note: Some(shortcuts::DECK_CARDS_NOTE),
    });
    use_page_keys(use_callback(move |key: String| {
        if shortcuts::card_key(&key) == Some(CardKey::Filter) {
            filter_open.set(true);
        }
    }));
    let catalog: Signal<Catalog> = use_signal(Catalog::default);
    let mut boards = use_signal(Boards::default);
    let show_zone = use_signal(|| true);
    let show_tokens = use_signal(|| false);
    let show_art = use_signal(|| true);
    let view = use_signal(|| CardsView::List);
    let collapsed: Signal<HashSet<String>> = use_signal(HashSet::new);
    let folded: Signal<HashSet<String>> =
        use_signal(|| FOLDED_STATS.into_iter().map(String::from).collect());
    let mut selected: Signal<Option<Uuid>> = use_signal(|| None);
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
    // The stacks deal in on entering Stacks, for as long as the longest stack takes.
    let mut deal_cue = use_signal(DealCue::default);
    let mut dealing = use_signal(|| false);
    use_effect(move || {
        if deal_cue.write().on(view(), true) {
            dealing.set(true);
            spawn(async move {
                let span = deal_delay(DEAL_STAGGER_CAP).saturating_add(DEAL_DURATION);
                sleep_ms(u32::try_from(span.as_millis()).unwrap_or(u32::MAX)).await;
                dealing.set(false);
            });
        }
    });
    let group_buy = use_group_buy();
    let mut deck_buy_open = use_signal(|| false);
    // The card whose printings show, to look at only.
    let mut printing: Signal<Option<Card>> = use_signal(|| None);
    let mut printing_open = use_signal(|| false);
    // The rows' hover stack, pinned in the corner away from the cursor.
    let hover_stack = use_hover_stack();
    // Featured cards with their image bytes in hand. The row stays unseen
    // until every card reports in, or 4 s pass, then deals.
    let mut featured_loaded = use_signal(|| 0usize);
    let mut featured_timed_out = use_signal(|| false);
    use_future(move || async move {
        sleep_ms(FEATURED_WAIT_MS).await;
        featured_timed_out.set(true);
    });
    // The full-art overlay: the card's data and the face it opens on.
    let mut overlay_card: Signal<Option<(ScryfallData, usize)>> = use_signal(|| None);
    let mut overlay_dismissing = use_signal(|| false);

    // The command zone in its slots, and every board's entries without it.
    let zone = Zone {
        commander: deck.commander.clone(),
        partner: deck.partner_commander.clone(),
        background: deck.background.clone(),
        signature: deck.signature_spell.clone(),
    };
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
    let featured_ready = featured_timed_out() || featured_loaded() >= featured.len();
    let featured_class = if featured_ready {
        "deck-head-deal ready"
    } else {
        "deck-head-deal"
    };
    let on_featured_load = move |()| {
        let loaded = featured_loaded();
        featured_loaded.set(loaded.saturating_add(1));
    };

    // The rows as the owner's deck page lists them, from the same pass.
    let applied = card_filter();
    let meta = listing::meta_of(&entries);
    let display = deck_cards::pass(&entries, &zone, &applied, group_by());
    let token_cards = if show_tokens() {
        deck.tokens.clone()
    } else {
        Vec::new()
    };
    let listed = listing::list_rows(
        &display,
        &meta,
        token_cards,
        &applied,
        boards(),
        show_zone(),
    );
    let all_rows = listed.rows.clone();
    let listed_ids = use_memo(use_reactive!(|all_rows| all_rows
        .iter()
        .map(|r| r.card.scryfall_data.id)
        .collect::<Vec<_>>()));
    use_stack_details_sync(reveal, details_open, listed_ids);
    let row_boards: HashMap<Uuid, Board> = meta.iter().map(|(id, m)| (*id, m.board)).collect();
    let sections = deck_sections(listed, &meta, deck.format);
    let empty = boards().main && sections.is_empty();
    let section_keys: Vec<String> = sections.iter().map(|(s, _)| s.key.clone()).collect();
    let all_folded = {
        let shut = collapsed.read();
        !section_keys.is_empty() && section_keys.iter().all(|k| shut.contains(k))
    };
    let fold_all = move |()| {
        let mut collapsed = collapsed;
        let mut shut = collapsed.write();
        if all_folded {
            for key in &section_keys {
                shut.remove(key);
            }
        } else {
            shut.extend(section_keys.iter().cloned());
        }
    };
    let filter_on = !applied.is_empty();
    let dot = filter::shows_dot(&applied);
    let sources = Sources::deck(&mainboard.iter().map(|e| e.card.clone()).collect::<Vec<_>>());

    // The head's facts: the main board plus each present command zone slot,
    // priced at each card's chosen printing.
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

    // The stat panels, as the owner's deck page draws them.
    let stats_profile = StatsProfile::from(&deck);
    let metrics = has_cards(&deck)
        .then(|| DeckMetrics::from_entries_and_command_zone(&deck.entries, &zone_cards));
    let charts = charted(metrics.as_ref());
    let land_count = metrics.as_ref().map_or(0, |m| m.land_count);
    let has_budget = stats_profile.has_budget(metrics.is_some());
    let has_tags = stats_profile.tag_count() > 0;
    let deck_buy = has_cards(&deck).then(|| buy::shared_deck_links(&deck));

    let host = DeckRowHost {
        on_image: Some(Callback::new(move |shown: (ScryfallData, usize)| {
            overlay_card.set(Some(shown));
        })),
        on_printing: None,
        on_view_printing: Some(Callback::new(move |card: Card| {
            printing.set(Some(card));
            printing_open.set(true);
        })),
        describe_tag: Some(describe_tag),
        on_examples: None,
    };

    rsx! {
        PageMeta {
            title: deck.name.clone(),
            description: "A Magic: The Gathering deck shared from Zwipe.".to_string(),
            path: "/deck".to_string(),
        }
        HoverStackHost { stack: hover_stack }
        // The page's actions float at the bottom right, as on the web's deck
        // page: the filter, with its reset while a filter is on, then the cart.
        div { class: "action-dock action-dock-right",
            DockFilter {
                label: "Filter",
                dot,
                active: filter_on,
                onclick: move |()| filter_open.set(true),
                on_reset: move |()| card_filter.set(CardQueryBuilder::default()),
                reset_label: "Reset filter",
            }
            if deck_buy.is_some() {
                span { class: "action-dock-divider" }
                button {
                    class: "action-dock-btn",
                    "aria-label": buy::CART_LABEL,
                    onclick: move |_| deck_buy_open.set(true),
                    DockIcon { path: CART }
                }
            }
        }
        CardFilterPanel {
            open: filter_open,
            filter: card_filter,
            config: SheetConfig { show_format: true, sources, ..SheetConfig::default() },
            on_apply: move |next: CardQueryBuilder| {
                card_filter.set(next);
                selected.set(None);
            },
            catalog,
            on_need: move |kinds: &'static [Kind]| fetch_lists(catalog, kinds),
        }
        // The full-art overlay: a dimmed backdrop with the card's art (its own
        // flip button for a card with faces); a tap anywhere dismisses it.
        if overlay_card().is_some() || overlay_dismissing() {
            div { class: "sd-image-overlay-backdrop" }
            div {
                class: if overlay_dismissing() { "sd-image-overlay dismissing" } else { "sd-image-overlay" },
                onclick: move |_| {
                    overlay_dismissing.set(true);
                    spawn(async move {
                        sleep_ms(OVERLAY_EXIT_MS).await;
                        overlay_card.set(None);
                        overlay_dismissing.set(false);
                    });
                },
                if let Some((sd, face)) = overlay_card() {
                    FlippableCardImage {
                        sd,
                        size: ImageSize::Large,
                        class: "sd-image-overlay-img".to_string(),
                        draggable: false,
                        initial_face: face,
                    }
                }
            }
        }
        div {
            class: "shared-deck content-enter deck-page-wide",
            onmousemove: move |e| hover_stack.track(e.client_coordinates().x),
            DeckPage {
                head: rsx! {
                    DeckHead {
                        name: deck.name.clone(),
                        byline: deck.owner.clone(),
                        facts,
                        has_featured: !featured.is_empty(),
                        featured_class: featured_class.to_string(),
                        for (card, role) in featured.iter().cloned() {
                            FeaturedCard {
                                key: "{card.scryfall_data.id}-{role}",
                                card,
                                role: role.to_string(),
                                on_open: move |(sd, face): (ScryfallData, usize)| overlay_card.set(Some((sd, face))),
                                on_load: on_featured_load,
                            }
                        }
                    }
                },
                controls: rsx! {
                    DeckControls {
                        group_by,
                        show_tokens: (!deck.tokens.is_empty()).then_some(show_tokens),
                        show_zone: zone.offers_chip(deck.format).then_some(show_zone),
                        show_art,
                        art_disabled: view() == CardsView::Stacks,
                        all_folded,
                        on_fold_all: fold_all,
                        on_change: move |()| selected.set(None),
                        boards: rsx! {
                            div { class: "chip-row", "data-settle": "",
                                span { class: "stat-chip stat-chip-p1 chip-row-label", "Boards" }
                                for (label, board, on, enabled) in [
                                    ("Main", Board::Deck, boards().main, true),
                                    ("Maybe", Board::Maybeboard, boards().maybe, has_maybe || boards().maybe),
                                    ("Side", Board::Sideboard, boards().side, has_side || boards().side),
                                ] {
                                    Chip {
                                        key: "{label}",
                                        selected: on,
                                        disabled: !enabled,
                                        onclick: move |_| {
                                            boards.write().toggle(board);
                                            selected.set(None);
                                        },
                                        "{label}"
                                    }
                                }
                            }
                            CardsViewSwitch { view, on_change: move |_| selected.set(None) }
                        },
                    }
                },
                empty,
                bands: rsx! {
                    DeckCardsView {
                        sections,
                        view: view(),
                        currency: PriceCurrency::Usd,
                        collapsed,
                        details,
                        selected,
                        face,
                        faces,
                        boards: row_boards.clone(),
                        group_buy,
                        dealing: dealing(),
                        show_art: Some(show_art),
                        actions: None,
                        host,
                    }
                },
                stats: rsx! {
                    if has_budget {
                        DeckFold {
                            fold_key: "Budget",
                            title: "Budget",
                            folded,
                            accessory: metrics.is_some().then(|| rsx! { CurrencyChips { currency } }),
                            BudgetBody { profile: stats_profile.clone(), metrics, currency }
                        }
                    }
                    if has_tags {
                        DeckFold { fold_key: "Tags", title: "Tags", folded,
                            TagsBody { profile: stats_profile }
                        }
                    }
                    if let Some(m) = charts.clone() {
                        DeckFold { fold_key: "Distributions", title: "Distributions", folded,
                            DistributionsBody { metrics: m, describe_tag: Some(describe_tag) }
                        }
                    }
                    if charts.is_some() || deck.land_target.is_some() {
                        DeckFold { fold_key: "Mana", title: "Mana", folded,
                            ManaBody { charts: charts.clone(), land_count, land_target: deck.land_target }
                        }
                    }
                    if let Some(m) = charts {
                        DeckFold { fold_key: "Draw odds", title: "Draw odds", folded,
                            DrawOddsStepper { metrics: m }
                        }
                    }
                },
            }
        }
        if let Some(links) = deck_buy {
            StoreDialog {
                open: deck_buy_open,
                title: buy::DECK_TITLE.to_owned(),
                body: buy::DECK_BODY.to_owned(),
                links,
            }
        }
        GroupBuyDialog { group_buy }
        if view() == CardsView::Stacks {
            DeckStackDrawer {
                rows: all_rows,
                boards: row_boards,
                details,
                faces,
                actions: None,
                host,
            }
        }
        if let Some(card) = printing() {
            PrintingSheet { key: "{card.scryfall_data.id}", card, open: printing_open }
        }
    }
}

/// Fetches each of `kinds` that `catalog` is missing, has failed or holds a
/// day old, from the public list endpoints. The deck tag list needs a
/// session, so it settles empty; the deck filter never reads it.
fn fetch_lists(catalog: Signal<Catalog>, kinds: &'static [Kind]) {
    let mut held = catalog;
    let claimed = held.write().claim(kinds, Utc::now());
    for kind in claimed {
        let mut cell = catalog;
        spawn(async move {
            let client = api::client();
            match kind {
                Kind::Keywords => {
                    let answer = client.fetch_keywords().await.ok();
                    cell.write().keywords.settle(answer, Utc::now());
                }
                Kind::KeywordReminders => {
                    let answer = client.fetch_keyword_reminders().await.ok();
                    cell.write().keyword_reminders.settle(answer, Utc::now());
                }
                Kind::OracleWords => {
                    let answer = client.fetch_oracle_words().await.ok();
                    cell.write().oracle_words.settle(answer, Utc::now());
                }
                Kind::CardTypes => {
                    let answer = client.fetch_card_types().await.ok();
                    cell.write().card_types.settle(answer, Utc::now());
                }
                Kind::Artists => {
                    let answer = client.fetch_artists().await.ok();
                    cell.write().artists.settle(answer, Utc::now());
                }
                Kind::Sets => {
                    let answer = client.fetch_sets().await.ok();
                    cell.write().sets.settle(answer, Utc::now());
                }
                Kind::CardRoles => {
                    let answer = client.fetch_card_roles().await.ok();
                    cell.write().card_roles.settle(answer, Utc::now());
                }
                Kind::OracleTags => {
                    let answer = client.fetch_oracle_tags().await.ok();
                    cell.write().oracle_tags.settle(answer, Utc::now());
                }
                Kind::UbFranchises => {
                    let answer = client.fetch_ub_franchises().await.ok();
                    cell.write().ub_franchises.settle(answer, Utc::now());
                }
                Kind::DeckTags => cell.write().deck_tags.settle(None, Utc::now()),
            }
        });
    }
}
