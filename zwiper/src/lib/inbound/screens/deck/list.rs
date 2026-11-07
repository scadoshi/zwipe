//! Deck list screen.
//!
//! The list carries the deck cards screen's chip-row grammar: a single-select
//! "Group by" row that folds the list into `card-group-header` sections
//! (format, color identity, or tag: a deck with several tags appears under
//! each), and a "Show" filter row seeded from the decks themselves (color
//! pips, deck tags, descriptive tags) with All as the reset. Both are
//! ephemeral per visit: with the 20-deck cap there is no state worth
//! persisting.
//!
//! What the screen knows and does is `zwipe_ui::screens::deck_list`; this
//! file draws its state and performs its effects.

use components::icons::{self};
use components::{Chip, DockAction};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use dioxus_primitives::toast::{Toasts, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::deck;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::deck::command_zone;
use zwipe_ui::deck::list::{self, GroupBy};
use zwipe_ui::screens::deck_list::{self, Action, Destination, Effect, State, Status};

use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::hint_dialog::OneTimeHintDialog;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::vocabulary::{DeckScreen, Screen};
use crate::inbound::components::{hint_dialog, ui_toast};
use crate::inbound::router::Router;
use crate::inbound::screens::deck::components::skeletons::{ChipRowSkeleton, DeckListSkeleton};

/// The deck list's model with the handles that perform its effects. Copy,
/// so every handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    navigator: Navigator,
    toast: Toasts,
    authed: Authed,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    hint_open: Signal<bool>,
}

impl Driver {
    /// Applies `action` to the model and performs what it returns.
    fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    /// Performs one effect, feeding any reply back through [`Self::dispatch`].
    fn perform(self, effect: Effect) {
        let Self {
            navigator,
            toast,
            authed,
            mut session,
            client,
            hint_open,
            ..
        } = self;
        match effect {
            Effect::RefreshUser => {
                let Some(s) = session.peek().clone() else {
                    return;
                };
                spawn(async move {
                    match client().fetch_user(&s).await {
                        Ok(fresh_user) => {
                            let current = session.peek().clone();
                            if let Some(mut current) = current {
                                current.user = fresh_user;
                                session.set(Some(current));
                            }
                        }
                        Err(e) => {
                            tracing::warn!("deck list user fetch failed: {e}");
                        }
                    }
                });
            }
            Effect::FetchDecks(ticket) => {
                spawn(async move {
                    // try_run reports a failure and raises its error toast.
                    let reply = authed
                        .try_run("load_decks", |c, s| async move {
                            c.fetch_deck_profiles(&s).await
                        })
                        .await;
                    self.dispatch(match reply {
                        Ok(decks) => Action::Loaded { ticket, decks },
                        Err(_) => Action::Failed { ticket },
                    });
                });
            }
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(hint.key(), session, client, hint_open);
            }
            Effect::Navigate(destination) => {
                navigator.push(route(destination));
            }
            Effect::Toast(warning) => ui_toast::raise(toast, warning),
        }
    }
}

/// The app's route for a place the deck list sends the user.
fn route(destination: Destination) -> Router {
    match destination {
        Destination::Home => Router::Home {},
        Destination::Deck(deck_id) => Router::ViewDeck { deck_id },
        Destination::CreateDeck => Router::CreateDeck,
    }
}

/// One collapsible section of the deck list: the card list's group grammar
/// (tap the header to fold, arrow rotates, rows stay mounted and hide via
/// CSS).
#[component]
fn DeckGroupSection(
    header: String,
    pips: Option<Vec<Color>>,
    count: usize,
    expanded: bool,
    ontoggle: EventHandler,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: if expanded { "card-group deck-group row-enter" } else { "card-group deck-group row-enter collapsed" },
            div {
                class: if expanded { "card-group-header group-collapsible expanded" } else { "card-group-header group-collapsible" },
                onclick: move |_| ontoggle.call(()),
                span { class: "card-row-arrow", "▸" }
                if let Some(pips) = pips.as_ref().filter(|p| !p.is_empty()) {
                    // Same wrapper the deck rows use, so a group's pips are the
                    // size of the pips on the decks beneath it.
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
            div {
                class: if expanded { "collapsible open" } else { "collapsible" },
                div { class: "collapsible-inner", {children} }
            }
        }
    }
}

