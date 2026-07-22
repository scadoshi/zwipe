use super::components::{
    clone_deck_dialog::CloneDeckDialog,
    collapsible_section::CollapsibleSection,
    deck_budget_section::{DeckBudgetSection, has_budget},
    deck_charts::{DeckCharts, DrawOdds, ManaBalanceRow, ManaCurve, ManaFulfillment},
    deck_profile::DeckProfileSection,
    deck_tags_section::{DeckTagsSection, has_any_tags, total_tag_count},
    deck_warnings::DeckWarnings,
    more_buttons::MoreButtons,
    skeletons::{DeckProfileSkeleton, DeckStatsSkeleton},
};
use crate::{
    inbound::{
        components::{
            alert_dialog::{
                AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
                AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
            },
            auth::ensure_session::EnsureFresh,
            hint_dialog::{HintBullet, HintBullets, HintDialog, HintKey, use_one_time_hint},
            screen_header::ScreenHeader,
        },
        router::Router,
    },
    outbound::{
        buy_links,
        client::{
            ZwipeClient,
            card::get_card::ClientGetCard,
            deck::{
                delete_deck::ClientDeleteDeck, get_deck::ClientGetDeck,
                get_deck_profile::ClientGetDeckProfile,
                update_deck_profile::ClientUpdateDeckProfile,
            },
            deck_card::update_deck_card::ClientUpdateDeckCard,
        },
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use uuid::Uuid;
use zwipe::inbound::http::ApiError;
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        card::Card,
        deck::{
            DeckEntry, deck_metrics::DeckMetrics, deck_profile::DeckProfile,
            deck_warning::DeckWarning,
        },
        user::models::hints::HINT_FIRST_DECK,
    },
    http::{
        contracts::{deck::HttpUpdateDeckProfile, deck_card::HttpUpdateDeckCard},
        helpers::Opdate,
    },
};

type DeckResult = Result<(Vec<DeckEntry>, Vec<DeckWarning>, Vec<Card>), ApiError>;

