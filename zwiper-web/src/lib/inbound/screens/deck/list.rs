//! Decks: every deck the user has, with Group by, the Show row and its tags,
//! collapsible groups, and Create as the dock's `+` behind its gate.
//!
//! On a wide window the tag row stays unfolded and each group lays its rows
//! out two to a line with larger art. Below the breakpoint it is the app's
//! list with the `Tags` fold chip.

use std::collections::HashSet;

use components::icons::{DockIcon, PLUS};
use components::{Chip, TOAST_LONG};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::deck::DeckProfile;
use zwipe_core::domain::deck::deck_tag::deck_tag_label;
use zwipe_ui::deck::list::{self, GroupBy, Show};
use zwipe_ui::deck::{card_count, cards_label, command_zone, count_fits};

use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, open_and_record_hint};
use crate::inbound::router::Route;
use crate::inbound::screens::deck::skeletons::{ChipRowSkeleton, DeckListSkeleton};
use crate::inbound::shell::ScreenFrame;
use crate::outbound::session_store::SessionStore;

/// The deck list screen.
#[component]
pub fn DeckList() -> Element {
    let navigator = use_navigator();
    let client: Signal<ZwipeClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();
    let toast = use_toast();
    let authed = use_authed();
    let hint_open = use_signal(|| false);
    let mut hint_fired = use_signal(|| false);

    let mut group_by = use_signal(GroupBy::default);
    let collapsed: Signal<HashSet<String>> = use_signal(HashSet::new);
    let mut show = use_signal(Show::default);
    let mut tags_open = use_signal(|| false);

    // The user, so the Create gate reads a current verification state. A
    // failure only logs.
    use_hook(move || {
        let Some(current) = session.peek().clone() else {
            return;
        };
        spawn(async move {
            let http = client.peek().clone();
            match http.fetch_user(&current).await {
                Ok(fresh) => {
                    let latest = session.peek().clone();
                    if let Some(mut latest) = latest {
                        latest.user = fresh;
                        SessionStore::browser().infallible_save(&latest);
                        session.set(Some(latest));
                    }
                }
                Err(e) => tracing::warn!("deck list user fetch failed: {e}"),
            }
        });
    });

    let decks: Resource<Result<Vec<DeckProfile>, ClientError>> = use_resource(move || async move {
        authed
            .try_run("load_decks", |c, s| async move {
                c.fetch_deck_profiles(&s).await
            })
            .await
    });

    // The hint teaches the chip rows, which an empty list hides, so it waits
    // for a list with decks.
    use_effect(move || {
        let has_decks = decks
            .read()
            .as_ref()
            .and_then(|r| r.as_ref().ok())
            .is_some_and(|p| !p.is_empty());
        if has_decks && !*hint_fired.peek() {
            hint_fired.set(true);
            open_and_record_hint(OneTimeHint::DeckList, session, client, hint_open);
        }
    });

    let on_create = move |_| {
        let at_limit = session.peek().as_ref().is_some_and(|s| {
            s.user.email_verified_at.is_none()
                && decks
                    .peek()
                    .as_ref()
                    .and_then(|r| r.as_ref().ok())
                    .is_some_and(|p| !p.is_empty())
        });
        if at_limit {
            toast.warning(
                "Verify your email to create more than 1 deck".to_string(),
                ToastOptions::default().duration(TOAST_LONG),
            );
        } else {
            navigator.push(Route::CreateDeck {});
        }
    };

    let profiles: Option<Vec<DeckProfile>> =
        decks.read().as_ref().and_then(|r| r.as_ref().ok()).cloned();
    let errored = matches!(&*decks.read(), Some(Err(_)));
    let loading = !errored && profiles.is_none();
    let show_rows = profiles.as_ref().is_some_and(|p| !p.is_empty());
    let tags_present: Vec<(String, String)> =
        profiles.as_deref().map(list::all_tags).unwrap_or_default();
    let filter = show();
    let shown: Option<Vec<DeckProfile>> = profiles.as_deref().map(|p| list::shown(p, &filter));

    rsx! {
        ScreenFrame {
            title: "Decks",
            description: Some("Every deck you've built, grouped and filtered your way".to_string()),
            on_hint: move |()| {
                let mut hint_open = hint_open;
                hint_open.set(true);
            },
            div { class: "screen-content",
                div { class: "flex-col web-deck-list",
                    if loading {
                        ChipRowSkeleton { label: "md", chips: vec!["sm", "md", "sm", "sm"] }
                        ChipRowSkeleton { label: "sm", chips: vec!["sm", "sm", "sm", "sm", "sm", "sm", "md"] }
                    } else if show_rows {
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "Group by" }
                            for option in GroupBy::ALL {
                                Chip {
                                    key: "{option.label()}",
                                    selected: group_by() == option,
                                    onclick: move |_| group_by.set(option),
                                    "{option.label()}"
                                }
                            }
                        }
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "Show" }
                            Chip {
                                selected: filter.is_all(),
                                onclick: move |_| show.write().clear(),
                                "All"
                            }
                            for color in Color::all() {
                                Chip {
                                    key: "{color.to_short_name()}",
                                    selected: filter.colors.contains(&color),
                                    onclick: move |_| show.write().toggle_color(color),
                                    i { class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost" }
                                }
                            }
                            if !tags_present.is_empty() {
                                span { class: "web-deck-tags-fold",
                                    Chip {
                                        selected: tags_open(),
                                        onclick: move |_| tags_open.toggle(),
                                        "Tags"
                                        span {
                                            class: if tags_open() { "chip-fold-arrow expanded" } else { "chip-fold-arrow" },
                                            "\u{25B8}"
                                        }
                                    }
                                    if !tags_open() {
                                        for (key, label) in tags_present.iter().filter(|(key, _)| filter.tags.contains(key)).cloned() {
                                            Chip {
                                                key: "{key}",
                                                selected: true,
                                                onclick: move |_| show.write().toggle_tag(&key),
                                                "{label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if !tags_present.is_empty() {
                            div { class: if tags_open() { "collapsible open web-deck-tags" } else { "collapsible web-deck-tags" },
                                div { class: "collapsible-inner",
                                    div { class: "chip-row",
                                        for (key, label) in tags_present {
                                            Chip {
                                                key: "{key}",
                                                selected: filter.tags.contains(&key),
                                                onclick: move |_| show.write().toggle_tag(&key),
                                                "{label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if errored {
                        div { class: "message-empty",
                            span { class: "chip-note", "Could not load decks" }
                        }
                    } else if loading {
                        DeckListSkeleton {}
                    } else if !show_rows {
                        div { class: "message-empty",
                            span { class: "chip-note", "No decks" }
                        }
                    } else if let Some(shown) = shown {
                        if shown.is_empty() {
                            div { class: "message-empty",
                                span { class: "chip-note", "No decks match" }
                            }
                        }
                        for group in list::group(&shown, group_by()) {
                            DeckGroupSection {
                                key: "{group.key()}",
                                group_key: group.key(),
                                header: group.header.clone(),
                                pips: group.pips.clone(),
                                count: group.decks.len(),
                                collapsed,
                                for profile in group.decks {
                                    DeckRow { key: "{profile.id}", profile }
                                }
                            }
                        }
                    }
                }
            }
            // Create as the action dock's +, pinned bottom right.
            div { class: "action-dock action-dock-right",
                button {
                    class: "action-dock-btn",
                    "aria-label": "Create",
                    onclick: on_create,
                    DockIcon { path: PLUS }
                }
            }
        }
        OneTimeHintDialog { hint: OneTimeHint::DeckList, open: hint_open }
    }
}

/// One group of the list: a header that folds it, then its rows, which stay
/// mounted while folded.
#[component]
fn DeckGroupSection(
    group_key: String,
    header: String,
    pips: Option<Vec<Color>>,
    count: usize,
    mut collapsed: Signal<HashSet<String>>,
    children: Element,
) -> Element {
    let expanded = !collapsed().contains(&group_key);
    rsx! {
        div {
            class: if expanded { "card-group row-enter web-deck-group" } else { "card-group row-enter web-deck-group collapsed" },
            div {
                class: if expanded { "card-group-header group-collapsible expanded" } else { "card-group-header group-collapsible" },
                onclick: move |_| {
                    let mut set = collapsed.write();
                    if !set.remove(&group_key) {
                        set.insert(group_key.clone());
                    }
                },
                span { class: "card-row-arrow", "\u{25b8}" }
                if let Some(pips) = pips.as_ref().filter(|p| !p.is_empty()) {
                    span { class: "identity-pips",
                        for color in pips.iter() {
                            i {
                                key: "{color.to_short_name()}",
                                class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost",
                            }
                        }
                    }
                    "({count})"
                } else {
                    "{header} ({count})"
                }
            }
            div { class: if expanded { "collapsible open" } else { "collapsible" },
                div { class: "collapsible-inner", {children} }
            }
        }
    }
}

/// One deck: its command-zone art, name, pips and chips. A click opens it.
#[component]
fn DeckRow(profile: DeckProfile) -> Element {
    let navigator = use_navigator();
    let count = card_count(&profile);
    let count_bad = !count_fits(&profile, count);
    let pips = list::identity_colors(&profile);
    let art = command_zone::art(&profile).into_iter().map(str::to_owned);
    let deck_id = profile.id;
    rsx! {
        div {
            class: "card-row deck-row row-enter",
            onclick: move |_| {
                navigator.push(Route::ViewDeck { deck_id });
            },
            div { class: "deck-list-row",
                for url in art {
                    img {
                        key: "{url}",
                        class: "deck-row-art-img",
                        src: "{url}",
                        loading: "lazy",
                        draggable: false,
                        alt: "",
                    }
                }
                h3 { class: "font-light text-base tracking-wide deck-list-name",
                    "{profile.name}"
                }
                if !pips.is_empty() {
                    span { class: "identity-pips",
                        for color in pips {
                            i {
                                key: "{color.to_short_name()}",
                                class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost",
                            }
                        }
                    }
                }
                // "1 card" for one, "{count} cards" otherwise.
                span {
                    class: if count_bad { "stat-chip stat-chip-warning" } else { "stat-chip" },
                    {cards_label(count)}
                }
                if let Some(format) = profile.format {
                    span { class: "stat-chip stat-chip-p3", "{format.display_name()}" }
                }
                if let Some(power) = profile.power_level {
                    span { class: "stat-chip stat-chip-p5", "{power.display_name()}" }
                }
                for name in [
                    profile.commander_name.clone(),
                    profile.partner_commander_name.clone(),
                    profile.background_name.clone(),
                    profile.signature_spell_name.clone(),
                ]
                    .into_iter()
                    .flatten()
                {
                    span { class: "stat-chip stat-chip-p3", "{name}" }
                }
                for tag in profile.tags.iter() {
                    span { key: "{tag}", class: "stat-chip stat-chip-muted", "{deck_tag_label(tag)}" }
                }
                for tag in profile.other_tags.iter() {
                    span { key: "{tag}", class: "stat-chip stat-chip-p5", "{tag.display_name()}" }
                }
            }
        }
    }
}
