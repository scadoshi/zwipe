//! Playstyle screen: what the user builds, read from every deck they
//! own in any format, over the top 10 commanders ranked against those decks,
//! each with the reasons it ranked, styled like the commander maybeboard.
//!
//! The screen fetches the ranking 100 deep once and shows the first 10. A
//! Show row atop the list holds one chip per color identity among the 100,
//! each with its pips and name; a tapped chip narrows the held list to that identity, and the Art chip
//! toggles the row thumbnails. An expanded row offers **Create deck** (seeds
//! the create screen), **Maybeboard** (saves to the commander maybeboard) and
//! **Remove** (hides it for a week). The action bar's Swipe opens the shared
//! overlay in its suggested mode, which deals the same held list and whose
//! Filter sheet holds the same Show chips over the same signal. Every action
//! that removes a row fills from the held list and refetches only when fewer
//! than ten remain, so the list stays a full top 10. Above the list, sections
//! in the deck profile's grammar sum up what the user builds, the profile the
//! ranking reads: a Summary table (the deck count, the top colors and the
//! top single color), then charts of the colors and card roles and a table of the
//! oracle tags across the decks. They share one accordion with Summary open
//! on entry; the list section folds on its own and starts open. With no deck
//! that has cards to read from, the screen shows a single line instead of the
//! sections and the list.

use std::collections::BTreeMap;

use components::{
    ActionBar, AlertDialog, Bar, BarChart, BarOrientation, BottomSheet, Button, ButtonVariant,
    CardRoleChips, Chip, DialogAction, HintBullet, HintBullets, HintDialog, HintKey, HintLine,
    TOAST_LONG, TOAST_QUICK,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::card_role::role_short_label;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::scryfall_data::colors::{Color, identity_name};
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::domain::user::models::hints::HINT_SUGGESTED_COMMANDERS;
use zwipe_core::http::contracts::suggestions::{
    HttpBuilderProfile, HttpColorAffinity, HttpCommanderSuggestion, HttpProfileRole,
    HttpProfileTag, HttpSharedTag, HttpSuggestionReasons, MAX_SUGGESTION_LIMIT,
};

use super::components::collapsible_section::CollapsibleSection;
use super::components::skeletons::{BuilderProfileSkeleton, ShowChipRowSkeleton};
use super::components::swipe_select::{SuggestedPile, SwipeMode, SwipeSelect};
use crate::inbound::components::auth::authed::use_authed;
use crate::inbound::components::catalog_cache::CatalogCache;
use crate::inbound::components::hint_dialog::{HintColored, use_one_time_hint};
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::InfoButton;
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

/// Suggestions fetched and held, the server's maximum; the list shows the first
/// [`LIST_SIZE`] of them and Swipe deals them all.
const HELD_SIZE: u32 = MAX_SUGGESTION_LIMIT;

/// A commander's exact color identity in WUBRG order, each color once; empty
/// is colorless.
pub(crate) fn identity_of(colors: &[Color]) -> Vec<Color> {
    Color::all()
        .into_iter()
        .filter(|c| colors.contains(c))
        .collect()
}

/// A suggestion's colors, the identity the Show chips read.
fn suggestion_colors(suggestion: &HttpCommanderSuggestion) -> &[Color] {
    &suggestion.card.scryfall_data.color_identity
}

/// The distinct color identities among `colors` (one entry per commander), in
/// the deck list's group order: fewest colors first, WUBRG order between
/// identities of the same size, colorless last.
pub(crate) fn identities_present<'a>(
    colors: impl IntoIterator<Item = &'a [Color]>,
) -> Vec<Vec<Color>> {
    let mut identities: Vec<Vec<Color>> = Vec::new();
    for commander in colors {
        let identity = identity_of(commander);
        if !identities.contains(&identity) {
            identities.push(identity);
        }
    }
    identities.sort_by_key(|identity| {
        let order: Vec<usize> = identity
            .iter()
            .map(|c| Color::all().iter().position(|a| a == c).unwrap_or(0))
            .collect();
        (identity.is_empty(), identity.len(), order)
    });
    identities
}

/// The items whose `colors` are exactly `identity`, in the order given; every
/// item when `identity` is `None` (the All chip). The server already spreads
/// its 100 across identities (at most four of each keep their rank, the rest
/// move behind, nothing dropped), so this is a plain predicate over that order
/// and no cap is applied here.
pub(crate) fn of_identity<T: Clone>(
    items: &[T],
    colors: impl Fn(&T) -> &[Color],
    identity: Option<&[Color]>,
) -> Vec<T> {
    items
        .iter()
        .filter(|item| identity.is_none_or(|wanted| identity_of(colors(item)) == wanted))
        .cloned()
        .collect()
}

/// The first [`LIST_SIZE`] of a filtered list, the rows the screen shows.
pub(crate) fn shown<T: Clone>(filtered: &[T]) -> Vec<T> {
    filtered.iter().take(LIST_SIZE).cloned().collect()
}

