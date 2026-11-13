//! The deck page's body, one assembly for the owner's deck and a shared deck: the head with its
//! facts and featured cards, the controls, the List or Stacks of cards, the stat panels, the
//! floating dock, the filter, the stores, the Stacks drawer, the image preview and printings, and
//! the Oracle tags' examples. The owner's page passes its edit actions and its own pieces (Quick
//! add, the foot, the dock's edit buttons); a shared deck passes none, and every edit control
//! stays out. The data and the keys come from the caller, so the body draws the same either way.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use components::icons::DockIcon;
use components::{
    BudgetBody, Button, ButtonVariant, CART, Chip, CurrencyChips, DeckCardsView, DeckControls,
    DeckEditActions, DeckFacts, DeckFold, DeckHead, DeckPage, DeckRowHost, DeckStackDrawer,
    DistributionsBody, DockFilter, DrawOddsStepper, FeaturedCard, GroupBuyDialog, HoverStackHost,
    ManaBody, StackDetails, StoreDialog, TagsBody, charted, deck_sections, use_group_buy,
    use_hover_stack, use_layer_presence, use_stack_details_sync,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::{Board, Format};
use zwipe_ui::buy::{self, BuyLink};
use zwipe_ui::deck::view::StatsProfile;
use zwipe_ui::deck_cards::listing::{Listed, RowMeta};
use zwipe_ui::deck_cards::stacks::CardsView;
use zwipe_ui::deck_cards::{self, Boards, FeaturedOpen};
use zwipe_ui::filter;
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::filter::picker::Sources;
use zwipe_ui::screens::deck_view::CARDS;

use super::cards_layer::DeckCardsLayer;
use super::ghost::{DeckBandsGhost, DeckControlsGhost, DeckHeadGhost, DeckStatsGhost};
use crate::inbound::card_filter::CardFilterPanel;
use crate::inbound::cards::card_row::{CardOverlays, CardOverlaysHost};
use crate::inbound::cards::otag::{OtagDescribe, OtagExamples, OtagExamplesHost, OtagExamplesOpen};
use crate::inbound::cards::stack_view::{CardsViewSwitch, use_stacks_dealing};
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::chrome::media::use_wide;
use crate::outbound::timer;

/// The stat panels' fold keys, shut when the page opens.
const FOLDED_STATS: [&str; 5] = ["Budget", "Tags", "Distributions", "Mana", "Draw odds"];

/// Scrolls the wide page to its cards' controls.
const CARDS_SCROLL_JS: &str = "document.querySelector('.deck-controls')?.scrollIntoView({ block: 'start', behavior: 'smooth' });";

/// How long the featured cards wait for their images before they deal anyway.
const FEATURED_WAIT: Duration = Duration::from_secs(4);

/// The deck page's body; see the module's doc. The owner's deck page fills it from its editor, a
/// shared deck from the shared deck.
#[component]
pub(crate) fn DeckPageBody(
    // Whether the deck is in; until then the page draws its ghosts.
    loaded: bool,
    // The deck's name over its facts.
    name: Option<String>,
    // Who shares the deck, beside its name; the owner's own page leaves it out.
    #[props(default)] byline: Option<String>,
    // The head's facts; the head is a ghost without them.
    facts: Option<DeckFacts>,
    // The featured cards with their roles.
    featured: Vec<(Card, &'static str)>,
    // The deck's format, which orders the groups.
    format: Option<Format>,
    // The rows as listed, and each card's quantity, board and star.
    listing: Listed,
    meta: HashMap<Uuid, RowMeta>,
    // The Group by choice.
    group_by: Signal<GroupByOption>,
    // Show's Tokens and Command zone chips, where the deck offers them, and Art.
    show_tokens: Option<Signal<bool>>,
    show_zone: Option<Signal<bool>>,
    show_art: Signal<bool>,
    // The Boards chips, and whether the maybeboard and sideboard have cards.
    mut boards: Signal<Boards>,
    has_maybe: bool,
    has_side: bool,
    // List or Stacks.
    view: Signal<CardsView>,
    // The folded groups, the open row, its face and every card's face.
    collapsed: Signal<HashSet<String>>,
    mut selected: Signal<Option<Uuid>>,
    face: Signal<usize>,
    faces: Signal<HashMap<Uuid, usize>>,
    // The Stacks view's reveal and drawer.
    details: StackDetails,
    // The filter, its sheet, its reset and the picker's sources.
    mut filter: Signal<CardQueryBuilder>,
    mut filter_open: Signal<bool>,
    on_reset_filter: Callback<()>,
    sources: Sources,
    // The rows' price currency, and the Budget panel's choice.
    row_currency: PriceCurrency,
    currency: Signal<PriceCurrency>,
    // The stat panels' inputs.
    stats: Option<StatsProfile>,
    metrics: Option<DeckMetrics>,
    land_target: Option<i32>,
    // The deck's store links, behind the dock's cart.
    deck_buy: Option<Vec<BuyLink>>,
    // The rows' edits; `None` for a shared deck, which edits nothing.
    actions: Option<DeckEditActions>,
    // The rows' buttons in the List, and in the Stacks drawer.
    host: DeckRowHost,
    drawer_host: DeckRowHost,
    // The image preview and the printing sheet to look at.
    overlays: CardOverlays,
    // The Oracle tags' example cards.
    examples: OtagExamples,
    // On a phone, whether the Cards layer is open over the Profile, and what opens it (`true`)
    // and closes it (`false`).
    #[props(default)] cards_open: bool,
    on_cards: Callback<bool>,
    // On a phone, the Profile's dock before its Cards button, and the Cards layer's dock after
    // the filter.
    #[props(default)] profile_dock: Option<Element>,
    // On a phone, the Profile in place of the page's head, stats and foot.
    #[props(default)] phone_profile: Option<Element>,
    #[props(default)] cards_dock_tail: Option<Element>,
    // On a phone, what the nav's `?` opens over the Cards layer.
    #[props(default)] cards_hint: Option<Callback<()>>,
    // Quick add, at the head of the controls.
    #[props(default)] quick_add: Option<Element>,
    // The foot under the stats.
    #[props(default)] foot: Option<Element>,
    // The dock's buttons before the filter, and after it; with none after it, a divider leads
    // the cart.
    #[props(default)] dock_lead: Option<Element>,
    #[props(default)] dock_tail: Option<Element>,
    // Anything else the page draws over itself.
    #[props(default)] extra: Option<Element>,
) -> Element {
    let CardOverlays { mut preview, .. } = overlays;
    // The rows' hover stack, drawn outside the rows and the page's entrance, whose transforms
    // would pin it to them instead of the window.
    let hover_stack = use_hover_stack();
    let folded: Signal<HashSet<String>> =
        use_signal(|| FOLDED_STATS.into_iter().map(String::from).collect());
    let dealing = use_stacks_dealing(view, loaded);
    // A phone draws the Profile as the page and the Cards as a layer over it.
    let wide = use_wide();
    let phone = !wide();
    // The Cards layer stays up while it slides out.
    let layer = use_layer_presence(phone && cards_open);
    // A wide window shows the cards on the page; a route that opens them scrolls there once
    // the cards are in.
    use_effect(use_reactive!(|loaded, cards_open| {
        if loaded && cards_open && *wide.peek() {
            document::eval(CARDS_SCROLL_JS);
        }
    }));
    // The store dialog a group's cart opens, in the List or the stacks.
    let group_buy = use_group_buy();
    // The deck's store dialog, from the dock's cart.
    let mut deck_buy_open = use_signal(|| false);
    let all_rows = listing.rows.clone();
    let listed_ids = use_memo(use_reactive!(|all_rows| all_rows
        .iter()
        .map(|r| r.card.scryfall_data.id)
        .collect::<Vec<_>>()));
    use_stack_details_sync(details.reveal, details.details_open, listed_ids);
    // The featured cards with their image bytes in hand. The row stays unseen until every card
    // reports in, or the wait runs out, then deals.
    let mut featured_loaded = use_signal(|| 0usize);
    let mut featured_timed_out = use_signal(|| false);
    use_future(move || async move {
        timer::sleep(FEATURED_WAIT).await;
        featured_timed_out.set(true);
    });
    let featured_ready = featured_timed_out() || featured_loaded() >= featured.len();
    let featured_class = if featured_ready {
        "deck-head-deal ready"
    } else {
        "deck-head-deal"
    };
    // A featured card opens its details in the drawer, as the same card in the List or Stacks
    // does, or its image when no listed card has details to show.
    let featured_cards: Vec<Card> = featured.iter().map(|(card, _)| card.clone()).collect();
    let open_featured =
        use_callback(
            move |id: Uuid| match deck_cards::featured_open(id, &listed_ids.peek()) {
                FeaturedOpen::Details => details.feature(id),
                FeaturedOpen::Image => {
                    if let Some(card) = featured_cards.iter().find(|c| c.scryfall_data.id == id) {
                        preview.set(Some((card.scryfall_data.clone(), 0)));
                    }
                }
            },
        );
    let on_featured_load = move |()| {
        let loaded = *featured_loaded.peek();
        featured_loaded.set(loaded.saturating_add(1));
    };

    let row_boards: HashMap<Uuid, Board> = meta.iter().map(|(id, m)| (*id, m.board)).collect();
    let applied = filter.read().clone();
    let shown_boards = boards();
    let filter_on = !applied.is_empty();
    let dot = filter::shows_dot(&applied);
    let no_main = listing.groups.is_empty() && listing.lands.is_empty();
    let sections = deck_sections(listing, &meta, format);
    let empty = loaded && shown_boards.main && sections.is_empty();
    // The fold-all button: every band shut, or every one open again.
    let section_keys: Vec<String> = sections.iter().map(|(s, _)| s.key.clone()).collect();
    let all_folded = {
        let shut = collapsed.read();
        !section_keys.is_empty() && section_keys.iter().all(|k| shut.contains(k))
    };
    let fold_all = move |()| {
        let mut collapsed = collapsed;
        let mut shut = collapsed.write();
        if all_folded {
            for key in &section_keys {
                shut.remove(key);
            }
        } else {
            shut.extend(section_keys.iter().cloned());
        }
    };
    let charts = charted(metrics.as_ref());
    let has_cards = metrics.is_some();
    let land_count = metrics.as_ref().map_or(0, |m| m.land_count);
    // The Oracle tag lookups the rows and the Distributions charts hand to their tags.
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let cart_divider = dock_tail.is_none();
    // The head: the name, its facts and the featured cards, on the page and atop the phone's
    // Cards layer.
    let head: Element = rsx! {
        if let Some(facts) = facts {
            DeckHead {
                name,
                byline,
                facts,
                has_featured: !featured.is_empty(),
                featured_class: featured_class.to_string(),
                for (card, role) in featured.iter().cloned() {
                    FeaturedCard {
                        key: "{card.scryfall_data.id}-{role}",
                        card,
                        role: role.to_string(),
                        on_select: open_featured,
                        on_load: on_featured_load,
                    }
                }
            }
        } else {
            DeckHeadGhost {}
        }
    };
    // The page's controls, on the wide page or in the phone's Cards layer.
    let controls: Element = rsx! {
        if !loaded {
            DeckControlsGhost {}
        } else {
            DeckControls {
                group_by,
                show_tokens,
                show_zone,
                show_art,
                art_disabled: view() == CardsView::Stacks,
                all_folded,
                on_fold_all: fold_all,
                on_change: move |()| selected.set(None),
                leading: quick_add,
                view: rsx! {
                    CardsViewSwitch { view, on_change: move |()| selected.set(None) }
                },
                boards: rsx! {
                    // Board chips ease the cards in and out (`settle`).
                    div { class: "chip-row", "data-settle": "",
                        span { class: "stat-chip stat-chip-p1 chip-row-label", "Boards" }
                        for (label, board, on, enabled) in [
                            ("Main", Board::Deck, shown_boards.main, true),
                            ("Maybe", Board::Maybeboard, shown_boards.maybe, has_maybe || shown_boards.maybe),
                            ("Side", Board::Sideboard, shown_boards.side, has_side || shown_boards.side),
                        ] {
                            Chip {
                                key: "{label}",
                                selected: on,
                                disabled: !enabled,
                                onclick: move |_| {
                                    boards.write().toggle(board);
                                    selected.set(None);
                                },
                                "{label}"
                            }
                        }
                    }
                },
            }
        }
    };

    let phone_profile = phone_profile.filter(|_| phone);
    rsx! {
        if let Some(profile) = phone_profile {
            {profile}
        } else {
        // The dock floats outside the page's entrance, whose transform would otherwise pin it to
        // the page instead of the window.
        div {
            class: "content-enter web-page deck-page-wide",
            onmousemove: move |e| hover_stack.track(e.client_coordinates().x),
            DeckPage {
                head: head.clone(),
                controls: (!phone).then(|| controls.clone()),
                split: phone,
                empty: empty && !phone,
                bands: rsx! {
                    // The ghost follows the saved view.
                    if !loaded && view() == CardsView::Stacks {
                        components::DeckStacksGhost { touch: phone }
                    } else if !loaded {
                        DeckBandsGhost {}
                    }
                    if !phone {
                    DeckCardsView {
                        sections: sections.clone(),
                        view: view(),
                        currency: row_currency,
                        collapsed,
                        details,
                        selected,
                        face,
                        faces,
                        boards: row_boards.clone(),
                        group_buy,
                        dealing: dealing(),
                        show_art: Some(show_art),
                        actions,
                        host,
                    }
                    }
                },
                stats: rsx! {
                    if !loaded {
                        DeckStatsGhost {}
                    }
                    if let Some(p) = stats {
                        if p.has_budget(has_cards) {
                            DeckFold {
                                fold_key: "Budget",
                                title: "Budget",
                                folded,
                                accessory: has_cards.then(|| rsx! { CurrencyChips { currency } }),
                                BudgetBody { profile: p.clone(), metrics, currency }
                            }
                        }
                        if p.tag_count() > 0 {
                            DeckFold { fold_key: "Tags", title: "Tags", folded,
                                TagsBody { profile: p }
                            }
                        }
                    }
                    if let Some(m) = charts.clone() {
                        DeckFold { fold_key: "Distributions", title: "Distributions", folded,
                            DistributionsBody {
                                metrics: m,
                                describe_tag,
                                on_examples,
                            }
                        }
                    }
                    if charts.is_some() || land_target.is_some() {
                        DeckFold { fold_key: "Mana", title: "Mana", folded,
                            ManaBody { charts: charts.clone(), land_count, land_target }
                        }
                    }
                    if let Some(m) = charts {
                        DeckFold {
                            fold_key: "Draw odds",
                            title: "Draw odds",
                            folded,
                            accessory: rsx! { InfoButton { topic: HintTopic::DrawOdds } },
                            DrawOddsStepper { metrics: m }
                        }
                    }
                },
                foot,
            }
        }
        }
        if phone {
            if layer.shown() {
                DeckCardsLayer {
                    sections,
                    view,
                    currency: row_currency,
                    collapsed,
                    selected,
                    face,
                    faces,
                    show_art,
                    boards: row_boards.clone(),
                    details,
                    group_buy,
                    dealing: dealing(),
                    loaded,
                    empty: loaded && shown_boards.main && no_main,
                    actions,
                    host,
                    head,
                    controls,
                    on_close: move |()| on_cards.call(false),
                    on_hint: cards_hint.unwrap_or_else(|| Callback::new(|()| {})),
                    leaving: layer.leaving(),
                    on_animation_end: move |name: String| layer.ended(&name),
                    // The layer's dock: the caller's lead, the filter, then the caller's tail.
                    dock: rsx! {
                        {dock_lead}
                        DockFilter {
                            label: "Filter",
                            dot,
                            active: filter_on,
                            onclick: move |()| filter_open.set(true),
                            on_reset: move |()| on_reset_filter.call(()),
                            reset_label: "Reset filter",
                        }
                        {cards_dock_tail}
                    },
                }
            }
            if !cards_open {
                // The Profile's dock: the caller's buttons, then Cards, the screen's main action.
                div { class: "action-dock action-dock-right content-enter-delayed",
                    {profile_dock}
                    if deck_buy.is_some() {
                        button {
                            class: "action-dock-btn",
                            "aria-label": buy::CART_LABEL,
                            onclick: move |_| deck_buy_open.set(true),
                            DockIcon { path: CART }
                        }
                    }
                    span { class: "action-dock-divider" }
                    Button {
                        variant: ButtonVariant::Action,
                        onclick: move |_| on_cards.call(true),
                        {CARDS}
                    }
                }
            }
        } else {
            // The page's actions as one floating icon bar at the bottom right: the caller's lead,
            // the filter (with its reset while a filter is on), the caller's tail, then the cart.
            div { class: "action-dock action-dock-right",
                {dock_lead}
                DockFilter {
                    label: "Filter",
                    dot,
                    active: filter_on,
                    onclick: move |()| filter_open.set(true),
                    on_reset: move |()| on_reset_filter.call(()),
                    reset_label: "Reset filter",
                }
                {dock_tail}
                if deck_buy.is_some() {
                    if cart_divider {
                        span { class: "action-dock-divider" }
                    }
                    button {
                        class: "action-dock-btn",
                        "aria-label": buy::CART_LABEL,
                        onclick: move |_| deck_buy_open.set(true),
                        DockIcon { path: CART }
                    }
                }
            }
        }
        CardFilterPanel {
            open: filter_open,
            filter,
            config: SheetConfig { show_format: true, sources, ..SheetConfig::default() },
            on_apply: move |next: CardQueryBuilder| {
                filter.set(next);
                selected.set(None);
            },
        }
        HoverStackHost { stack: hover_stack }
        if let Some(links) = deck_buy {
            StoreDialog {
                open: deck_buy_open,
                title: buy::DECK_TITLE.to_owned(),
                body: buy::DECK_BODY.to_owned(),
                links,
            }
        }
        GroupBuyDialog { group_buy }
        // The drawer a stack card opens, and in either view a featured card.
        DeckStackDrawer {
            rows: all_rows,
            boards: row_boards,
            details,
            faces,
            actions,
            host: drawer_host,
        }
        CardOverlaysHost { overlays }
        OtagExamplesHost { examples }
        {extra}
    }
}
