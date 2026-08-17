//! Deck list screen.
//!
//! The list carries the deck cards screen's chip-row grammar: a single-select
//! "Group by:" row that folds the list into `card-group-header` sections
//! (format, color identity, or tag — a deck with several tags appears under
//! each), and a "Show:" filter row seeded from the decks themselves (color
//! pips, deck tags, descriptive tags) with All as the reset. Both are
//! ephemeral per visit — with the 20-deck cap there is no state worth
//! persisting.

use crate::{
    inbound::{
        components::{
            auth::ensure_session::EnsureFresh,
            bottom_sheet::BottomSheet,
            hint_dialog::{HintBullet, HintBullets, HintDialog, HintKey, open_and_record_hint},
            screen_header::ScreenHeader,
            telemetry::{
                usage_buffer::UsageBuffer,
                vocabulary::{component, screen},
            },
        },
        router::Router,
        screens::deck::components::skeletons::DeckListSkeleton,
    },
    outbound::client::{
        ClientError, ZwipeClient, deck::get_deck_profiles::ClientGetDeckList,
        user::get_user::ClientGetUser,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::{collections::HashSet, time::Duration};
use zwipe_components::{ActionBar, Button, ButtonVariant, Chip};
use zwipe_core::domain::{
    auth::models::session::Session,
    card::scryfall_data::colors::Color,
    deck::{deck_profile::DeckProfile, deck_tag_label},
    user::models::hints::HINT_DECK_LIST,
};

/// Grouping dimensions for the deck list.
#[derive(Clone, Copy, Debug, PartialEq)]
enum DeckGroupBy {
    /// Flat alphabetical list (the pre-grouping behavior).
    None,
    /// Sections per format; format-less decks land in "No format".
    Format,
    /// Sections per exact color identity, colorless last.
    Color,
    /// Sections per tag (deck + descriptive); a deck appears under each of
    /// its tags, tagless decks land in "Untagged".
    Tag,
}

impl DeckGroupBy {
    fn all() -> Vec<Self> {
        vec![Self::None, Self::Format, Self::Color, Self::Tag]
    }
}

impl std::fmt::Display for DeckGroupBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::None => "None",
            Self::Format => "Format",
            Self::Color => "Color",
            Self::Tag => "Tag",
        };
        write!(f, "{label}")
    }
}

/// The deck's identity as ordered [`Color`]s (WUBRG order, unknowns dropped).
fn identity_colors(profile: &DeckProfile) -> Vec<Color> {
    let present: HashSet<Color> = profile
        .color_identity
        .iter()
        .filter_map(|s| Color::try_from(s.as_str()).ok())
        .collect();
    Color::all()
        .into_iter()
        .filter(|c| present.contains(c))
        .collect()
}

