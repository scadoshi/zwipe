//! The view deck screen: a deck's profile, sections and charts.
//!
//! What the screen knows and does is `zwipe_ui::screens::deck_view`; this
//! file draws its state and performs its effects.

use components::{
    AlertDialog, Button, ButtonVariant, DialogAction, HintBullet, HintBullets, HintDialog, HintKey,
    TOAST_NORMAL, ToastOptions, Toasts, use_toast,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::user::models::hints::HINT_FIRST_DECK;
use zwipe_core::http::contracts::deck::HttpCloneDeck;
use zwipe_core::http::contracts::deck_card::HttpPatchDeckCard;
use zwipe_ui::deck::command_zone::Slot;
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::deck_view::{
    self, Action, Destination, Dialog, Effect, MoreItem, Section, State,
};

use super::components::clone_deck_dialog::CloneDeckDialog;
use super::components::collapsible_section::CollapsibleSection;
use super::components::deck_budget_section::DeckBudgetSection;
use super::components::deck_charts::{
    ColorChart, DeckCharts, DrawOdds, ManaBalanceRow, ManaCurve, ManaFulfillment,
};
use super::components::deck_profile::DeckProfileSection;
use super::components::deck_tags_section::DeckTagsSection;
use super::components::deck_warnings::DeckWarnings;
use super::components::more_buttons::{DeckDialogs, MoreButtons, use_deck_dialogs};
use super::components::skeletons::{DeckProfileSkeleton, DeckStatsSkeleton};
use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::catalog_cache::CatalogCache;
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::InfoButton;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::flush_loop;
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::components::telemetry::vocabulary::{DeckScreen, Screen, component, screen};
use crate::inbound::components::{guide_link, hint_dialog, ui_toast};
use crate::inbound::router::Router;
use crate::inbound::screens::oracle_tag_examples::OracleTagExamples;
use crate::outbound::open_url;

/// The deck view's model with the handles that perform its effects. Copy,
/// so every handler and task holds its own.
#[derive(Clone, Copy)]
pub(crate) struct Driver {
    model: Signal<State>,
    navigator: Navigator,
    toast: Toasts,
    authed: Authed,
    usage_buffer: Signal<UsageBuffer>,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    hint_open: Signal<bool>,
    dialogs: DeckDialogs,
}

// By hand: the handles have no `PartialEq`. One model is one driver.
impl PartialEq for Driver {
    fn eq(&self, other: &Self) -> bool {
        self.model == other.model
    }
}

impl Driver {
    /// Applies `action` to the model and performs what it returns.
    pub(crate) fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    /// Clones the deck under `name` and waits for the reply, for the Clone
    /// dialog's busy Save. Returns whether the clone went through.
    pub(crate) async fn clone_deck(self, name: String) -> bool {
        let mut model = self.model;
        let effects = model.write().update(Action::Clone { name });
        let mut cloned = false;
        for effect in effects {
            if let Effect::CloneDeck {
                ticket,
                deck_id,
                name,
            } = effect
            {
                cloned = self.run_clone(ticket, deck_id, name).await;
            } else {
                self.perform(effect);
            }
        }
        cloned
    }

    /// Sends the clone and feeds its reply back. Returns whether it went
    /// through.
    async fn run_clone(self, ticket: Ticket, deck_id: Uuid, name: String) -> bool {
        let body = HttpCloneDeck { new_name: name };
        let reply = self
            .authed
            .run_at(
                component::CLONE_DECK_DIALOG,
                "clone_deck",
                |c, s| async move { c.clone_deck(deck_id, &body, &s).await },
            )
            .await;
        let cloned = reply.is_some();
        self.settle(ticket, reply, |cloned| Action::Cloned {
            ticket,
            deck_id: cloned.deck_id,
        });
        cloned
    }

    /// Performs one effect, feeding any reply back through [`Self::dispatch`].
    fn perform(self, effect: Effect) {
        match effect {
            Effect::FetchProfile { ticket, deck_id } => self.fetch_profile(ticket, deck_id),
            Effect::FetchDeck { ticket, deck_id } => self.fetch_deck(ticket, deck_id),
            Effect::FetchCommander { ticket, card } => self.fetch_commander(ticket, card),
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(
                    hint.key(),
                    self.session,
                    self.client,
                    self.hint_open,
                );
            }
            Effect::SetQuantity {
                ticket,
                deck_id,
                card,
                quantity,
            } => self.set_quantity(ticket, deck_id, card, quantity),
            Effect::ClearCommander { ticket, deck_id } => self.clear_commander(ticket, deck_id),
            Effect::RemoveCard {
                ticket,
                deck_id,
                card,
            } => self.remove_card(ticket, deck_id, card),
            Effect::CloneDeck {
                ticket,
                deck_id,
                name,
            } => {
                spawn(async move {
                    self.run_clone(ticket, deck_id, name).await;
                });
            }
            Effect::ShareDeck { ticket, deck_id } => self.share(ticket, deck_id),
            Effect::UnshareDeck { ticket, deck_id } => self.unshare(ticket, deck_id),
            Effect::ClearSkips { ticket, deck_id } => self.clear_skips(ticket, deck_id),
            Effect::DeleteDeck { ticket, deck_id } => self.delete(ticket, deck_id),
            Effect::Copy { text, done } => {
                // The link holds only a UUID, but it is serialized anyway.
                let js = format!(
                    "navigator.clipboard.writeText({})",
                    serde_json::to_string(&text).unwrap_or_default()
                );
                document::eval(&js);
                ui_toast::raise(self.toast, done);
            }
            Effect::OpenDialog(dialog) => self.dialogs.open(dialog),
            Effect::Navigate(destination) => {
                self.navigator.push(route(destination));
            }
            Effect::Toast(raised) => ui_toast::raise(self.toast, raised),
        }
    }

    /// Feeds `reply` back as `landed`, or as a failure of `ticket` when it is
    /// `None`. The request runner has already reported a failure and raised
    /// its toast.
    fn settle<T>(self, ticket: Ticket, reply: Option<T>, landed: impl FnOnce(T) -> Action) {
        self.dispatch(reply.map_or(Action::Failed { ticket }, landed));
    }

    fn fetch_profile(self, ticket: Ticket, deck_id: Uuid) {
        spawn(async move {
            // try_run reports a failure and raises its error toast.
            let reply = self
                .authed
                .try_run("load_deck", |c, s| async move {
                    c.fetch_deck_profile(deck_id, &s).await
                })
                .await;
            self.settle(ticket, reply.ok(), |profile| Action::ProfileLoaded {
                ticket,
                profile: Box::new(profile),
            });
        });
    }

    fn fetch_deck(self, ticket: Ticket, deck_id: Uuid) {
        spawn(async move {
            let reply = self
                .authed
                .try_run("load_deck", |c, s| async move {
                    c.fetch_deck(deck_id, &s).await
                })
                .await;
            self.settle(ticket, reply.ok(), |deck| Action::DeckLoaded {
                ticket,
                deck: Box::new(deck),
            });
        });
    }

    fn fetch_commander(self, ticket: Ticket, card: Uuid) {
        let Self {
            toast,
            usage_buffer,
            client,
            ..
        } = self;
        spawn(async move {
            let reply = match client().fetch_card(card).await {
                Ok(card) => Some(card),
                Err(e) => {
                    usage_buffer.peek().report_error(
                        screen::DECK_VIEW,
                        component::NONE,
                        "load_commander",
                        &e,
                    );
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(TOAST_NORMAL),
                    );
                    None
                }
            };
            self.settle(ticket, reply, |card| Action::CommanderLoaded {
                ticket,
                card: Box::new(card),
            });
        });
    }

    fn set_quantity(self, ticket: Ticket, deck_id: Uuid, card: Uuid, quantity: i32) {
        // PATCH sets the absolute target, not a delta from a possibly stale
        // quantity.
        let request = HttpPatchDeckCard::new(Some(quantity), None);
        spawn(async move {
            let reply = self
                .authed
                .run("update_card_quantity", |c, s| async move {
                    c.update_deck_card(deck_id, card, &request, &s).await
                })
                .await;
            self.settle(ticket, reply, |_| Action::QuantitySet { ticket });
        });
    }

    fn clear_commander(self, ticket: Ticket, deck_id: Uuid) {
        let request = Slot::Commander.update_request(None);
        spawn(async move {
            let reply = self
                .authed
                .run("remove_commander", |c, s| async move {
                    c.update_deck_profile(deck_id, &request, &s).await
                })
                .await;
            self.settle(ticket, reply, |_| Action::CommanderCleared { ticket });
        });
    }

    /// Goes straight to the client, so a missing session reads `Session
    /// expired`, and reports a failure itself.
    fn remove_card(self, ticket: Ticket, deck_id: Uuid, card: Uuid) {
        let Self {
            toast,
            usage_buffer,
            session,
            client,
            ..
        } = self;
        spawn(async move {
            let result: Result<(), ClientError> = async {
                let session = session()
                    .ok_or_else(|| ClientError::Unauthorized("Session expired".to_string()))?;
                client().delete_deck_card(deck_id, card, &session).await
            }
            .await;
            let reply = match result {
                Ok(()) => Some(()),
                Err(e) => {
                    usage_buffer.peek().report_error(
                        screen::DECK_VIEW,
                        component::DECK_WARNINGS,
                        "remove_flagged_card",
                        &e,
                    );
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(TOAST_NORMAL),
                    );
                    None
                }
            };
            self.settle(ticket, reply, |()| Action::CardRemoved { ticket });
        });
    }

    fn share(self, ticket: Ticket, deck_id: Uuid) {
        spawn(async move {
            let reply = self
                .authed
                .run_at(component::MORE_BUTTONS, "share_deck", |c, s| async move {
                    c.share_deck(deck_id, &s).await
                })
                .await;
            self.settle(ticket, reply, |shared| Action::Shared {
                ticket,
                token: shared.share_token,
            });
        });
    }

    fn unshare(self, ticket: Ticket, deck_id: Uuid) {
        spawn(async move {
            let reply = self
                .authed
                .run_at(component::MORE_BUTTONS, "unshare_deck", |c, s| async move {
                    c.unshare_deck(deck_id, &s).await
                })
                .await;
            self.settle(ticket, reply, |()| Action::Unshared { ticket });
        });
    }

    fn clear_skips(self, ticket: Ticket, deck_id: Uuid) {
        let Self {
            usage_buffer,
            session,
            client,
            ..
        } = self;
        spawn(async move {
            // Flush pending skips first so this window's not-yet-sent skips
            // are wiped too, not re-suppressed by the next flush.
            flush_loop::flush_once(&usage_buffer(), &client, &session).await;
            let reply = self
                .authed
                .run_at(
                    component::MORE_BUTTONS,
                    "clear_suppressions",
                    |c, s| async move { c.clear_deck_suppressions(deck_id, &s).await },
                )
                .await;
            self.settle(ticket, reply, |_| Action::SkipsCleared { ticket });
        });
    }

    fn delete(self, ticket: Ticket, deck_id: Uuid) {
        spawn(async move {
            let reply = self
                .authed
                .run("delete_deck", |c, s| async move {
                    c.delete_deck(deck_id, &s).await
                })
                .await;
            self.settle(ticket, reply, |()| Action::Deleted { ticket });
        });
    }
}

