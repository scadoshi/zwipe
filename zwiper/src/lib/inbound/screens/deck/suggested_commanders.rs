//! Suggested commanders screen: the top 10 commanders ranked against the
//! user's own decks, each with the reasons it ranked, styled like the
//! commander maybeboard.
//!
//! An expanded row offers **Create deck** (seeds the create screen), **Maybeboard**
//! (saves to the commander maybeboard) and **Remove** (hides it for a week). The action bar's Swipe opens the shared overlay in its suggested
//! mode, which deals the same ranking down to 100. Every action that removes
//! a row refetches when fewer than ten remain, so the list stays a full top 10.
//! Over the list, a panel sums up what the user builds: the deck count with
//! the most common color set and color, then charts of the colors, card roles
//! and oracle tags across the decks, the profile the ranking reads. With no
//! commander deck to read from, the screen shows a single line instead of a
//! list and no panel.

use std::collections::BTreeMap;

use components::{
    ActionBar, AlertDialog, Bar, BarChart, BarOrientation, BottomSheet, Button, ButtonVariant,
    CardRoleChips, Chip, DialogAction, HintBullet, HintBullets, HintDialog, HintKey, HintLine,
    LabelCase, TOAST_LONG, TOAST_QUICK,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::card_role::role_short_label;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::domain::user::models::hints::HINT_SUGGESTED_COMMANDERS;
use zwipe_core::http::contracts::suggestions::{
    HttpBuilderProfile, HttpColorAffinity, HttpCommanderSuggestion, HttpProfileRole,
    HttpProfileTag, HttpSharedTag, HttpSuggestionReasons,
};

use super::components::collapsible_section::CollapsibleSection;
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

/// Pips with the words after them, one "most common" line of the profile
/// panel's lead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LeadPips {
    /// WUBRG codes drawn as pips; empty when `tail` names the identity.
    pub pips: Vec<String>,
    /// The deck count after the pips, "(4)", or "colorless (4)".
    pub tail: String,
}

/// The profile panel's lead: the deck count, then the most common color set
/// and the most common color, each absent when the profile has none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProfileLead {
    /// "11 decks" or "1 deck".
    pub decks: String,
    /// The top identity with its count; `None` when no identity repeats.
    pub color_set: Option<LeadPips>,
    /// The color in the most decks with its count; `None` when every deck is
    /// colorless.
    pub color: Option<LeadPips>,
}

/// Words the lead from the profile. The top identity's colorless case is
/// named instead of drawn; the most common color is the first the server
/// sent, which is the one in the most decks.
pub(crate) fn profile_lead(profile: &HttpBuilderProfile) -> ProfileLead {
    let decks = if profile.deck_count == 1 {
        "1 deck".to_string()
    } else {
        format!("{} decks", profile.deck_count)
    };
    let color_set = profile.top_identity.as_ref().map(|top| {
        if top.colors.is_empty() {
            LeadPips {
                pips: vec![],
                tail: format!("colorless ({})", top.decks),
            }
        } else {
            LeadPips {
                pips: top.colors.clone(),
                tail: format!("({})", top.decks),
            }
        }
    });
    let color = profile.colors.first().map(|c| LeadPips {
        pips: vec![c.color.clone()],
        tail: format!("({})", c.decks),
    });
    ProfileLead {
        decks,
        color_set,
        color,
    }
}

/// Max-normalized bar sizes, the deck charts' math.
fn bars_from<T>(
    items: &[T],
    count: impl Fn(&T) -> usize,
    bar: impl Fn(&T, u32) -> Bar,
) -> Vec<Bar> {
    let max = items.iter().map(&count).max().unwrap_or(0);
    items
        .iter()
        .map(|item| bar(item, DeckMetrics::bar_pct(count(item), max)))
        .collect()
}

/// A deck count as a bar size.
fn decks_usize(decks: u32) -> usize {
    usize::try_from(decks).unwrap_or(usize::MAX)
}