/// One tappable deck row: name, identity pips, and the stat chips.
#[component]
fn DeckRow(profile: DeckProfile, onopen: EventHandler) -> Element {
    let count = zwipe_ui::deck::card_count(&profile);
    // Out of bounds only when the format defines a size rule the count breaks.
    let count_bad = !zwipe_ui::deck::count_fits(&profile, count);
    let pips = list::identity_colors(&profile);
    let art = command_zone::art(&profile);
    rsx! {
        div {
            class: "card-row deck-row row-enter",
            onclick: move |_| onopen.call(()),
            div { class: "deck-list-row",
                // The command zone's art leads the row beside the name. Two
                // crops (a pair of partners, or oathbreaker + signature spell)
                // sit one after another.
                for url in art.iter() {
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
                    {profile.name.to_string()}
                }
                // The pips, the count and the chips take the line under the name.
                div { class: "deck-row-chips",
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
                    span {
                        class: if count_bad { "stat-chip stat-chip-warning" } else { "stat-chip" },
                        {zwipe_ui::deck::cards_label(count)}
                    }
                    if let Some(ref fmt) = profile.format {
                        span { class: "stat-chip stat-chip-p3", "{fmt.display_name()}" }
                    }
                    if let Some(pl) = profile.power_level {
                        span { class: "stat-chip stat-chip-p5", "{pl.display_name()}" }
                    }
                    if let Some(ref cmd) = profile.commander_name {
                        span { class: "stat-chip stat-chip-p3", "{cmd}" }
                    }
                    if let Some(ref name) = profile.partner_commander_name {
                        span { class: "stat-chip stat-chip-p3", "{name}" }
                    }
                    if let Some(ref name) = profile.background_name {
                        span { class: "stat-chip stat-chip-p3", "{name}" }
                    }
                    if let Some(ref name) = profile.signature_spell_name {
                        span { class: "stat-chip stat-chip-p3", "{name}" }
                    }
                    for tag in profile.tags.iter() {
                        span { key: "{tag}", class: "stat-chip stat-chip-muted", "{deck::deck_tag_label(tag)}" }
                    }
                    for tag in profile.other_tags.iter() {
                        span { key: "{tag}", class: "stat-chip stat-chip-p5", "{tag.display_name()}" }
                    }
                }
            }
        }
    }
}

