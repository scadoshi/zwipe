//! Playstyle screen: what the user builds, read from every deck they
//! own in any format, over the top 10 commanders ranked against those decks,
//! each with the reasons it ranked, styled like the commander maybeboard.
//!
//! The screen fetches the ranking 100 deep once and shows the first 10. A
//! Show row atop the list holds one chip per color identity among the 100,
//! each with its pips and name; a tapped chip narrows the held list to that identity, and the Art chip
//! toggles the row thumbnails. An expanded row offers **Create deck** (seeds
//! the create screen), **Maybeboard** (saves to the commander maybeboard) and
//! **Remove** (hides it for a week). The Swipe button opens the shared
//! overlay in its suggested mode, which deals the same held list and whose
//! Filter sheet holds the same Show chips. Every action
//! that removes a row fills from the held list and refetches only when fewer
//! than ten remain, so the list stays a full top 10. Above the list, sections
//! in the deck profile's grammar sum up what the user builds, the profile the
//! ranking reads: a bar per color of the cards, a Summary table (the
//! deck count, the top colors and the top single color), a chart of the
//! card roles and a table of the oracle tags across the decks. They share
//! one accordion with Colors open on entry. Below the list, the staples
//! (up to 100 cards in the most decks) show as the shared card rows in the
//! list's group grammar, the first 10 that match their own Type and Color chip
//! rows, which the Show row does not touch. With no deck that has cards to
//! read from, the screen shows a single line instead of the sections and the
//! list.
//!
//! What the screen knows and does is `zwipe_ui::screens::playstyle`; this
//! file draws its state and performs its effects.

