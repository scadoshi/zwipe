use crate::{
    inbound::components::{
        fields::text_input::TextInput,
        hint_dialog::{HintBullet, HintBullets, HintColored, HintDialog},
        telemetry::usage_buffer::UsageBuffer,
    },
    outbound::client::{ZwipeClient, card::search_cards::ClientSearchCards},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, Toasts, use_toast};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use zwipe_core::domain::{
    auth::models::session::Session,
    card::{
        Card,
        oracle_tag::prettify_oracle_tag_slug,
        search_card::{
            card_filter::{
                builder::CardQueryBuilder, error::InvalidCardCriteria,
                price_currency::PriceCurrency,
            },
            commander_eligibility::{PartnerKind, has_choose_a_background, partner_kind},
        },
    },
    deck::{
        DeckName, DeckOtherTag, DeckTag, MAX_DECK_ORACLE_TAGS, MAX_DECK_TAGS, PowerLevel,
        format::Format,
    },
};

/// Upper bound for the land-target stepper — no deck runs more lands than this.
const MAX_LAND_TARGET: i32 = 100;

/// "Partner with [Name]" names exactly one legal partner, so a commander pick
/// fills the partner slot automatically: fetch the named mate and set it, with
/// a toast so the user knows something automatic happened. Generic Partner /
/// Friends forever / Doctor's companion pair freely and are untouched.
///
/// Call this only from explicit commander selections (typeahead click, Zwipe
/// select) — never from the commander-change effect, which also fires on
/// edit-screen load and would toast on every open of a partner deck. The
/// commander-change clear effect runs before this fetch returns, so the fill
/// lands on a freshly cleared slot; the completion guard bails if the user
/// changed commander again (or somehow set a partner) in the meantime.
pub(crate) fn autofill_named_partner(
    selected: &Card,
    client: Signal<ZwipeClient>,
    session: Signal<Option<Session>>,
    commander: Signal<Option<Card>>,
    mut partner_commander: Signal<Option<Card>>,
    mut partner_commander_display: Signal<String>,
    toast: Toasts,
) {
    let Some(PartnerKind::Named(mate)) = partner_kind(selected) else {
        return;
    };
    let selected_id = selected.scryfall_data.id;
    spawn(async move {
        let Some(session) = session.peek().clone() else {
            return;
        };
        let mut builder = CardQueryBuilder::with_name_contains(&mate);
        builder.set_limit(5);
        let Ok(filter) = builder.build() else {
            return;
        };
        match client().search_cards(&filter, &session).await {
            Ok(cards) => {
                // Exact full-name match first — the front-face tier is only a
                // fallback for mates that exist solely as a DFC, and must not
                // pick a reversible `A // A` printing over the real card.
                let exact = cards
                    .iter()
                    .position(|c| c.scryfall_data.name.eq_ignore_ascii_case(&mate));
                let front_face = || {
                    cards.iter().position(|c| {
                        c.scryfall_data
                            .name
                            .split(" // ")
                            .next()
                            .is_some_and(|front| front.trim().eq_ignore_ascii_case(&mate))
                    })
                };
                let Some(found) = exact
                    .or_else(front_face)
                    .and_then(|i| cards.into_iter().nth(i))
                else {
                    tracing::warn!("partner autofill: no exact match for {mate}");
                    return;
                };
                let commander_unchanged = commander
                    .peek()
                    .as_ref()
                    .is_some_and(|c| c.scryfall_data.id == selected_id);
                if !commander_unchanged || partner_commander.peek().is_some() {
                    return;
                }
                let name = found.scryfall_data.name.clone();
                partner_commander_display.set(name.clone());
                partner_commander.set(Some(found));
                toast.info(
                    format!("Partner found and selected: {name}"),
                    ToastOptions::default().duration(Duration::from_millis(2500)),
                );
            }
            Err(e) => tracing::warn!("partner autofill search failed: {e}"),
        }
    });
}