/// The deck's command-zone art crops, in zone order: commander (or
/// oathbreaker), then partner, background, and signature spell. Server-resolved
/// on the profile, so the row draws its command zone without fetching cards.
/// Realistically one or two — no printed card fills more than two slots.
fn command_zone_art(profile: &DeckProfile) -> Vec<&str> {
    [
        profile.commander_art_url.as_deref(),
        profile.partner_commander_art_url.as_deref(),
        profile.background_art_url.as_deref(),
        profile.signature_spell_art_url.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Every tag on the deck as (key, label): deck tags keyed by raw slug,
/// descriptive tags keyed by display name. One namespace — the Show row and
/// tag grouping treat them alike.
fn tag_labels(profile: &DeckProfile) -> Vec<(String, String)> {
    profile
        .tags
        .iter()
        .map(|t| (t.clone(), deck_tag_label(t)))
        .chain(
            profile
                .other_tags
                .iter()
                .map(|t| (t.display_name().to_string(), t.display_name().to_string())),
        )
        .collect()
}

/// True when the deck passes the Show row: every selected color is in its
/// identity (build toward a combo), and, if any tags are selected, it carries
/// at least one of them. Empty selections mean All.
fn matches_filters(profile: &DeckProfile, colors: &HashSet<Color>, tags: &HashSet<String>) -> bool {
    let identity: HashSet<Color> = identity_colors(profile).into_iter().collect();
    if !colors.iter().all(|c| identity.contains(c)) {
        return false;
    }
    if !tags.is_empty() {
        let deck_tags: HashSet<String> = tag_labels(profile).into_iter().map(|(k, _)| k).collect();
        if deck_tags.is_disjoint(tags) {
            return false;
        }
    }
    true
}

/// One rendered section: header text, header pips (color grouping renders
/// mana icons instead of text), and the decks inside.
struct DeckGroup {
    header: String,
    pips: Option<Vec<Color>>,
    decks: Vec<DeckProfile>,
}

impl DeckGroup {
    /// Stable identity for the collapse set. Color groups render as pips with
    /// no header text, so they key off the pip sequence instead — otherwise
    /// every color group would share the empty string and fold as one.
    fn key(&self) -> String {
        if !self.header.is_empty() {
            return self.header.clone();
        }
        self.pips
            .as_ref()
            .map(|pips| {
                pips.iter()
                    .map(Color::to_short_name)
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default()
    }
}

/// One collapsible section of the deck list: the card list's group grammar
/// (tap the header to fold, arrow rotates, rows stay mounted and hide via
/// CSS). Ephemeral per visit, like the deck cards screen.
#[component]
fn DeckGroupSection(
    group_key: String,
    header: String,
    pips: Option<Vec<Color>>,
    count: usize,
    collapsed_groups: Signal<HashSet<String>>,
    children: Element,
) -> Element {
    let mut collapsed_groups = collapsed_groups;
    let expanded = !collapsed_groups().contains(&group_key);
    rsx! {
        div {
            class: if expanded { "card-group row-enter" } else { "card-group row-enter collapsed" },
            div {
                class: if expanded { "card-group-header group-collapsible expanded" } else { "card-group-header group-collapsible" },
                onclick: move |_| {
                    let mut set = collapsed_groups.write();
                    if !set.remove(&group_key) {
                        set.insert(group_key.clone());
                    }
                },
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

/// Folds an (already filtered, name-sorted) list into sections for the chosen
/// dimension. `DeckGroupBy::None` returns one headerless group.
fn group_decks(profiles: &[DeckProfile], by: DeckGroupBy) -> Vec<DeckGroup> {
    match by {
        // Ungrouped is still one group, headed "All", so the list looks the
        // same whether or not Group by is on — one container, one collapsible
        // header.
        DeckGroupBy::None => vec![DeckGroup {
            header: "All".to_string(),
            pips: None,
            decks: profiles.to_vec(),
        }],
        DeckGroupBy::Format => {
            let mut groups: Vec<DeckGroup> = Vec::new();
            for profile in profiles {
                let header = profile
                    .format
                    .as_ref()
                    .map(|f| f.display_name().to_string())
                    .unwrap_or_else(|| "No format".to_string());
                match groups.iter_mut().find(|g| g.header == header) {
                    Some(group) => group.decks.push(profile.clone()),
                    None => groups.push(DeckGroup {
                        header,
                        pips: None,
                        decks: vec![profile.clone()],
                    }),
                }
            }
            // Alphabetical, format-less decks last.
            groups.sort_by_key(|g| (g.header == "No format", g.header.clone()));
            groups
        }
        DeckGroupBy::Color => {
            let mut groups: Vec<DeckGroup> = Vec::new();
            for profile in profiles {
                let pips = identity_colors(profile);
                let header = if pips.is_empty() {
                    "Colorless".to_string()
                } else {
                    String::new()
                };
                match groups
                    .iter_mut()
                    .find(|g| g.pips.as_deref() == Some(pips.as_slice()))
                {
                    Some(group) => group.decks.push(profile.clone()),
                    None => groups.push(DeckGroup {
                        header,
                        pips: Some(pips),
                        decks: vec![profile.clone()],
                    }),
                }
            }
            // Fewest colors first in WUBRG order, colorless last.
            groups.sort_by_key(|g| {
                let pips = g.pips.clone().unwrap_or_default();
                let order: Vec<usize> = pips
                    .iter()
                    .map(|c| Color::all().iter().position(|a| a == c).unwrap_or(0))
                    .collect();
                (pips.is_empty(), pips.len(), order)
            });
            groups
        }
        DeckGroupBy::Tag => {
            let mut groups: Vec<DeckGroup> = Vec::new();
            let mut untagged: Vec<DeckProfile> = Vec::new();
            for profile in profiles {
                let labels = tag_labels(profile);
                if labels.is_empty() {
                    untagged.push(profile.clone());
                    continue;
                }
                for (_, label) in labels {
                    match groups.iter_mut().find(|g| g.header == label) {
                        Some(group) => group.decks.push(profile.clone()),
                        None => groups.push(DeckGroup {
                            header: label,
                            pips: None,
                            decks: vec![profile.clone()],
                        }),
                    }
                }
            }
            groups.sort_by_key(|g| g.header.clone());
            if !untagged.is_empty() {
                groups.push(DeckGroup {
                    header: "Untagged".to_string(),
                    pips: None,
                    decks: untagged,
                });
            }
            groups
        }
    }
}

/// One tappable deck row: name, identity pips, and the stat chips.
#[component]
fn DeckRow(profile: DeckProfile) -> Element {
    let navigator = use_navigator();
    let mut count = profile.card_count;
    if profile.format.as_ref().is_some_and(|f| f.has_commander()) && profile.commander_id.is_some()
    {
        count += 1;
    }
    if profile.partner_commander_id.is_some() {
        count += 1;
    }
    if profile.background_id.is_some() {
        count += 1;
    }
    if profile.signature_spell_id.is_some() {
        count += 1;
    }
    // Out of bounds only when the format defines a size rule the count breaks.
    let count_bad = profile.format.as_ref().is_some_and(|f| {
        f.min_cards().is_some_and(|m| count < m as i64)
            || f.max_cards().is_some_and(|m| count > m as i64)
    });
    let pips = identity_colors(&profile);
    let art = command_zone_art(&profile);
    let deck_id = profile.id;
    rsx! {
        div {
            class: "card-row deck-row row-enter",
            onclick: move |_| {
                navigator.push(Router::ViewDeck { deck_id });
            },
            div { class: "deck-list-row",
                // The command zone leads the row inline, as flex items beside
                // the name — so the chips wrap onto the next line underneath
                // the art rather than into a column beside it. Two crops (a
                // pair of partners, or oathbreaker + signature spell) simply
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
                    class: if count_bad { "stat-chip stat-chip-bad" } else { "stat-chip" },
                    "{count} cards"
                }
                if let Some(ref fmt) = profile.format {
                    span { class: "stat-chip stat-chip-format", "{fmt.display_name()}" }
                }
                if let Some(pl) = profile.power_level {
                    span { class: "stat-chip stat-chip-power", "{pl.display_name()}" }
                }
                if let Some(ref cmd) = profile.commander_name {
                    span { class: "stat-chip stat-chip-zone", "{cmd}" }
                }
                if let Some(ref name) = profile.partner_commander_name {
                    span { class: "stat-chip stat-chip-zone", "{name}" }
                }
                if let Some(ref name) = profile.background_name {
                    span { class: "stat-chip stat-chip-zone", "{name}" }
                }
                if let Some(ref name) = profile.signature_spell_name {
                    span { class: "stat-chip stat-chip-zone", "{name}" }
                }
                for tag in profile.tags.iter() {
                    span { key: "{tag}", class: "stat-chip stat-chip-tag", "{deck_tag_label(tag)}" }
                }
                for tag in profile.other_tags.iter() {
                    span { key: "{tag}", class: "stat-chip stat-chip-other", "{tag.display_name()}" }
                }
            }
        }
    }
}

/// Screen displaying all user's decks with navigation to view/edit.
#[component]
pub fn DeckList() -> Element {
    let navigator = use_navigator();
    let auth_client: Signal<ZwipeClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();
    let toast = use_toast();
    let usage_buffer: Signal<UsageBuffer> = use_context();
    let decks_hint_open = use_signal(|| false);

    // Deck-list hint: fires once decks have loaded and only if any exist —
    // its main job is teaching the group/filter chip rows, which an empty
    // list hides, so it waits for a later visit instead of burning its one
    // showing (same gate as the deck cards screen's hint).
    let mut decks_hint_fired = use_signal(|| false);

    let mut group_by = use_signal(|| DeckGroupBy::None);
    // Collapsed group keys, ephemeral per visit — the deck cards screen's rule:
    // everything opens fresh next time.
    let collapsed_groups: Signal<HashSet<String>> = use_signal(HashSet::new);
    let mut selected_colors: Signal<HashSet<Color>> = use_signal(HashSet::new);
    let mut selected_tags: Signal<HashSet<String>> = use_signal(HashSet::new);
    let mut show_more_sheet = use_signal(|| false);

    // Refresh user on mount so email_verified_at is current without re-login.
    use_effect(move || {
        let Some(s) = session.peek().clone() else {
            return;
        };
        spawn(async move {
            match auth_client().get_user(&s).await {
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
    });

    let mut deck_profiles_resource: Resource<Result<Vec<DeckProfile>, ClientError>> =
        use_resource(move || async move {
            let session = session.ensure_fresh(auth_client).await?;

            auth_client().get_deck_profiles(&session).await
        });

    // Restart resource on component mount to ensure fresh data
    use_effect(move || {
        deck_profiles_resource.restart();
    });

    use_effect(move || {
        if let Some(Err(e)) = &*deck_profiles_resource.read() {
            usage_buffer
                .peek()
                .report_error(screen::DECK_LIST, component::NONE, "load_decks", &e);
            toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
        }
    });

    use_effect(move || {
        let has_decks = deck_profiles_resource
            .read()
            .as_ref()
            .and_then(|r| r.as_ref().ok())
            .is_some_and(|p| !p.is_empty());
        if has_decks && !*decks_hint_fired.peek() {
            decks_hint_fired.set(true);
            open_and_record_hint(HINT_DECK_LIST, session, auth_client, decks_hint_open);
        }
    });

    rsx! {
            div { class: "screen",
                ScreenHeader { title: "Decks", hint: decks_hint_open }

                // Auto-opens once per account (HINT_DECK_LIST) via the gated
                // effect above; the header's ? reopens it on demand.
                HintDialog {
                    open: decks_hint_open,
                    title: "Your decks",
                    HintBullets {
                        HintBullet { "Scroll through your decks and tap one to open it" }
                        HintBullet {
                            HintKey { color: "--color-success", "Group by" }
                            " folds the list into sections, and "
                            HintKey { color: "--accent-secondary", "Show" }
                            " narrows it by color or tag"
                        }
                        HintBullet {
                            "Tap "
                            HintKey { "Create" }
                            " to start a new deck"
                        }
                    }
                }

                div { class: "screen-content",
                div { class: "flex-col",
                    // Same wrapper as the deck cards and maybeboard screens,
                    // so the console starts at the same distance from the
                    // header on all three.
                    style: "max-width: 40rem; width: 100%; padding: 0 1rem;",

                    {
                        // Snapshot the resource once: Some(profiles) when loaded,
                        // None while loading (errors render their own message).
                        let profiles: Option<Vec<DeckProfile>> = deck_profiles_resource
                            .read()
                            .as_ref()
                            .and_then(|r| r.as_ref().ok())
                            .cloned();
                        let errored = matches!(&*deck_profiles_resource.read(), Some(Err(_)));
                        let loading = !errored && profiles.is_none();

                        // Static chrome renders real from the first frame, and the
                        // rule is closed vs open vocabularies: Group by's options
                        // and the five colors are closed sets, so they render
                        // permanently (a color with no matching decks just yields
                        // "No decks match" — never a selected chip that vanishes).
                        // Tags are open and deck-derived, so they ghost while
                        // loading; the deck tiles below stay skeleton.
                        let colors_shown: Vec<Color> = Color::all().to_vec();
                        let tags_present: Vec<(String, String)> = profiles
                            .as_ref()
                            .map(|deck_profiles| {
                                let mut seen = HashSet::new();
                                let mut tags: Vec<(String, String)> = deck_profiles
                                    .iter()
                                    .flat_map(|p| tag_labels(p))
                                    .filter(|(key, _)| seen.insert(key.clone()))
                                    .collect();
                                tags.sort_by_key(|(_, label)| label.clone());
                                tags
                            })
                            .unwrap_or_default();
                        let all_on = selected_colors().is_empty() && selected_tags().is_empty();
                        let filtered: Option<Vec<DeckProfile>> = profiles.as_ref().map(|deck_profiles| {
                            let mut kept: Vec<DeckProfile> = deck_profiles
                                .iter()
                                .filter(|p| matches_filters(p, &selected_colors(), &selected_tags()))
                                .cloned()
                                .collect();
                            kept.sort_by_key(|p| p.name.to_lowercase());
                            kept
                        });
                        // Hide the chrome only when there is nothing it could act
                        // on: a loaded-and-empty list, or a load error.
                        let show_rows = loading || profiles.as_ref().is_some_and(|p| !p.is_empty());
                        rsx! {
                            if show_rows {
                                div { class: "chip-row",
                                    span { class: "chip-row-label", "Group by:" }
                                    for option in DeckGroupBy::all() {
                                        Chip {
                                            key: "{option}",
                                            selected: group_by() == option,
                                            onclick: move |_| group_by.set(option),
                                            "{option}"
                                        }
                                    }
                                }
                                div { class: "chip-row",
                                    span { class: "chip-row-label", "Show:" }
                                    Chip {
                                        selected: all_on,
                                        onclick: move |_| {
                                            selected_colors.write().clear();
                                            selected_tags.write().clear();
                                        },
                                        "All"
                                    }
                                    for color in colors_shown {
                                        Chip {
                                            key: "{color.to_short_name()}",
                                            selected: selected_colors().contains(&color),
                                            onclick: move |_| {
                                                let mut colors = selected_colors.write();
                                                if !colors.remove(&color) {
                                                    colors.insert(color);
                                                }
                                            },
                                            i { class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost" }
                                        }
                                    }
                                    if loading {
                                        // Deck-derived tag chips ghost until the list lands.
                                        for (i, size) in ["md", "sm", "lg", "sm", "md", "sm", "lg", "md"].iter().enumerate() {
                                            div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-{size}" }
                                        }
                                    } else {
                                        for (key, label) in tags_present {
                                            Chip {
                                                key: "{key}",
                                                selected: selected_tags().contains(&key),
                                                onclick: move |_| {
                                                    let mut tags = selected_tags.write();
                                                    if !tags.remove(&key) {
                                                        tags.insert(key.clone());
                                                    }
                                                },
                                                "{label}"
                                            }
                                        }
                                    }
                                }
                            }
                            if errored {
                                div { class: "message-empty",
                                    p { "Could not load decks" }
                                }
                            } else if loading {
                                DeckListSkeleton {}
                            } else if profiles.as_ref().is_some_and(|p| p.is_empty()) {
                                div { class: "message-empty",
                                    p { "No decks" }
                                }
                            } else if let Some(filtered) = filtered {
                                if filtered.is_empty() {
                                    div { class: "message-empty",
                                        p { "No decks match" }
                                    }
                                }
                                // Every mode renders the same way: contained
                                // groups with collapsible headers, the same
                                // grammar as a deck's card list. Ungrouped is
                                // simply the single "All" group.
                                for group in group_decks(&filtered, group_by()) {
                                    DeckGroupSection {
                                        key: "{group.key()}",
                                        group_key: group.key(),
                                        header: group.header.clone(),
                                        pips: group.pips.clone(),
                                        count: group.decks.len(),
                                        collapsed_groups,
                                        for profile in group.decks {
                                            DeckRow { key: "{profile.id}", profile }
                                        }
                                    }
                                }
                            }
                        }
                    }

                }
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        navigator.push(Router::Home {});
                    },
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        // Proactive guard: unverified users are limited to 1 deck.
                        // The backend enforces this too, but we surface it here first.
                        let at_limit = session().is_some_and(|s| {
                            s.user.email_verified_at.is_none()
                                && deck_profiles_resource
                                    .read()
                                    .as_ref()
                                    .and_then(|r| r.as_ref().ok())
                                    .is_some_and(|p| !p.is_empty())
                        });
                        if at_limit {
                            toast.warning(
                                "Verify your email to create more than 1 deck".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(4000)),
                            );
                        } else {
                            navigator.push(Router::CreateDeck);
                        }
                    },
                    "Create"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| show_more_sheet.set(true),
                    "More"
                }
            }

            // Side features live here rather than crowding the action bar.
            BottomSheet { open: show_more_sheet, title: "More actions",
                Button {
                    onclick: move |_| {
                        show_more_sheet.set(false);
                        navigator.push(Router::CommanderMaybeboard {});
                    },
                    "Commander maybeboard"
                }
            }
            }
    }
}