/// Screen displaying all user's decks with navigation to view/edit.
#[component]
pub fn DeckList() -> Element {
    let driver = Driver {
        model: use_signal(State::new),
        navigator: use_navigator(),
        toast: use_toast(),
        authed: use_authed(Screen::Deck(DeckScreen::List)),
        session: use_context(),
        client: use_context(),
        hint_open: use_signal(|| false),
    };
    let decks_hint_open = driver.hint_open;
    let session = driver.session;

    // Refreshes the user, so email_verified_at is current without re-login,
    // and fetches the decks, once per mount.
    use_effect(move || driver.dispatch(Action::Load));

    // Snapshot the model once per render.
    let state = driver.model.read();
    let loading = state.status() == Status::Loading;
    // Hide the console when there is nothing it could act on: a
    // loaded-and-empty list, or a load error.
    let show_rows = state.chips_shown();
    let group_by = state.group_by();
    let all_on = state.show().is_all();
    let picked_colors = state.show().colors.clone();
    let picked_tags = state.show().tags.clone();
    let tags_open = state.tags_open();
    let tags_present = state.tags();
    let folded_picks = state.folded_picks();
    let message = state.message();
    let mut groups: Vec<(list::DeckGroup, bool)> = Vec::new();
    for group in state.groups() {
        let expanded = !state.is_collapsed(&group.key());
        groups.push((group, expanded));
    }
    drop(state);

    rsx! {
            div { class: "screen",
                ScreenHeader { title: deck_list::TITLE, hint: decks_hint_open, back: move |()| driver.dispatch(Action::Back) }

                // Auto-opens once per account (HINT_DECK_LIST) when the model
                // offers it; the header's ? reopens it on demand.
                OneTimeHintDialog { open: decks_hint_open, hint: OneTimeHint::DeckList }

                div { class: "screen-content",
                div { class: "flex-col deck-list-stack",
                    // Same wrapper as the deck cards and maybeboard screens,
                    // so the console starts at the same distance from the
                    // header on all three.
                    style: "max-width: 40rem; width: 100%; padding: 0 1rem;",

                    // While the decks load the whole console ghosts with the
                    // list. Once loaded, the five colors always show (a color
                    // with no matching decks just yields "No decks match",
                    // never a selected chip that vanishes), and the
                    // deck-derived tags fold behind the Tags chip.
                    if loading {
                        // Group by, then Show: All, the five pips, Tags.
                        ChipRowSkeleton { label: "md", chips: vec!["sm", "md", "sm", "sm"] }
                        ChipRowSkeleton { label: "sm", chips: vec!["sm", "sm", "sm", "sm", "sm", "sm", "md"] }
                    } else if show_rows {
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", {deck_list::GROUP_BY} }
                            for option in GroupBy::ALL {
                                Chip {
                                    key: "{option.label()}",
                                    selected: group_by == option,
                                    onclick: move |_| driver.dispatch(Action::PickGroupBy(option)),
                                    "{option.label()}"
                                }
                            }
                        }
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", {deck_list::SHOW} }
                            Chip {
                                selected: all_on,
                                onclick: move |_| driver.dispatch(Action::ShowAll),
                                {deck_list::SHOW_ALL}
                            }
                            for color in Color::all() {
                                Chip {
                                    key: "{color.to_short_name()}",
                                    selected: picked_colors.contains(&color),
                                    onclick: move |_| driver.dispatch(Action::ToggleColor(color)),
                                    i { class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost" }
                                }
                            }
                            // The Tags fold chip shows only when some deck has a tag.
                            if !tags_present.is_empty() {
                                Chip {
                                    selected: tags_open,
                                    onclick: move |_| driver.dispatch(Action::ToggleTagFold),
                                    // The card rows' disclosure arrow at the left, as on every
                                    // fold: sideways folded, down open.
                                    span {
                                        class: if tags_open { "chip-fold-arrow expanded" } else { "chip-fold-arrow" },
                                        "\u{25B8}"
                                    }
                                    {deck_list::TAGS}
                                }
                                // The tags that are on stay in the row while folded.
                                if !tags_open {
                                    for (key, label) in folded_picks {
                                        Chip {
                                            key: "{key}",
                                            selected: true,
                                            onclick: move |_| driver.dispatch(Action::ToggleTag(key.clone())),
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                        // The full tag row folds open and closed under the Show row.
                        if !tags_present.is_empty() {
                            div { class: if tags_open { "collapsible open" } else { "collapsible" },
                                div { class: "collapsible-inner",
                                    div { class: "chip-row",
                                        for (key, label) in tags_present {
                                            Chip {
                                                key: "{key}",
                                                selected: picked_tags.contains(&key),
                                                onclick: move |_| driver.dispatch(Action::ToggleTag(key.clone())),
                                                "{label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if loading {
                        DeckListSkeleton {}
                    }
                    if let Some(message) = message {
                        div { class: "message-empty",
                            span { class: "chip-note", {message} }
                        }
                    }
                    // Every mode renders the same way: contained groups with
                    // collapsible headers, the same grammar as a deck's card
                    // list. Ungrouped is simply the single "All" group.
                    for (group, expanded) in groups {
                        DeckGroupSection {
                            key: "{group.key()}",
                            header: group.header.clone(),
                            pips: group.pips.clone(),
                            count: group.decks.len(),
                            expanded,
                            ontoggle: {
                                let group_key = group.key();
                                move |()| driver.dispatch(Action::ToggleGroup(group_key.clone()))
                            },
                            for profile in group.decks {
                                DeckRow {
                                    key: "{profile.id}",
                                    onopen: {
                                        let deck_id = profile.id;
                                        move |()| driver.dispatch(Action::OpenDeck(deck_id))
                                    },
                                    profile,
                                }
                            }
                        }
                    }
                }
            }

            // Create as the action dock's +, pinned bottom right.
            div { class: "action-dock action-dock-right",
                DockAction {
                    label: deck_list::CREATE,
                    icon: icons::PLUS,
                    onclick: move |()| {
                        // Unverified users are limited to 1 deck. The backend
                        // enforces this too, but the model surfaces it first.
                        // With no session only the server can refuse.
                        let verified = session
                            .peek()
                            .as_ref()
                            .is_none_or(|s| s.user.email_verified_at.is_some());
                        driver.dispatch(Action::Create { verified });
                    },
                }
            }
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures;

    #[test]
    fn every_destination_has_the_app_route() {
        let deck_id = test_fixtures::deck_profile().id;
        assert_eq!(route(Destination::Home), Router::Home {});
        assert_eq!(
            route(Destination::Deck(deck_id)),
            Router::ViewDeck { deck_id }
        );
        assert_eq!(route(Destination::CreateDeck), Router::CreateDeck);
    }
}