use components::{
    AlertDialog, Bar, BarChart, BarOrientation, Button, ButtonVariant, CardRoleChips, Chip,
    ColorBars, DialogAction, OtagDef, RoleTags, Toasts, use_toast,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_type::CardType;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_core::http::contracts::suggestions::{
    HttpCommanderSuggestion, HttpStaple, HttpSuggestionReasons,
};
use zwipe_ui::colors::ColorCounts;
use zwipe_ui::generation::Ticket;
use zwipe_ui::playstyle::{
    self, ProfileSummary, StapleColor, SummaryValue, TagRow, identity_label,
};
use zwipe_ui::screens::playstyle::{
    self as model, Action, Destination, Effect, Section, State, Status,
};

use super::components::collapsible_section::CollapsibleSection;
use super::components::danger_zone::DangerZoneAction;
use super::components::skeletons::{BuilderProfileSkeleton, ShowChipRowSkeleton};
use super::components::swipe_select::{SuggestedPile, SwipeMode, SwipeSelect};
use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::catalog_cache::CatalogCache;
use crate::inbound::components::hint_dialog::{self, OneTimeHintDialog};
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::home_play_style::ColorsEmpty;
use crate::inbound::components::info_button::InfoButton;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::vocabulary::{Screen, screen};
use crate::inbound::components::ui_toast;
use crate::inbound::router::Router;
use crate::inbound::screens::deck::card::components::card_row::{
    CardRow, OtagDescribe, OtagExamplesOpen, ShowRowArt,
};
use crate::inbound::screens::deck::card::components::image_preview::ImagePreview;
use crate::inbound::screens::deck::card::components::printing_sheet::PrintingSheet;
use crate::inbound::screens::deck::create::CreateDeckCommanderSeed;
use crate::inbound::screens::oracle_tag_examples::OracleTagExamples;

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
pub(crate) fn IdentityPips(
    identity: Vec<Color>,
    count: Option<String>,
    /// Leads with the identity's name; off where the host sets the name apart.
    #[props(default = true)]
    show_name: bool,
) -> Element {
    let name = identity_label(&identity).filter(|_| show_name);
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
/// toggle). Single select over `active_identity`, `None` being All; a tap
/// reports the chip, `None` for All, and the model decides the pick. The
/// screen and the Swipe overlay's Filter sheet both render it over the same
/// model, so a tap in either filters both.
#[component]
pub(crate) fn ShowIdentityChips(
    active_identity: Option<Vec<Color>>,
    identities: Vec<Vec<Color>>,
    on_pick: EventHandler<Option<Vec<Color>>>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "chip-row",
            span { class: "stat-chip stat-chip-p1 chip-row-label", {model::SHOW} }
            Chip {
                selected: active_identity.is_none(),
                onclick: move |_| on_pick.call(None),
                {model::ALL}
            }
            for identity in identities {
                Chip {
                    key: "{identity_key(&identity)}",
                    selected: active_identity.as_deref() == Some(identity.as_slice()),
                    onclick: {
                        let identity = identity.clone();
                        move |_| on_pick.call(Some(identity.clone()))
                    },
                    IdentityPips { identity: identity.clone() }
                }
            }
            {children}
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
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
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
                span { class: "settings-row-label", {model::TAG_COLUMN} }
            }
            div { class: "tag-table-head tag-table-head-p3",
                span { class: "settings-row-label", {model::DECKS_COLUMN} }
                InfoButton { topic: HintTopic::SuggestionTagDecks }
            }
            div { class: "tag-table-head tag-table-head-p5",
                span { class: "settings-row-label", {model::CARDS_COLUMN} }
                InfoButton { topic: HintTopic::SuggestionTagCards }
            }
            div { class: "tag-table-head tag-table-head-p2",
                span { class: "settings-row-label", {model::IMPACT_COLUMN} }
                InfoButton { topic: HintTopic::SuggestionTagImpact }
            }
            for row in rows.iter() {
                div { class: "tag-table-tag",
                    button {
                        class: if is_open(&row.slug) { "chip selected" } else { "chip" },
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
                            OtagDef { slug: row.slug.clone(), describe_tag, on_examples }
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
fn SummaryIdentityRow(
    label: &'static str,
    value: SummaryValue,
    /// A name before the pips where they draw none: the single top color's.
    #[props(default)]
    name: Option<String>,
) -> Element {
    rsx! {
        div { class: "settings-row",
            span { class: "settings-row-label", "{label}" }
            span { class: "info-row-value identity-pips",
                if let Some(name) = name {
                    span { "{name}" }
                }
                IdentityPips { identity: value.pips, count: value.count }
            }
        }
    }
}

/// One staple: the shared expandable card row with its percent of the user's
/// decks as a tag where a deck list puts copies, and the deck count under its
/// roles when expanded. No actions beyond the row's own Printing.
#[component]
fn StapleRow(
    staple: HttpStaple,
    deck_count: u32,
    expanded_card: Signal<Option<Uuid>>,
    preview_card: Signal<Option<(ScryfallData, usize)>>,
    preview_dismissing: Signal<bool>,
    on_printing: EventHandler<Card>,
) -> Element {
    let decks_label = playstyle::staple_decks(staple.decks, deck_count);
    rsx! {
        CardRow {
            card: staple.card,
            qty: i32::try_from(staple.decks).unwrap_or(i32::MAX),
            qty_tags: playstyle::staple_percent(staple.decks, deck_count).into_iter().collect::<Vec<String>>(),
            expanded_card,
            preview_card,
            preview_dismissing,
            on_printing: move |c: Card| on_printing.call(c),
            footer: rsx! {
                div { class: "keyword-chips",
                    span { class: "detail-chip detail-chip-pt", "{decks_label}" }
                }
            },
        }
    }
}

/// The Staples Type row in the Show row's grammar: the label, All, then one
/// chip per type in `types`. Single select over `active_type`, `None` being
/// All; a tap reports the chip, `None` for All.
#[component]
fn StapleTypeChips(
    active_type: Option<CardType>,
    types: Vec<CardType>,
    on_pick: EventHandler<Option<CardType>>,
) -> Element {
    rsx! {
        div { class: "chip-row",
            span { class: "stat-chip stat-chip-p1 chip-row-label", {model::TYPE} }
            Chip {
                selected: active_type.is_none(),
                onclick: move |_| on_pick.call(None),
                {model::ALL}
            }
            for card_type in types {
                Chip {
                    key: "{card_type}",
                    selected: active_type == Some(card_type),
                    onclick: move |_| on_pick.call(Some(card_type)),
                    "{card_type}"
                }
            }
        }
    }
}

/// The Staples Color row in the Show row's grammar: the label, All, then one
/// chip per entry in `colors`, a color as its pip and colorless by name.
/// Single select over `active_color`, `None` being All; a tap reports the
/// chip, `None` for All.
#[component]
fn StapleColorChips(
    active_color: Option<StapleColor>,
    colors: Vec<StapleColor>,
    on_pick: EventHandler<Option<StapleColor>>,
) -> Element {
    rsx! {
        div { class: "chip-row",
            span { class: "stat-chip stat-chip-p1 chip-row-label", {model::COLOR} }
            Chip {
                selected: active_color.is_none(),
                onclick: move |_| on_pick.call(None),
                {model::ALL}
            }
            for chip in colors {
                Chip {
                    key: "{identity_key(&chip.pips())}",
                    selected: active_color == Some(chip),
                    onclick: move |_| on_pick.call(Some(chip)),
                    IdentityPips { identity: chip.pips() }
                }
            }
        }
    }
}

/// The staples under the suggestions list: the Type and Color rows, then
/// `rows`, the staples they match, as [`StapleRow`]s in a group headed like
/// the list's, or `message` when none match.
#[component]
fn StaplesPanel(
    rows: Vec<HttpStaple>,
    message: Option<&'static str>,
    types: Vec<CardType>,
    colors: Vec<StapleColor>,
    active_type: Option<CardType>,
    active_color: Option<StapleColor>,
    deck_count: u32,
    expanded_card: Signal<Option<Uuid>>,
    preview_card: Signal<Option<(ScryfallData, usize)>>,
    preview_dismissing: Signal<bool>,
    on_printing: EventHandler<Card>,
    on_action: EventHandler<Action>,
) -> Element {
    rsx! {
        // One block, so the two rows stack at the chip rows' own spacing
        // instead of the screen column's panel gap.
        div {
            StapleTypeChips {
                active_type,
                types,
                on_pick: move |picked: Option<CardType>| {
                    on_action.call(picked.map_or(Action::TypeAll, Action::ToggleType));
                },
            }
            StapleColorChips {
                active_color,
                colors,
                on_pick: move |picked: Option<StapleColor>| {
                    on_action.call(picked.map_or(Action::ColorAll, Action::ToggleColor));
                },
            }
        }
        if let Some(message) = message {
            div { class: "message-empty",
                span { class: "chip-note", {message} }
            }
        } else {
            div { class: "maybeboard-list card-group row-enter",
                div { class: "card-group-header card-group-header-with-hint",
                    span { {model::STAPLES_TITLE} }
                    InfoButton { topic: HintTopic::SuggestionStaples }
                }
                for staple in rows {
                    StapleRow {
                        key: "{staple.card.scryfall_data.id}",
                        staple,
                        deck_count,
                        expanded_card,
                        preview_card,
                        preview_dismissing,
                        on_printing,
                    }
                }
            }
        }
    }
}

/// What the user builds, as sections over the suggestions list in the deck
/// profile's grammar: Home's [`ColorBars`], then a
/// Summary table (the deck count, the most common color set and color) in
/// the Budget section's rows, a chart and a table. The four share one
/// accordion with Colors open on entry. Colors are a bar per color, or
/// Home's empty line when no deck has a card; card roles are rows like the deck
/// profile's role chart with chip labels, a tapped role revealing its oracle
/// tags (from `role_tags`) under its row; oracle tags are a [`TagTable`].
/// `sections` says which draw.
#[component]
pub(crate) fn BuilderProfileSections(
    sections: Vec<Section>,
    color_counts: Option<ColorCounts>,
    summary: ProfileSummary,
    roles: Vec<Bar>,
    tags: Vec<TagRow>,
    role_tags: Callback<String, Vec<String>>,
) -> Element {
    let mut selected_role: Signal<Option<String>> = use_signal(|| None);
    let open_section: Signal<Option<String>> = use_signal(|| {
        Section::ALL
            .into_iter()
            .find(|section| section.opens_on_entry())
            .map(|section| section.title().to_string())
    });
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let role_reveal = use_callback(move |role: String| {
        let tags = role_tags.call(role);
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
        if sections.contains(&Section::Colors) {
            CollapsibleSection {
                title: Section::Colors.title(),
                open_section,
                header_accessory: rsx! {
                    InfoButton { topic: HintTopic::Colors }
                },
                if let Some(counts) = color_counts {
                    ColorBars { bars: counts.bars() }
                } else {
                    ColorsEmpty {}
                }
            }
        }
        if sections.contains(&Section::Summary) {
            CollapsibleSection { title: Section::Summary.title(), open_section,
                div { style: "display:flex;flex-direction:column;",
                    div { class: "settings-row",
                        span { class: "settings-row-label", {model::SUMMARY_DECKS} }
                        span { class: "info-row-value", "{summary.decks}" }
                    }
                    if let Some(value) = summary.color_set {
                        SummaryIdentityRow { label: model::SUMMARY_TOP_COLORS, value }
                    }
                    if let Some(value) = summary.color {
                        SummaryIdentityRow {
                            label: model::SUMMARY_TOP_COLOR,
                            name: value.pips.first().map(Color::to_long_name),
                            value,
                        }
                    }
                }
            }
        }
        if sections.contains(&Section::Roles) {
            CollapsibleSection {
                title: Section::Roles.title(),
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
        if sections.contains(&Section::Tags) {
            CollapsibleSection {
                title: Section::Tags.title(),
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
    let Some(model::Reasons { tags, colors }) = model::reasons(&reasons, deck_count) else {
        return rsx! {};
    };
    let describe_tag = describe_tag.or(context_describe);
    let on_examples = on_examples.or(context_examples);
    let bullets = rsx! {
        ul { class: "card-detail-bullets",
            if let Some(tags) = tags {
                li {
                    {model::SHARES_TAGS}
                    CardRoleChips {
                        roles: tags.roles,
                        tags_by_role: tags.by_role,
                        other_tags: tags.other,
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
                    // The card rows' disclosure arrow at the left, as on every
                    // fold: sideways folded, down open.
                    span {
                        class: if reasons_open() { "chip-fold-arrow expanded" } else { "chip-fold-arrow" },
                        "\u{25B8}"
                    }
                    {model::WHY}
                }
                // The bullets stay mounted and ease open and closed under the chip.
                div { class: if reasons_open() { "collapsible open" } else { "collapsible" },
                    div { class: "collapsible-inner", {bullets} }
                }
            } else {
                // Open, headed like the details' Keywords and Card roles.
                span { class: "chips-label", {model::WHY} }
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
                    {model::CREATE_DECK}
                }
                if let Some(oid) = oracle_id {
                    button {
                        class: "card-action-btn",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            on_maybe.call(oid);
                        },
                        {model::MAYBEBOARD}
                    }
                    button {
                        class: "card-action-btn",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            on_dismiss.call(oid);
                        },
                        {model::REMOVE}
                    }
                }
            },
        }
    }
}

/// Playstyle's model with the handles that perform its effects. Copy, so
/// every handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    navigator: Navigator,
    toast: Toasts,
    authed: Authed,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    hint_open: Signal<bool>,
    seed: CreateDeckCommanderSeed,
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
        let authed = self.authed;
        match effect {
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(
                    hint.key(),
                    self.session,
                    self.client,
                    self.hint_open,
                );
            }
            Effect::FetchSuggestions(ticket) => {
                spawn(async move {
                    // try_run reports a failure and raises its error toast.
                    let reply = authed
                        .try_run("load_suggestions", |c, s| async move {
                            c.commander_suggestions(model::HELD_SIZE, &s).await
                        })
                        .await;
                    self.dispatch(match reply {
                        Ok(found) => Action::Loaded {
                            ticket,
                            found: Box::new(found),
                        },
                        Err(_) => Action::Failed { ticket },
                    });
                });
            }
            Effect::FetchDeckProfiles => {
                spawn(async move {
                    // Quiet: backs the create gate only; a failure leaves the
                    // gate open and telemetry records it.
                    let found = authed
                        .run_quiet("load_deck_profiles", |c, s| async move {
                            c.fetch_deck_profiles(&s).await
                        })
                        .await;
                    if let Some(profiles) = found {
                        self.dispatch(Action::DeckProfilesLoaded {
                            count: profiles.len(),
                        });
                    }
                });
            }
            Effect::AddToMaybeboard { ticket, oracle_id } => self.edit(
                "add_commander_maybeboard",
                ticket,
                Action::Saved { ticket },
                move |c, s| async move { c.add_commander_maybeboard_card(oracle_id, &s).await },
            ),
            Effect::DismissCommander { ticket, oracle_id } => self.edit(
                "dismiss_commander",
                ticket,
                Action::Dismissed { ticket },
                move |c, s| async move { c.dismiss_commander(oracle_id, &s).await },
            ),
            Effect::ClearDismissals(ticket) => self.edit(
                "clear_commander_dismissals",
                ticket,
                Action::Cleared { ticket },
                |c, s| async move { c.clear_commander_dismissals(&s).await },
            ),
            Effect::SeedCommander(card) => {
                let mut slot = self.seed.0;
                slot.set(Some(*card));
            }
            Effect::Navigate(destination) => match destination {
                // Back to wherever it opened from: Home's panel or the Decks
                // screen's More menu.
                Destination::Previous => self.navigator.go_back(),
                Destination::CreateDeck => {
                    self.navigator.push(Router::CreateDeck);
                }
            },
            Effect::Toast(toast) => ui_toast::raise(self.toast, toast),
        }
    }

    /// Runs the ranking edit `call`, reported under `op`, and answers with
    /// `done`, or with [`Action::Failed`] carrying `ticket` once the runner
    /// has raised its error toast.
    fn edit<T, Fut>(
        self,
        op: &'static str,
        ticket: Ticket,
        done: Action,
        call: impl FnOnce(ZwipeClient, Session) -> Fut + 'static,
    ) where
        T: 'static,
        Fut: Future<Output = Result<T, ClientError>> + 'static,
    {
        let authed = self.authed;
        spawn(async move {
            let reply = authed.run(op, call).await;
            self.dispatch(match reply {
                Some(_) => done,
                None => Action::Failed { ticket },
            });
        });
    }

    /// Starts a deck with `card`, through the create gate.
    fn create(self, card: Card) {
        let verified = self
            .session
            .peek()
            .as_ref()
            .is_none_or(|s| s.user.email_verified_at.is_some());
        self.dispatch(Action::Create {
            card: Box::new(card),
            verified,
        });
    }

    /// The Show chip tapped, `None` being All.
    fn pick_identity(self, picked: Option<Vec<Color>>) {
        self.dispatch(picked.map_or(Action::ShowAll, Action::ToggleIdentity));
    }
}

/// Screen summing up the user's playstyle over their top 10 suggested
/// commanders with the reasons behind each, Show chips narrowing them to one color identity, a Swipe
/// overlay going deeper, and Clear skips under More.
#[component]
pub fn SuggestedCommanders() -> Element {
    let driver = Driver {
        model: use_signal(State::new),
        navigator: use_navigator(),
        toast: use_toast(),
        authed: use_authed(Screen::SuggestedCommanders),
        session: use_context(),
        client: use_context(),
        hint_open: use_signal(|| false),
        seed: use_context(),
    };
    let auth_client = driver.client;
    // Auto-opens once per account, reopenable from the header "?".
    let hint_open = driver.hint_open;
    // Offers the hint and fetches the ranking and the decks once per mount,
    // while the screen first draws, so the hint opens with the screen.
    use_hook(move || driver.dispatch(Action::Load));
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
    let mut show_clear_dialog = use_signal(|| false);

    // Tells the model when the Swipe overlay closes: its swipes dismissed and
    // saved commanders the list must not show anymore, so it fetches again.
    let mut swipe_was_open = use_signal(|| false);
    use_effect(move || {
        if swipe_open() {
            swipe_was_open.set(true);
        } else if *swipe_was_open.peek() {
            swipe_was_open.set(false);
            driver.dispatch(Action::SwipeClosed);
        }
    });

    // What Swipe deals: the held list as the Show chip narrows it, with the
    // deck count the reasons count against.
    let pile = use_memo(move || {
        let state = driver.model.read();
        SuggestedPile {
            suggestions: state.filtered(),
            deck_count: state.deck_count(),
            identity: state.identity().map(<[Color]>::to_vec),
        }
    });

    // The tags a role bar reveals, read from the profile the model holds.
    let role_tags = use_callback(move |role: String| driver.model.read().role_tags(&role));

    let on_start = move |card: Card| driver.create(card);
    let on_maybe = move |oracle_id: Uuid| driver.dispatch(Action::Maybeboard(oracle_id));
    let on_dismiss = move |oracle_id: Uuid| driver.dispatch(Action::Dismiss(oracle_id));
    let on_action = move |action: Action| driver.dispatch(action);

    let clear_dismissals = use_callback(move |()| {
        driver.dispatch(Action::ClearSkips);
    });

    let state = driver.model.read();
    let status = state.status();
    let message = state.message();
    let shows_swipe = state.shows_swipe();
    let deck_count = state.deck_count();
    let sections = state.sections();
    let color_counts = state.colors();
    let summary = state.summary();
    let role_bars = state.role_bars();
    let tag_rows = state.tag_rows();
    let active_identity = state.identity().map(<[Color]>::to_vec);
    let identities = state.identities();
    let rows = state.rows();
    let list_message = state.list_message();
    let shows_staples = state.shows_staples();
    let staple_rows = state.matching_staples();
    let staples_message = state.staples_message();
    let staple_types = state.staple_types();
    let staple_colors = state.staple_colors();
    let staple_type = state.staple_type();
    let staple_color = state.staple_color();
    drop(state);

    rsx! {
        div { class: "screen",
            ScreenHeader { title: model::TITLE, hint: hint_open, back: move |()| driver.dispatch(Action::Back) }

            OneTimeHintDialog { open: hint_open, hint: OneTimeHint::SuggestedCommanders }

            div { class: "screen-content",
                // Bounded like the commander maybeboard: screen-content centers
                // its children, so an unbounded column would grow to content
                // width and overflow the viewport on long rows.
                div {
                    class: "flex-col",
                    style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
                    if status == Status::Loading {
                        // First load only: a refetch keeps the data in place.
                        // The profile ghosts as its five sections, Summary open
                        // on three rows, then the Show row and the list, its
                        // header title a bar.
                        div { style: "width:100%;display:flex;flex-direction:column;gap:var(--panel-gap);",
                            BuilderProfileSkeleton {}
                            ShowChipRowSkeleton {}
                            div { class: "maybeboard-list card-group row-enter",
                                div { class: "card-group-header card-group-header-with-hint",
                                    div { class: "skeleton-bar skeleton-card-bar-header-wide" }
                                    div { class: "skeleton-bar skeleton-chip skeleton-info-button" }
                                }
                                for i in 0..6 {
                                    div { key: "{i}", class: "skeleton-card-row",
                                        div { class: "skeleton-bar skeleton-card-thumb" }
                                        div { class: "skeleton-bar skeleton-card-bar-row" }
                                    }
                                }
                            }
                        }
                    } else if let Some(message) = message {
                        div { class: "message-empty",
                            span { class: "chip-note", {message} }
                        }
                    } else {
                        // The deck view's column of sibling sections.
                        div { style: "width:100%;display:flex;flex-direction:column;gap:var(--panel-gap);",
                            if let Some(summary) = summary {
                                BuilderProfileSections {
                                    sections,
                                    color_counts,
                                    summary,
                                    roles: role_bars,
                                    tags: tag_rows,
                                    role_tags,
                                }
                            }
                            // The commander maybeboard's Show row on the grid, over
                            // the rows it narrows, the Art toggle at its end.
                            ShowIdentityChips {
                                active_identity: active_identity.clone(),
                                identities: identities.clone(),
                                on_pick: move |picked| driver.pick_identity(picked),
                                Chip {
                                    selected: show_row_art(),
                                    onclick: move |_| show_row_art.set(!show_row_art()),
                                    {model::ART}
                                }
                            }
                            if let Some(line) = list_message {
                                div { class: "message-empty",
                                    span { class: "chip-note", {line} }
                                }
                            } else {
                                div { class: "maybeboard-list card-group row-enter",
                                    // Swipe sits in the list's header, so it
                                    // says what gets swiped.
                                    div { class: "card-group-header card-group-header-with-hint",
                                        span { {model::LIST_TITLE} }
                                        div { class: "card-group-header-actions",
                                            if shows_swipe {
                                                Button {
                                                    variant: ButtonVariant::Action,
                                                    onclick: move |_| swipe_open.set(true),
                                                    {model::SWIPE}
                                                }
                                            }
                                            InfoButton { topic: HintTopic::SuggestionList }
                                        }
                                    }
                                    for suggestion in rows {
                                        SuggestionRow {
                                            key: "{suggestion.card.scryfall_data.id}",
                                            suggestion,
                                            deck_count,
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
                            // Below the list in its group grammar, outside the
                            // Show filter, with its own Type and Color rows.
                            if shows_staples {
                                StaplesPanel {
                                    rows: staple_rows,
                                    message: staples_message,
                                    types: staple_types,
                                    colors: staple_colors,
                                    active_type: staple_type,
                                    active_color: staple_color,
                                    deck_count,
                                    expanded_card,
                                    preview_card,
                                    preview_dismissing,
                                    on_printing: move |c: Card| {
                                        printing_card.set(Some(c));
                                        printing_open.set(true);
                                    },
                                    on_action,
                                }
                            }
                            DangerZoneAction {
                                label: model::CLEAR_SKIPS,
                                button: model::CLEAR,
                                onclick: move |()| show_clear_dialog.set(true),
                            }
                        }
                    }
                }
            }

            AlertDialog {
                open: show_clear_dialog,
                title: model::CLEAR_SKIPS,
                actions: vec![
                    DialogAction::Dismiss(model::CANCEL.into()),
                    DialogAction::Confirm {
                        label: model::CLEAR.into(),
                        danger: true,
                        on_confirm: clear_dismissals,
                    },
                ],
                {model::CLEAR_BODY}
            }

            // The shared overlay in its suggested mode deals the held list as
            // the Show chip narrows it: right seeds a new deck with the pick,
            // up saves it, left dismisses it. Its Filter sheet holds the same
            // Show chips over the same model.
            SwipeSelect {
                host_screen: screen::SUGGESTED_COMMANDERS,
                open: swipe_open,
                mode: swipe_mode,
                suggested_pile: Some(pile.into()),
                suggested_filter: rsx! {
                    ShowIdentityChips {
                        active_identity,
                        identities,
                        on_pick: move |picked| driver.pick_identity(picked),
                    }
                },
                on_select: move |card: Card| {
                    swipe_open.set(false);
                    driver.create(card);
                },
                on_close: move |()| swipe_open.set(false),
            }

            // Pick a printing: swap that row's card in place and keep the row
            // expanded under its new scryfall id.
            if let Some(card) = printing_card() {
                PrintingSheet {
                    card,
                    open: printing_open,
                    host_screen: screen::SUGGESTED_COMMANDERS,
                    on_save: move |new_card: Card| {
                        if new_card.scryfall_data.oracle_id.is_some() {
                            let mut expanded = expanded_card;
                            expanded.set(Some(new_card.scryfall_data.id));
                        }
                        driver.dispatch(Action::PickPrinting(Box::new(new_card)));
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
