use crate::{API_BASE, Footer, Nav, Route, components::PageMeta};
use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use zwipe_components::{CardRow as SharedCardRow, Chip};
use zwipe_core::{
    domain::{
        card::{
            Card,
            scryfall_data::{ImageSize, colors::Color},
            search_card::{
                card_filter::{
                    builder::CardQueryBuilder, card_sort_key::CardSortKey,
                    price_currency::PriceCurrency,
                },
                card_type::CardType,
                cards::Cards,
                group_cards::{GroupByOption, GroupCards},
            },
        },
        deck::{Board, DeckEntry, deck_metrics::mainboard_total_price},
    },
    http::contracts::deck::HttpSharedDeck,
};

/// How a shared-deck fetch can fail, from the reader's point of view.
#[derive(Clone, PartialEq)]
enum FetchError {
    /// 404: never shared, or sharing was stopped.
    NotShared,
    /// Anything else — worth a retry.
    Network(String),
}

/// A card in the top-left hover-preview stack. The stack holds up to 5, newest
/// first; each card has its own timer and fades out on its own.
#[derive(Clone, PartialEq)]
struct PreviewCard {
    /// Monotonic id, unique per hover (so the same card hovered twice stacks).
    id: u64,
    url: String,
    /// True once the card's timer has fired and it's playing its exit fade.
    leaving: bool,
}

/// Browser `setTimeout` as a future. No-op off wasm (the server render never
/// hovers, so this never actually runs there); the `.await` only exists on
/// wasm, hence the allow.
#[allow(clippy::unused_async)]
async fn sleep_ms(ms: u32) {
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::TimeoutFuture::new(ms).await;
    #[cfg(not(target_arch = "wasm32"))]
    let _ = ms;
}

/// One pinned command zone card: image (when available) above name + role.
#[component]
fn CommandZoneCard(card: Card, role: String) -> Element {
    let name = card.scryfall_data.name.clone();
    let image = card.scryfall_data.primary_image_url(ImageSize::Normal);
    rsx! {
        div { class: "sd-cz-card",
            if let Some(url) = image {
                img { class: "sd-cz-image", src: "{url}", alt: "{name}", loading: "lazy" }
            }
            div { class: "sd-cz-name", "{name}" }
            if !role.is_empty() {
                div {
                    class: if role == "MVP" { "sd-cz-role sd-cz-role-mvp" } else { "sd-cz-role sd-cz-role-zone" },
                    "{role}"
                }
            }
        }
    }
}

/// Read-only wrapper over the shared [`SharedCardRow`]: wires zite's desktop
/// hover-preview stack and the mobile tap-to-open image overlay, passing no
/// edit callbacks (this page has none).
#[component]
fn CardRow(
    card: Card,
    qty: i32,
    mvp: bool,
    expanded_card: Signal<Option<Uuid>>,
    mut preview_stack: Signal<Vec<PreviewCard>>,
    mut preview_next_id: Signal<u64>,
    /// Tap-to-open full-art overlay target (mobile only). Set to this card's art
    /// when the in-detail "Image" button is tapped.
    mut overlay_image: Signal<Option<String>>,
) -> Element {
    // This row's live stack entry id (while the cursor is on it); the leave
    // handler times it out. `None` when the row isn't hovered.
    let mut my_preview_id = use_signal(|| None::<u64>);
    let sd = &card.scryfall_data;
    // Full-size art shown in the pinned hover preview (desktop). Front face for
    // double-faced layouts.
    let hover_image = sd.primary_image_url(ImageSize::Normal).map(str::to_string);
    // Larger art for the tap-to-open overlay (mobile, where hover is
    // unavailable). Mirrors the app's fullscreen ImagePreview (ImageSize::Large).
    let overlay_url = sd.primary_image_url(ImageSize::Large).map(str::to_string);

    rsx! {
        SharedCardRow {
            card,
            qty,
            expanded_card,
            // Star indicator on starred rows only; no Star button (read-only).
            mvp: mvp.then_some(true),
            on_image: move |()| {
                if let Some(url) = overlay_url.clone() {
                    overlay_image.set(Some(url));
                }
            },
            on_hover_enter: move |()| {
                let Some(url) = hover_image.clone() else {
                    return;
                };
                // Push onto the top of the stack (newest first), cap at 5.
                // No timer yet: the card is held as long as the cursor stays
                // on the row — the leave handler starts its 2s countdown.
                let id = preview_next_id();
                preview_next_id.set(id + 1);
                my_preview_id.set(Some(id));
                preview_stack.with_mut(|s| {
                    s.insert(0, PreviewCard { id, url, leaving: false });
                    s.truncate(5);
                });
            },
            on_hover_leave: move |()| {
                let Some(id) = my_preview_id() else {
                    return;
                };
                my_preview_id.set(None);
                // Now the card's 2s life runs; then a short fade-out, then drop.
                spawn(async move {
                    sleep_ms(2000).await;
                    preview_stack.with_mut(|s| {
                        if let Some(c) = s.iter_mut().find(|c| c.id == id) {
                            c.leaving = true;
                        }
                    });
                    sleep_ms(400).await;
                    preview_stack.with_mut(|s| s.retain(|c| c.id != id));
                });
            },
        }
    }
}

