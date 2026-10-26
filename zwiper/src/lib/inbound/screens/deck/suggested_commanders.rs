//! Suggested commanders screen: the top 10 commanders ranked against the
//! user's own decks, each with the reasons it ranked, styled like the
//! commander maybeboard.
//!
//! An expanded row offers **Create deck** (seeds the create screen), **Maybeboard**
//! (saves to the commander maybeboard) and **Remove** (hides it for a week). The action bar's Swipe opens the shared overlay in its suggested
//! mode, which deals the same ranking down to 100. Every action that removes
//! a row refetches when fewer than ten remain, so the list stays a full top 10.
//! Over the list, a panel sums up what the user builds: the decks and their
//! most-built identity, the colors sized by deck count, and the strongest
//! oracle tags, the profile the ranking reads. With no commander deck to read
//! from, the screen shows a single line instead of a list and no panel.

use std::collections::BTreeMap;

use components::{
    ActionBar, AlertDialog, BottomSheet, Button, ButtonVariant, CardRoleChips, Chip, DialogAction,
    HintBullet, HintBullets, HintDialog, HintKey, HintLine, TOAST_LONG, TOAST_QUICK,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::domain::user::models::hints::HINT_SUGGESTED_COMMANDERS;
use zwipe_core::http::contracts::suggestions::{
    HttpBuilderProfile, HttpColorAffinity, HttpColorIdentityCount, HttpCommanderSuggestion,
    HttpProfileTag, HttpSharedTag, HttpSuggestionReasons,
};

use super::components::swipe_select::{SwipeMode, SwipeSelect};
use crate::inbound::components::auth::authed::use_authed;
use crate::inbound::components::catalog_cache::CatalogCache;
use crate::inbound::components::hint_dialog::{HintColored, use_one_time_hint};
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::vocabulary::{Screen, screen};
use crate::inbound::router::Router;
use crate::inbound::screens::deck::card::components::card_row::{
    CardRow, OtagDescribe, OtagExamplesOpen, ShowRowArt,
};
use crate::inbound::screens::deck::card::components::image_preview::ImagePreview;
use crate::inbound::screens::deck::card::components::printing_sheet::PrintingSheet;
use crate::inbound::screens::deck::create::CreateDeckCommanderSeed;
use crate::inbound::screens::oracle_tag_examples::OracleTagExamples;

/// Suggestions the list shows; a shorter list after a dismissal refetches.
const LIST_SIZE: usize = 10;

/// One line per color the user builds in, most decks first, in the order the
/// server sent. A color code outside WUBRG is skipped.
pub(crate) fn color_lines(colors: &[HttpColorAffinity], deck_count: u32) -> Vec<String> {
    colors
        .iter()
        .filter_map(|c| {
            let name = color_name(&c.color)?;
            Some(format!(
                "You build in {name} ({} of your {deck_count} {})",
                c.decks,
                if deck_count == 1 { "deck" } else { "decks" }
            ))
        })
        .collect()
}

fn color_name(code: &str) -> Option<&'static str> {
    match code {
        "W" => Some("white"),
        "U" => Some("blue"),
        "B" => Some("black"),
        "R" => Some("red"),
        "G" => Some("green"),
        _ => None,
    }
}

/// The shared tags in the shape [`CardRoleChips`] takes: the roles in order of
/// first appearance (the tags come strongest first), each role's tags, and the
/// role-less tags for the "Other" bucket.
pub(crate) fn group_shared_tags(
    tags: &[HttpSharedTag],
) -> (Vec<String>, BTreeMap<String, Vec<String>>, Vec<String>) {
    let mut roles: Vec<String> = Vec::new();
    let mut by_role: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut other: Vec<String> = Vec::new();
    for tag in tags {
        match &tag.role {
            Some(role) => {
                if !roles.contains(role) {
                    roles.push(role.clone());
                }
                by_role
                    .entry(role.clone())
                    .or_default()
                    .push(tag.slug.clone());
            }
            None => other.push(tag.slug.clone()),
        }
    }
    (roles, by_role, other)
}