/// A chip's stable key: the identity's codes, `C` for colorless.
fn identity_key(identity: &[Color]) -> String {
    if identity.is_empty() {
        "C".to_string()
    } else {
        identity.iter().map(Color::to_short_name).collect()
    }
}

/// An identity's name, then its pips in the deck list's group-header
/// grammar, then `count` when given. A single color is its pip alone, since a
/// pip already says which color it is; colorless is the name alone.
#[component]
pub(crate) fn IdentityPips(identity: Vec<Color>, count: Option<String>) -> Element {
    let name = identity_label(&identity);
    rsx! {
        span { class: "identity-pips",
            if let Some(name) = name {
                span { "{name}" }
            }
            for color in identity.iter() {
                i {
                    key: "{color.to_short_name()}",
                    class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost",
                }
            }
            if let Some(count) = count {
                span { "{count}" }
            }
        }
    }
}

/// The Show chip row: the label, All, then one chip per identity in
/// `identities` with its pips and name, then `children` (the screen's Art
/// toggle). Single select over `active_identity`, `None` being All; the
/// selected chip tapped again is All. The screen and the Swipe overlay's
/// Filter sheet both render it over the same signal, so a tap in either
/// filters both.
#[component]
pub(crate) fn ShowIdentityChips(
    mut active_identity: Signal<Option<Vec<Color>>>,
    identities: Vec<Vec<Color>>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "chip-row",
            span { class: "stat-chip stat-chip-p1 chip-row-label", "Show" }
            Chip {
                selected: active_identity().is_none(),
                onclick: move |_| active_identity.set(None),
                "All"
            }
            for identity in identities {
                Chip {
                    key: "{identity_key(&identity)}",
                    selected: active_identity().as_deref() == Some(identity.as_slice()),
                    onclick: {
                        let identity = identity.clone();
                        move |_| {
                            if active_identity().as_deref() == Some(identity.as_slice()) {
                                active_identity.set(None);
                            } else {
                                active_identity.set(Some(identity.clone()));
                            }
                        }
                    },
                    IdentityPips { identity: identity.clone() }
                }
            }
            {children}
        }
    }
}

/// The name shown beside an identity's pips: none for a single color, the
/// identity's name otherwise.
pub(crate) fn identity_label(identity: &[Color]) -> Option<&'static str> {
    (identity.len() != 1).then(|| identity_name(identity))
}

/// The list's empty state under a Show chip whose commanders are all gone,
/// "No Jeskai commanders in your top 100". The names that describe rather
/// than name (colorless, mono-color, five-color) go lowercase mid-sentence.
pub(crate) fn empty_identity_line(identity: &[Color]) -> String {
    let identity = identity_of(identity);
    let name = identity_name(&identity);
    let name = match identity.len() {
        0 | 1 | 5 => name.to_lowercase(),
        _ => name.to_string(),
    };
    format!("No {name} commanders in your top 100")
}

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

/// A Summary row's value: an identity, drawn by [`IdentityPips`], and its
/// deck count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SummaryValue {
    /// The colors drawn as pips in WUBRG order; empty for colorless.
    pub pips: Vec<Color>,
    /// The deck count after the pips, "(4)".
    pub count: String,
}

/// The Summary section's rows: the deck count, then the most common color
/// set and the top color, each absent when the profile has none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProfileSummary {
    /// The Decks row's value, the bare count.
    pub decks: String,
    /// The top identity with its count; `None` when no identity repeats.
    pub color_set: Option<SummaryValue>,
    /// The color in the most decks with its count; `None` when every deck is
    /// colorless.
    pub color: Option<SummaryValue>,
}

/// The Summary rows from the profile. The top color is the first WUBRG
/// color the server sent, which is the one in the most decks; codes outside
/// WUBRG are skipped.
pub(crate) fn profile_summary(profile: &HttpBuilderProfile) -> ProfileSummary {
    let color_set = profile.top_identity.as_ref().map(|top| {
        let parsed: Vec<Color> = top
            .colors
            .iter()
            .filter_map(|code| Color::try_from(code.as_str()).ok())
            .collect();
        SummaryValue {
            pips: identity_of(&parsed),
            count: format!("({})", top.decks),
        }
    });
    let color = profile.colors.iter().find_map(|c| {
        let color = Color::try_from(c.color.as_str()).ok()?;
        Some(SummaryValue {
            pips: vec![color],
            count: format!("({})", c.decks),
        })
    });
    ProfileSummary {
        decks: profile.deck_count.to_string(),
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
            revealable: true,
        },
    )
}

/// The WUBRG codes in order.
const WUBRG: [&str; 5] = ["W", "U", "B", "R", "G"];

/// The list's group header, the deck view's non-collapsible Commander
/// header grammar.
const LIST_TITLE: &str = "Suggested commanders";

/// The profile sections' titles, also their accordion keys.
const SUMMARY_TITLE: &str = "Summary";
const COLORS_TITLE: &str = "Colors";
const ROLES_TITLE: &str = "Card roles";
const TAGS_TITLE: &str = "Oracle tags";