/// Format chip selector and card search inputs with debounced dropdowns.
///
/// Shows/hides commander, partner, background, and signature spell fields
/// based on the selected format and commander's abilities.
///
/// Reads `Signal<Option<Session>>` and `Signal<ZwipeClient>` from context.
#[component]
pub(crate) fn DeckFields(
    mut deck_name: Signal<String>,
    mut selected_format: Signal<Option<Format>>,
    mut selected_tags: Signal<Vec<DeckTag>>,
    mut commander: Signal<Option<Card>>,
    mut commander_display: Signal<String>,
    mut partner_commander: Signal<Option<Card>>,
    mut partner_commander_display: Signal<String>,
    mut background: Signal<Option<Card>>,
    mut background_display: Signal<String>,
    mut signature_spell: Signal<Option<Card>>,
    mut signature_spell_display: Signal<String>,
    // Each toggles its field's Zwipe-select overlay. Owned by the parent screen
    // (it replaces its own content with the picker), set true by these chips.
    mut show_commander_swipe: Signal<bool>,
    mut show_partner_swipe: Signal<bool>,
    mut show_background_swipe: Signal<bool>,
    mut show_signature_spell_swipe: Signal<bool>,
    mut show_tags_select: Signal<bool>,
    mut show_format_select: Signal<bool>,
    mut deck_name_error: Signal<Option<String>>,
    // Land target. `None` = Not set, which falls back to the format heuristic.
    // A stepper edits it; the × in the label row returns it to None.
    mut land_target: Signal<Option<i32>>,
    // Price target (budget) as amount text ("" = no budget) + currency chips.
    mut price_target: Signal<String>,
    mut price_target_currency: Signal<PriceCurrency>,
    // Power level (WotC bracket). `None` = Not set. Single-select chips.
    mut power_level: Signal<Option<PowerLevel>>,
    // Other tags (non-gameplay labels). Multi-select chips.
    mut other_tags: Signal<Vec<DeckOtherTag>>,
    // Oracle tags (granular strategy slugs). Opens a fetched-catalog picker;
    // seeded from the selected deck tags by the parent screen.
    mut oracle_tags: Signal<Vec<String>>,
    mut show_oracle_tags_select: Signal<bool>,
) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let usage_buffer: Signal<UsageBuffer> = use_context();
    let toast = use_toast();

    // Deck name inline validation — mirrors the auth/profile per-field pattern:
    // show the error under the field as the user types (after first input), so a
    // bad name surfaces here instead of as a toast on save.
    let mut deck_name_touched = use_signal(|| false);
    use_effect(move || {
        let value = deck_name();
        if !value.is_empty() && !deck_name_touched() {
            deck_name_touched.set(true);
        }
        if deck_name_touched() {
            match DeckName::new(value) {
                Ok(_) => deck_name_error.set(None),
                Err(e) => deck_name_error.set(Some(e.to_string())),
            }
        }
    });

    // ========================================
    // Commander search state
    // ========================================
    let mut cmd_search_query = use_signal(String::new);
    let mut cmd_search_results = use_signal(Vec::<Card>::new);
    let mut cmd_is_searching = use_signal(|| false);
    let mut cmd_show_dropdown = use_signal(|| false);
    let mut cmd_filter_on = use_signal(|| true);

    // ========================================
    // Partner search state
    // ========================================
    let mut partner_search_query = use_signal(String::new);
    let mut partner_search_results = use_signal(Vec::<Card>::new);
    let mut partner_is_searching = use_signal(|| false);
    let mut partner_show_dropdown = use_signal(|| false);
    let mut partner_filter_on = use_signal(|| true);

    // ========================================
    // Background search state
    // ========================================
    let mut bg_search_query = use_signal(String::new);
    let mut bg_search_results = use_signal(Vec::<Card>::new);
    let mut bg_is_searching = use_signal(|| false);
    let mut bg_show_dropdown = use_signal(|| false);
    let mut bg_filter_on = use_signal(|| true);

    // ========================================
    // Signature spell search state
    // ========================================
    let mut spell_search_query = use_signal(String::new);
    let mut spell_search_results = use_signal(Vec::<Card>::new);
    let mut spell_is_searching = use_signal(|| false);
    let mut spell_show_dropdown = use_signal(|| false);
    let mut spell_filter_on = use_signal(|| true);

    // ========================================
    // Visibility memos
    // ========================================
    let show_commander = use_memo(move || selected_format().is_some_and(|f| f.has_commander()));

    let show_partner = use_memo(move || {
        show_commander() && commander().is_some_and(|c| partner_kind(&c).is_some())
    });

    let show_background = use_memo(move || {
        show_commander() && commander().is_some_and(|c| has_choose_a_background(&c))
    });

    let show_signature_spell =
        use_memo(move || selected_format().is_some_and(|f| f.has_signature_spell()));

    let is_oathbreaker =
        use_memo(move || selected_format().is_some_and(|f| f.has_signature_spell()));

    let commander_label = use_memo(move || {
        if is_oathbreaker() {
            "Oathbreaker"
        } else {
            "Commander"
        }
    });

    // Format-derived land heuristic. The stepper seeds from it on the first tick
    // so an empty field starts at the sensible count rather than zero.
    let land_heuristic = use_memo(move || selected_format().and_then(|f| f.default_land_target()));

    // ========================================
    // Cascading clear effects
    // ========================================

    // Format change → reset all filter toggles
    use_effect(move || {
        let _ = selected_format();
        cmd_filter_on.set(true);
        partner_filter_on.set(true);
        bg_filter_on.set(true);
        spell_filter_on.set(true);
    });

    // Commander change → clear partner and background (they depend on the
    // commander's abilities). Guarded by the previous commander's id: the edit
    // screen loads commander and partner through independent racing resources,
    // so the None → Some transition of the initial load must NOT clear — if
    // the partner resolved first, the commander's arrival would wipe it (the
    // field showed empty + a phantom "Save changes" on every edit open of a
    // partner deck). A genuine change is Some → Some(other) or Some → None;
    // re-setting the same commander also keeps the partner.
    let mut prev_commander_id = use_signal(|| Option::<Uuid>::None);
    use_effect(move || {
        let current = commander().map(|c| c.scryfall_data.id);
        let previous = *prev_commander_id.peek();
        prev_commander_id.set(current);
        if previous.is_none() || previous == current {
            return;
        }
        partner_commander.set(None);
        partner_commander_display.set(String::new());
        partner_search_query.set(String::new());
        partner_show_dropdown.set(false);
        background.set(None);
        background_display.set(String::new());
        bg_search_query.set(String::new());
        bg_show_dropdown.set(false);
    });

    // ========================================
    // Commander debounced search
    // ========================================
    use_effect(move || {
        let query = cmd_search_query();

        if query.len() < 3 {
            cmd_show_dropdown.set(false);
            cmd_is_searching.set(false);
            return;
        }

        cmd_is_searching.set(true);
        // Reveal the dropdown immediately so the "Searching..." indicator shows
        // during the debounce — otherwise the field looks empty for ~1s and a
        // slow reveal reads as "card missing."
        cmd_show_dropdown.set(true);

        spawn(async move {
            sleep(Duration::from_millis(800)).await;

            if cmd_search_query() != query {
                return;
            }

            if let Some(session) = session() {
                let mut builder = CardQueryBuilder::with_name_contains(&query);
                if cmd_filter_on()
                    && let Some(fmt) = selected_format()
                {
                    builder.set_is_commander_in_format(fmt);
                }
                builder.set_limit(5);
                let Ok(card_filter) = builder.build() else {
                    tracing::error!("{}", InvalidCardCriteria::Empty.to_string());
                    return;
                };
                usage_buffer().record_search();
                match client().search_cards(&card_filter, &session).await {
                    Ok(cards) => {
                        cmd_search_results.set(cards);
                        cmd_is_searching.set(false);
                        cmd_show_dropdown.set(true);
                    }
                    Err(e) => {
                        tracing::error!("search error: {}", e);
                        cmd_is_searching.set(false);
                        cmd_show_dropdown.set(false);
                    }
                }
            }
        });
    });

    // ========================================
    // Partner debounced search (1 char minimum — small card pool)
    // ========================================
    use_effect(move || {
        let query = partner_search_query();

        if query.is_empty() {
            partner_show_dropdown.set(false);
            partner_is_searching.set(false);
            return;
        }

        partner_is_searching.set(true);
        partner_show_dropdown.set(true);

        spawn(async move {
            sleep(Duration::from_millis(800)).await;

            if partner_search_query() != query {
                return;
            }

            if let Some(session) = session() {
                let mut builder = CardQueryBuilder::new();
                if !query.is_empty() {
                    builder.set_name_contains(&query);
                }
                if partner_filter_on() {
                    builder.set_is_partner(true);
                }
                builder.set_limit(10);
                let Ok(card_filter) = builder.build() else {
                    return;
                };
                usage_buffer().record_search();
                match client().search_cards(&card_filter, &session).await {
                    Ok(cards) => {
                        partner_search_results.set(cards);
                        partner_is_searching.set(false);
                        partner_show_dropdown.set(true);
                    }
                    Err(e) => {
                        tracing::error!("search error: {}", e);
                        partner_is_searching.set(false);
                        partner_show_dropdown.set(false);
                    }
                }
            }
        });
    });

    // ========================================
    // Background debounced search (1 char minimum — small card pool)
    // ========================================
    use_effect(move || {
        let query = bg_search_query();

        if query.is_empty() {
            bg_show_dropdown.set(false);
            bg_is_searching.set(false);
            return;
        }

        bg_is_searching.set(true);
        bg_show_dropdown.set(true);

        spawn(async move {
            sleep(Duration::from_millis(800)).await;

            if bg_search_query() != query {
                return;
            }

            if let Some(session) = session() {
                let mut builder = CardQueryBuilder::new();
                if !query.is_empty() {
                    builder.set_name_contains(&query);
                }
                if bg_filter_on() {
                    builder.set_is_background(true);
                }
                builder.set_limit(10);
                let Ok(card_filter) = builder.build() else {
                    return;
                };
                usage_buffer().record_search();
                match client().search_cards(&card_filter, &session).await {
                    Ok(cards) => {
                        bg_search_results.set(cards);
                        bg_is_searching.set(false);
                        bg_show_dropdown.set(true);
                    }
                    Err(e) => {
                        tracing::error!("search error: {}", e);
                        bg_is_searching.set(false);
                        bg_show_dropdown.set(false);
                    }
                }
            }
        });
    });

    // ========================================
    // Signature spell debounced search
    // ========================================
    use_effect(move || {
        let query = spell_search_query();

        if query.len() < 3 {
            spell_show_dropdown.set(false);
            spell_is_searching.set(false);
            return;
        }

        spell_is_searching.set(true);
        spell_show_dropdown.set(true);

        spawn(async move {
            sleep(Duration::from_millis(800)).await;

            if spell_search_query() != query {
                return;
            }

            if let Some(session) = session() {
                let mut builder = CardQueryBuilder::with_name_contains(&query);
                if spell_filter_on() {
                    builder.set_is_signature_spell(true);
                    // Also restrict to oathbreaker's color identity
                    if let Some(ob) = commander() {
                        builder.set_color_identity_within(ob.scryfall_data.color_identity);
                    }
                }
                builder.set_limit(5);
                let Ok(card_filter) = builder.build() else {
                    return;
                };
                usage_buffer().record_search();
                match client().search_cards(&card_filter, &session).await {
                    Ok(cards) => {
                        spell_search_results.set(cards);
                        spell_is_searching.set(false);
                        spell_show_dropdown.set(true);
                    }
                    Err(e) => {
                        tracing::error!("search error: {}", e);
                        spell_is_searching.set(false);
                        spell_show_dropdown.set(false);
                    }
                }
            }
        });
    });

    rsx! {
        // ========================================
        // Profile — name, format, command zone, power level.
        // ========================================
        div { style: "margin-bottom: 0.5rem;",
            span { class: "card-title", "Profile" }
        }

        // ========================================
        // Deck name
        // ========================================
        TextInput {
            label: "Deck name",
            value: deck_name,
            id: "deck_name",
            placeholder: "Not set",
            error: deck_name_error(),
        }

        // ========================================
        // Format (open the full-screen picker to choose)
        // ========================================
        div {
            div { class: "label-row",
                label { class: "label", "Format" }
                if selected_format().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            selected_format.set(None);
                            commander.set(None);
                            commander_display.set(String::new());
                            signature_spell.set(None);
                            signature_spell_display.set(String::new());
                        },
                        "\u{00d7}"
                    }
                }
            }

            if let Some(fmt) = selected_format() {
                div { class: "input input-tappable", onclick: move |_| show_format_select.set(true), "{fmt.display_name()}" }
            } else {
                div { class: "input input-placeholder input-tappable", onclick: move |_| show_format_select.set(true), "Not set" }
            }
        }

        // ========================================
        // Commander selector
        // ========================================
        Collapsible { show: show_commander(),
            div {
                div { class: "label-row",
                    label { class: "label", "{commander_label}" }
                    div {
                        class: if cmd_filter_on() { "chip-xs selected" } else { "chip-xs" },
                        onclick: move |_| {
                            cmd_filter_on.set(!cmd_filter_on());
                        },
                        "Filter"
                    }
                    if commander().is_some() {
                        button {
                            class: "clear-btn",
                            onclick: move |_| {
                                commander.set(None);
                                commander_display.set(String::new());
                                cmd_search_query.set(String::new());
                                cmd_show_dropdown.set(false);
                            },
                            "\u{00d7}"
                        }
                    }
                    div {
                        class: "chip-xs chip-primary",
                        onclick: move |_| show_commander_swipe.set(true),
                        "Zwipe"
                    }
                }

                if cmd_show_dropdown() {
                    if cmd_is_searching() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "Searching..." }
                        }
                    } else if cmd_search_results().is_empty() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "No results" }
                        }
                    } else {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            for card in cmd_search_results().iter().cloned() {
                                div { class: "chip-unselected",
                                    onclick: move |_| {
                                        commander.set(Some(card.clone()));
                                        commander_display.set(card.scryfall_data.name.clone());
                                        cmd_show_dropdown.set(false);
                                        autofill_named_partner(
                                            &card,
                                            client,
                                            session,
                                            commander,
                                            partner_commander,
                                            partner_commander_display,
                                            toast,
                                        );
                                    },
                                    { card.scryfall_data.name.clone() }
                                }
                            }
                        }
                    }
                }

                input { class: "input",
                    id: "commander",
                    r#type: "text",
                    placeholder: "Not set",
                    value: "{commander_display}",
                    autocapitalize: "none",
                    autocorrect: "off",
                    spellcheck: "false",
                    onclick: move |_| {
                        cmd_search_query.set(String::new());
                        commander_display.set(String::new());
                        commander.set(None);
                    },
                    oninput: move |event| {
                        cmd_search_query.set(event.value());
                        commander_display.set(event.value());
                    }
                }
            }
        }

        // ========================================
        // Partner commander selector
        // ========================================
        Collapsible { show: show_partner(),
            div {
                div { class: "label-row",
                    label { class: "label", "Partner" }
                    div {
                        class: if partner_filter_on() { "chip-xs selected" } else { "chip-xs" },
                        onclick: move |_| {
                            partner_filter_on.set(!partner_filter_on());
                        },
                        "Filter"
                    }
                    if partner_commander().is_some() {
                        button {
                            class: "clear-btn",
                            onclick: move |_| {
                                partner_commander.set(None);
                                partner_commander_display.set(String::new());
                                partner_search_query.set(String::new());
                                partner_show_dropdown.set(false);
                            },
                            "\u{00d7}"
                        }
                    }
                    div {
                        class: "chip-xs chip-primary",
                        onclick: move |_| show_partner_swipe.set(true),
                        "Zwipe"
                    }
                }

                if partner_show_dropdown() {
                    if partner_is_searching() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "Searching..." }
                        }
                    } else if partner_search_results().is_empty() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "No results" }
                        }
                    } else {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            for card in partner_search_results().iter().cloned() {
                                div { class: "chip-unselected",
                                    onclick: move |_| {
                                        partner_commander.set(Some(card.clone()));
                                        partner_commander_display.set(card.scryfall_data.name.clone());
                                        partner_show_dropdown.set(false);
                                    },
                                    { card.scryfall_data.name.clone() }
                                }
                            }
                        }
                    }
                }

                input { class: "input",
                    id: "partner_commander",
                    r#type: "text",
                    placeholder: "Not set",
                    value: "{partner_commander_display}",
                    autocapitalize: "none",
                    autocorrect: "off",
                    spellcheck: "false",
                    onclick: move |_| {
                        partner_search_query.set(String::new());
                        partner_commander_display.set(String::new());
                        partner_commander.set(None);
                    },
                    oninput: move |event| {
                        partner_search_query.set(event.value());
                        partner_commander_display.set(event.value());
                    }
                }
            }
        }

        // ========================================
        // Background selector
        // ========================================
        Collapsible { show: show_background(),
            div {
                div { class: "label-row",
                    label { class: "label", "Background" }
                    div {
                        class: if bg_filter_on() { "chip-xs selected" } else { "chip-xs" },
                        onclick: move |_| {
                            bg_filter_on.set(!bg_filter_on());
                        },
                        "Filter"
                    }
                    if background().is_some() {
                        button {
                            class: "clear-btn",
                            onclick: move |_| {
                                background.set(None);
                                background_display.set(String::new());
                                bg_search_query.set(String::new());
                                bg_show_dropdown.set(false);
                            },
                            "\u{00d7}"
                        }
                    }
                    div {
                        class: "chip-xs chip-primary",
                        onclick: move |_| show_background_swipe.set(true),
                        "Zwipe"
                    }
                }

                if bg_show_dropdown() {
                    if bg_is_searching() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "Searching..." }
                        }
                    } else if bg_search_results().is_empty() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "No results" }
                        }
                    } else {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            for card in bg_search_results().iter().cloned() {
                                div { class: "chip-unselected",
                                    onclick: move |_| {
                                        background.set(Some(card.clone()));
                                        background_display.set(card.scryfall_data.name.clone());
                                        bg_show_dropdown.set(false);
                                    },
                                    { card.scryfall_data.name.clone() }
                                }
                            }
                        }
                    }
                }

                input { class: "input",
                    id: "background",
                    r#type: "text",
                    placeholder: "Not set",
                    value: "{background_display}",
                    autocapitalize: "none",
                    autocorrect: "off",
                    spellcheck: "false",
                    onclick: move |_| {
                        bg_search_query.set(String::new());
                        background_display.set(String::new());
                        background.set(None);
                    },
                    oninput: move |event| {
                        bg_search_query.set(event.value());
                        background_display.set(event.value());
                    }
                }
            }
        }

        // ========================================
        // Signature spell selector
        // ========================================
        Collapsible { show: show_signature_spell(),
            div {
                div { class: "label-row",
                    label { class: "label", "Signature spell" }
                    div {
                        class: if spell_filter_on() { "chip-xs selected" } else { "chip-xs" },
                        onclick: move |_| {
                            spell_filter_on.set(!spell_filter_on());
                        },
                        "Filter"
                    }
                    if signature_spell().is_some() {
                        button {
                            class: "clear-btn",
                            onclick: move |_| {
                                signature_spell.set(None);
                                signature_spell_display.set(String::new());
                                spell_search_query.set(String::new());
                                spell_show_dropdown.set(false);
                            },
                            "\u{00d7}"
                        }
                    }
                    div {
                        class: "chip-xs chip-primary",
                        onclick: move |_| show_signature_spell_swipe.set(true),
                        "Zwipe"
                    }
                }

                if spell_show_dropdown() {
                    if spell_is_searching() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "Searching..." }
                        }
                    } else if spell_search_results().is_empty() {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            div { class: "chip-unselected", "No results" }
                        }
                    } else {
                        div { class: "flex flex-wrap gap-1 mb-1 flex-center",
                            for card in spell_search_results().iter().cloned() {
                                div { class: "chip-unselected",
                                    onclick: move |_| {
                                        signature_spell.set(Some(card.clone()));
                                        signature_spell_display.set(card.scryfall_data.name.clone());
                                        spell_show_dropdown.set(false);
                                    },
                                    { card.scryfall_data.name.clone() }
                                }
                            }
                        }
                    }
                }

                input { class: "input",
                    id: "signature_spell",
                    r#type: "text",
                    placeholder: "Not set",
                    value: "{signature_spell_display}",
                    autocapitalize: "none",
                    autocorrect: "off",
                    spellcheck: "false",
                    onclick: move |_| {
                        spell_search_query.set(String::new());
                        signature_spell_display.set(String::new());
                        signature_spell.set(None);
                    },
                    oninput: move |event| {
                        spell_search_query.set(event.value());
                        signature_spell_display.set(event.value());
                    }
                }
            }
        }

        // ========================================
        // Power level (single-select WotC bracket; Not set = none)
        // ========================================
        div { style: "margin-top: 1rem;",
            div { class: "label-row",
                label { class: "label", "Power level" }
                if power_level().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| power_level.set(None),
                        "\u{00d7}"
                    }
                }
            }
            div { class: "chip-row", style: "flex-wrap: wrap; justify-content: center;",
                for pl in PowerLevel::all().iter().copied() {
                    div {
                        key: "{pl}",
                        class: if power_level() == Some(pl) { "chip selected" } else { "chip" },
                        onclick: move |_| {
                            if power_level() == Some(pl) {
                                power_level.set(None);
                            } else {
                                power_level.set(Some(pl));
                            }
                        },
                        "{pl.display_name()}"
                    }
                }
            }
        }

        // ========================================
        // Budget — land target + price target.
        // ========================================
        div { style: "margin-top: 1.5rem;",
            span { class: "card-title", "Budget" }
        }

        // ========================================
        // Land target (Not set = use the format heuristic)
        // ========================================
        div { style: "margin-top: 1rem;",
            div { class: "label-row",
                label { class: "label", "Land target" }
                if land_target().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| land_target.set(None),
                        "\u{00d7}"
                    }
                }
            }

            div { class: "stepper",
                button {
                    class: "stepper-btn",
                    r#type: "button",
                    onclick: move |_| {
                        let next = match land_target() {
                            None => land_heuristic().unwrap_or(0),
                            Some(v) => v.saturating_sub(1),
                        };
                        land_target.set(Some(next));
                    },
                    "-"
                }
                span { class: "stepper-value", style: "width: auto; min-width: 4rem;",
                    if let Some(v) = land_target() { "{v}" } else { "Not set" }
                }
                button {
                    class: "stepper-btn",
                    r#type: "button",
                    onclick: move |_| {
                        let next = match land_target() {
                            None => land_heuristic().unwrap_or(0),
                            Some(v) => (v + 1).min(MAX_LAND_TARGET),
                        };
                        land_target.set(Some(next));
                    },
                    "+"
                }
            }
        }

        // ========================================
        // Price target (budget; empty = no budget)
        // ========================================
        div { style: "margin-top: 1rem;",
            div { class: "label-row",
                label { class: "label", "Price target" }
                if !price_target().is_empty() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| price_target.set(String::new()),
                        "\u{00d7}"
                    }
                }
            }

            div { class: "chip-row", style: "margin-bottom: 0.5rem; justify-content: center;",
                for currency in PriceCurrency::all().iter().copied() {
                    div {
                        class: if price_target_currency() == currency { "chip selected" } else { "chip" },
                        onclick: move |_| price_target_currency.set(currency),
                        "{currency.label()}"
                    }
                }
            }

            input { class: "input",
                id: "price_target",
                r#type: "text",
                inputmode: "decimal",
                placeholder: "Not set",
                value: "{price_target}",
                autocapitalize: "none",
                autocorrect: "off",
                spellcheck: "false",
                oninput: move |event| {
                    // Digits and a single decimal point only.
                    let filtered: String = event.value().chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
                    price_target.set(filtered);
                }
            }
        }

        // ========================================
        // Tags — deck tags, oracle tags, other tags. Grouped at the bottom
        // (under a "Tags" heading) to mirror the deck view's Tags section.
        // ========================================
        div { style: "margin-top: 1.5rem;",
            span { class: "card-title", "Tags" }
        }

        // Deck tags (open the full-screen picker to choose)
        div { style: "margin-top: 1rem;",
            div { class: "label-row",
                label { class: "label", "Deck tags" }
                span { class: "field-count", "{selected_tags().len()}/{MAX_DECK_TAGS}" }
                if !selected_tags().is_empty() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| selected_tags.set(Vec::new()),
                        "\u{00d7}"
                    }
                }
            }

            if selected_tags().is_empty() {
                div { class: "input input-placeholder input-tappable", onclick: move |_| show_tags_select.set(true), "Not set" }
            } else {
                div { class: "chip-box input-tappable", onclick: move |_| show_tags_select.set(true),
                    for tag in selected_tags().iter().copied() {
                        div {
                            key: "{tag}",
                            class: "chip selected flex items-center gap-05",
                            "{tag.display_name()}"
                            // Remove just this tag; stop propagation so the
                            // chip-box's own onclick doesn't also open the picker.
                            button {
                                class: "chip-remove",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    selected_tags.write().retain(|t| *t != tag);
                                },
                                "\u{00d7}"
                            }
                        }
                    }
                }
            }
        }

        // Oracle tags (granular strategy; deck tags seed these)
        div { style: "margin-top: 1rem;",
            div { class: "label-row",
                label { class: "label", "Oracle tags" }
                span { class: "field-count", "{oracle_tags().len()}/{MAX_DECK_ORACLE_TAGS}" }
                if !oracle_tags().is_empty() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| oracle_tags.set(Vec::new()),
                        "\u{00d7}"
                    }
                }
            }

            if oracle_tags().is_empty() {
                div {
                    class: "input input-placeholder input-tappable",
                    onclick: move |_| show_oracle_tags_select.set(true),
                    "Not set"
                }
            } else {
                div { class: "chip-box input-tappable", onclick: move |_| show_oracle_tags_select.set(true),
                    for slug in oracle_tags().iter().cloned() {
                        div {
                            key: "{slug}",
                            class: "chip selected flex items-center gap-05",
                            { prettify_oracle_tag_slug(&slug) }
                            button {
                                class: "chip-remove",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    oracle_tags.write().retain(|s| s != &slug);
                                },
                                "\u{00d7}"
                            }
                        }
                    }
                }
            }
        }

        // Other tags (non-gameplay labels; multi-select)
        div { style: "margin-top: 1rem;",
            div { class: "label-row",
                label { class: "label", "Other tags" }
                if !other_tags().is_empty() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| other_tags.set(Vec::new()),
                        "\u{00d7}"
                    }
                }
            }
            div { class: "chip-row", style: "flex-wrap: wrap; justify-content: center;",
                for tag in DeckOtherTag::all().iter().copied() {
                    div {
                        key: "{tag}",
                        class: if other_tags().contains(&tag) { "chip selected" } else { "chip" },
                        onclick: move |_| {
                            let mut v = other_tags();
                            if let Some(pos) = v.iter().position(|t| *t == tag) {
                                v.remove(pos);
                            } else {
                                v.push(tag);
                            }
                            other_tags.set(v);
                        },
                        "{tag.display_name()}"
                    }
                }
            }
        }
    }
}