/// The profile panel's lead line in three parts: the words before the
/// identity pips, the pips as WUBRG codes (empty when no pips follow), and
/// the words after them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProfileLead {
    /// The text the line opens with.
    pub text: String,
    /// The top identity's colors, rendered as pips after `text`.
    pub pips: Vec<String>,
    /// The deck count after the pips, empty when the text already says it.
    pub tail: String,
}

/// Words the lead line: "7 decks, mostly" and pips with "(4)", "3 decks, all
/// in" and pips, "1 deck in" and pips, or "7 decks, no two alike" when no
/// identity repeats. A colorless identity is named instead of drawn.
pub(crate) fn profile_lead(deck_count: u32, top: Option<&HttpColorIdentityCount>) -> ProfileLead {
    let decks = if deck_count == 1 {
        "1 deck".to_string()
    } else {
        format!("{deck_count} decks")
    };
    let Some(top) = top else {
        return ProfileLead {
            text: format!("{decks}, no two alike"),
            pips: vec![],
            tail: String::new(),
        };
    };
    let colorless = top.colors.is_empty();
    let (text, tail) = if deck_count == 1 {
        (
            if colorless {
                format!("{decks}, colorless")
            } else {
                format!("{decks} in")
            },
            String::new(),
        )
    } else if top.decks >= deck_count {
        (
            if colorless {
                format!("{decks}, all colorless")
            } else {
                format!("{decks}, all in")
            },
            String::new(),
        )
    } else if colorless {
        (
            format!("{decks}, mostly colorless ({})", top.decks),
            String::new(),
        )
    } else {
        (format!("{decks}, mostly"), format!("({})", top.decks))
    };
    ProfileLead {
        text,
        pips: if colorless {
            vec![]
        } else {
            top.colors.clone()
        },
        tail,
    }
}

/// Oracle tags named in the counts line under the profile's chips.
const PROFILE_TAG_COUNTS_SHOWN: usize = 3;

/// The deck counts of the strongest tags as one muted line, "sacrifice-outlet
/// in 6 decks, treasure in 4, lifegain in 3". `None` with one deck, where
/// every count is 1 and the line would say nothing.
pub(crate) fn tag_count_line(tags: &[HttpProfileTag], deck_count: u32) -> Option<String> {
    if deck_count < 2 {
        return None;
    }
    let parts: Vec<String> = tags
        .iter()
        .take(PROFILE_TAG_COUNTS_SHOWN)
        .enumerate()
        .map(|(i, tag)| {
            let decks = tag.decks;
            let unit = if i == 0 {
                if decks == 1 { " deck" } else { " decks" }
            } else {
                ""
            };
            format!("{} in {decks}{unit}", tag.slug)
        })
        .collect();
    (!parts.is_empty()).then(|| parts.join(", "))
}

/// What the user builds, over the suggestions list: a lead line with the
/// most-built identity, a row of the user's colors sized by deck count, and
/// the strongest oracle tags grouped by card role the way the card's own
/// chips are, with the deck counts of the top three under them. Open on
/// entry, quieter than the rows beneath it.
#[component]
pub(crate) fn BuilderProfilePanel(profile: HttpBuilderProfile) -> Element {
    let context_describe = try_use_context::<OtagDescribe>().map(|d| d.0);
    let context_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let lead = profile_lead(profile.deck_count, profile.top_identity.as_ref());
    let shared: Vec<HttpSharedTag> = profile
        .tags
        .iter()
        .map(|tag| HttpSharedTag {
            slug: tag.slug.clone(),
            role: tag.role.clone(),
        })
        .collect();
    let (roles, tags_by_role, other_tags) = group_shared_tags(&shared);
    let counts = tag_count_line(&profile.tags, profile.deck_count);
    rsx! {
        div { class: "screen-panel profile-panel",
            div { class: "profile-lead",
                span { "{lead.text}" }
                if !lead.pips.is_empty() {
                    span { class: "identity-pips",
                        for code in lead.pips.iter() {
                            i { key: "{code}", class: "ms ms-{code.to_lowercase()} ms-cost" }
                        }
                    }
                }
                if !lead.tail.is_empty() {
                    span { "{lead.tail}" }
                }
            }
            if !profile.colors.is_empty() {
                div { class: "profile-section",
                    span { class: "chips-label", "Colors you build" }
                    div { class: "profile-color-bar",
                        for color in profile.colors.iter() {
                            div {
                                key: "{color.color}",
                                class: "profile-color-segment",
                                style: "flex-grow: {color.decks};",
                                i { class: "ms ms-{color.color.to_lowercase()} ms-cost" }
                                span { "{color.decks}" }
                            }
                        }
                    }
                }
            }
            if !profile.tags.is_empty() {
                div { class: "profile-section",
                    span { class: "chips-label", "Oracle tags you build with" }
                    CardRoleChips {
                        roles,
                        tags_by_role,
                        other_tags,
                        describe_tag: context_describe,
                        on_examples: context_examples,
                        show_label: false,
                    }
                    if let Some(counts) = counts {
                        div { class: "profile-tag-counts", "{counts}" }
                    }
                }
            }
        }
    }
}