/// One bar per role in the order sent, labeled as the deck role chart labels
/// the role and keyed by the role slug. A role with none of `tags` under it
/// has nothing to reveal, so its bar is not revealable.
pub(crate) fn role_bars(roles: &[HttpProfileRole], tags: &[HttpProfileTag]) -> Vec<Bar> {
    bars_from(
        roles,
        |r| decks_usize(r.decks),
        |r, pct| Bar {
            key: r.role.clone(),
            label: role_short_label(&r.role),
            count: decks_usize(r.decks),
            pct,
            revealable: tags
                .iter()
                .any(|t| t.role.as_deref() == Some(r.role.as_str())),
        },
    )
}

/// The impact the top tag reads.
const TOP_IMPACT: u32 = 100;

/// One row of the oracle tags table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagRow {
    /// The oracle tag slug, the row's chip.
    pub(crate) slug: String,
    /// Profiled decks carrying the tag.
    pub(crate) decks: u32,
    /// Cards carrying the tag across those decks.
    pub(crate) cards: u32,
    /// The tag's score against the top tag's, rounded, the top tag reading
    /// [`TOP_IMPACT`]. None when no tag scores above 0, as from an older
    /// server.
    pub(crate) impact: Option<u32>,
}

/// `score` against `max` on a 0 to [`TOP_IMPACT`] scale, rounded. None when
/// `max` is not above 0; a negative or NaN score reads 0.
fn impact(score: f64, max: f64) -> Option<u32> {
    if max.is_nan() || max <= 0.0 {
        return None;
    }
    let scaled = (score.max(0.0) / max * f64::from(TOP_IMPACT)).round();
    // No TryFrom from f64 to an integer, so find the whole number instead.
    Some(
        (0..=TOP_IMPACT)
            .find(|n| f64::from(*n) >= scaled)
            .unwrap_or(TOP_IMPACT),
    )
}

/// Rows the oracle tags table shows: the user's 10 most impactful tags, as
/// the hint and the guide say. The profile sends more so each card role can
/// reveal its own tags.
const TAG_TABLE_ROWS: usize = 10;

/// One row per oracle tag in the order sent (highest score first), the first
/// [`TAG_TABLE_ROWS`] only, its impact scaled so the top tag reads
/// [`TOP_IMPACT`].
pub(crate) fn tag_rows(tags: &[HttpProfileTag]) -> Vec<TagRow> {
    let max = tags.iter().map(|t| t.score).fold(0.0, f64::max);
    tags.iter()
        .take(TAG_TABLE_ROWS)
        .map(|t| TagRow {
            slug: t.slug.clone(),
            decks: t.decks,
            cards: t.cards,
            impact: impact(t.score, max),
        })
        .collect()
}

/// The profile's tags under `role`, highest score first.
pub(crate) fn role_tags(tags: &[HttpProfileTag], role: &str) -> Vec<String> {
    tags.iter()
        .filter(|t| t.role.as_deref() == Some(role))
        .map(|t| t.slug.clone())
        .collect()
}

