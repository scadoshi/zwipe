//! Playstyle's profile: the Colors, Summary, Card roles and Oracle tags
//! sections over the suggestions.

use std::collections::HashSet;

use components::{
    BarChart, BarOrientation, ColorBars, ColorsEmpty, DataBar, DeckFold, OtagDef, RoleTags,
};
use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::http::contracts::suggestions::HttpBuilderProfile;
use zwipe_ui::playstyle::{
    SummaryValue, TagBars, TagRow, profile_summary, role_bars, role_tags, tag_rows,
};
use zwipe_ui::screens::playstyle::{SUMMARY_DECKS, SUMMARY_TOP_COLOR, SUMMARY_TOP_COLORS};

use super::chips::IdentityPips;
use crate::inbound::cards::charts::{chart_bars, color_bars};
use crate::inbound::cards::otag::{OtagDescribe, OtagExamplesOpen};
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::chrome::section::CollapsibleSection;

/// The Colors section's title, the one open on entry.
pub(crate) const COLORS_TITLE: &str = "Colors";
const SUMMARY_TITLE: &str = "Summary";
const ROLES_TITLE: &str = "Card roles";
const TAGS_TITLE: &str = "Oracle tags";

/// One profile section: a [`DeckFold`] keyed by its title when `folds` is
/// given (a wide window), else a section of the accordion over
/// `open_section`. `accessory` sits at the header's right end.
#[component]
fn ProfilePanel(
    title: &'static str,
    open_section: Signal<Option<String>>,
    folds: Option<Signal<HashSet<String>>>,
    #[props(default)] accessory: Option<Element>,
    children: Element,
) -> Element {
    if let Some(folded) = folds {
        rsx! {
            DeckFold { fold_key: title, title, folded, accessory,
                div { class: "web-playstyle-fold-body", {children} }
            }
        }
    } else {
        rsx! {
            CollapsibleSection { title, open_section, header_accessory: accessory, {children} }
        }
    }
}