/// The "Why this commander?" block under a suggestion's details: a fold chip
/// in the deck list's Tags chip grammar, closed on every mount, over the
/// shared oracle tags grouped by card role (the card's own role-chip grammar)
/// and one bullet per shared color, each omitted when empty. Renders nothing
/// at all when both are. A tapped tag shows its description and its examples
/// button through `describe_tag` and `on_examples`; a host that leaves them
/// unset gets the `OtagDescribe` and `OtagExamplesOpen` contexts the card rows
/// read, which resolve only inside the screen's own subtree.
#[component]
pub(crate) fn SuggestionReasons(
    reasons: HttpSuggestionReasons,
    deck_count: u32,
    /// Resolves a shared tag's description; falls back to the `OtagDescribe` context.
    #[props(default)]
    describe_tag: Option<Callback<String, Option<String>>>,
    /// Opens a shared tag's examples; falls back to the `OtagExamplesOpen` context.
    #[props(default)]
    on_examples: Option<Callback<String>>,
) -> Element {
    let context_describe = try_use_context::<OtagDescribe>().map(|d| d.0);
    let context_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    // Closed on every mount: the sheet remounts per card and a list row on
    // expand, so each gets a folded block.
    let mut reasons_open = use_signal(|| false);
    let colors = color_lines(&reasons.colors, deck_count);
    if reasons.tags.is_empty() && colors.is_empty() {
        return rsx! {};
    }
    let (roles, tags_by_role, other_tags) = group_shared_tags(&reasons.tags);
    let describe_tag = describe_tag.or(context_describe);
    let on_examples = on_examples.or(context_examples);
    rsx! {
        div { class: "card-detail-footer",
            Chip {
                selected: reasons_open(),
                onclick: move |evt: MouseEvent| {
                    evt.stop_propagation();
                    reasons_open.set(!reasons_open());
                },
                "Why this commander?"
                // The card rows' disclosure arrow: sideways folded, down open.
                span {
                    class: if reasons_open() { "chip-fold-arrow expanded" } else { "chip-fold-arrow" },
                    "\u{25B8}"
                }
            }
            // The bullets stay mounted and ease open and closed under the chip.
            div { class: if reasons_open() { "collapsible open" } else { "collapsible" },
                div { class: "collapsible-inner",
                    ul { class: "card-detail-bullets",
                        if !reasons.tags.is_empty() {
                            li {
                                "Shares oracle tags with your decks"
                                CardRoleChips {
                                    roles,
                                    tags_by_role,
                                    other_tags,
                                    describe_tag,
                                    on_examples,
                                    show_label: false,
                                }
                            }
                        }
                        for line in colors {
                            li { key: "{line}", "{line}" }
                        }
                    }
                }
            }
        }
    }
}