/// An oracle tag's description with its Examples button, through the
/// screen's `OtagDescribe` and `OtagExamplesOpen` contexts.
#[component]
fn OtagDef(slug: String) -> Element {
    let describe = try_use_context::<OtagDescribe>().map(|d| d.0);
    let examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let def = describe
        .and_then(|cb| cb.call(slug.clone()))
        .unwrap_or_else(|| "No description yet".to_string());
    rsx! {
        div { class: "otag-def",
            p { class: "otag-def-text", "{def}" }
            if let Some(handler) = examples {
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

/// A tapped role's oracle tags from the profile as chips, each tappable for
/// its description eased open beneath them, the "Why this commander?" chips'
/// reveal. Renders nothing for an empty list, so no bare frame shows.
#[component]
fn RoleTags(tags: Vec<String>) -> Element {
    if tags.is_empty() {
        return rsx! {};
    }
    let mut open_tag: Signal<Option<String>> = use_signal(|| None);
    // The last opened tag, kept through a close so its description eases out.
    let mut shown_tag: Signal<Option<String>> = use_signal(|| None);
    let open = open_tag().filter(|slug| tags.contains(slug));
    let shown = shown_tag().filter(|slug| tags.contains(slug));
    let reveal_class = if open.is_some() {
        "keyword-reveal open"
    } else {
        "keyword-reveal"
    };
    rsx! {
        div { class: "otag-reveal-block",
            div { class: "card-detail-meta card-detail-otags",
                for tag in tags.iter().cloned() {
                    {
                        let slug = tag.clone();
                        rsx! {
                            button {
                                key: "{tag}",
                                class: if open.as_deref() == Some(tag.as_str()) { "keyword-chip active" } else { "keyword-chip" },
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    if open_tag().as_deref() == Some(slug.as_str()) {
                                        open_tag.set(None);
                                    } else {
                                        open_tag.set(Some(slug.clone()));
                                        shown_tag.set(Some(slug.clone()));
                                    }
                                },
                                "{tag}"
                            }
                        }
                    }
                }
            }
            div { class: "{reveal_class}",
                div { class: "keyword-reveal-inner",
                    if let Some(slug) = shown {
                        OtagDef { slug }
                    }
                }
            }
        }
    }
}

/// The oracle tags as a table: a header row of column labels, each number
/// column with its "?", then one row per tag with its chip and its Decks,
/// Cards and Impact. A tapped chip eases its description and Examples open
/// directly under its row in the deck charts' chip grammar, one row at a
/// time; tapping the open chip closes it.
#[component]
fn TagTable(rows: Vec<TagRow>) -> Element {
    let mut open_tag: Signal<Option<String>> = use_signal(|| None);
    // The last opened tag, kept through a close so its description eases out.
    let mut shown_tag: Signal<Option<String>> = use_signal(|| None);
    let open = open_tag();
    let shown = shown_tag();
    let is_open = |slug: &str| open.as_deref() == Some(slug);
    let tap = move |slug: String| {
        move |evt: MouseEvent| {
            evt.stop_propagation();
            if open_tag().as_deref() == Some(slug.as_str()) {
                open_tag.set(None);
            } else {
                open_tag.set(Some(slug.clone()));
                shown_tag.set(Some(slug.clone()));
            }
        }
    };
    rsx! {
        div { class: "tag-table",
            div { class: "tag-table-head",
                span { class: "settings-row-label", "Tag" }
            }
            div { class: "tag-table-head tag-table-head-p3",
                span { class: "settings-row-label", "Decks" }
                InfoButton { topic: HintTopic::SuggestionTagDecks }
            }
            div { class: "tag-table-head tag-table-head-p5",
                span { class: "settings-row-label", "Cards" }
                InfoButton { topic: HintTopic::SuggestionTagCards }
            }
            div { class: "tag-table-head tag-table-head-p2",
                span { class: "settings-row-label", "Impact" }
                InfoButton { topic: HintTopic::SuggestionTagImpact }
            }
            for row in rows.iter() {
                div { class: "tag-table-tag",
                    button {
                        class: if is_open(&row.slug) { "keyword-chip active" } else { "keyword-chip" },
                        onclick: tap(row.slug.clone()),
                        "{row.slug}"
                    }
                }
                span { class: "tag-table-num",
                    span { class: "stat-chip stat-chip-p3", "{row.decks}" }
                }
                span { class: "tag-table-num",
                    span { class: "stat-chip stat-chip-p5", "{row.cards}" }
                }
                span { class: "tag-table-num",
                    if let Some(impact) = row.impact {
                        span { class: "stat-chip stat-chip-p2", "{impact}" }
                    }
                }
                // Mounted shut under every row so opening one is a class
                // change the reveal can ease, not a fresh node.
                div {
                    class: if is_open(&row.slug) { "keyword-reveal open" } else { "keyword-reveal" },
                    style: "grid-column:1 / -1;",
                    div { class: "keyword-reveal-inner",
                        if shown.as_deref() == Some(row.slug.as_str()) {
                            OtagDef { slug: row.slug.clone() }
                        }
                    }
                }
            }
        }
    }
}

/// A Summary row whose value is an identity: the label, then the pips with
/// the name and count after them.
#[component]
fn SummaryIdentityRow(label: &'static str, value: SummaryValue) -> Element {
    rsx! {
        div { class: "settings-row",
            span { class: "settings-row-label", "{label}" }
            span { class: "info-row-value",
                IdentityPips { identity: value.pips, count: value.count }
            }
        }
    }
}

/// What the user builds, as sections over the suggestions list in the deck
/// profile's grammar: a Summary table (the deck count, the most common color
/// set and color) in the Budget section's rows, then two charts and a table.
/// The four share one accordion with Summary open on entry. Colors are
/// columns in WUBRG order; card roles are rows like the deck profile's role
/// chart with chip labels, a tapped role revealing its oracle tags from the
/// profile under its row; oracle tags are a [`TagTable`].
#[component]
pub(crate) fn BuilderProfileSections(profile: HttpBuilderProfile) -> Element {
    let summary = profile_summary(&profile);
    let open_section: Signal<Option<String>> = use_signal(|| Some(SUMMARY_TITLE.to_string()));
    let mut selected_role: Signal<Option<String>> = use_signal(|| None);
    let colors = color_bars(&profile.colors);
    let roles = role_bars(&profile.roles, &profile.tags);
    let tags = tag_rows(&profile.tags);
    let profile_tags = profile.tags;
    let role_reveal = use_callback(move |role: String| {
        let tags = role_tags(&profile_tags, &role);
        rsx! {
            RoleTags { tags }
        }
    });
    let toggle_role = use_callback(move |key: String| {
        if selected_role().as_deref() == Some(key.as_str()) {
            selected_role.set(None);
        } else {
            selected_role.set(Some(key));
        }
    });
    rsx! {
        CollapsibleSection { title: SUMMARY_TITLE, open_section,
            div { style: "display:flex;flex-direction:column;",
                div { class: "settings-row",
                    span { class: "settings-row-label", "Decks" }
                    span { class: "info-row-value", "{summary.decks}" }
                }
                if let Some(value) = summary.color_set {
                    SummaryIdentityRow { label: "Top colors", value }
                }
                if let Some(value) = summary.color {
                    SummaryIdentityRow { label: "Top color", value }
                }
            }
        }
        if !colors.is_empty() {
            CollapsibleSection {
                title: COLORS_TITLE,
                open_section,
                header_accessory: rsx! {
                    InfoButton { topic: HintTopic::SuggestionColorChart }
                },
                BarChart { bars: colors }
            }
        }
        if !roles.is_empty() {
            CollapsibleSection {
                title: ROLES_TITLE,
                open_section,
                header_accessory: rsx! {
                    InfoButton { topic: HintTopic::SuggestionRoleChart }
                },
                BarChart {
                    bars: roles,
                    orientation: BarOrientation::Horizontal,
                    on_bar: toggle_role,
                    selected: selected_role(),
                    reveal: role_reveal,
                }
            }
        }
        if !tags.is_empty() {
            CollapsibleSection {
                title: TAGS_TITLE,
                open_section,
                header_accessory: rsx! {
                    InfoButton { topic: HintTopic::SuggestionTagChart }
                },
                TagTable { rows: tags }
            }
        }
    }
}

/// The "Why this commander?" block under a suggestion's details: folded
/// under a chip in a list row (`foldable`), or open under a `chips-label`
/// heading like the details' Keywords and Card roles in the details sheet,
/// over the shared oracle tags grouped by card role (the card's own role-chip grammar)
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
    /// Folds the bullets under a "Why this commander?" chip, closed on mount,
    /// where a list row keeps them; unset, they sit open under a heading.
    #[props(default)]
    foldable: bool,
) -> Element {
    let mut reasons_open = use_signal(|| false);
    let context_describe = try_use_context::<OtagDescribe>().map(|d| d.0);
    let context_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let colors = color_lines(&reasons.colors, deck_count);
    if reasons.tags.is_empty() && colors.is_empty() {
        return rsx! {};
    }
    let (roles, tags_by_role, other_tags) = group_shared_tags(&reasons.tags);
    let describe_tag = describe_tag.or(context_describe);
    let on_examples = on_examples.or(context_examples);
    let bullets = rsx! {
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
    };
    rsx! {
        div { class: "card-detail-footer",
            if foldable {
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
                    div { class: "collapsible-inner", {bullets} }
                }
            } else {
                // Open, headed like the details' Keywords and Card roles.
                span { class: "chips-label", "Why this commander?" }
                {bullets}
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
                SuggestionReasons { reasons: suggestion.reasons, deck_count, foldable: true }
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

/// Screen summing up the user's playstyle over their top 10 suggested
/// commanders with the reasons behind each, Show chips narrowing them to one color identity, a Swipe
/// overlay going deeper, and Clear skips under More.
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
    // Art-crop thumbnails on by default; the Show row's Art chip toggles
    // them, as on the commander maybeboard.
    let mut show_row_art = use_signal(|| true);
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

    // The held ranking, 100 deep, in a plain signal so a dismissal can drop
    // its row in place. `reload` refetches: after a dismissal leaves the
    // filtered list short, when the Swipe overlay closes, and after Clear
    // skips. A refetch with entries present is silent (no skeleton flash).
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
                    c.commander_suggestions(HELD_SIZE, &s).await
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

    // The Show chip in effect: `None` is All. The chips are the identities
    // among the held list plus the chosen one, so a chip stays put after its
    // last commander is removed and the empty state says so.
    let active_identity: Signal<Option<Vec<Color>>> = use_signal(|| None);
    let identities = use_memo(move || {
        let held = entries();
        let chosen = active_identity();
        identities_present(held.iter().map(suggestion_colors).chain(chosen.as_deref()))
    });
    let filtered =
        use_memo(move || of_identity(&entries(), suggestion_colors, active_identity().as_deref()));
    // What Swipe deals: the same filtered list, with the deck count the
    // reasons count against.
    let pile = use_memo(move || SuggestedPile {
        suggestions: filtered(),
        deck_count: deck_count(),
        identity: active_identity(),
    });

    // Drops a row in place; the next held suggestion of that identity moves
    // up, and the server is asked again only when the filtered list runs
    // short of ten.
    let mut drop_row = move |oracle_id: Uuid| {
        entries
            .write()
            .retain(|s| s.card.scryfall_data.oracle_id != Some(oracle_id));
        let remaining = of_identity(
            &entries.peek(),
            suggestion_colors,
            active_identity.peek().as_deref(),
        )
        .len();
        if remaining < LIST_SIZE {
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
    let rows = shown(&filtered());

    rsx! {
        div { class: "screen",
            ScreenHeader { title: "Playstyle", hint: hint_open }

            HintDialog {
                open: hint_open,
                title: "Playstyle",
                HintLine { "Your colors, card roles and top oracle tags, read from every deck you own in any format" }
                HintBullets {
                    HintBullet { HintKey { "Create deck" } " begins a deck with that commander" }
                    HintBullet { HintKey { "Maybeboard" } " saves it to your commander maybeboard" }
                    HintBullet { HintKey { color: "--color-error", "Remove" } " hides it for a week and pulls the next one up" }
                    HintBullet { HintKey { "Swipe" } " deals the same ranking: " HintColored { color: "--color-success", "right" } " starts a deck, " HintColored { color: "--color-warning", "up" } " saves, " HintColored { color: "--color-error", "left" } " skips it" }
                    HintBullet { HintKey { "Show" } " narrows the list to one color identity" }
                }
                HintLine { "Below them, your top 10 commanders ranked against all your decks, best fit first. Swipe keeps going down the same ranking" }
            }

            div { class: "screen-content",
                // Bounded like the commander maybeboard: screen-content centers
                // its children, so an unbounded column would grow to content
                // width and overflow the viewport on long rows.
                div {
                    class: "flex-col",
                    style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
                    if is_loading() {
                        // First load only: a refetch keeps the data in place.
                        // The profile ghosts as its four sections, Summary open
                        // on three rows; the list's group and its header are
                        // static chrome, so they render real while the Show
                        // chips and the data rows ghost (the closed-vocabulary
                        // skeleton rule).
                        div { style: "width:100%;display:flex;flex-direction:column;gap:var(--panel-gap);",
                            BuilderProfileSkeleton {}
                            ShowChipRowSkeleton {}
                            div { class: "maybeboard-list card-group row-enter",
                                div { class: "card-group-header", "{LIST_TITLE}" }
                                for i in 0..6 {
                                    div { key: "{i}", class: "skeleton-card-row",
                                        div { class: "skeleton-bar skeleton-card-thumb" }
                                        div { class: "skeleton-bar skeleton-card-bar-row" }
                                    }
                                }
                            }
                        }
                    } else if load_failed() {
                        div { class: "message-empty",
                            span { class: "chip-note", "Could not load your suggestions" }
                        }
                    } else if cold_start {
                        div { class: "message-empty",
                            span { class: "chip-note", "Add cards to a deck in any format to see your playstyle and the commanders that fit it." }
                        }
                    } else {
                        // The deck view's column of sibling sections.
                        div { style: "width:100%;display:flex;flex-direction:column;gap:var(--panel-gap);",
                            if let Some(profile) = profile() {
                                BuilderProfileSections { profile }
                            }
                            // The commander maybeboard's Show row on the grid, over
                            // the rows it narrows, the Art toggle at its end.
                            ShowIdentityChips { active_identity, identities: identities(),
                                Chip {
                                    selected: show_row_art(),
                                    onclick: move |_| show_row_art.set(!show_row_art()),
                                    "Art"
                                }
                            }
                            if !has_entries {
                                div { class: "message-empty",
                                    span { class: "chip-note", "No suggestions right now" }
                                }
                            } else if rows.is_empty() {
                                div { class: "message-empty",
                                    span { class: "chip-note",
                                        {empty_identity_line(&active_identity().unwrap_or_default())}
                                    }
                                }
                            } else {
                                div { class: "maybeboard-list card-group row-enter",
                                    div { class: "card-group-header", "{LIST_TITLE}" }
                                    for suggestion in rows {
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
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Action,
                    // Back to wherever it opened from: Home's panel or the
                    // Decks screen's More menu.
                    onclick: move |_| navigator.go_back(),
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

            // The shared overlay in its suggested mode deals the held list as
            // the Show chip narrows it: right seeds a new deck with the pick,
            // up saves it, left dismisses it. Its Filter sheet holds the same
            // Show chips over the same signal.
            SwipeSelect {
                host_screen: screen::SUGGESTED_COMMANDERS,
                open: swipe_open,
                mode: swipe_mode,
                suggested_pile: Some(pile.into()),
                suggested_filter: rsx! {
                    ShowIdentityChips { active_identity, identities: identities() }
                },
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

    /// A commander as the helpers see it: a name and its colors.
    type Named = (&'static str, Vec<Color>);

    fn commander(name: &'static str, colors: &[Color]) -> Named {
        (name, colors.to_vec())
    }

    fn colors_of(commander: &Named) -> &[Color] {
        &commander.1
    }

    fn names(commanders: &[Named]) -> Vec<&str> {
        commanders.iter().map(|c| c.0).collect()
    }

    use Color::{Black, Blue, Green, Red, White};

    #[test]
    fn identity_of_runs_wubrg_once_each() {
        assert_eq!(identity_of(&[Red, White, Red]), vec![White, Red]);
        assert!(identity_of(&[]).is_empty());
    }

    #[test]
    fn identities_present_are_distinct_fewest_colors_first_colorless_last() {
        let held = [
            commander("Narset", &[White, Blue, Red]),
            commander("Karlov", &[Black, White]),
            commander("Kozilek", &[]),
            commander("Krenko", &[Red]),
            commander("Talrand", &[Blue]),
            commander("Ghired", &[Red, Green, White]),
            commander("Thalia", &[White, Black]),
            commander("Zada", &[Red]),
        ];
        assert_eq!(
            identities_present(held.iter().map(colors_of)),
            vec![
                vec![Blue],
                vec![Red],
                vec![White, Black],
                vec![White, Blue, Red],
                vec![White, Red, Green],
                vec![],
            ]
        );
        assert!(identities_present(std::iter::empty()).is_empty());
        let chosen: &[Color] = &[Green];
        assert_eq!(
            identities_present(held.iter().map(colors_of).chain([chosen])),
            vec![
                vec![Blue],
                vec![Red],
                vec![Green],
                vec![White, Black],
                vec![White, Blue, Red],
                vec![White, Red, Green],
                vec![],
            ],
            "a chosen identity without a commander still gets its chip"
        );
    }

    #[test]
    fn of_identity_keeps_server_order_and_all_passes_everything() {
        let held = [
            commander("Narset", &[White, Blue, Red]),
            commander("Krenko", &[Red]),
            commander("Kozilek", &[]),
            commander("Zada", &[Red]),
            commander("Ghired", &[Red, Green, White]),
        ];
        assert_eq!(
            names(&of_identity(&held, colors_of, Some(&[Red]))),
            vec!["Krenko", "Zada"]
        );
        assert_eq!(
            names(&of_identity(&held, colors_of, Some(&[]))),
            vec!["Kozilek"]
        );
        assert_eq!(
            names(&of_identity(&held, colors_of, Some(&[Red, Green]))),
            Vec::<&str>::new(),
            "a superset identity is not a match"
        );
        assert_eq!(of_identity(&held, colors_of, None).len(), held.len());
    }

    #[test]
    fn shown_is_the_first_ten() {
        let held: Vec<usize> = (0..25).collect();
        let rows = shown(&held);
        assert_eq!(rows.len(), LIST_SIZE);
        assert_eq!(rows.first().copied(), Some(0));
        assert_eq!(rows.last().copied(), Some(9));
        assert_eq!(shown(&[1, 2, 3]).len(), 3, "a short list is shown whole");
    }

    #[test]
    fn empty_identity_line_names_the_identity() {
        assert_eq!(
            empty_identity_line(&[Red, White, Blue]),
            "No Jeskai commanders in your top 100"
        );
        assert_eq!(
            empty_identity_line(&[Blue, Black, Red, Green]),
            "No Glint-Eye commanders in your top 100"
        );
        assert_eq!(
            empty_identity_line(&[Green]),
            "No mono-green commanders in your top 100"
        );
        assert_eq!(
            empty_identity_line(&[]),
            "No colorless commanders in your top 100"
        );
        assert_eq!(
            empty_identity_line(&[White, Blue, Black, Red, Green]),
            "No five-color commanders in your top 100"
        );
    }

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

    fn value(pips: &[Color], count: &str) -> SummaryValue {
        SummaryValue {
            pips: pips.to_vec(),
            count: count.to_string(),
        }
    }

    #[test]
    fn a_single_color_is_its_pip_alone_and_others_are_named() {
        assert_eq!(identity_label(&[Blue]), None);
        assert_eq!(identity_label(&[White, Blue, Red]), Some("Jeskai"));
        assert_eq!(identity_label(&[]), Some("Colorless"));
    }

    #[test]
    fn profile_summary_counts_decks_with_the_top_set_and_color() {
        let summary = profile_summary(&profile(
            7,
            vec![color("U", 5), color("W", 4)],
            Some(top(&["R", "W", "U"], 4)),
        ));
        assert_eq!(summary.decks, "7");
        assert_eq!(
            summary.color_set,
            Some(value(&[White, Blue, Red], "(4)")),
            "pips in WUBRG order whatever order the server sent"
        );
        assert_eq!(summary.color, Some(value(&[Blue], "(5)")));

        let summary = profile_summary(&profile(1, vec![color("B", 1)], Some(top(&["B"], 1))));
        assert_eq!(summary.decks, "1");
        assert_eq!(summary.color_set, Some(value(&[Black], "(1)")));
        assert_eq!(summary.color, Some(value(&[Black], "(1)")));
    }

    #[test]
    fn profile_summary_omits_what_the_profile_lacks() {
        let summary = profile_summary(&profile(3, vec![color("U", 2)], None));
        assert_eq!(summary.color_set, None, "no identity repeats");
        assert_eq!(summary.color, Some(value(&[Blue], "(2)")));

        let summary = profile_summary(&profile(2, vec![], Some(top(&[], 2))));
        assert_eq!(summary.color_set, Some(value(&[], "(2)")));
        assert_eq!(summary.color, None, "every deck colorless");

        let summary = profile_summary(&profile(2, vec![color("C", 2), color("G", 1)], None));
        assert_eq!(
            summary.color,
            Some(value(&[Green], "(1)")),
            "a code outside WUBRG is skipped"
        );
    }

    fn profile_tag(
        slug: &str,
        role: Option<&str>,
        decks: u32,
        cards: u32,
        score: f64,
    ) -> HttpProfileTag {
        HttpProfileTag {
            slug: slug.to_string(),
            role: role.map(str::to_string),
            decks,
            cards,
            score,
        }
    }

    fn bar(key: &str, label: &str, count: usize, pct: u32) -> Bar {
        Bar {
            key: key.to_string(),
            label: label.to_string(),
            count,
            pct,
            revealable: true,
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
        let tags = [
            profile_tag("sacrifice-outlet", Some("sacrifice"), 6, 23, 1.6),
            profile_tag("treasure", Some("ramp"), 1, 4, 0.2),
        ];
        assert_eq!(
            role_bars(&roles, &tags),
            vec![
                bar("sacrifice", "sacrf", 50, 100),
                bar("ramp", "ramp", 1, 4)
            ],
            "a nonzero bar is at least the 4% sliver"
        );
    }

    #[test]
    fn a_role_with_no_tags_sent_is_not_revealable() {
        let roles = [
            HttpProfileRole {
                role: "sacrifice".to_string(),
                decks: 2,
            },
            HttpProfileRole {
                role: "ramp".to_string(),
                decks: 1,
            },
        ];
        let tags = [profile_tag("aristocrats", Some("sacrifice"), 2, 5, 0.4)];
        let revealable: Vec<(String, bool)> = role_bars(&roles, &tags)
            .into_iter()
            .map(|b| (b.key, b.revealable))
            .collect();
        assert_eq!(
            revealable,
            vec![("sacrifice".to_string(), true), ("ramp".to_string(), false)]
        );
    }

    #[test]
    fn tag_rows_show_the_top_ten_while_roles_read_every_tag_sent() {
        let mut tags: Vec<HttpProfileTag> = (0..20)
            .map(|i| profile_tag(&format!("removal-{i:02}"), Some("removal"), 1, 1, 1.0))
            .collect();
        tags.push(profile_tag("mana-rock", Some("ramp"), 1, 1, 0.5));
        let rows = tag_rows(&tags);
        assert_eq!(rows.len(), TAG_TABLE_ROWS);
        assert!(rows.iter().all(|r| r.slug.starts_with("removal-")));
        assert_eq!(role_tags(&tags, "ramp"), vec!["mana-rock"]);
    }

    #[test]
    fn tag_rows_keep_the_order_sent_and_scale_impact_to_the_top_tag() {
        let tags = [
            profile_tag("sacrifice-outlet", Some("sacrifice"), 6, 23, 1.6),
            profile_tag("treasure", Some("ramp"), 3, 40, 0.8),
            profile_tag("aristocrats", Some("sacrifice"), 3, 9, 0.4),
            profile_tag("typal-goblin", None, 1, 2, 0.01),
            profile_tag("blink", None, 1, 1, 0.0),
        ];
        let rows = tag_rows(&tags);
        let got: Vec<(&str, u32, u32, Option<u32>)> = rows
            .iter()
            .map(|r| (r.slug.as_str(), r.decks, r.cards, r.impact))
            .collect();
        assert_eq!(
            got,
            vec![
                ("sacrifice-outlet", 6, 23, Some(100)),
                ("treasure", 3, 40, Some(50)),
                ("aristocrats", 3, 9, Some(25)),
                ("typal-goblin", 1, 2, Some(1)),
                ("blink", 1, 1, Some(0)),
            ],
            "the order sent, impact rounded against the top score"
        );
        assert!(tag_rows(&[]).is_empty());
    }

    #[test]
    fn tag_rows_round_impact_to_the_nearest_whole() {
        let rows = tag_rows(&[
            profile_tag("a", None, 1, 1, 3.0),
            profile_tag("b", None, 1, 1, 2.0),
            profile_tag("c", None, 1, 1, 1.0),
        ]);
        let got: Vec<Option<u32>> = rows.iter().map(|r| r.impact).collect();
        assert_eq!(got, vec![Some(100), Some(67), Some(33)]);
    }

    #[test]
    fn tag_rows_with_no_score_show_no_impact() {
        let rows = tag_rows(&[
            profile_tag("treasure", None, 2, 5, 0.0),
            profile_tag("blink", None, 1, 3, 0.0),
        ]);
        let got: Vec<(u32, u32, Option<u32>)> =
            rows.iter().map(|r| (r.decks, r.cards, r.impact)).collect();
        assert_eq!(
            got,
            vec![(2, 5, None), (1, 3, None)],
            "an older server's rows keep their counts and leave impact empty"
        );
    }

    #[test]
    fn role_tags_keep_the_order_sent() {
        let tags = [
            profile_tag("sacrifice-outlet", Some("sacrifice"), 6, 23, 1.6),
            profile_tag("treasure", Some("ramp"), 3, 40, 0.8),
            profile_tag("aristocrats", Some("sacrifice"), 3, 9, 0.4),
            profile_tag("typal-goblin", None, 1, 2, 0.01),
        ];
        assert_eq!(
            role_tags(&tags, "sacrifice"),
            vec!["sacrifice-outlet", "aristocrats"]
        );
        assert!(role_tags(&tags, "draw").is_empty());
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