/// Loading placeholder in the shape of the loaded deck: a featured card row,
/// the controls panel, and grouped card columns. Pulses while the fetch runs.
#[component]
fn SharedDeckSkeleton() -> Element {
    rsx! {
        div { class: "shared-deck content-enter",
            header { class: "sd-header",
                div { class: "sk sk-title" }
                div { class: "sd-header-meta",
                    for i in 0..4 {
                        div { key: "{i}", class: "sk sk-chip" }
                    }
                }
            }
            section { class: "sd-featured",
                for i in 0..4 {
                    div { key: "{i}", class: "sd-cz-card",
                        div { class: "sk sk-card" }
                        div { class: "sk sk-line sk-line-name" }
                        div { class: "sk sk-line sk-line-role" }
                    }
                }
            }
            div { class: "sk sk-controls" }
            section { class: "sd-groups",
                for g in 0..3 {
                    div { key: "{g}", class: "sd-group",
                        div { class: "sd-group-header",
                            div { class: "sk sk-line sk-group-title" }
                        }
                        for r in 0..6 {
                            div { class: "card-row", key: "{r}",
                                div { class: "sk sk-row" }
                            }
                        }
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
            let client = reqwest::Client::new();
            let res = client
                .get(format!("{API_BASE}/api/share/deck/{token}"))
                .send()
                .await
                .map_err(|e| FetchError::Network(e.to_string()))?;
            if res.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(FetchError::NotShared);
            }
            if !res.status().is_success() {
                return Err(FetchError::Network(format!("status {}", res.status())));
            }
            res.json::<HttpSharedDeck>()
                .await
                .map_err(|e| FetchError::Network(e.to_string()))
        }
    });

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
                div { class: "form-page content-enter",
                    h1 { "This deck is no longer shared" }
                    p { class: "subtitle",
                        "The owner may have stopped sharing it, or the link may be incomplete."
                    }
                    Link { to: Route::Home {}, class: "sd-cta-link", "Explore Zwipe" }
                }
            },
            Some(Err(FetchError::Network(e))) => rsx! {
                PageMeta {
                    title: "Shared deck".to_string(),
                    description: "A Magic: The Gathering deck shared from Zwipe.".to_string(),
                    path: "/deck".to_string(),
                }
                div { class: "form-page content-enter",
                    h1 { "Could not load this deck" }
                    p { class: "subtitle", "Check your connection and try again." }
                    div { class: "status-message error", "{e}" }
                    button { class: "sd-retry", onclick: move |_| result.restart(), "Retry" }
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
    let mut group_option = use_signal(|| GroupByOption::CardType);
    let mut name_filter = use_signal(String::new);
    let mut selected_types = use_signal(Vec::<CardType>::new);
    let mut selected_colors = use_signal(Vec::<Color>::new);
    // Lands hidden by default (they dominate the list and add little to a
    // read-through); the Lands toggle brings them in.
    let mut show_lands = use_signal(|| false);
    let mut show_command_zone = use_signal(|| true);
    let expanded_card: Signal<Option<Uuid>> = use_signal(|| None);
    // Full-art hover-preview stack pinned top-left (desktop). Each hovered row
    // pushes a card on top; each card fades out on its own 2s timer. Never set
    // on touch devices (no mouseenter), so it simply stays empty there.
    let preview_stack: Signal<Vec<PreviewCard>> = use_signal(Vec::new);
    let preview_next_id: Signal<u64> = use_signal(|| 0);
    // Tap-to-open full-art overlay for touch/hamburger-width screens (no hover).
    // Holds the art URL; a short dismiss fade mirrors the app's ImagePreview.
    let mut overlay_image: Signal<Option<String>> = use_signal(|| None);
    let mut overlay_dismissing = use_signal(|| false);

    // Mainboard only: the maybeboard is scratch space, not the deck statement.
    let mainboard: Vec<&DeckEntry> = deck
        .entries
        .iter()
        .filter(|e| e.deck_card.board == Board::Deck)
        .collect();
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
    // Starred cards, oldest star first (the vesting clock), for the featured
    // MVP row — the deck's personality statement.
    let mut mvp_entries: Vec<&DeckEntry> = mainboard
        .iter()
        .copied()
        .filter(|e| e.deck_card.mvp_at.is_some())
        .collect();
    mvp_entries.sort_by_key(|e| e.deck_card.mvp_at);
    let mvp_cards: Vec<Card> = mvp_entries.iter().map(|e| e.card.clone()).collect();
    // A format with a signature spell is Oathbreaker: the "commander" is the
    // oathbreaker planeswalker.
    let is_oathbreaker = deck.format.as_ref().is_some_and(|f| f.has_signature_spell());

    let price = mainboard_total_price(&deck.entries, PriceCurrency::Usd);

    // Card count mirrors the app's deck-list tile: mainboard sum plus each
    // present command-zone slot (commander only when the format uses one).
    let mut display_count = card_count;
    if deck.format.as_ref().is_some_and(|f| f.has_commander()) && deck.commander.is_some() {
        display_count += 1;
    }
    if deck.partner_commander.is_some() {
        display_count += 1;
    }
    if deck.background.is_some() {
        display_count += 1;
    }
    if deck.signature_spell.is_some() {
        display_count += 1;
    }
    // Command-zone names as chips, in the app's order: commander, partner,
    // background, signature spell.
    let zone_names: Vec<String> = [
        &deck.commander,
        &deck.partner_commander,
        &deck.background,
        &deck.signature_spell,
    ]
    .into_iter()
    .filter_map(|c| c.as_ref().map(|c| c.scryfall_data.name.clone()))
    .collect();

    // The same in-memory path the app's deck view runs: criteria match, name
    // sort, then group. Filter parse failures degrade to the unfiltered list.
    let cards: Vec<Card> = mainboard
        .iter()
        .map(|e| e.card.clone())
        .filter(|c| {
            show_lands()
                || !c
                    .scryfall_data
                    .type_line
                    .as_deref()
                    .is_some_and(|t| t.contains("Land"))
        })
        .collect();
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
    let filtered: Vec<Card> = if builder.is_empty() {
        cards
    } else {
        match builder.build_criteria() {
            Ok(criteria) => Cards::from(cards).matching(&criteria).into(),
            Err(_) => cards,
        }
    };
    let filtered: Vec<Card> = Cards::from(filtered).sorted(CardSortKey::Name, true).into();
    let groups = filtered.group_by(group_option());
    let no_matching = groups.is_empty();

    // Flatten command zone + type groups into one ordered section list, then
    // greedily balance them into independent columns. CSS multi-column would
    // reflow the whole layout when a card expands (shifting groups between
    // columns); independent flex columns let a column just grow taller.
    let mut sections: Vec<(String, Vec<Card>)> = Vec::new();
    if show_command_zone() {
        let mut cz: Vec<Card> = Vec::new();
        if let Some(c) = &deck.commander {
            cz.push(c.clone());
        }
        if let Some(c) = &deck.partner_commander {
            cz.push(c.clone());
        }
        if !cz.is_empty() {
            let header = if is_oathbreaker {
                "Oathbreaker"
            } else if deck.partner_commander.is_some() {
                "Commanders"
            } else {
                "Commander"
            };
            sections.push((header.to_string(), cz));
        }
        if let Some(c) = &deck.background {
            sections.push(("Background".to_string(), vec![c.clone()]));
        }
        if let Some(c) = &deck.signature_spell {
            sections.push(("Signature spell".to_string(), vec![c.clone()]));
        }
    }
    for group in groups {
        let qty: i64 = group
            .cards
            .iter()
            .map(|c| i64::from(*qty_by_id.get(&c.scryfall_data.id).unwrap_or(&1)))
            .sum();
        sections.push((format!("{} ({})", group.label, qty), group.cards));
    }

    const COLS: usize = 3;
    let mut columns: Vec<Vec<(String, Vec<Card>)>> = vec![Vec::new(); COLS];
    let mut heights = [0usize; COLS];
    for (header, cards) in sections {
        let h = cards.len() + 2;
        // Assign to the currently shortest column.
        let ci = heights
            .iter()
            .enumerate()
            .min_by_key(|(_, x)| **x)
            .map(|(i, _)| i)
            .unwrap_or(0);
        if let Some(height) = heights.get_mut(ci) {
            *height += h;
        }
        if let Some(col) = columns.get_mut(ci) {
            col.push((header, cards));
        }
    }

    rsx! {
        PageMeta {
            title: deck.name.clone(),
            description: "A Magic: The Gathering deck shared from Zwipe.".to_string(),
            path: "/deck".to_string(),
        }
        // Pinned full-art preview stack, top-left, populated by row hover. Sits
        // in the wide-screen left gutter; empty on touch (no mouseenter) and
        // hidden below the gutter width so it never overlaps the content.
        if !preview_stack().is_empty() {
            div { class: "sd-hover-stack",
                for (i, card) in preview_stack().into_iter().enumerate() {
                    div {
                        key: "{card.id}",
                        class: if card.leaving { "sd-preview-card leaving" } else { "sd-preview-card" },
                        style: "top: calc({i} * 2.6rem); z-index: {200 - i};",
                        img { src: "{card.url}", alt: "Card preview" }
                    }
                }
            }
        }
        // Tap-to-open full-art overlay (mobile). Mirrors the app's ImagePreview:
        // a dimmed backdrop with the card art, tap anywhere to dismiss.
        if overlay_image().is_some() || overlay_dismissing() {
            div { class: "sd-image-overlay-backdrop" }
            div {
                class: if overlay_dismissing() { "sd-image-overlay dismissing" } else { "sd-image-overlay" },
                onclick: move |_| {
                    overlay_dismissing.set(true);
                    spawn(async move {
                        sleep_ms(200).await;
                        overlay_image.set(None);
                        overlay_dismissing.set(false);
                    });
                },
                if let Some(url) = overlay_image() {
                    img { class: "sd-image-overlay-img", src: "{url}", alt: "Card image" }
                }
            }
        }
        div { class: "shared-deck content-enter",
            header { class: "sd-header",
                h1 { "{deck.name}" }
                div { class: "sd-header-meta",
                    span { class: "stat-chip", "{display_count} cards" }
                    if let Some(fmt) = deck.format {
                        span { class: "stat-chip stat-chip-format", "{fmt.display_name()}" }
                    }
                    if let Some(pl) = deck.power_level {
                        span { class: "stat-chip stat-chip-power", "{pl.display_name()}" }
                    }
                    for name in zone_names.iter() {
                        span { key: "{name}", class: "stat-chip stat-chip-zone", "{name}" }
                    }
                    for tag in deck.tags.iter() {
                        span { key: "{tag}", class: "stat-chip stat-chip-tag", "{tag.display_name()}" }
                    }
                    for tag in deck.other_tags.iter() {
                        span { key: "{tag}", class: "stat-chip stat-chip-other", "{tag.display_name()}" }
                    }
                    if price > 0.0 {
                        span { class: "stat-chip stat-chip-price", "~${price:.0}" }
                    }
                }
            }

            // Featured row: the command zone and the deck MVPs together on one
            // line, each card labeled underneath — the deck's identity at a
            // glance. Always pinned at the top (the Command zone toggle only
            // governs list inclusion below, like the app).
            if deck.commander.is_some() || deck.partner_commander.is_some()
                || deck.background.is_some() || deck.signature_spell.is_some()
                || !mvp_cards.is_empty() {
                section { class: "sd-featured",
                    if let Some(card) = deck.commander.clone() {
                        CommandZoneCard {
                            card,
                            role: if is_oathbreaker { "Oathbreaker".to_string() } else { "Commander".to_string() },
                        }
                    }
                    if let Some(card) = deck.partner_commander.clone() {
                        CommandZoneCard { card, role: "Partner".to_string() }
                    }
                    if let Some(card) = deck.background.clone() {
                        CommandZoneCard { card, role: "Background".to_string() }
                    }
                    if let Some(card) = deck.signature_spell.clone() {
                        CommandZoneCard { card, role: "Signature spell".to_string() }
                    }
                    for card in mvp_cards.iter().cloned() {
                        CommandZoneCard { card, role: "MVP".to_string() }
                    }
                }
            }

            section { class: "sd-controls",
                div { class: "sd-control-row",
                    span { class: "sd-control-label", "Group by:" }
                    for option in GroupByOption::all() {
                        Chip {
                            key: "{option}",
                            selected: group_option() == option,
                            onclick: move |_| group_option.set(option),
                            "{option}"
                        }
                    }
                }
                div { class: "sd-control-row",
                    span { class: "sd-control-label", "Show:" }
                    Chip {
                        selected: show_lands(),
                        onclick: move |_| show_lands.set(!show_lands()),
                        "Lands"
                    }
                    Chip {
                        selected: show_command_zone(),
                        onclick: move |_| show_command_zone.set(!show_command_zone()),
                        "Command zone"
                    }
                }
                div { class: "sd-control-row",
                    span { class: "sd-control-label", "Filter by:" }
                    input {
                        class: "sd-filter-input",
                        r#type: "search",
                        placeholder: "Name",
                        value: "{name_filter}",
                        oninput: move |evt| name_filter.set(evt.value()),
                    }
                    for card_type in [
                        CardType::Creature, CardType::Instant, CardType::Sorcery,
                        CardType::Artifact, CardType::Enchantment,
                        CardType::Planeswalker, CardType::Land,
                    ] {
                        Chip {
                            key: "{card_type}",
                            selected: selected_types().contains(&card_type),
                            onclick: move |_| {
                                let mut current = selected_types();
                                match current.iter().position(|t| *t == card_type) {
                                    Some(idx) => { current.remove(idx); }
                                    None => current.push(card_type),
                                }
                                selected_types.set(current);
                            },
                            "{card_type}"
                        }
                    }
                }
                div { class: "sd-control-row",
                    for color in Color::all() {
                        button {
                            key: "{color:?}",
                            class: if selected_colors().contains(&color) { "sd-chip sd-chip-color active" } else { "sd-chip sd-chip-color" },
                            onclick: move |_| {
                                let mut current = selected_colors();
                                match current.iter().position(|c| *c == color) {
                                    Some(idx) => { current.remove(idx); }
                                    None => current.push(color),
                                }
                                selected_colors.set(current);
                            },
                            i { class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost" }
                        }
                    }
                }
            }

            if no_matching {
                p { class: "sd-empty", "No cards found" }
            }
            section { class: "sd-columns",
                for (ci, col) in columns.into_iter().enumerate() {
                    div { class: "sd-column", key: "{ci}",
                        for (header, cards) in col {
                            div { class: "sd-group", key: "{header}",
                                div { class: "sd-group-header", "{header}" }
                                for card in cards {
                                    CardRow {
                                        key: "{card.scryfall_data.id}",
                                        qty: *qty_by_id.get(&card.scryfall_data.id).unwrap_or(&1),
                                        mvp: mvp_ids.contains(&card.scryfall_data.id),
                                        card,
                                        expanded_card,
                                        preview_stack,
                                        preview_next_id,
                                        overlay_image,
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
