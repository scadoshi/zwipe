use std::collections::{HashMap, HashSet};

use components::{
    CardRow as SharedCardRow, Chip, ColorChart, DeckBand, DeckCharts, DeckControls, DeckFacts,
    DeckFilter, DeckFold, DeckHead, DeckPage, DrawOdds, FeaturedCard, FlippableCardImage,
    HoverStackHost, ManaCurve, ManaFulfillment, Panel, use_hover_stack,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::ClientError;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::card_sort_key::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::card_filter::sort_direction::SortDirection;
use zwipe_core::domain::card::search_card::card_type::CardType;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::deck_metrics::{self, DeckMetrics};
use zwipe_core::domain::deck::{Board, DeckEntry, Format, deck_tag_label};
use zwipe_core::http::contracts::deck::HttpSharedDeck;
use zwipe_ui::colors::ColorCounts;
use zwipe_ui::deck_cards::{self, Zone};

use crate::components::{OVERLAY_EXIT_MS, PageMeta, sleep_ms};
use crate::{Footer, Nav, api};

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

/// Read-only wrapper over the shared [`SharedCardRow`]: wires the mobile
/// tap-to-open image overlay, passing no edit callbacks (this page has none).
/// The row feeds the page's hover stack itself.
#[component]
fn CardRow(
    card: Card,
    qty: i32,
    mvp: bool,
    expanded_card: Signal<Option<Uuid>>,
    /// Tap-to-open full-art overlay target. Set to this card's data + the face
    /// currently shown when the in-detail "Image" button is tapped.
    mut overlay_card: Signal<Option<(ScryfallData, usize)>>,
    /// Resolve an oracle tag's description, so the card-role tags reveal their
    /// definition inline. No `on_examples` on the web (no swipe browse).
    describe_tag: Callback<String, Option<String>>,
    /// Art-crop thumbnail visibility (the "Art" chip in Show). Always
    /// `Some(..)` here so the toggle eases thumbs in/out instead of clipping.
    show_art: bool,
) -> Element {
    // Which face the expanded detail is flipped to (0 = front), the side the
    // overlay opens on. Kept in sync via the detail's `on_face_change`.
    let mut current_face = use_signal(|| 0usize);
    // Owned copies for the move closures below (each may fire many times).
    let sd_overlay = card.scryfall_data.clone();

    rsx! {
        SharedCardRow {
            card,
            qty,
            show_art: Some(show_art),
            expanded_card,
            show_classification: true,
            describe_tag: Some(describe_tag),
            // Star indicator on starred rows only; no Star button (read-only).
            mvp: mvp.then_some(true),
            on_image: move |()| {
                overlay_card.set(Some((sd_overlay.clone(), current_face())));
            },
            on_face_change: move |face: usize| current_face.set(face),
        }
    }
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
                        div { class: "sk sk-title" }
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
                div { class: "deck-controls",
                    for (row, chips) in [3usize, 2, 7].into_iter().enumerate() {
                        div { key: "{row}", class: "chip-row",
                            div { class: "sk sk-chip sk-chip-label" }
                            for i in 0..chips {
                                div { key: "{i}", class: "sk sk-chip" }
                            }
                        }
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

#[component]
pub fn SharedDeck(token: String) -> Element {
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

    // Drives the Retry button's transient "Retrying…" state so a click is
    // legible even when the refetch fails again instantly (held for a minimum
    // beat on click, below; the request itself fires without delay).
    let mut retrying = use_signal(|| false);

    rsx! {
        // Reachable only by having the link; keep crawlers out.
        document::Meta { name: "robots", content: "noindex" }
        Nav {}
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
                        title: "Could not load this deck",
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
        Footer {}
    }
}

#[component]
fn SharedDeckView(deck: HttpSharedDeck) -> Element {
    // Oracle-tag catalog for the card-role tag definitions, fetched once and held
    // in browser memory (the endpoint is public, no auth). On failure it stays
    // empty and exposed tags read "No description yet". No examples browse on the
    // web, so tags reveal their definition only (no "Examples" button).
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
    let name_filter = use_signal(String::new);
    let selected_types = use_signal(Vec::<CardType>::new);
    let selected_colors = use_signal(Vec::<Color>::new);
    let show_zone = use_signal(|| true);
    let show_tokens = use_signal(|| false);
    // Art-crop thumbnails on the card rows, on by default (matches the app).
    let show_row_art = use_signal(|| true);
    // Featured cards with their image bytes in hand. The row stays unseen
    // until every card reports in, or 4 s pass, then deals.
    let mut featured_loaded = use_signal(|| 0usize);
    let mut featured_timed_out = use_signal(|| false);
    use_future(move || async move {
        sleep_ms(FEATURED_WAIT_MS).await;
        featured_timed_out.set(true);
    });
    // Budget panel currency, mirroring the app's Budget section chips.
    let mut selected_currency = use_signal(|| "usd");
    // The folded panels, by key. The card groups open; the stat panels fold.
    let mut folded: Signal<HashSet<String>> = use_signal(|| {
        ["Budget", "Tags", "Distributions", "Mana", "Draw odds"]
            .into_iter()
            .map(String::from)
            .collect()
    });
    // The hover stack the rows feed, pinned in the corner away from the
    // cursor; it stays empty on touch (no mouseenter).
    let hover_stack = use_hover_stack();
    let expanded_card: Signal<Option<Uuid>> = use_signal(|| None);
    // Tap-to-open full-art overlay. Holds the card's data + the face to open on
    // (so it lands on whichever side the row was flipped to) and flips from
    // there via the shared FlippableCardImage. A short dismiss fade mirrors the
    // app's ImagePreview.
    let mut overlay_card: Signal<Option<(ScryfallData, usize)>> = use_signal(|| None);
    let mut overlay_dismissing = use_signal(|| false);

    // The command zone in its slots, and the main board without it: the
    // maybeboard is scratch space, not the deck statement.
    let zone = Zone {
        commander: deck.commander.clone(),
        partner: deck.partner_commander.clone(),
        background: deck.background.clone(),
        signature: deck.signature_spell.clone(),
    };
    let mut mainboard: Vec<DeckEntry> = deck
        .entries
        .iter()
        .filter(|e| e.deck_card.board == Board::Deck)
        .cloned()
        .collect();
    zone.strip(&mut mainboard);
    let card_count: i64 = mainboard
        .iter()
        .map(|e| i64::from(*e.deck_card.quantity))
        .sum();
    let qty_by_id: HashMap<Uuid, i32> = mainboard
        .iter()
        .map(|e| (e.deck_card.scryfall_data_id, *e.deck_card.quantity))
        .collect();
    let mvp_ids: HashSet<Uuid> = mainboard
        .iter()
        .filter(|e| e.deck_card.mvp_at.is_some())
        .map(|e| e.deck_card.scryfall_data_id)
        .collect();
    // A format with a signature spell is Oathbreaker: the "commander" is the
    // oathbreaker planeswalker.
    let is_oathbreaker = deck
        .format
        .as_ref()
        .is_some_and(Format::has_signature_spell);
    // The command zone with its roles, then the MVPs: the deck's identity at
    // a glance, and the one strip the Command zone chip never hides.
    let featured = deck_cards::featured(&zone, &mainboard, is_oathbreaker);
    let featured_ready = featured_timed_out() || featured_loaded() >= featured.len();
    let featured_class = if featured_ready {
        "sd-featured ready"
    } else {
        "sd-featured"
    };
    // A count of image loads, compared against a handful of featured cards.
    let on_featured_load = move |()| {
        let loaded = featured_loaded();
        featured_loaded.set(loaded.saturating_add(1));
    };

    let price = deck_metrics::mainboard_total_price(&deck.entries, PriceCurrency::Usd);

    // Card count mirrors the app's deck-list tile: mainboard sum plus each
    // present command-zone slot (commander only when the format uses one).
    let zone_slots = [
        deck.format.as_ref().is_some_and(Format::has_commander) && deck.commander.is_some(),
        deck.partner_commander.is_some(),
        deck.background.is_some(),
        deck.signature_spell.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    // A display count: it saturates rather than wraps on a corrupt quantity.
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

    // The same pass the app's deck view runs: criteria match, name sort, then
    // group, with the lands pulled out. A filter that fails to parse lists
    // every card.
    let mut builder = CardQueryBuilder::new();
    let name_query = name_filter();
    if !name_query.trim().is_empty() {
        builder.set_name_contains(name_query.trim());
    }
    if !selected_types().is_empty() {
        builder.set_card_type_contains_any(selected_types());
    }
    if !selected_colors().is_empty() {
        builder.set_color_identity_within(selected_colors().into_iter().collect());
    }
    let listed_zone = if show_zone() {
        zone.clone()
    } else {
        Zone::default()
    };
    let display = deck_cards::pass(&mainboard, &listed_zone, &builder, group_by());
    // Tokens the deck's cards produce (server-derived), shown like the app's
    // token list. Name-sorted, not run through the card filter.
    let tokens: Option<Vec<Card>> = show_tokens().then(|| {
        Cards::from(deck.tokens.clone())
            .sorted(CardSortKey::Name, SortDirection::Ascending)
            .into()
    });
    let sections = components::sections(&display, deck.format, tokens.as_deref(), &qty_by_id);
    // The fold-all button: every card group shut, or every one open again.
    let section_keys: Vec<String> = sections.iter().map(|s| s.key.clone()).collect();
    let all_folded = {
        let shut = folded.read();
        !section_keys.is_empty() && section_keys.iter().all(|k| shut.contains(k))
    };
    let fold_all = move |()| {
        let mut shut = folded.write();
        if all_folded {
            for key in &section_keys {
                shut.remove(key);
            }
        } else {
            shut.extend(section_keys.iter().cloned());
        }
    };
    let empty = sections.is_empty();

    // Distribution charts, mirroring the app's deck screen sections: metrics
    // over the full entry list (mainboard-only inside, same as the app) plus
    // the command zone; suppressed while the deck holds only its command zone.
    let cz_cards: Vec<Card> = zone.cards().into_iter().map(|(_, c)| c.clone()).collect();
    let metrics = DeckMetrics::from_entries_and_command_zone(&deck.entries, &cz_cards);
    let show_charts = metrics.land_count > 0 || metrics.nonland_count > 0;
    let has_tags =
        !deck.tags.is_empty() || !deck.oracle_tags.is_empty() || !deck.other_tags.is_empty();
    let currency = PriceCurrency::from_key(selected_currency()).unwrap_or_default();
    let (total, avg) = match currency {
        PriceCurrency::Eur => (metrics.total_price_eur, metrics.avg_price_eur),
        PriceCurrency::Tix => (metrics.total_price_tix, metrics.avg_price_tix),
        PriceCurrency::Usd => (metrics.total_price_usd, metrics.avg_price_usd),
    };
    // Total shown as `total / target` when a price target is set; the target
    // keeps its own stored currency, never converted (same as the app's
    // Budget section).
    let running = currency.format_amount(total.unwrap_or(0.0));
    let total_fmt = match deck.price_target {
        Some(target) => {
            let target_currency = deck.price_target_currency.unwrap_or_default();
            format!("{running} / {}", target_currency.format_amount(target))
        }
        None => running,
    };
    let avg_fmt = avg.map_or_else(|| "N/A".to_string(), |v| currency.format_amount(v));
    let avg_pt = if metrics.avg_power.is_some() || metrics.avg_toughness.is_some() {
        let fmt = |v: Option<f64>| v.map_or_else(|| "-".to_string(), |v| format!("{v:.1}"));
        Some(format!(
            "{} / {}",
            fmt(metrics.avg_power),
            fmt(metrics.avg_toughness)
        ))
    } else {
        None
    };
    // `actual / target` when a land target is set, like the app's Mana section.
    let lands_line = match deck.land_target {
        Some(target) => format!("{} / {target}", metrics.land_count),
        None => metrics.land_count.to_string(),
    };
    let (deck_size, buckets) = metrics.draw_odds_buckets();

    rsx! {
        PageMeta {
            title: deck.name.clone(),
            description: "A Magic: The Gathering deck shared from Zwipe.".to_string(),
            path: "/deck".to_string(),
        }
        HoverStackHost { stack: hover_stack }
        // Tap-to-open full-art overlay. Mirrors the app's ImagePreview: a dimmed
        // backdrop with the card art (its own flip button for DFCs), tap anywhere
        // to dismiss. The flip button stops propagation, so flipping never
        // dismisses.
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
            class: "shared-deck content-enter",
            onmousemove: move |e| hover_stack.track(e.client_coordinates().x),
            DeckPage {
                head: rsx! {
                    DeckHead {
                        name: deck.name.clone(),
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
                        all_folded,
                        on_fold_all: fold_all,
                        show_tokens: (!deck.tokens.is_empty()).then_some(show_tokens),
                        show_zone,
                        show_art: show_row_art,
                        filter: Some(DeckFilter {
                            name: name_filter,
                            types: selected_types,
                            colors: selected_colors,
                        }),
                    }
                },
                empty,
                bands: rsx! {
                    for section in sections {
                        DeckBand {
                            key: "{section.key}",
                            section: section.clone(),
                            folded,
                            for card in section.cards {
                                CardRow {
                                    key: "{card.scryfall_data.id}",
                                    qty: *qty_by_id.get(&card.scryfall_data.id).unwrap_or(&1),
                                    mvp: mvp_ids.contains(&card.scryfall_data.id),
                                    card,
                                    expanded_card,
                                    overlay_card,
                                    describe_tag,
                                    show_art: show_row_art(),
                                }
                            }
                        }
                    }
                },
                // The app's deck-view sections (Budget, Tags, Distributions,
                // Mana, Draw odds), folded. Warnings stays app-only: a
                // builder's tool.
                stats: show_charts.then(|| rsx! {
                    DeckFold { fold_key: "Budget", title: "Budget", folded,
                        div { class: "deck-stat-body",
                            div { class: "chip-row", style: "padding: 0 1rem;",
                                for (label, key) in [("USD", "usd"), ("EUR", "eur"), ("TIX", "tix")] {
                                    Chip {
                                        key: "{key}",
                                        selected: selected_currency() == key,
                                        onclick: move |_| selected_currency.set(key),
                                        "{label}"
                                    }
                                }
                            }
                            div { class: "info-row",
                                span { class: "info-row-label", "Total price" }
                                span { class: "info-row-value", "{total_fmt}" }
                            }
                            div { class: "info-row",
                                span { class: "info-row-label", "Average card price" }
                                span { class: "info-row-value", "{avg_fmt}" }
                            }
                        }
                    }
                    if has_tags {
                        DeckFold { fold_key: "Tags", title: "Tags", folded,
                            div { class: "deck-stat-body",
                                if !deck.tags.is_empty() {
                                    div { class: "info-row info-row-stacked",
                                        span { class: "info-row-label", "Deck tags" }
                                        span { class: "info-row-value info-row-tags",
                                            for tag in deck.tags.iter() {
                                                span { key: "{tag}", class: "stat-chip stat-chip-muted", "{deck_tag_label(tag)}" }
                                            }
                                        }
                                    }
                                }
                                if !deck.oracle_tags.is_empty() {
                                    div { class: "info-row info-row-stacked",
                                        span { class: "info-row-label", "Oracle tags" }
                                        span { class: "info-row-value info-row-tags",
                                            for slug in deck.oracle_tags.iter() {
                                                span { key: "{slug}", class: "stat-chip stat-chip-muted", "{slug}" }
                                            }
                                        }
                                    }
                                }
                                if !deck.other_tags.is_empty() {
                                    div { class: "info-row info-row-stacked",
                                        span { class: "info-row-label", "Other tags" }
                                        span { class: "info-row-value info-row-tags",
                                            for tag in deck.other_tags.iter() {
                                                span { key: "{tag}", class: "stat-chip stat-chip-muted", "{tag.display_name()}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    DeckFold { fold_key: "Distributions", title: "Distributions", folded,
                        div { class: "deck-stat-body",
                            if let Some(value) = avg_pt {
                                div { class: "info-row",
                                    span { class: "info-row-label", "Average P/T" }
                                    span { class: "info-row-value", style: "white-space:nowrap;", "{value}" }
                                }
                            }
                            DeckCharts {
                                type_bars: Some(metrics.type_bars()),
                                category_bars: metrics.card_role_bars(),
                                oracle_tag_bars: metrics.oracle_tag_bars(),
                                role_tags: Some(metrics.card_role_tags.clone()),
                                describe_tag,
                            }
                        }
                    }
                    DeckFold { fold_key: "Mana", title: "Mana", folded,
                        div { class: "deck-stat-body",
                            div { class: "info-row",
                                span { class: "info-row-label", "Lands" }
                                span { class: "info-row-value", "{lands_line}" }
                            }
                            div { class: "info-row",
                                span { class: "info-row-label", "Average mana value" }
                                span { class: "info-row-value", "{metrics.avg_cmc:.1}" }
                            }
                            ManaCurve { mana_curve_bars: metrics.mana_curve_bars() }
                            ColorChart { bars: ColorCounts::from(&metrics).bars() }
                            ManaFulfillment { rows: metrics.mana_balance_rows() }
                        }
                    }
                    DeckFold { fold_key: "Draw odds", title: "Draw odds", folded,
                        div { class: "deck-stat-body",
                            DrawOdds { deck_size, buckets }
                        }
                    }
                }),
            }
        }
    }
}