/// Explainer dialog for the deck create/edit form. Shared by both screens so
/// the command-zone behavior is described in one place. `open` is owned by the
/// screen (a one-time hint plus a "?" button).
#[component]
pub(crate) fn DeckFieldsHint(open: Signal<bool>) -> Element {
    rsx! {
        HintDialog {
            open,
            title: "Building a deck",
            HintBullets {
                HintBullet { "Name your deck and choose a format." }
                HintBullet {
                    "Add up to "
                    HintColored { color: "--accent-tertiary", "5 tags" }
                    " to label your deck's strategy."
                }
                HintBullet {
                    "Command-zone fields are dynamic: "
                    HintColored { color: "--accent-tertiary", "Partner" }
                    ", "
                    HintColored { color: "--accent-tertiary", "Background" }
                    ", and "
                    HintColored { color: "--accent-tertiary", "Signature spell" }
                    " appear only when your commander or format needs them."
                }
                HintBullet {
                    "Commander search auto-limits to legal commanders. Tap "
                    HintColored { color: "--accent-secondary", "Filter" }
                    " to search any card instead."
                }
                HintBullet {
                    "Tap "
                    HintColored { color: "--accent-primary", "Zwipe" }
                    " on a field to swipe-pick."
                }
            }
        }
    }
}

/// Eases a command-zone field's height + opacity open/closed instead of popping
/// it in and out. Always rendered (the field's search/clear effects live at the
/// `DeckFields` top level), so this only animates appearance, never logic.
#[component]
fn Collapsible(show: bool, children: Element) -> Element {
    let class = if show {
        "collapsible open"
    } else {
        "collapsible"
    };
    rsx! {
        div { class: "{class}",
            div { class: "collapsible-inner", {children} }
        }
    }
}