/// One bar per color in WUBRG order, labeled the way the deck color chart
/// labels its bars and keyed by the color code. A code outside WUBRG is
/// skipped.
pub(crate) fn color_bars(colors: &[HttpColorAffinity]) -> Vec<Bar> {
    let mut known: Vec<&HttpColorAffinity> = colors
        .iter()
        .filter(|c| color_name(&c.color).is_some())
        .collect();
    known.sort_by_key(|c| WUBRG.iter().position(|code| *code == c.color));
    bars_from(
        &known,
        |c| decks_usize(c.decks),
        |c, pct| Bar {
            key: c.color.clone(),
            label: color_name(&c.color).unwrap_or_default().to_string(),
            count: decks_usize(c.decks),
            pct,
        },
    )
}

/// The WUBRG codes in order.
const WUBRG: [&str; 5] = ["W", "U", "B", "R", "G"];

/// One bar per role in the order sent, labeled as the deck role chart labels
/// the role and keyed by the role slug.
pub(crate) fn role_bars(roles: &[HttpProfileRole]) -> Vec<Bar> {
    bars_from(
        roles,
        |r| decks_usize(r.decks),
        |r, pct| Bar {
            key: r.role.clone(),
            label: role_short_label(&r.role),
            count: decks_usize(r.decks),
            pct,
        },
    )
}

/// One bar per oracle tag in the order sent, the slug as label and key.
pub(crate) fn tag_bars(tags: &[HttpProfileTag]) -> Vec<Bar> {
    bars_from(
        tags,
        |t| decks_usize(t.decks),
        |t, pct| Bar::new(t.slug.clone(), decks_usize(t.decks), pct),
    )
}

/// The shown tags under `role`, strongest first.
pub(crate) fn role_tags(tags: &[HttpProfileTag], role: &str) -> Vec<String> {
    tags.iter()
        .filter(|t| t.role.as_deref() == Some(role))
        .map(|t| t.slug.clone())
        .collect()
}

/// Width of the oracle tag chart's label column: room for a slug like
/// `sacrifice-outlet` on one line, a longer one wraps at its hyphens.
const TAG_LABEL_WIDTH: &str = "14ch";

