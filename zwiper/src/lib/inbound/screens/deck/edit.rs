//! Edit deck screen.

use components::{ActionBar, Button, ButtonVariant, TOAST_NORMAL};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::domain::deck::format::Format;
use zwipe_core::domain::deck::requests::update_deck_profile::InvalidUpdateDeckProfile;
use zwipe_core::domain::deck::{
    self, Deck, DeckName, DeckOtherTag, DeckTagView, MAX_DECK_ORACLE_TAGS, PowerLevel,
};
use zwipe_core::domain::user::models::hints::HINT_EDIT_DECK;
use zwipe_core::http::contracts::deck::HttpUpdateDeckProfile;
use zwipe_core::http::helpers::Patch;

use super::components::deck_fields::{self, DeckFields, DeckFieldsHint};
use super::components::format_select::FormatSelect;
use super::components::oracle_tag_select::OracleTagSelect;
use super::components::skeletons::EditDeckSkeleton;
use super::components::swipe_select::{SwipeMode, SwipeSelect};
use super::components::tag_select::TagSelect;
use crate::inbound::components::auth::authed::use_authed;
use crate::inbound::components::catalog_cache::CatalogCache;
use crate::inbound::components::hint_dialog::use_one_time_hint;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::components::telemetry::vocabulary::{DeckScreen, Screen, component, screen};
use crate::inbound::router::Router;

/// Screen for editing a deck with name and settings.
/// Snapshot of the format + command-zone state, captured when the format picker
/// opens so Cancel can revert it: (format, commander, commander name, signature
/// spell, signature spell name).
type FormatSnapshot = (Option<Format>, Option<Card>, String, Option<Card>, String);