/// One suggestion: the shared expandable card row with the reasons block in
/// its footer and this screen's three actions on its action bar.
#[component]
fn SuggestionRow(
    suggestion: HttpCommanderSuggestion,
    deck_count: u32,
    expanded_card: Signal<Option<Uuid>>,
    preview_card: Signal<Option<(ScryfallData, usize)>>,
    preview_dismissing: Signal<bool>,
    on_start: EventHandler<Card>,
    on_maybe: EventHandler<Uuid>,
    on_dismiss: EventHandler<Uuid>,
    on_printing: EventHandler<Card>,
) -> Element {
    let card = suggestion.card;
    let oracle_id = card.scryfall_data.oracle_id;
    let card_for_start = card.clone();
    rsx! {
        CardRow {
            card,
            qty: 1,
            expanded_card,
            preview_card,
            preview_dismissing,
            on_printing: move |c: Card| on_printing.call(c),
            footer: rsx! {
                SuggestionReasons { reasons: suggestion.reasons, deck_count }
            },
            extra_actions: rsx! {
                button {
                    class: "card-action-btn",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        on_start.call(card_for_start.clone());
                    },
                    "Create deck"
                }
                if let Some(oid) = oracle_id {
                    button {
                        class: "card-action-btn",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            on_maybe.call(oid);
                        },
                        "Maybeboard"
                    }
                    button {
                        class: "card-action-btn",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            on_dismiss.call(oid);
                        },
                        "Remove"
                    }
                }
            },
        }
    }
}