#[component]
pub fn ViewDeck(deck_id: Uuid) -> Element {
    // config
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    // original deck information
    let mut commander: Signal<Option<Card>> = use_signal(|| None);
    let toast = use_toast();

    // `use_reactive!` ties these resources to `deck_id` so they re-fetch when the
    // route param changes without a remount — e.g. cloning navigates ViewDeck →
    // ViewDeck, and a plain `move ||` closure would keep serving the old deck.
    let mut deck_profile_resource: Resource<Result<DeckProfile, ApiError>> =
        use_resource(use_reactive!(|deck_id| async move {
            let session = session.ensure_fresh(client).await?;
            client().get_deck_profile(deck_id, &session).await
        }));
    let commander_resource: Resource<Result<Option<Card>, ApiError>> =
        use_resource(move || async move {
            let Some(Ok(DeckProfile {
                commander_id: Some(original_commander_id),
                ..
            })) = deck_profile_resource()
            else {
                return Ok(None);
            };
            client().get_card(original_commander_id).await.map(Some)
        });
    let mut deck_resource: Resource<DeckResult> =
        use_resource(use_reactive!(|deck_id| async move {
            let session = session.ensure_fresh(client).await?;
            client()
                .get_deck(deck_id, &session)
                .await
                .map(|d| (d.entries, d.warnings, d.command_zone_cards))
        }));
    use_effect(move || {
        if let Some(Err(e)) = &*deck_profile_resource.read() {
            toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
        }
    });

    use_effect(move || match commander_resource() {
        Some(Ok(Some(original_commander))) => {
            commander.set(Some(original_commander));
        }
        Some(Err(e)) => {
            toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
        }
        Some(Ok(None)) | None => (),
    });

    // First-deck tour: auto-opens once per account on first opening any
    // deck profile; the header "?" reopens it on demand.
    let first_deck_hint_open = use_one_time_hint(HINT_FIRST_DECK);

    let show_buy_dialog = use_signal(|| false);
    // Accordion state for the deck-view sections — holds the title of the one
    // open section. Stats is auto-expanded on load.
    let open_section: Signal<Option<String>> = use_signal(|| Some("Budget".to_string()));
    // Currency selected in the Budget header chips, shared with the price rows.
    let mut selected_currency: Signal<&'static str> = use_signal(|| "usd");
    // Seed the chip once from the deck's price-target currency so the price
    // fraction (total / target) reads in one currency by default; the user can
    // still switch chips afterward without it snapping back.
    let mut currency_seeded = use_signal(|| false);
    use_effect(move || {
        if !currency_seeded()
            && let Some(Ok(p)) = &*deck_profile_resource.read()
        {
            if let Some(c) = p.price_target_currency {
                selected_currency.set(c.json_key());
            }
            currency_seeded.set(true);
        }
    });
    let mut show_more_sheet = use_signal(|| false);
    let mut show_delete_dialog = use_signal(|| false);
    let show_clone_dialog = use_signal(|| false);
    // Live share-link state, seeded from the loaded profile and driven by the
    // More sheet's Share / Stop-sharing actions so the buttons reflect reality
    // without a round-trip re-fetch.
    let mut share_token = use_signal(|| None::<Uuid>);
    use_effect(move || {
        if let Some(Ok(p)) = &*deck_profile_resource.read() {
            share_token.set(p.share_token);
        }
    });
    let attempt_delete = move || {
        spawn(async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(e) => {
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    return;
                }
            };

            match client().delete_deck(deck_id, &session).await {
                Ok(_) => {
                    navigator.push(Router::DeckList {});
                }
                Err(e) => {
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                }
            }
        });
    };

    // pre-compute metrics and chart data before rsx!
    // True only while the deck fetch is in flight. The stats skeleton must
    // key off this, not off metrics being absent: a loaded deck with a
    // commander but no cards never produces metrics, and an absence-keyed
    // skeleton pulses forever on exactly that deck.
    let deck_loading = deck_resource().is_none();
    let deck_data = deck_resource().and_then(|r| r.ok());
    let warnings: Vec<DeckWarning> = deck_data
        .as_ref()
        .map(|(_, w, _)| w.clone())
        .unwrap_or_default();
    // Metrics fold the command zone (commander, partner, background, signature
    // spell) into card count + prices; present when there is a mainboard card or
    // a command-zone card. Distributions/charts stay mainboard-only.
    let metrics = deck_data
        .as_ref()
        .filter(|(entries, _, command_zone)| !entries.is_empty() || !command_zone.is_empty())
        .map(|(entries, _, command_zone)| {
            DeckMetrics::from_entries_and_command_zone(entries, command_zone)
        });
    // Chart data reflects the mainboard only, so suppress it for a deck that so
    // far holds only its command zone (no mainboard cards to distribute).
    let chart_metrics = metrics
        .as_ref()
        .filter(|m| m.land_count + m.nonland_count > 0);

    let command_zone_names: Vec<String> = deck_profile_resource()
        .and_then(|r| r.ok())
        .map(|p| {
            [
                p.commander_name.as_deref(),
                p.partner_commander_name.as_deref(),
                p.background_name.as_deref(),
                p.signature_spell_name.as_deref(),
            ]
            .into_iter()
            .flatten()
            .map(|s| s.to_string())
            .collect()
        })
        .unwrap_or_default();
    let cz_refs: Vec<&str> = command_zone_names.iter().map(|s| s.as_str()).collect();

    let tcg_url = deck_data.as_ref().map(|(entries, _, _)| {
        let active: Vec<_> = entries
            .iter()
            .filter(|e| e.deck_card.board.is_active() || e.deck_card.board.is_sideboard())
            .cloned()
            .collect();
        buy_links::tcgplayer_url(&active, &cz_refs)
    });
    let ck_url = deck_data.as_ref().map(|(entries, _, _)| {
        let active: Vec<_> = entries
            .iter()
            .filter(|e| e.deck_card.board.is_active() || e.deck_card.board.is_sideboard())
            .cloned()
            .collect();
        buy_links::cardkingdom_url(&active, &cz_refs)
    });

    let mana_curve_bars: Option<[(usize, u32); 7]> = chart_metrics.map(|m| {
        let max_count = m.cmc_histogram.iter().copied().max().unwrap_or(0);
        std::array::from_fn(|i| {
            let count = m.cmc_histogram.get(i).copied().unwrap_or(0);
            let pct = if max_count > 0 && count > 0 {
                ((count * 100) / max_count).max(4) as u32
            } else {
                0
            };
            (count, pct)
        })
    });

    let type_bars: Option<Vec<(&str, usize, u32)>> = chart_metrics.map(|m| {
        let max_count = m.type_counts.iter().map(|(_, c)| *c).max().unwrap_or(0);
        m.type_counts
            .iter()
            .map(|(label, count)| {
                let pct = if max_count > 0 && *count > 0 {
                    ((count * 100) / max_count).max(4) as u32
                } else {
                    0
                };
                (DeckMetrics::abbreviate_type(label), *count, pct)
            })
            .collect()
    });

    let color_bars: Option<Vec<(&str, usize, u32)>> = chart_metrics.map(|m| {
        let max_count = m.color_counts.iter().map(|(_, c)| *c).max().unwrap_or(0);
        m.color_counts
            .iter()
            .map(|(label, count)| {
                let pct = if max_count > 0 && *count > 0 {
                    ((count * 100) / max_count).max(4) as u32
                } else {
                    0
                };
                (DeckMetrics::abbreviate_color(label), *count, pct)
            })
            .collect()
    });

    let category_bars: Option<Vec<(&str, usize, u32)>> = chart_metrics.and_then(|m| {
        if m.card_role_counts.is_empty() {
            return None;
        }
        let max_count = m
            .card_role_counts
            .iter()
            .map(|(_, c)| *c)
            .max()
            .unwrap_or(0);
        Some(
            m.card_role_counts
                .iter()
                .map(|(label, count)| {
                    let pct = if max_count > 0 && *count > 0 {
                        ((count * 100) / max_count).max(4) as u32
                    } else {
                        0
                    };
                    (*label, *count, pct)
                })
                .collect(),
        )
    });

    // Draw-odds buckets: (deck_size, [(label, count)]). Library size = the
    // mainboard (the commander sits in the command zone, not the library). The
    // DrawOdds component turns these into per-turn P(>=1) live.
    let draw_odds_data: Option<(u32, Vec<(&'static str, u32)>)> = chart_metrics.map(|m| {
        let deck_size = (m.land_count + m.nonland_count) as u32;
        let mut buckets: Vec<(&'static str, u32)> = vec![("Land", m.land_count as u32)];
        buckets.extend(
            m.card_role_counts
                .iter()
                .map(|(label, count)| (*label, *count as u32)),
        );
        (deck_size, buckets)
    });

    let mana_balance_rows = chart_metrics.map(|m| -> Vec<_> {
        let labels = ["W", "U", "B", "R", "G"];
        labels
            .iter()
            .zip(m.mana_balance.iter())
            .filter(|(_, (consumed, _produced))| *consumed > 0)
            .map(|(label, (consumed, produced))| {
                let bar_max = (*consumed).max(*produced);
                // Floor a nonzero share at 4% so a barely-produced color still
                // shows a visible sliver, matching the curve/type/color bars.
                let fill_pct = if *produced > 0 {
                    (produced * 100).checked_div(bar_max).unwrap_or(0).max(4) as u32
                } else {
                    0
                };
                let is_surplus = produced >= consumed;
                ManaBalanceRow {
                    label,
                    consumed: *consumed,
                    produced: *produced,
                    fill_pct,
                    is_surplus,
                }
            })
            .collect()
    });

    rsx! {
            div { class: "screen",
                ScreenHeader { title: "Deck", hint: first_deck_hint_open }

                div { class: "screen-content",
                    match deck_profile_resource() {
                        Some(Ok(deck_profile)) => {
                            let metrics_possible = deck_profile.card_count > 0
                                || deck_profile.commander_id.is_some()
                                || deck_profile.partner_commander_id.is_some()
                                || deck_profile.background_id.is_some()
                                || deck_profile.signature_spell_id.is_some();
                            // Copied out before the rsx! moves deck_profile into
                            // the Tags section; the Lands row (below, in Mana) needs it.
                            let land_target = deck_profile.land_target;
                            rsx! {
                            div { class: "content-enter",
                                  style: "width: calc(100% - 4rem); display: flex; flex-direction: column; gap: 1rem; padding: 1rem 0;",
                                DeckProfileSection {
                                    deck_profile: deck_profile.clone(),
                                    commander: commander(),
                                    card_count: metrics.as_ref().map(|m| m.total_cards).unwrap_or_else(|| deck_profile.card_count.max(0) as usize),
                                }

                                if has_budget(&deck_profile, metrics.is_some()) {
                                    CollapsibleSection {
                                        title: "Budget",
                                        open_section: open_section,
                                        header_accessory: {
                                            if metrics.is_some() {
                                                rsx! {
                                                    div { class: "chip-row", style: "margin-bottom:0;",
                                                        for (label, key) in [("USD", "usd"), ("EUR", "eur"), ("TIX", "tix")] {
                                                            div {
                                                                class: if selected_currency() == key { "chip selected" } else { "chip" },
                                                                onclick: move |_| selected_currency.set(key),
                                                                "{label}"
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                rsx! {}
                                            }
                                        },
                                        DeckBudgetSection {
                                            deck_profile: deck_profile.clone(),
                                            metrics: metrics.clone(),
                                            selected_currency: selected_currency,
                                        }
                                    }
                                }

                                if has_any_tags(&deck_profile) {
                                    CollapsibleSection {
                                        title: "Tags",
                                        badge: format!("{}", total_tag_count(&deck_profile)),
                                        open_section: open_section,
                                        DeckTagsSection { deck_profile }
                                    }
                                }

                                {
                                    // Lands presence is its own reason to show the Mana
                                    // section: a curve (mainboard cards) OR just a land
                                    // target set, so the Lands goal-vs-actual stays
                                    // visible even on an otherwise-empty deck.
                                    let land_count = metrics.as_ref().map(|m| m.land_count).unwrap_or(0);
                                    rsx! {
                                        if mana_curve_bars.is_some() || land_target.is_some() {
                                          div { class: "content-enter",
                                                style: "display: flex; flex-direction: column; gap: 0.75rem;",
                                            // Distributions stays mainboard-only (its bars
                                            // are None without mainboard cards).
                                            if mana_curve_bars.is_some() {
                                                CollapsibleSection {
                                                    title: "Distributions",
                                                    open_section: open_section,
                                                    if let Some(m) = metrics.as_ref() {
                                                        if m.avg_power.is_some() || m.avg_toughness.is_some() {
                                                            {
                                                                let fmt = |v: Option<f64>| v.map_or_else(|| "-".to_string(), |v| format!("{v:.1}"));
                                                                let value = format!("{} / {}", fmt(m.avg_power), fmt(m.avg_toughness));
                                                                rsx! {
                                                                    div { class: "info-row",
                                                                        span { class: "info-row-label", "Average P/T" }
                                                                        span { class: "info-row-value", style: "white-space:nowrap;", "{value}" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    DeckCharts {
                                                        type_bars: type_bars.clone(),
                                                        category_bars: category_bars.clone(),
                                                        color_bars: color_bars.clone(),
                                                    }
                                                }
                                            }

                                            CollapsibleSection {
                                                title: "Mana",
                                                open_section: open_section,
                                                // Lands: actual count, shown as `actual / target`
                                                // when a land target is set. Moved here from the
                                                // Budget section in 1.6.0.
                                                {
                                                    let value = match land_target {
                                                        Some(target) => format!("{land_count} / {target}"),
                                                        None => land_count.to_string(),
                                                    };
                                                    rsx! {
                                                        div { class: "info-row",
                                                            span { class: "info-row-label", "Lands" }
                                                            span { class: "info-row-value", "{value}" }
                                                        }
                                                    }
                                                }
                                                if let Some(m) = metrics.as_ref() {
                                                    if mana_curve_bars.is_some() {
                                                        div { class: "info-row",
                                                            span { class: "info-row-label", "Average mana value" }
                                                            span { class: "info-row-value", "{m.avg_cmc:.1}" }
                                                        }
                                                    }
                                                }
                                                if let Some(mana_curve_bars) = mana_curve_bars.as_ref() {
                                                    ManaCurve { mana_curve_bars: *mana_curve_bars }
                                                }
                                                if let Some(rows) = mana_balance_rows {
                                                    ManaFulfillment { rows: rows }
                                                }
                                            }

                                            if let Some((deck_size, buckets)) = draw_odds_data.clone() {
                                                CollapsibleSection {
                                                    title: "Draw odds",
                                                    open_section: open_section,
                                                    DrawOdds { deck_size: deck_size, buckets: buckets }
                                                }
                                            }
                                          }
                                        } else if metrics_possible && deck_loading {
                                            DeckStatsSkeleton {}
                                        }
                                    }
                                }

                                if !warnings.is_empty() {
                                    CollapsibleSection {
                                        title: "Warnings",
                                        warn: true,
                                        open_section: open_section,
                                        badge: warnings.len().to_string(),
                                    DeckWarnings {
                                        warnings: warnings,
                                        deck_id: deck_id,
                                        on_remove: move |_| {
                                            deck_resource.restart();
                                        },
                                        on_fix_quantity: move |(card_id, target_qty): (Uuid, i32)| {
                                            let current_qty = deck_resource()
                                                .and_then(|r| r.ok())
                                                .and_then(|(entries, _, _)| {
                                                    entries.iter()
                                                        .find(|e| e.card.scryfall_data.id == card_id)
                                                        .map(|e| *e.deck_card.quantity)
                                                })
                                                .unwrap_or(1);
                                            let delta = target_qty - current_qty;
                                            let request = HttpUpdateDeckCard::new(Some(delta), None);

                                            spawn(async move {
                                                let session = match session.ensure_fresh(client).await {
                                                    Ok(session) => session,
                                                    Err(_) => return,
                                                };

                                                match client().update_deck_card(deck_id, card_id, &request, &session).await {
                                                    Ok(_) => {
                                                        toast.info(
                                                            format!("Quantity set to {target_qty}"),
                                                            ToastOptions::default().duration(Duration::from_millis(1500)),
                                                        );
                                                        deck_resource.restart();
                                                    }
                                                    Err(e) => {
                                                        toast.error(e.to_user_message(), ToastOptions::default().duration(Duration::from_millis(3000)));
                                                    }
                                                }
                                            });
                                        },
                                        on_clear_commander: move |_| {
                                            let request = HttpUpdateDeckProfile::builder()
                                                .commander_id(Opdate::Set(None))
                                                .build();

                                            spawn(async move {
                                                let session = match session.ensure_fresh(client).await {
                                                    Ok(session) => session,
                                                    Err(_) => return,
                                                };

                                                match client().update_deck_profile(deck_id, &request, &session).await {
                                                    Ok(_) => {
                                                        let label = if deck_profile_resource().is_some_and(|r| r.as_ref().ok().is_some_and(|p| p.format.as_ref().is_some_and(|f| f.has_signature_spell()))) {
                                                            "Oathbreaker"
                                                        } else {
                                                            "Commander"
                                                        };
                                                        toast.info(
                                                            format!("{label} cleared"),
                                                            ToastOptions::default().duration(Duration::from_millis(1500)),
                                                        );
                                                        commander.set(None);
                                                        deck_profile_resource.restart();
                                                        deck_resource.restart();
                                                    }
                                                    Err(e) => {
                                                        toast.error(e.to_user_message(), ToastOptions::default().duration(Duration::from_millis(3000)));
                                                    }
                                                }
                                            });
                                        },
                                    }
                                    }
                                }

                            }
                            }
                        },
                        Some(Err(_)) => rsx! { p { class: "text-muted", "Could not load deck" } },
                        None => rsx! {
                            div { class: "content-enter",
                                  style: "width: calc(100% - 4rem); display: flex; flex-direction: column; gap: 1rem; padding: 1rem 0;",
                                DeckProfileSkeleton {}
                                DeckStatsSkeleton {}
                            }
                        }
                    }
                }

            ActionBar { class: "content-enter-delayed",
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        navigator.push(Router::DeckList {});
                    },
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        navigator.push(Router::EditDeck { deck_id });
                    },
                    "Edit"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        navigator.push(Router::ViewDeckCard { deck_id });
                    },
                    "Cards"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| show_more_sheet.set(true),
                    "More"
                }
            }

            AlertDialogRoot {
                open: show_delete_dialog(),
                on_open_change: move |open| show_delete_dialog.set(open),
                AlertDialogContent {
                    AlertDialogTitle { "Delete deck" }
                    hr { class: "dialog-rule" }
                    AlertDialogDescription { "Are you sure you want to delete this deck?" }
                    hr { class: "dialog-rule" }
                    AlertDialogActions {
                        AlertDialogCancel {
                            on_click: move |_| show_delete_dialog.set(false),
                            "Cancel"
                        }
                        AlertDialogAction {
                            danger: true,
                            on_click: move |_| attempt_delete(),
                            "Delete"
                        }
                    }
                }
            }

            MoreButtons {
                deck_id: deck_id,
                show_buy_dialog: show_buy_dialog,
                show_more_sheet: show_more_sheet,
                show_delete_dialog: show_delete_dialog,
                show_clone_dialog: show_clone_dialog,
                share_token: share_token,
                has_cards: metrics.is_some(),
                tcg_url: tcg_url,
                ck_url: ck_url,
            }

            CloneDeckDialog {
                source_deck_id: deck_id,
                default_name: deck_profile_resource()
                    .and_then(|r| r.ok())
                    .map(|p| format!("{} (clone)", p.name))
                    .unwrap_or_default(),
                open: show_clone_dialog,
            }

            HintDialog {
                open: first_deck_hint_open,
                title: "Welcome to your deck",
                HintBullets {
                    HintBullet {
                        "Tap "
                        HintKey { color: "--accent-primary", "Cards" }
                        " to browse your deck's cards, and to add or remove cards"
                    }
                    HintBullet {
                        "Tap "
                        HintKey { color: "--accent-secondary", "Edit" }
                        " to change name, format, command zone, tags, land target, or price target"
                    }
                    HintBullet {
                        "Tap "
                        HintKey { "More" }
                        " to import, export, clone, buy, share, or delete the deck"
                    }
                    HintBullet {
                        "Cards you skip or remove stay out of the add stack for this deck. "
                        HintKey { "More" }
                        " then Clear skips brings them back"
                    }
                    HintBullet {
                        "Stats appear as the deck grows, tap a section like "
                        HintKey { color: "--color-success", "Stats" }
                        " or "
                        HintKey { color: "--color-warning", "Mana" }
                        " to expand it"
                    }
                    HintBullet {
                        "Warnings call out rule problems and offer one-tap fixes"
                    }
                }
            }

            }
    }
}