#[component]
pub fn EditDeck(deck_id: Uuid) -> Element {
    // config
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    // current values
    let mut deck_name: Signal<String> = use_signal(String::new);
    let mut deck_name_error: Signal<Option<String>> = use_signal(|| None);
    let mut commander: Signal<Option<Card>> = use_signal(|| None);
    let mut commander_display = use_signal(String::new);
    let mut show_commander_swipe = use_signal(|| false);
    let edit_hint = use_one_time_hint(HINT_EDIT_DECK);
    let mut selected_format: Signal<Option<Format>> = use_signal(|| None);
    let mut selected_tags: Signal<Vec<String>> = use_signal(Vec::new);
    // Server-driven deck-tag catalog for the picker: read from the app-wide
    // catalog cache (warmed after session), empty until loaded.
    let cache: CatalogCache = use_context();
    use_effect(move || {
        if let Some(s) = session() {
            cache.ensure_deck_tags(client, s);
        }
    });
    let deck_tags_catalog = use_memo(move || -> Option<Vec<DeckTagView>> {
        cache.deck_tags.cell().read().loaded().cloned()
    });
    let mut partner_commander: Signal<Option<Card>> = use_signal(|| None);
    let mut partner_commander_display = use_signal(String::new);
    let mut background: Signal<Option<Card>> = use_signal(|| None);
    let mut background_display = use_signal(String::new);
    let mut signature_spell: Signal<Option<Card>> = use_signal(|| None);
    let mut signature_spell_display = use_signal(String::new);
    let mut show_partner_swipe = use_signal(|| false);
    let mut show_background_swipe = use_signal(|| false);
    let mut show_signature_spell_swipe = use_signal(|| false);
    let mut show_tags_select = use_signal(|| false);
    let mut show_format_select = use_signal(|| false);
    let mut land_target = use_signal(|| None::<i32>);
    let mut price_target = use_signal(String::new);
    let mut price_target_currency = use_signal(|| PriceCurrency::Usd);
    let mut power_level: Signal<Option<PowerLevel>> = use_signal(|| None);
    let mut other_tags: Signal<Vec<DeckOtherTag>> = use_signal(Vec::new);
    let mut oracle_tags: Signal<Vec<String>> = use_signal(Vec::new);
    let mut show_oracle_tags_select = use_signal(|| false);
    // Oracle tags currently contributed by deck-tag seeding. Reconciled when the
    // deck-tag picker closes (drop old seed, add new, keep manual picks). Init to
    // the loaded tags' seed so editing an existing deck reconciles cleanly.
    let mut applied_seed: Signal<Vec<String>> = use_signal(Vec::new);

    // Snapshot the format + its command-zone cascade when the format picker opens,
    // so Cancel can revert (format edits clear commander/spell live).
    let mut fmt_snapshot: Signal<FormatSnapshot> =
        use_signal(|| (None, None, String::new(), None, String::new()));
    use_effect(move || {
        if show_format_select() {
            fmt_snapshot.set((
                *selected_format.peek(),
                commander.peek().clone(),
                commander_display.peek().clone(),
                signature_spell.peek().clone(),
                signature_spell_display.peek().clone(),
            ));
        }
    });

    // Reactive Zwipe-select modes: derived from the current format / commander.
    let commander_mode = use_memo(move || selected_format().map(SwipeMode::Commander));
    let partner_mode = use_memo(|| Some(SwipeMode::Partner));
    let background_mode = use_memo(|| Some(SwipeMode::Background));
    let spell_mode = use_memo(move || {
        commander().map(|c| SwipeMode::SignatureSpell(c.scryfall_data.color_identity))
    });

    // original values (for change detection)
    let mut original_deck_name: Signal<String> = use_signal(String::new);
    let mut original_commander: Signal<Option<Card>> = use_signal(|| None);
    let mut original_format: Signal<Option<Format>> = use_signal(|| None);
    let mut original_tags: Signal<Vec<String>> = use_signal(Vec::new);
    let mut original_land_target: Signal<Option<i32>> = use_signal(|| None);
    let mut original_price_target: Signal<String> = use_signal(String::new);
    let mut original_price_target_currency: Signal<PriceCurrency> =
        use_signal(|| PriceCurrency::Usd);
    let mut original_power_level: Signal<Option<PowerLevel>> = use_signal(|| None);
    let mut original_other_tags: Signal<Vec<DeckOtherTag>> = use_signal(Vec::new);
    let mut original_oracle_tags: Signal<Vec<String>> = use_signal(Vec::new);
    let mut original_partner: Signal<Option<Card>> = use_signal(|| None);
    let mut original_background: Signal<Option<Card>> = use_signal(|| None);
    let mut original_signature_spell: Signal<Option<Card>> = use_signal(|| None);

    let toast = use_toast();
    let usage_buffer: Signal<UsageBuffer> = use_context();
    let authed = use_authed(Screen::Deck(DeckScreen::Edit));

    // Fetch deck profile
    let original_deck_resource: Resource<Result<Deck, ClientError>> =
        use_resource(move || async move {
            // try_run keeps the Result the watcher matches on; reporting and
            // the error toast live in the facade.
            authed
                .try_run("load_deck", |c, s| async move {
                    c.fetch_deck(deck_id, &s).await
                })
                .await
        });
    use_effect(move || {
        if let Some(Ok(deck)) = original_deck_resource() {
            original_deck_name.set(deck.deck_profile.name.to_string());
            deck_name.set(deck.deck_profile.name.to_string());
            original_format.set(deck.deck_profile.format);
            selected_format.set(deck.deck_profile.format);
            original_tags.set(deck.deck_profile.tags.clone());
            selected_tags.set(deck.deck_profile.tags.clone());
            original_land_target.set(deck.deck_profile.land_target);
            land_target.set(deck.deck_profile.land_target);
            let pt = deck
                .deck_profile
                .price_target
                .map(|v| format!("{v}"))
                .unwrap_or_default();
            original_price_target.set(pt.clone());
            price_target.set(pt);
            let ptc = deck
                .deck_profile
                .price_target_currency
                .unwrap_or(PriceCurrency::Usd);
            original_price_target_currency.set(ptc);
            price_target_currency.set(ptc);
            original_power_level.set(deck.deck_profile.power_level);
            power_level.set(deck.deck_profile.power_level);
            original_other_tags.set(deck.deck_profile.other_tags.clone());
            other_tags.set(deck.deck_profile.other_tags);
            original_oracle_tags.set(deck.deck_profile.oracle_tags.clone());
            oracle_tags.set(deck.deck_profile.oracle_tags.clone());
            // Seed contribution the saved deck tags account for, so the next
            // deck-tag reconcile drops exactly those and keeps manual picks.
            applied_seed.set(deck::seed_oracle_tags(&deck.deck_profile.tags));
        }
    });

    // Fetch commander card
    let original_commander_resource: Resource<Result<Option<Card>, ClientError>> =
        use_resource(move || async move {
            let Some(Ok(Deck {
                deck_profile:
                    DeckProfile {
                        commander_id: Some(original_commander_id),
                        ..
                    },
                ..
            })) = original_deck_resource()
            else {
                return Ok(None);
            };
            client().fetch_card(original_commander_id).await.map(Some)
        });
    use_effect(move || match original_commander_resource() {
        Some(Ok(Some(original))) => {
            original_commander.set(Some(original.clone()));
            commander.set(Some(original.clone()));
            commander_display.set(original.scryfall_data.name);
        }
        Some(Ok(None)) | None => (),
        Some(Err(e)) => {
            usage_buffer.peek().report_error(
                screen::DECK_EDIT,
                component::NONE,
                "load_commander",
                &e,
            );
            toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(TOAST_NORMAL),
            );
        }
    });

    // Fetch partner commander card
    let original_partner_resource: Resource<Result<Option<Card>, ClientError>> =
        use_resource(move || async move {
            let Some(Ok(Deck {
                deck_profile:
                    DeckProfile {
                        partner_commander_id: Some(id),
                        ..
                    },
                ..
            })) = original_deck_resource()
            else {
                return Ok(None);
            };
            client().fetch_card(id).await.map(Some)
        });
    use_effect(move || match original_partner_resource() {
        Some(Ok(Some(original))) => {
            original_partner.set(Some(original.clone()));
            partner_commander.set(Some(original.clone()));
            partner_commander_display.set(original.scryfall_data.name);
        }
        Some(Ok(None)) | None => (),
        Some(Err(e)) => {
            usage_buffer.peek().report_error(
                screen::DECK_EDIT,
                component::NONE,
                "load_partner",
                &e,
            );
            toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(TOAST_NORMAL),
            );
        }
    });

    // Fetch background card
    let original_background_resource: Resource<Result<Option<Card>, ClientError>> =
        use_resource(move || async move {
            let Some(Ok(Deck {
                deck_profile:
                    DeckProfile {
                        background_id: Some(id),
                        ..
                    },
                ..
            })) = original_deck_resource()
            else {
                return Ok(None);
            };
            client().fetch_card(id).await.map(Some)
        });
    use_effect(move || match original_background_resource() {
        Some(Ok(Some(original))) => {
            original_background.set(Some(original.clone()));
            background.set(Some(original.clone()));
            background_display.set(original.scryfall_data.name);
        }
        Some(Ok(None)) | None => (),
        Some(Err(e)) => {
            usage_buffer.peek().report_error(
                screen::DECK_EDIT,
                component::NONE,
                "load_background",
                &e,
            );
            toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(TOAST_NORMAL),
            );
        }
    });

    // Fetch signature spell card
    let original_spell_resource: Resource<Result<Option<Card>, ClientError>> =
        use_resource(move || async move {
            let Some(Ok(Deck {
                deck_profile:
                    DeckProfile {
                        signature_spell_id: Some(id),
                        ..
                    },
                ..
            })) = original_deck_resource()
            else {
                return Ok(None);
            };
            client().fetch_card(id).await.map(Some)
        });
    use_effect(move || match original_spell_resource() {
        Some(Ok(Some(original))) => {
            original_signature_spell.set(Some(original.clone()));
            signature_spell.set(Some(original.clone()));
            signature_spell_display.set(original.scryfall_data.name);
        }
        Some(Ok(None)) | None => (),
        Some(Err(e)) => {
            usage_buffer.peek().report_error(
                screen::DECK_EDIT,
                component::NONE,
                "load_signature_spell",
                &e,
            );
            toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(TOAST_NORMAL),
            );
        }
    });

    // Change tracking
    let deck_name_update = use_memo(move || {
        if deck_name() == original_deck_name() {
            None
        } else {
            Some(deck_name())
        }
    });
    let commander_id_update = use_memo(move || {
        if commander() == original_commander() {
            Patch::Unchanged
        } else {
            Patch::Set(commander().map(|c| c.scryfall_data.id))
        }
    });
    let partner_id_update = use_memo(move || {
        if partner_commander() == original_partner() {
            Patch::Unchanged
        } else {
            Patch::Set(partner_commander().map(|c| c.scryfall_data.id))
        }
    });
    let background_id_update = use_memo(move || {
        if background() == original_background() {
            Patch::Unchanged
        } else {
            Patch::Set(background().map(|c| c.scryfall_data.id))
        }
    });
    let signature_spell_id_update = use_memo(move || {
        if signature_spell() == original_signature_spell() {
            Patch::Unchanged
        } else {
            Patch::Set(signature_spell().map(|c| c.scryfall_data.id))
        }
    });
    let format_update = use_memo(move || {
        if selected_format() == original_format() {
            Patch::Unchanged
        } else {
            Patch::Set(selected_format().map(|f| f.to_legality_key().to_string()))
        }
    });
    let tags_update = use_memo(move || {
        if selected_tags() == original_tags() {
            Patch::Unchanged
        } else {
            Patch::Set(Some(selected_tags()))
        }
    });
    let land_target_update = use_memo(move || {
        if land_target() == original_land_target() {
            Patch::Unchanged
        } else {
            // `None` here clears the override back to the heuristic.
            Patch::Set(land_target())
        }
    });
    let price_target_update = use_memo(move || {
        let cur: Option<f64> = price_target().parse().ok().filter(|v: &f64| *v > 0.0);
        let orig: Option<f64> = original_price_target()
            .parse()
            .ok()
            .filter(|v: &f64| *v > 0.0);
        if cur == orig {
            Patch::Unchanged
        } else {
            // Empty/zero parses to `None` → clears the budget.
            Patch::Set(cur)
        }
    });
    let price_currency_update = use_memo(move || {
        if price_target_currency() == original_price_target_currency() {
            Patch::Unchanged
        } else {
            Patch::Set(Some(price_target_currency()))
        }
    });
    let power_level_update = use_memo(move || {
        if power_level() == original_power_level() {
            Patch::Unchanged
        } else {
            Patch::Set(power_level().map(|p| p.to_string()))
        }
    });
    let other_tags_update = use_memo(move || {
        if other_tags() == original_other_tags() {
            Patch::Unchanged
        } else {
            Patch::Set(Some(
                other_tags()
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            ))
        }
    });
    let oracle_tags_update = use_memo(move || {
        if oracle_tags() == original_oracle_tags() {
            Patch::Unchanged
        } else {
            Patch::Set(Some(oracle_tags()))
        }
    });
    let has_made_changes = use_memo(move || {
        deck_name_update().is_some()
            || commander_id_update().is_changed()
            || partner_id_update().is_changed()
            || background_id_update().is_changed()
            || signature_spell_id_update().is_changed()
            || format_update().is_changed()
            || tags_update().is_changed()
            || land_target_update().is_changed()
            || price_target_update().is_changed()
            || price_currency_update().is_changed()
            || power_level_update().is_changed()
            || other_tags_update().is_changed()
            || oracle_tags_update().is_changed()
    });

    // save state
    let mut is_saving = use_signal(|| false);

    let mut do_submit = move || {
        if let Err(e) = DeckName::new(deck_name()) {
            deck_name_error.set(Some(e.to_string()));
            return;
        }
        is_saving.set(true);

        spawn(async move {
            if !has_made_changes() {
                toast.error(
                    InvalidUpdateDeckProfile::NoUpdates.to_string(),
                    ToastOptions::default().duration(TOAST_NORMAL),
                );
                is_saving.set(false);
                return;
            }

            let request = HttpUpdateDeckProfile::builder()
                .name(deck_name_update().as_deref())
                .commander_id(commander_id_update())
                .partner_commander_id(partner_id_update())
                .background_id(background_id_update())
                .signature_spell_id(signature_spell_id_update())
                .format(format_update())
                .tags(tags_update())
                .land_target(land_target_update())
                .price_target(price_target_update())
                .price_target_currency(price_currency_update())
                .power_level(power_level_update())
                .other_tags(other_tags_update())
                .oracle_tags(oracle_tags_update())
                .build();

            match authed
                .try_run("save_profile", |c, s| async move {
                    c.update_deck_profile(deck_id, &request, &s).await
                })
                .await
            {
                Ok(_updated) => {
                    is_saving.set(false);
                    navigator.push(Router::ViewDeck { deck_id });
                }
                Err(_) => {
                    is_saving.set(false);
                }
            }
        });
    };

    let mut attempt_submit = move || {
        do_submit();
    };

    rsx! {
            div { class: "screen",
                ScreenHeader { title: "Edit deck", hint: edit_hint }

                div { class: "screen-content content-enter",
                div { class : "container-sm",
                    match &*original_deck_resource.read() {
                        Some(Ok(_deck)) => rsx! {
                            form { class: "flex-col text-center",
                                DeckFields {
                                    deck_name,
                                    deck_name_error,
                                    selected_format,
                                    selected_tags,
                                    commander,
                                    commander_display,
                                    partner_commander,
                                    partner_commander_display,
                                    background,
                                    background_display,
                                    signature_spell,
                                    signature_spell_display,
                                    show_commander_swipe,
                                    show_partner_swipe,
                                    show_background_swipe,
                                    show_signature_spell_swipe,
                                    show_tags_select,
                                    show_format_select,
                                    land_target,
                                    price_target,
                                    price_target_currency,
                                    power_level,
                                    other_tags,
                                    oracle_tags,
                                    show_oracle_tags_select,
                                }

                            }

                        },
                        Some(Err(_)) => rsx! { p { class: "text-muted", "Could not load deck" } },
                        None => rsx! { EditDeckSkeleton {} }
                    }
                }
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Action,
                    disabled: is_saving(),
                    onclick: move |_| {
                        navigator.push(Router::ViewDeck { deck_id });
                    },
                    "Back"
                }
                if has_made_changes() {
                    Button {
                        variant: ButtonVariant::Action,
                        disabled: is_saving(),
                        onclick : move |_| attempt_submit(),
                            if is_saving() { "Saving..." } else { "Save changes" }
                    }
                }
            }

            }
            SwipeSelect {
            host_screen: screen::DECK_EDIT,
                open: show_commander_swipe,
                mode: commander_mode,
                on_select: move |card: Card| {
                    commander_display.set(card.scryfall_data.name.clone());
                    deck_fields::autofill_named_partner(
                        &card,
                        client,
                        session,
                        commander,
                        partner_commander,
                        partner_commander_display,
                        toast,
                    );
                    commander.set(Some(card));
                    show_commander_swipe.set(false);
                },
                on_close: move |()| show_commander_swipe.set(false),
            }
            SwipeSelect {
            host_screen: screen::DECK_EDIT,
                open: show_partner_swipe,
                mode: partner_mode,
                on_select: move |card: Card| {
                    partner_commander_display.set(card.scryfall_data.name.clone());
                    partner_commander.set(Some(card));
                    show_partner_swipe.set(false);
                },
                on_close: move |()| show_partner_swipe.set(false),
            }
            SwipeSelect {
            host_screen: screen::DECK_EDIT,
                open: show_background_swipe,
                mode: background_mode,
                on_select: move |card: Card| {
                    background_display.set(card.scryfall_data.name.clone());
                    background.set(Some(card));
                    show_background_swipe.set(false);
                },
                on_close: move |()| show_background_swipe.set(false),
            }
            SwipeSelect {
            host_screen: screen::DECK_EDIT,
                open: show_signature_spell_swipe,
                mode: spell_mode,
                on_select: move |card: Card| {
                    signature_spell_display.set(card.scryfall_data.name.clone());
                    signature_spell.set(Some(card));
                    show_signature_spell_swipe.set(false);
                },
                on_close: move |()| show_signature_spell_swipe.set(false),
            }

            TagSelect {
                open: show_tags_select,
                selected_tags,
                catalog: deck_tags_catalog().unwrap_or_default(),
                on_close: move |()| {
                    show_tags_select.set(false);
                    // Reconcile seeded oracle tags with the final deck-tag set:
                    // drop the previous seed contribution, add the new one, keep
                    // manual picks. Cap at MAX; toast if anything was auto-added.
                    // Seed from the server catalog so a newly-served tag's otags
                    // apply without a client release; empty catalog seeds nothing.
                    let new_seed = deck::seed_oracle_tags_from_catalog(
                        &selected_tags(),
                        &deck_tags_catalog().unwrap_or_default(),
                    );
                    let old_seed = applied_seed();
                    let mut ot: Vec<String> = oracle_tags()
                        .into_iter()
                        .filter(|s| !old_seed.contains(s))
                        .collect();
                    let mut added = false;
                    for slug in &new_seed {
                        if !ot.contains(slug) && ot.len() < MAX_DECK_ORACLE_TAGS {
                            ot.push(slug.clone());
                            added = true;
                        }
                    }
                    oracle_tags.set(ot);
                    applied_seed.set(new_seed);
                    if added {
                        toast.info(
                            "Oracle tags derived from your deck tags".to_string(),
                            ToastOptions::default().duration(TOAST_NORMAL),
                        );
                    }
                },
            }

            OracleTagSelect {
                open: show_oracle_tags_select,
                selected: oracle_tags,
                on_close: move |()| show_oracle_tags_select.set(false),
            }

            FormatSelect {
                open: show_format_select,
                selected_format,
                on_select: move |fmt: Format| {
                    selected_format.set(Some(fmt));
                    commander.set(None);
                    commander_display.set(String::new());
                    if !fmt.has_signature_spell() {
                        signature_spell.set(None);
                        signature_spell_display.set(String::new());
                    }
                },
                on_clear: move |()| {
                    selected_format.set(None);
                    commander.set(None);
                    commander_display.set(String::new());
                    signature_spell.set(None);
                    signature_spell_display.set(String::new());
                },
                on_close: move |()| show_format_select.set(false),
                on_cancel: move |()| {
                    let (f, c, cd, ss, ssd) = fmt_snapshot();
                    selected_format.set(f);
                    commander.set(c);
                    commander_display.set(cd);
                    signature_spell.set(ss);
                    signature_spell_display.set(ssd);
                    show_format_select.set(false);
                },
            }

            DeckFieldsHint { open: edit_hint }
    }
}