/// An oracle tag's description eased open under a chart, with its Examples
/// button, through the screen's `OtagDescribe` and `OtagExamplesOpen`
/// contexts. Collapsed while `slug` is `None`.
#[component]
fn OtagReveal(slug: Option<String>) -> Element {
    let describe = try_use_context::<OtagDescribe>().map(|d| d.0);
    let examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let def = slug.as_ref().map(|s| {
        describe
            .and_then(|cb| cb.call(s.clone()))
            .unwrap_or_else(|| "No description yet".to_string())
    });
    let reveal_class = if slug.is_some() {
        "keyword-reveal open"
    } else {
        "keyword-reveal"
    };
    rsx! {
        div { class: "{reveal_class}",
            div { class: "keyword-reveal-inner",
                if let Some(def) = def {
                    div { class: "otag-def",
                        p { class: "otag-def-text", "{def}" }
                        if let (Some(handler), Some(slug)) = (examples, slug) {
                            button {
                                class: "otag-examples-btn",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    handler.call(slug.clone());
                                },
                                "Examples"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A tapped role's oracle tags from the profile as chips under the role
/// chart, each tappable for its description, the "Why this commander?"
/// chips' reveal.
#[component]
fn RoleTagsReveal(tags: Vec<String>) -> Element {
    let mut open_tag: Signal<Option<String>> = use_signal(|| None);
    let reveal_class = if tags.is_empty() {
        "keyword-reveal"
    } else {
        "keyword-reveal open"
    };
    let shown = open_tag().filter(|slug| tags.contains(slug));
    rsx! {
        div { class: "{reveal_class}",
            div { class: "keyword-reveal-inner",
                if !tags.is_empty() {
                    div { class: "otag-reveal-block",
                        div { class: "card-detail-meta card-detail-otags",
                            for tag in tags.iter().cloned() {
                                {
                                    let slug = tag.clone();
                                    rsx! {
                                        button {
                                            key: "{tag}",
                                            class: if shown.as_deref() == Some(tag.as_str()) { "keyword-chip active" } else { "keyword-chip" },
                                            onclick: move |evt| {
                                                evt.stop_propagation();
                                                if open_tag().as_deref() == Some(slug.as_str()) {
                                                    open_tag.set(None);
                                                } else {
                                                    open_tag.set(Some(slug.clone()));
                                                }
                                            },
                                            "{tag}"
                                        }
                                    }
                                }
                            }
                        }
                        OtagReveal { slug: shown }
                    }
                }
            }
        }
    }
}

/// What the user builds, over the suggestions list: the deck count with the
/// most common color set and color under it, then three collapsible charts
/// in the deck profile's grammar, all open on entry. Colors are columns in
/// WUBRG order; card roles are rows like the deck profile's role chart, a
/// tapped role revealing its oracle tags from the profile; oracle tags are
/// rows, a tapped tag revealing its description and Examples. Quieter than
/// the rows beneath it.
#[component]
pub(crate) fn BuilderProfilePanel(profile: HttpBuilderProfile) -> Element {
    let lead = profile_lead(&profile);
    // One signal per section so each folds on its own and all start open.
    let colors_open: Signal<Option<String>> = use_signal(|| Some("Colors".to_string()));
    let roles_open: Signal<Option<String>> = use_signal(|| Some("Card roles".to_string()));
    let tags_open: Signal<Option<String>> = use_signal(|| Some("Oracle tags".to_string()));
    let mut selected_role: Signal<Option<String>> = use_signal(|| None);
    let mut selected_tag: Signal<Option<String>> = use_signal(|| None);
    let colors = color_bars(&profile.colors);
    let roles = role_bars(&profile.roles);
    let tags = tag_bars(&profile.tags);
    let revealed_role_tags = selected_role()
        .map(|role| role_tags(&profile.tags, &role))
        .unwrap_or_default();
    let toggle_role = use_callback(move |key: String| {
        if selected_role().as_deref() == Some(key.as_str()) {
            selected_role.set(None);
        } else {
            selected_role.set(Some(key));
        }
    });
    let toggle_tag = use_callback(move |key: String| {
        if selected_tag().as_deref() == Some(key.as_str()) {
            selected_tag.set(None);
        } else {
            selected_tag.set(Some(key));
        }
    });
    rsx! {
        div { class: "screen-panel profile-panel",
            div { class: "profile-lead", span { "{lead.decks}" } }
            if let Some(set) = lead.color_set.as_ref() {
                div { class: "profile-fact",
                    span { "Most common color set" }
                    if !set.pips.is_empty() {
                        span { class: "identity-pips",
                            for code in set.pips.iter() {
                                i { key: "{code}", class: "ms ms-{code.to_lowercase()} ms-cost" }
                            }
                        }
                    }
                    span { "{set.tail}" }
                }
            }
            if let Some(color) = lead.color.as_ref() {
                div { class: "profile-fact",
                    span { "Most common color" }
                    span { class: "identity-pips",
                        for code in color.pips.iter() {
                            i { key: "{code}", class: "ms ms-{code.to_lowercase()} ms-cost" }
                        }
                    }
                    span { "{color.tail}" }
                }
            }
            div { class: "profile-charts",
                if !colors.is_empty() {
                    CollapsibleSection { title: "Colors", open_section: colors_open,
                        BarChart { bars: colors }
                    }
                }
                if !roles.is_empty() {
                    CollapsibleSection { title: "Card roles", open_section: roles_open,
                        BarChart {
                            bars: roles,
                            orientation: BarOrientation::Horizontal,
                            on_bar: toggle_role,
                            selected: selected_role(),
                        }
                        RoleTagsReveal { tags: revealed_role_tags }
                    }
                }
                if !tags.is_empty() {
                    CollapsibleSection { title: "Oracle tags", open_section: tags_open,
                        BarChart {
                            bars: tags,
                            orientation: BarOrientation::Horizontal,
                            label_case: LabelCase::AsIs,
                            label_width: TAG_LABEL_WIDTH,
                            on_bar: toggle_tag,
                            selected: selected_tag(),
                        }
                        OtagReveal { slug: selected_tag() }
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
    use zwipe_core::http::contracts::suggestions::HttpColorIdentityCount;

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

    fn profile(
        deck_count: u32,
        colors: Vec<HttpColorAffinity>,
        top: Option<HttpColorIdentityCount>,
    ) -> HttpBuilderProfile {
        HttpBuilderProfile {
            deck_count,
            colors,
            top_identity: top,
            tags: vec![],
            roles: vec![],
        }
    }

    fn pips(codes: &[&str], tail: &str) -> LeadPips {
        LeadPips {
            pips: codes.iter().map(|c| c.to_string()).collect(),
            tail: tail.to_string(),
        }
    }

    #[test]
    fn profile_lead_counts_decks_and_names_the_most_common_set_and_color() {
        let lead = profile_lead(&profile(
            7,
            vec![color("R", 5), color("W", 4)],
            Some(top(&["W", "U", "R"], 4)),
        ));
        assert_eq!(lead.decks, "7 decks");
        assert_eq!(
            lead.color_set.as_ref(),
            Some(&pips(&["W", "U", "R"], "(4)"))
        );
        assert_eq!(lead.color.as_ref(), Some(&pips(&["R"], "(5)")));

        let lead = profile_lead(&profile(1, vec![color("B", 1)], Some(top(&["B"], 1))));
        assert_eq!(lead.decks, "1 deck");
        assert_eq!(lead.color_set.as_ref(), Some(&pips(&["B"], "(1)")));
        assert_eq!(lead.color.as_ref(), Some(&pips(&["B"], "(1)")));
    }

    #[test]
    fn profile_lead_omits_what_the_profile_lacks_and_names_colorless() {
        let lead = profile_lead(&profile(3, vec![color("U", 2)], None));
        assert_eq!(lead.color_set, None, "no identity repeats");
        assert_eq!(lead.color.as_ref(), Some(&pips(&["U"], "(2)")));

        let lead = profile_lead(&profile(2, vec![], Some(top(&[], 2))));
        assert_eq!(lead.color_set.as_ref(), Some(&pips(&[], "colorless (2)")));
        assert_eq!(lead.color, None, "every deck colorless");
    }

    fn profile_tag(slug: &str, role: Option<&str>, decks: u32) -> HttpProfileTag {
        HttpProfileTag {
            slug: slug.to_string(),
            role: role.map(str::to_string),
            decks,
        }
    }

    fn bar(key: &str, label: &str, count: usize, pct: u32) -> Bar {
        Bar {
            key: key.to_string(),
            label: label.to_string(),
            count,
            pct,
        }
    }

    #[test]
    fn color_bars_run_wubrg_with_the_deck_chart_labels_and_math() {
        let bars = color_bars(&[color("R", 4), color("W", 2), color("C", 9), color("G", 1)]);
        assert_eq!(
            bars,
            vec![
                bar("W", "white", 2, 50),
                bar("R", "red", 4, 100),
                bar("G", "green", 1, 25),
            ],
            "WUBRG order, unknown code dropped, pct over the max"
        );
    }

    #[test]
    fn role_bars_keep_the_order_sent_and_abbreviate_like_the_deck_chart() {
        let roles = [
            HttpProfileRole {
                role: "sacrifice".to_string(),
                decks: 50,
            },
            HttpProfileRole {
                role: "ramp".to_string(),
                decks: 1,
            },
        ];
        assert_eq!(
            role_bars(&roles),
            vec![
                bar("sacrifice", "sacrf", 50, 100),
                bar("ramp", "ramp", 1, 4)
            ],
            "a nonzero bar is at least the 4% sliver"
        );
    }

    #[test]
    fn tag_bars_and_role_tags_read_the_slugs() {
        let tags = [
            profile_tag("sacrifice-outlet", Some("sacrifice"), 6),
            profile_tag("treasure", Some("ramp"), 3),
            profile_tag("aristocrats", Some("sacrifice"), 3),
            profile_tag("typal-goblin", None, 1),
        ];
        assert_eq!(
            tag_bars(&tags),
            vec![
                bar("sacrifice-outlet", "sacrifice-outlet", 6, 100),
                bar("treasure", "treasure", 3, 50),
                bar("aristocrats", "aristocrats", 3, 50),
                bar("typal-goblin", "typal-goblin", 1, 16),
            ]
        );
        assert_eq!(
            role_tags(&tags, "sacrifice"),
            vec!["sacrifice-outlet", "aristocrats"]
        );
        assert!(role_tags(&tags, "draw").is_empty());
        assert!(tag_bars(&[]).is_empty());
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