/// What the user builds: Colors, Summary, Card roles (a role chart whose
/// roles reveal their tags) and Oracle tags (the [`TagTable`]). With `folds`
/// (a wide window) each is a fold, all open on entry; without, they are one
/// accordion with Colors open. The host draws Summary as its hero on a wide
/// window, so this one hides there.
#[component]
pub fn BuilderProfileSections(
    profile: HttpBuilderProfile,
    folds: Option<Signal<HashSet<String>>>,
) -> Element {
    let summary = profile.clone();
    let mut selected_role: Signal<Option<String>> = use_signal(|| None);
    let colors = profile.color_map.as_ref().and_then(color_bars);
    let open_section: Signal<Option<String>> = use_signal(|| Some(COLORS_TITLE.to_string()));
    let roles = chart_bars(role_bars(&profile.roles, &profile.tags));
    let tags = tag_rows(&profile.tags, profile.deck_count);
    let profile_tags = profile.tags;
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let role_reveal = use_callback(move |role: String| {
        let tags = role_tags(&profile_tags, &role);
        rsx! {
            RoleTags { tags, describe_tag, on_examples }
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
        div { class: "web-playstyle-cell web-playstyle-colors",
            ProfilePanel {
                title: COLORS_TITLE,
                open_section,
                folds,
                accessory: rsx! { InfoButton { topic: HintTopic::Colors } },
                if let Some(bars) = colors {
                    ColorBars { bars }
                } else {
                    ColorsEmpty {}
                }
            }
        }
        div { class: "web-playstyle-summary-inline",
            ProfileSummary { profile: summary, open_section, folds }
        }
        if !roles.is_empty() {
            div { class: "web-playstyle-cell web-playstyle-roles",
                ProfilePanel {
                    title: ROLES_TITLE,
                    open_section,
                    folds,
                    accessory: rsx! { InfoButton { topic: HintTopic::SuggestionRoleChart } },
                    BarChart {
                        bars: roles,
                        orientation: BarOrientation::Horizontal,
                        on_bar: toggle_role,
                        selected: selected_role(),
                        reveal: role_reveal,
                    }
                }
            }
        }
        if !tags.is_empty() {
            div { class: "web-playstyle-cell web-playstyle-tags",
                ProfilePanel {
                    title: TAGS_TITLE,
                    open_section,
                    folds,
                    accessory: rsx! { InfoButton { topic: HintTopic::SuggestionTagChart } },
                    TagTable { rows: tags }
                }
            }
        }
    }
}

/// The Summary section: the deck count, then the top color set and the top
/// color with their deck counts. In the profile accordion below the
/// breakpoint, and Playstyle's hero on a wide window, a fold there through
/// `folds`.
#[component]
pub fn ProfileSummary(
    profile: HttpBuilderProfile,
    open_section: Signal<Option<String>>,
    folds: Option<Signal<HashSet<String>>>,
) -> Element {
    let summary = profile_summary(&profile);
    rsx! {
        ProfilePanel { title: SUMMARY_TITLE, open_section, folds,
            div { class: "flex-col",
                div { class: "settings-row",
                    span { class: "settings-row-label", "{SUMMARY_DECKS}" }
                    span { class: "info-row-value", "{summary.decks}" }
                }
                if let Some(value) = summary.color_set {
                    SummaryIdentityRow { label: SUMMARY_TOP_COLORS, value }
                }
                if let Some(value) = summary.color {
                    SummaryIdentityRow {
                        label: SUMMARY_TOP_COLOR,
                        name: value.pips.first().map(Color::to_long_name),
                        value,
                    }
                }
            }
        }
    }
}

/// A Summary row whose value is an identity: `name` when given (the long
/// color name), then the identity's pips and its deck count.
#[component]
fn SummaryIdentityRow(
    label: &'static str,
    value: SummaryValue,
    #[props(default)] name: Option<String>,
) -> Element {
    // A shown name says the color, so the pips say nothing more.
    let speak = name.is_none();
    rsx! {
        div { class: "settings-row",
            span { class: "settings-row-label", "{label}" }
            span { class: "info-row-value identity-pips",
                if let Some(name) = name {
                    span { "{name}" }
                }
                IdentityPips { identity: value.pips, count: value.count, speak }
            }
        }
    }
}

/// The top oracle tags as a table: Tag, Decks, Cards and Impact, each
/// number column with its `?`. Tapping a tag eases its description open
/// under the row, one at a time.
#[component]
fn TagTable(rows: Vec<TagRow>) -> Element {
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let mut open_tag: Signal<Option<String>> = use_signal(|| None);
    // The last opened tag, kept through a close so its description eases out.
    let mut shown_tag: Signal<Option<String>> = use_signal(|| None);
    let open = open_tag();
    let shown = shown_tag();
    let is_open = |slug: &str| open.as_deref() == Some(slug);
    let bars = TagBars::of(&rows);
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
        table { class: "data-table tag-data-table",
            thead {
                tr {
                    th { scope: "col", "Tag" }
                    th { scope: "col", class: "data-table-bar",
                        "Decks"
                        InfoButton { topic: HintTopic::SuggestionTagDecks }
                    }
                    th { scope: "col", class: "data-table-bar",
                        "Cards"
                        InfoButton { topic: HintTopic::SuggestionTagCards }
                    }
                    th { scope: "col", class: "data-table-bar",
                        "Impact"
                        InfoButton { topic: HintTopic::SuggestionTagImpact }
                    }
                }
            }
            tbody {
                for row in rows.iter() {
                    Fragment { key: "{row.slug}",
                        tr {
                            th { scope: "row",
                                button {
                                    class: if is_open(&row.slug) { "chip selected" } else { "chip" },
                                    onclick: tap(row.slug.clone()),
                                    "{row.slug}"
                                }
                            }
                            td { class: "data-table-bar",
                                DataBar { percent: bars.decks(row), tone: "p3", value: row.decks.to_string() }
                            }
                            td { class: "data-table-bar",
                                DataBar { percent: bars.cards(row), tone: "p5", value: row.cards.to_string() }
                            }
                            td { class: "data-table-bar",
                                if let (Some(impact), Some(percent)) = (row.impact, bars.impact(row)) {
                                    DataBar { percent, tone: "p2", value: impact.to_string() }
                                }
                            }
                        }
                        tr { class: "data-table-detail",
                            td { colspan: "4",
                                div { class: if is_open(&row.slug) { "keyword-reveal open" } else { "keyword-reveal" },
                                    div { class: "keyword-reveal-inner",
                                        if shown.as_deref() == Some(row.slug.as_str()) {
                                            OtagDef { slug: row.slug.clone(), describe_tag, on_examples }
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
}