/// Screen listing the user's top 10 suggested commanders with the reasons
/// behind each, a Swipe overlay going deeper, and Clear skips under More.
#[component]
pub fn SuggestedCommanders() -> Element {
    let navigator = use_navigator();
    let auth_client: Signal<ZwipeClient> = use_context();
    let session: Signal<Option<Session>> = use_context();
    let toast = use_toast();
    let authed = use_authed(Screen::SuggestedCommanders);
    let seed: CreateDeckCommanderSeed = use_context();
    // Auto-opens once per account, reopenable from the header "?".
    let hint_open = use_one_time_hint(HINT_SUGGESTED_COMMANDERS);
    // Art-crop thumbnails on, as on the commander maybeboard.
    let show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));

    // Oracle-tag reveal inside expanded rows, wired the same way as the
    // commander maybeboard.
    let cache: CatalogCache = use_context();
    use_effect(move || {
        cache.ensure_oracle_tags(auth_client);
    });
    let describe_tag = use_callback(move |slug: String| {
        cache.oracle_tags.cell().read().loaded().and_then(|tags| {
            tags.iter()
                .find(|t| t.slug == slug)
                .and_then(|t| t.description.clone())
        })
    });
    let mut otag_examples_open = use_signal(|| false);
    let mut otag_examples_slug = use_signal(String::new);
    let open_examples = use_callback(move |slug: String| {
        otag_examples_slug.set(slug);
        otag_examples_open.set(true);
    });
    use_context_provider(|| OtagDescribe(describe_tag));
    use_context_provider(|| OtagExamplesOpen(open_examples));

    let expanded_card: Signal<Option<Uuid>> = use_signal(|| None);
    let preview_card: Signal<Option<(ScryfallData, usize)>> = use_signal(|| None);
    let preview_dismissing = use_signal(|| false);
    let mut swipe_open = use_signal(|| false);
    let swipe_mode = use_memo(|| Some(SwipeMode::Suggested));
    let mut printing_card: Signal<Option<Card>> = use_signal(|| None);
    let mut printing_open = use_signal(|| false);
    let mut show_more_sheet = use_signal(|| false);
    let mut show_clear_dialog = use_signal(|| false);

    // Entries live in a plain signal so a dismissal can drop its row in place.
    // `reload` refetches: after a dismissal leaves the list short, when the
    // Swipe overlay closes, and after Clear skips. A refetch with entries
    // present is silent (no skeleton flash).
    let mut entries: Signal<Vec<HttpCommanderSuggestion>> = use_signal(Vec::new);
    let mut deck_count = use_signal(|| 0u32);
    let mut profile: Signal<Option<HttpBuilderProfile>> = use_signal(|| None);
    let mut is_loading = use_signal(|| true);
    let mut load_failed = use_signal(|| false);
    let mut reload = use_signal(|| 0u32);
    use_effect(move || {
        let _tick = reload();
        spawn(async move {
            match authed
                .try_run("load_suggestions", |c, s| async move {
                    c.commander_suggestions(LIST_SIZE as u32, &s).await
                })
                .await
            {
                Ok(found) => {
                    deck_count.set(found.deck_count);
                    profile.set(found.profile);
                    entries.set(found.suggestions);
                    load_failed.set(false);
                }
                Err(_) => load_failed.set(entries.peek().is_empty()),
            }
            is_loading.set(false);
        });
    });
    let mut refetch = move || {
        let tick = *reload.peek();
        reload.set(tick + 1);
    };

    // Refetch when the Swipe overlay closes: its swipes dismissed and saved
    // commanders the list must not show anymore.
    let mut swipe_was_open = use_signal(|| false);
    use_effect(move || {
        if swipe_open() {
            swipe_was_open.set(true);
        } else if *swipe_was_open.peek() {
            swipe_was_open.set(false);
            refetch();
        }
    });

    // Drops a row in place and tops the list back up to ten from the server.
    let mut drop_row = move |oracle_id: Uuid| {
        entries
            .write()
            .retain(|s| s.card.scryfall_data.oracle_id != Some(oracle_id));
        if entries.peek().len() < LIST_SIZE {
            refetch();
        }
    };

    // Deck profiles back the same create gate as the deck list's Create button:
    // unverified accounts are limited to 1 deck.
    let deck_profiles_resource: Resource<Option<Vec<DeckProfile>>> =
        use_resource(move || async move {
            authed
                .run_quiet("load_deck_profiles", |c, s| async move {
                    c.get_deck_profiles(&s).await
                })
                .await
        });

    let at_deck_limit = move || {
        session().is_some_and(|s| {
            s.user.email_verified_at.is_none()
                && deck_profiles_resource
                    .read()
                    .as_ref()
                    .and_then(|o| o.as_ref())
                    .is_some_and(|p| !p.is_empty())
        })
    };

    let create_with = move |card: Card| {
        if at_deck_limit() {
            toast.warning(
                "Verify your email to create more than 1 deck".to_string(),
                ToastOptions::default().duration(TOAST_LONG),
            );
        } else {
            let mut slot = seed.0;
            slot.set(Some(card));
            navigator.push(Router::CreateDeck);
        }
    };

    let on_start = move |card: Card| create_with(card);

    // A saved commander leaves the suggestions (the server excludes the
    // maybeboard), so its row goes too.
    let on_maybe = move |oracle_id: Uuid| {
        spawn(async move {
            if authed
                .run("add_commander_maybeboard", |c, s| async move {
                    c.add_commander_maybeboard_card(oracle_id, &s).await
                })
                .await
                .is_some()
            {
                drop_row(oracle_id);
                toast.success(
                    "Added to your commander maybeboard".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    };

    let on_dismiss = move |oracle_id: Uuid| {
        spawn(async move {
            if authed
                .run("dismiss_commander", |c, s| async move {
                    c.dismiss_commander(oracle_id, &s).await
                })
                .await
                .is_some()
            {
                drop_row(oracle_id);
                toast.info(
                    "Skipped for a week".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    };

    let clear_dismissals = use_callback(move |()| {
        show_more_sheet.set(false);
        spawn(async move {
            if authed
                .run("clear_commander_dismissals", |c, s| async move {
                    c.clear_commander_dismissals(&s).await
                })
                .await
                .is_some()
            {
                toast.info(
                    "Skips cleared".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
                refetch();
            }
        });
    });

    let cold_start = deck_count() == 0;
    let has_entries = !entries().is_empty();

    rsx! {
        div { class: "screen",
            ScreenHeader { title: "Suggested commanders", hint: hint_open }

            HintDialog {
                open: hint_open,
                title: "Suggested commanders",
                HintBullets {
                    HintBullet { HintKey { "Create deck" } " begins a deck with that commander" }
                    HintBullet { HintKey { "Maybeboard" } " saves it to your commander maybeboard" }
                    HintBullet { HintKey { color: "--color-error", "Remove" } " hides it for a week and pulls the next one up" }
                    HintBullet { HintKey { "Swipe" } " deals the same ranking: " HintColored { color: "--color-success", "right" } " starts a deck, " HintColored { color: "--color-warning", "up" } " saves, " HintColored { color: "--color-error", "left" } " skips it" }
                }
                HintLine { "Your top 10 for the decks you already build, best fit first. Swipe keeps going down the same ranking" }
            }

            div { class: "screen-content",
                // Bounded like the commander maybeboard: screen-content centers
                // its children, so an unbounded column would grow to content
                // width and overflow the viewport on long rows.
                div {
                    class: "flex-col",
                    style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
                    if is_loading() {
                        // The group and its header are static chrome, so they
                        // render real while only the data rows ghost (the
                        // closed-vocabulary skeleton rule).
                        div { class: "maybeboard-list card-group row-enter",
                            div { class: "card-group-header", "Suggested commanders" }
                            for i in 0..6 {
                                div { key: "{i}", class: "skeleton-card-row",
                                    div { class: "skeleton-bar skeleton-card-thumb" }
                                    div { class: "skeleton-bar skeleton-card-bar-row" }
                                }
                            }
                        }
                    } else if load_failed() {
                        div { class: "message-empty",
                            span { class: "chip-note", "Could not load your suggestions" }
                        }
                    } else if cold_start {
                        div { class: "message-empty",
                            span { class: "chip-note", "Build a deck and we'll suggest commanders that fit how you play." }
                        }
                    } else {
                        if let Some(profile) = profile() {
                            BuilderProfilePanel { profile }
                        }
                        if !has_entries {
                            div { class: "message-empty",
                                span { class: "chip-note", "No suggestions right now" }
                            }
                        } else {
                            div { class: "maybeboard-list card-group row-enter",
                                div { class: "card-group-header", "Suggested commanders" }
                                for suggestion in entries() {
                                    SuggestionRow {
                                        key: "{suggestion.card.scryfall_data.id}",
                                        suggestion,
                                        deck_count: deck_count(),
                                        expanded_card,
                                        preview_card,
                                        preview_dismissing,
                                        on_start,
                                        on_maybe,
                                        on_dismiss,
                                        on_printing: move |c: Card| {
                                            printing_card.set(Some(c));
                                            printing_open.set(true);
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| {
                        navigator.push(Router::DeckList {});
                    },
                    "Back"
                }
                if !cold_start {
                    Button {
                        variant: ButtonVariant::Action,
                        onclick: move |_| swipe_open.set(true),
                        "Swipe"
                    }
                }
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| show_more_sheet.set(true),
                    "More"
                }
            }

            BottomSheet { open: show_more_sheet, title: "More actions",
                Button {
                    danger: true,
                    onclick: move |_| {
                        show_clear_dialog.set(true);
                    },
                    "Clear skips"
                }
            }

            AlertDialog {
                open: show_clear_dialog,
                title: "Clear skips",
                actions: vec![
                    DialogAction::Dismiss("Cancel".into()),
                    DialogAction::Confirm {
                        label: "Clear".into(),
                        danger: true,
                        on_confirm: clear_dismissals,
                    },
                ],
                "Commanders you skipped will be suggested again. This can't be undone."
            }

            // The shared overlay in its suggested mode deals this ranking down
            // to 100: right seeds a new deck with the pick, up saves it, left
            // dismisses it.
            SwipeSelect {
                host_screen: screen::SUGGESTED_COMMANDERS,
                open: swipe_open,
                mode: swipe_mode,
                on_select: move |card: Card| {
                    swipe_open.set(false);
                    create_with(card);
                },
                on_close: move |_| swipe_open.set(false),
            }

            // Pick a printing: swap that row's card in place and keep the row
            // expanded under its new scryfall id.
            if let Some(card) = printing_card() {
                PrintingSheet {
                    card,
                    open: printing_open,
                    host_screen: screen::SUGGESTED_COMMANDERS,
                    on_save: move |new_card: Card| {
                        let target = new_card.scryfall_data.oracle_id;
                        if target.is_some() {
                            let mut expanded = expanded_card;
                            expanded.set(Some(new_card.scryfall_data.id));
                            if let Some(slot) = entries
                                .write()
                                .iter_mut()
                                .find(|s| s.card.scryfall_data.oracle_id == target)
                            {
                                slot.card = new_card;
                            }
                        }
                    },
                }
            }

            if otag_examples_open() {
                OracleTagExamples { open: otag_examples_open, slug: otag_examples_slug() }
            }

            ImagePreview { card: preview_card, dismissing: preview_dismissing }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn color(code: &str, decks: u32) -> HttpColorAffinity {
        HttpColorAffinity {
            color: code.to_string(),
            decks,
        }
    }

    #[test]
    fn color_lines_name_each_color_and_count_decks() {
        let lines = color_lines(&[color("R", 3), color("G", 1)], 4);
        assert_eq!(
            lines,
            vec![
                "You build in red (3 of your 4 decks)",
                "You build in green (1 of your 4 decks)"
            ]
        );
        assert_eq!(
            color_lines(&[color("W", 1)], 1),
            vec!["You build in white (1 of your 1 deck)"]
        );
    }

    #[test]
    fn color_lines_skip_unknown_codes() {
        assert!(color_lines(&[color("C", 2)], 2).is_empty());
    }

    fn tag(slug: &str, role: Option<&str>) -> HttpSharedTag {
        HttpSharedTag {
            slug: slug.to_string(),
            role: role.map(str::to_string),
        }
    }

    fn top(colors: &[&str], decks: u32) -> HttpColorIdentityCount {
        HttpColorIdentityCount {
            colors: colors.iter().map(|c| c.to_string()).collect(),
            decks,
        }
    }

    fn lead(text: &str, pips: &[&str], tail: &str) -> ProfileLead {
        ProfileLead {
            text: text.to_string(),
            pips: pips.iter().map(|c| c.to_string()).collect(),
            tail: tail.to_string(),
        }
    }

    #[test]
    fn profile_lead_words_the_deck_count_and_identity() {
        assert_eq!(
            profile_lead(7, Some(&top(&["W", "U", "R"], 4))),
            lead("7 decks, mostly", &["W", "U", "R"], "(4)")
        );
        assert_eq!(
            profile_lead(3, Some(&top(&["R"], 3))),
            lead("3 decks, all in", &["R"], "")
        );
        assert_eq!(
            profile_lead(1, Some(&top(&["B", "G"], 1))),
            lead("1 deck in", &["B", "G"], "")
        );
        assert_eq!(
            profile_lead(7, None),
            lead("7 decks, no two alike", &[], "")
        );
    }

    #[test]
    fn profile_lead_names_colorless_instead_of_drawing_it() {
        assert_eq!(
            profile_lead(1, Some(&top(&[], 1))),
            lead("1 deck, colorless", &[], "")
        );
        assert_eq!(
            profile_lead(5, Some(&top(&[], 2))),
            lead("5 decks, mostly colorless (2)", &[], "")
        );
        assert_eq!(
            profile_lead(2, Some(&top(&[], 2))),
            lead("2 decks, all colorless", &[], "")
        );
    }

    fn profile_tag(slug: &str, decks: u32) -> HttpProfileTag {
        HttpProfileTag {
            slug: slug.to_string(),
            role: None,
            decks,
        }
    }

    #[test]
    fn tag_count_line_names_the_top_three_with_one_unit() {
        let tags = [
            profile_tag("sacrifice-outlet", 6),
            profile_tag("treasure", 4),
            profile_tag("lifegain", 3),
            profile_tag("ramp", 2),
        ];
        assert_eq!(
            tag_count_line(&tags, 7).as_deref(),
            Some("sacrifice-outlet in 6 decks, treasure in 4, lifegain in 3")
        );
        assert_eq!(
            tag_count_line(&tags[..1], 7).as_deref(),
            Some("sacrifice-outlet in 6 decks")
        );
        assert_eq!(
            tag_count_line(&[profile_tag("ramp", 1)], 3).as_deref(),
            Some("ramp in 1 deck")
        );
        assert_eq!(tag_count_line(&tags, 1), None, "one deck: every count is 1");
        assert_eq!(tag_count_line(&[], 4), None);
    }

    #[test]
    fn shared_tags_group_by_role_in_first_appearance_order() {
        let (roles, by_role, other) = group_shared_tags(&[
            tag("sacrifice-outlet", Some("sacrifice")),
            tag("typal-goblin", None),
            tag("spot-removal", Some("removal")),
            tag("aristocrats", Some("sacrifice")),
        ]);
        assert_eq!(roles, vec!["sacrifice", "removal"]);
        let expected: BTreeMap<String, Vec<String>> = BTreeMap::from([
            (
                "sacrifice".to_string(),
                vec!["sacrifice-outlet".to_string(), "aristocrats".to_string()],
            ),
            ("removal".to_string(), vec!["spot-removal".to_string()]),
        ]);
        assert_eq!(by_role, expected);
        assert_eq!(other, vec!["typal-goblin"]);
    }
}