/// The app's route for a place the deck view sends the user.
fn route(destination: Destination) -> Router {
    match destination {
        Destination::DeckList => Router::DeckList {},
        Destination::EditDeck(deck_id) => Router::EditDeck { deck_id },
        Destination::DeckCards(deck_id) => Router::ViewDeckCard { deck_id },
        Destination::AddCards(deck_id) => Router::AddDeckCard { deck_id },
        Destination::ImportDeck(deck_id) => Router::ImportDeck { deck_id },
        Destination::ExportDeck(deck_id) => Router::ExportDeck { deck_id },
        Destination::Deck(deck_id) => Router::ViewDeck { deck_id },
    }
}

#[component]
pub fn ViewDeck(deck_id: Uuid) -> Element {
    let client: Signal<ZwipeClient> = use_context();
    let driver = Driver {
        model: use_signal(State::new),
        navigator: use_navigator(),
        toast: use_toast(),
        authed: use_authed(Screen::Deck(DeckScreen::View)),
        usage_buffer: use_context(),
        session: use_context(),
        client,
        hint_open: use_signal(|| false),
        dialogs: use_deck_dialogs(),
    };
    // First-deck tour: opens once per account when the model offers it on
    // the first Load; the header "?" reopens it on demand.
    let first_deck_hint_open = driver.hint_open;

    // Fetches the deck on mount and again when the route's deck id changes
    // without a remount, as cloning does (ViewDeck to ViewDeck).
    use_effect(use_reactive!(
        |deck_id| driver.dispatch(Action::Load(deck_id))
    ));

    // The Roles chart's tags: a description lookup from the
    // shared catalog cache and an examples-browse opener, as on the deck
    // cards view, with the examples overlay owned here.
    let cache: CatalogCache = use_context();
    use_effect(move || {
        cache.ensure_oracle_tags(client);
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

    // Accordion state for the deck-view sections, holds the title of the one
    // open section. Budget is open on load.
    let open_section: Signal<Option<String>> =
        use_signal(|| Some(Section::Budget.title().to_string()));
    let attempt_delete = use_callback(move |()| driver.dispatch(Action::Delete));

    // Snapshot the model once per render.
    let state = driver.model.read();
    let failed = state.status() == deck_view::Status::Failed;
    let profile = state.profile().cloned();
    let commander = state.commander().cloned();
    let card_count = state.card_count();
    let chart_metrics = state.charted().cloned();
    let sections = state.sections();
    let stats_loading = state.stats_loading();
    let warnings = state.warnings().to_vec();
    let currency = state.currency();
    let currency_chips = state.currency_chips_shown();
    let total_line = state.total_price_line();
    let average_line = state.average_price_line();
    let pt_line = state.average_pt_line();
    let lands_line = state.lands_line();
    let mana_value_line = state.average_mana_value_line();
    let tags_badge = state.badge(Section::Tags).unwrap_or_default();
    let warnings_badge = state.badge(Section::Warnings).unwrap_or_default();
    let more_enabled: Vec<MoreItem> = MoreItem::ALL
        .into_iter()
        .filter(|item| state.is_enabled(*item))
        .collect();
    let buy_links = state.buy_links();
    let buy_body = state.buy_body();
    let shared = state.share_token().is_some();
    let share_body = state.share_body();
    let clone_name = state.clone_name();
    drop(state);

    let mana_curve_bars: Option<[(usize, u32); 7]> =
        chart_metrics.as_ref().map(DeckMetrics::mana_curve_bars);
    let type_bars: Option<Vec<(&str, usize, u32)>> =
        chart_metrics.as_ref().map(DeckMetrics::type_bars);
    let color_bars: Option<Vec<(&str, usize, u32)>> =
        chart_metrics.as_ref().map(DeckMetrics::color_bars);
    let category_bars: Option<Vec<(&str, usize, u32)>> =
        chart_metrics.as_ref().and_then(DeckMetrics::card_role_bars);
    let role_tags = chart_metrics.as_ref().map(|m| m.card_role_tags.clone());

    let draw_odds_data: Option<(u32, Vec<(&'static str, u32)>)> =
        chart_metrics.as_ref().map(DeckMetrics::draw_odds_buckets);
    let mana_balance_rows: Option<Vec<ManaBalanceRow>> =
        chart_metrics.as_ref().map(DeckMetrics::mana_balance_rows);

    rsx! {
            div { class: "screen",
                ScreenHeader {
                    title: deck_view::TITLE,
                    hint: first_deck_hint_open,
                    back: move |()| driver.dispatch(Action::Back),
                }

                div { class: "screen-content",
                    match profile {
                        Some(deck_profile) => {
                            rsx! {
                            div { class: "content-enter",
                                  style: "width: calc(100% - 4rem); display: flex; flex-direction: column; gap: var(--panel-gap); padding: 1rem 0;",
                                DeckProfileSection {
                                    deck_profile: deck_profile.clone(),
                                    commander: commander,
                                    card_count: card_count,
                                    actions: rsx! {
                                        Button {
                                            variant: ButtonVariant::Action,
                                            onclick: move |_| driver.dispatch(Action::Edit),
                                            {deck_view::EDIT}
                                        }
                                        MoreItemButton { item: MoreItem::CloneDeck, enabled: more_enabled.clone(), driver }
                                        MoreItemButton { item: MoreItem::ShareDeck, enabled: more_enabled.clone(), driver }
                                    },
                                }

                                if sections.contains(&Section::Budget) {
                                    CollapsibleSection {
                                        title: Section::Budget.title().to_string(),
                                        open_section: open_section,
                                        header_accessory: {
                                            if currency_chips {
                                                rsx! {
                                                    div { class: "chip-row", style: "margin-bottom:0;",
                                                        for option in PriceCurrency::all().iter().copied() {
                                                            div {
                                                                class: if currency == option { "chip selected" } else { "chip" },
                                                                onclick: move |_| driver.dispatch(Action::PickCurrency(option)),
                                                                "{option.label()}"
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                rsx! {}
                                            }
                                        },
                                        DeckBudgetSection {
                                            total: total_line,
                                            average: average_line,
                                        }
                                        if let (true, Some((tcg, ck))) = (more_enabled.contains(&MoreItem::BuyDeck), buy_links.clone()) {
                                            div { class: "deck-buy-row",
                                                Button {
                                                    variant: ButtonVariant::Action,
                                                    onclick: move |_| open_url::open(&tcg),
                                                    {deck_view::TCGPLAYER}
                                                }
                                                Button {
                                                    variant: ButtonVariant::Action,
                                                    onclick: move |_| open_url::open(&ck),
                                                    {deck_view::CARD_KINGDOM}
                                                }
                                            }
                                        }
                                    }
                                }

                                if sections.contains(&Section::Tags) {
                                    CollapsibleSection {
                                        title: Section::Tags.title().to_string(),
                                        badge: tags_badge,
                                        open_section: open_section,
                                        DeckTagsSection { deck_profile }
                                    }
                                }

                                // Mana shows for a curve (mainboard cards) or just a
                                // land target, so the Lands goal-vs-actual stays
                                // visible even on an otherwise-empty deck.
                                if sections.contains(&Section::Mana) {
                                  // Nested column: keep its gap equal to the outer
                                  // sections column so Distributions/Mana/Draw odds
                                  // space like every other section pair.
                                  div { class: "content-enter",
                                        style: "display: flex; flex-direction: column; gap: var(--panel-gap);",
                                    // Distributions stays mainboard-only.
                                    if sections.contains(&Section::Distributions) {
                                        CollapsibleSection {
                                            title: Section::Distributions.title().to_string(),
                                            open_section: open_section,
                                            if let Some(value) = pt_line {
                                                div { class: "settings-row",
                                                    span { class: "settings-row-label", {deck_view::AVERAGE_PT} }
                                                    span { class: "info-row-value", style: "white-space:nowrap;", "{value}" }
                                                }
                                            }
                                            DeckCharts {
                                                type_bars: type_bars.clone(),
                                                category_bars: category_bars.clone(),
                                                role_tags: role_tags.clone(),
                                                describe_tag,
                                                on_examples: open_examples,
                                                type_hint: rsx! { InfoButton { topic: HintTopic::TypeChart } },
                                                role_hint: rsx! { InfoButton { topic: HintTopic::RoleChart } },
                                            }
                                        }
                                    }

                                    CollapsibleSection {
                                        title: Section::Mana.title().to_string(),
                                        open_section: open_section,
                                        // Lands: actual count, shown as `actual / target`
                                        // when a land target is set.
                                        div { class: "settings-row",
                                            span { class: "settings-row-label", {deck_view::LANDS} }
                                            span { class: "info-row-value", "{lands_line}" }
                                        }
                                        if let Some(value) = mana_value_line {
                                            div { class: "settings-row",
                                                span { class: "settings-row-label", {deck_view::AVERAGE_MANA_VALUE} }
                                                span { class: "info-row-value", "{value}" }
                                            }
                                        }
                                        if let Some(mana_curve_bars) = mana_curve_bars.as_ref() {
                                            ManaCurve {
                                                mana_curve_bars: *mana_curve_bars,
                                                hint: rsx! { InfoButton { topic: HintTopic::ManaCurve } },
                                            }
                                        }
                                        if let Some(color_bars) = color_bars.clone() {
                                            ColorChart {
                                                color_bars,
                                                hint: rsx! { InfoButton { topic: HintTopic::ColorChart } },
                                            }
                                        }
                                        if let Some(rows) = mana_balance_rows {
                                            ManaFulfillment {
                                                rows: rows,
                                                hint: rsx! { InfoButton { topic: HintTopic::ManaFulfillment } },
                                            }
                                        }
                                    }

                                    if let Some((deck_size, buckets)) = draw_odds_data.clone() {
                                        CollapsibleSection {
                                            title: Section::DrawOdds.title().to_string(),
                                            open_section: open_section,
                                            header_accessory: rsx! { InfoButton { topic: HintTopic::DrawOdds } },
                                            DrawOdds {
                                                deck_size: deck_size,
                                                buckets: buckets,
                                            }
                                        }
                                    }
                                  }
                                } else if stats_loading {
                                    DeckStatsSkeleton {}
                                }

                                if sections.contains(&Section::Warnings) {
                                    CollapsibleSection {
                                        title: Section::Warnings.title().to_string(),
                                        warn: true,
                                        open_section: open_section,
                                        badge: warnings_badge,
                                        DeckWarnings {
                                            warnings: warnings,
                                            ondispatch: move |action| driver.dispatch(action),
                                        }
                                    }
                                }

                                // The less used actions, then the destructive two.
                                div { class: "deck-page-actions",
                                    div { class: "deck-page-actions-row",
                                        MoreItemButton { item: MoreItem::ImportCards, enabled: more_enabled.clone(), driver }
                                        MoreItemButton { item: MoreItem::ExportCards, enabled: more_enabled.clone(), driver }
                                    }
                                    div { class: "deck-page-actions-row",
                                        MoreItemButton { item: MoreItem::ClearSkips, enabled: more_enabled.clone(), driver }
                                        MoreItemButton { item: MoreItem::DeleteDeck, enabled: more_enabled, driver }
                                    }
                                }

                            }
                            }
                        },
                        None if failed => rsx! { p { class: "text-muted", {deck_view::LOAD_FAILED} } },
                        None => rsx! {
                            div { class: "content-enter",
                                  style: "width: calc(100% - 4rem); display: flex; flex-direction: column; gap: var(--panel-gap); padding: 1rem 0;",
                                DeckProfileSkeleton {}
                                DeckStatsSkeleton {}
                            }
                        }
                    }
                }

            // Cards, the screen's main action, floats over the page.
            div { class: "app-float content-enter-delayed",
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| driver.dispatch(Action::Cards),
                    {deck_view::CARDS}
                }
            }

            AlertDialog {
                open: driver.dialogs.delete,
                title: Dialog::Delete.title().to_string(),
                actions: vec![
                    DialogAction::Dismiss(deck_view::CANCEL.into()),
                    DialogAction::Confirm {
                        label: deck_view::DELETE.into(),
                        danger: true,
                        on_confirm: attempt_delete,
                    },
                ],
                {deck_view::DELETE_BODY}
            }

            MoreButtons {
                dialogs: driver.dialogs,
                buy_links: buy_links,
                buy_body: buy_body,
                shared: shared,
                share_body: share_body,
                ondispatch: move |action| driver.dispatch(action),
            }

            CloneDeckDialog {
                driver: driver,
                default_name: clone_name,
                open: driver.dialogs.clone,
            }

            if otag_examples_open() {
                OracleTagExamples { open: otag_examples_open, slug: otag_examples_slug() }
            }

            HintDialog {
                open: first_deck_hint_open,
                actions: guide_link::one_time(HINT_FIRST_DECK),
                title: "Welcome to your deck",
                HintBullets {
                    HintBullet { HintKey { "Cards" } " browses, adds and removes cards" }
                    HintBullet { HintKey { "Edit" } " changes the name, format, command zone, tags and targets" }
                    HintBullet { HintKey { "Clone" } " and " HintKey { "Share" } " sit beside Edit; import, export and delete are at the bottom" }
                    HintBullet { "Skipped and removed cards stay out of the add stack. " HintKey { "Clear skips" } " at the bottom brings them back" }
                    HintBullet { "Stats fill in as the deck grows. Tap a section to expand it" }
                    HintBullet { "Warnings flag rule problems, each with a one-tap fix" }
                }
            }

            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_destination_has_the_app_route() {
        let deck_id = Uuid::new_v4();
        assert_eq!(route(Destination::DeckList), Router::DeckList {});
        assert_eq!(
            route(Destination::EditDeck(deck_id)),
            Router::EditDeck { deck_id }
        );
        assert_eq!(
            route(Destination::DeckCards(deck_id)),
            Router::ViewDeckCard { deck_id }
        );
        assert_eq!(
            route(Destination::AddCards(deck_id)),
            Router::AddDeckCard { deck_id }
        );
        assert_eq!(
            route(Destination::ImportDeck(deck_id)),
            Router::ImportDeck { deck_id }
        );
        assert_eq!(
            route(Destination::ExportDeck(deck_id)),
            Router::ExportDeck { deck_id }
        );
        assert_eq!(
            route(Destination::Deck(deck_id)),
            Router::ViewDeck { deck_id }
        );
    }
}

/// One of the deck's actions as a button: its label, the danger color for
/// Clear skips and Delete, disabled while the model says it can't run.
#[component]
fn MoreItemButton(item: MoreItem, enabled: Vec<MoreItem>, driver: Driver) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Action,
            disabled: !enabled.contains(&item),
            danger: item.is_danger(),
            onclick: move |_| driver.dispatch(Action::More(item)),
            {item.short_label()}
        }
    }
}
