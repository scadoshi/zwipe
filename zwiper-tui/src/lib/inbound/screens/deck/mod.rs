//! One deck: its Profile card and the app's sections in as many panels as the terminal fits, the warning fixes, and the More sheet with its dialogs.
//!
//! What the screen knows and decides is zwipe-ui's [`State`]; this view adds the focus, which pane is zoomed and scrolled, the open role and tag, the draw odds' turn, the oracle tag catalog for definitions, and which sheet or dialog is up.

pub(crate) mod draw;
pub(crate) mod keys;
pub(crate) mod layout;
#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::domain::deck::{Deck, DeckProfile};
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_core::http::contracts::deck::HttpCloneDeck;
use zwipe_core::http::contracts::deck_card::HttpPatchDeckCard;
use zwipe_ui::deck::command_zone::Slot;
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::deck_view::{
    self, Action, CANCEL, CARD_KINGDOM, CLEAR, CLEAR_SKIPS_BODY, CLOSE, COPY_LINK, CREATE_LINK,
    DELETE, DELETE_BODY, Destination, Effect, MORE_TITLE, MoreItem, STOP_SHARE, Section, State,
    TCGPLAYER,
};
use zwipe_ui::toast::Toast;

use self::layout::Pane;
use super::Screen;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::alert::AlertDialog;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::info::InfoPopup;
use crate::inbound::widgets::menu::{Menu, MenuItem};
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// A section's "?" explainers, as the app's info buttons word them.
pub(crate) fn info(section: Section) -> InfoPopup {
    match section {
        Section::Budget => InfoPopup::new(
            "Price target",
            &["A budget for the mainboard. Going over warns on the deck"],
        ),
        Section::Tags => InfoPopup::new(
            "Tags",
            &[
                "Deck tags: the archetypes the deck is built around, like Aristocrats or Voltron",
                "Oracle tags: the specific things the deck does, like spot removal, ramp or reanimation",
                "Other tags: labels that do not change suggestions, like Budget or Jank",
            ],
        ),
        Section::Distributions => InfoPopup::new(
            "Distributions",
            &[
                "Types: mainboard cards by type",
                "Roles: mainboard cards by role, like ramp or removal. Cards with no role are left out. Tap a role to see its tags",
            ],
        ),
        Section::Mana => InfoPopup::new(
            "Mana",
            &[
                "Mana curve: nonland mainboard cards by mana value",
                "Colors: mainboard cards by color",
                "Mana cost fulfillment: per color, the pips your lands and sources make against the pips your spells need",
                "A check means you make at least as much as you need",
            ],
        ),
        Section::DrawOdds => InfoPopup::new(
            "Draw odds",
            &[
                "Chance of at least one card from each group in the opening hand, or by a turn. - and + step the turn",
                "Random draw only. Mulligans, tutors and card draw are not counted",
            ],
        ),
        Section::Warnings => InfoPopup::new(
            "Warnings",
            &["Warnings flag rule problems, each with a one-tap fix"],
        ),
    }
}

/// Something inside a pane the focus can rest on and press.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Item {
    /// A role in the Roles chart that has tags to show.
    Role(String),
    /// An oracle tag under the open role.
    Tag(String),
    /// The open tag's Examples button.
    Examples(String),
    /// One fix button of one warning.
    Fix { warning: usize, button: usize },
}

/// A dialog over the deck screen.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Dialog {
    Buy(AlertDialog),
    Clone(CloneDialog),
    Share(AlertDialog),
    ClearSkips(AlertDialog),
    Delete(AlertDialog),
    Info(InfoPopup),
}

/// The Clone dialog: a name for the copy, and whether the copy is on its way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CloneDialog {
    pub(crate) name: TextInput,
    /// Focused button: 0 Cancel, 1 Save.
    pub(crate) focused: usize,
    pub(crate) busy: bool,
}

impl CloneDialog {
    /// Prefilled with `prefill`, the model's "{deck name} (clone)".
    pub(crate) fn new(prefill: &str) -> Self {
        Self {
            name: TextInput::default().with_value(prefill),
            focused: 1,
            busy: false,
        }
    }
}

/// The furthest turn the draw odds look ahead, as in the app.
pub(crate) const MAX_TURN: u32 = 20;

/// The deck screen's state: the model plus what only the terminal keeps.
#[derive(Debug)]
pub(crate) struct DeckView {
    pub(crate) model: State,
    /// The pane the focus was put on. While it is not showing, the focus rests on [`DeckView::focused`]'s stand-in.
    pub(crate) focus: Pane,
    /// Panes newest first by when they were focused, so a tabbed panel shows the tab looked at last.
    pub(crate) recent: Vec<Pane>,
    /// The focused item in the focused pane, kept in range when read.
    pub(crate) item: usize,
    /// Whether the focused pane fills the body.
    pub(crate) zoomed: bool,
    /// How far the focused pane is scrolled when it has no items to move through, in rows.
    pub(crate) scroll: u16,
    /// The role whose tags show under it.
    pub(crate) role_open: Option<String>,
    /// The tag whose definition shows under the role's tags.
    pub(crate) tag_open: Option<String>,
    /// The turn the draw odds are for; 0 is the opening hand.
    pub(crate) turn: u32,
    /// The oracle tag catalog, for tag definitions.
    pub(crate) oracle_tags: Vec<OracleTag>,
    pub(crate) more: Option<Menu>,
    pub(crate) dialog: Option<Dialog>,
}

impl Default for DeckView {
    fn default() -> Self {
        Self {
            model: State::new(),
            focus: Pane::Profile,
            recent: Vec::new(),
            item: 0,
            zoomed: false,
            scroll: 0,
            role_open: None,
            tag_open: None,
            turn: 0,
            oracle_tags: Vec::new(),
            more: None,
            dialog: None,
        }
    }
}

impl DeckView {
    /// The deck's id, once the model has been told it.
    pub(crate) fn deck_id(&self) -> Option<Uuid> {
        self.model.deck_id()
    }

    /// The panes that show, Profile first and the sections in the app's order, with ghost rows standing in for the stats while they load.
    pub(crate) fn panes(&self) -> Vec<Pane> {
        let mut panes = vec![Pane::Profile];
        let sections = self.model.sections();
        let warnings = sections.contains(&Section::Warnings);
        panes.extend(
            sections
                .into_iter()
                .filter(|s| *s != Section::Warnings)
                .map(Pane::Section),
        );
        if self.model.stats_loading() {
            panes.push(Pane::Loading);
        }
        if warnings {
            panes.push(Pane::Section(Section::Warnings));
        }
        panes
    }

    /// The pane the focus rests on: the one it was put on while that shows, else the stats that took its place, else Profile.
    pub(crate) fn focused(&self) -> Pane {
        let panes = self.panes();
        if panes.contains(&self.focus) {
            return self.focus;
        }
        let stats = |p: &Pane| {
            matches!(
                p,
                Pane::Loading
                    | Pane::Section(Section::Distributions | Section::Mana | Section::DrawOdds)
            )
        };
        if stats(&self.focus) {
            // The skeleton stands in first, since the pane comes back when it goes.
            let loading = panes.iter().find(|p| **p == Pane::Loading);
            if let Some(stand_in) = loading.or_else(|| panes.iter().find(|p| stats(p))) {
                return *stand_in;
            }
        }
        Pane::Profile
    }

    /// Puts the focus on `pane`, at its first item.
    pub(crate) fn focus_on(&mut self, pane: Pane) {
        self.focus = pane;
        self.item = 0;
        self.scroll = 0;
        self.recent.retain(|p| *p != pane);
        self.recent.insert(0, pane);
    }

    /// Moves the focus to the next pane that shows, wrapping around to Profile.
    pub(crate) fn next_pane(&mut self) {
        let panes = self.panes();
        let at = self.pane_index(&panes);
        let next = at.saturating_add(1).checked_rem(panes.len()).unwrap_or(0);
        if let Some(pane) = panes.get(next) {
            self.focus_on(*pane);
        }
    }

    /// Moves the focus to the previous pane that shows, wrapping around to the last.
    pub(crate) fn previous_pane(&mut self) {
        let panes = self.panes();
        let at = self.pane_index(&panes);
        let previous = at
            .checked_sub(1)
            .unwrap_or_else(|| panes.len().saturating_sub(1));
        if let Some(pane) = panes.get(previous) {
            self.focus_on(*pane);
        }
    }

    fn pane_index(&self, panes: &[Pane]) -> usize {
        let focused = self.focused();
        panes.iter().position(|p| *p == focused).unwrap_or(0)
    }

    /// The pane a panel holding `panes` shows: the focused one, else the one focused last, else its first.
    pub(crate) fn shown(&self, panes: &[Pane]) -> Option<Pane> {
        let focused = self.focused();
        if panes.contains(&focused) {
            return Some(focused);
        }
        self.recent
            .iter()
            .find(|p| panes.contains(p))
            .or_else(|| panes.first())
            .copied()
    }

    /// The focused pane's items in order: the roles with their open tags, or the warnings' fix buttons.
    pub(crate) fn items(&self) -> Vec<Item> {
        let mut items = Vec::new();
        match self.focused() {
            Pane::Section(Section::Distributions) => self.role_items(&mut items),
            Pane::Section(Section::Warnings) => {
                for (i, w) in self.model.warnings().iter().enumerate() {
                    items.extend(
                        (0..deck_view::fix_buttons(w).len())
                            .map(|button| Item::Fix { warning: i, button }),
                    );
                }
            }
            _ => {}
        }
        items
    }

    fn role_items(&self, items: &mut Vec<Item>) {
        let Some(metrics) = self.model.charted() else {
            return;
        };
        for (role, _, _) in metrics.card_role_bars().unwrap_or_default() {
            let Some(tags) = metrics.card_role_tags.get(role) else {
                continue;
            };
            items.push(Item::Role(role.to_owned()));
            if self.role_open.as_deref() != Some(role) {
                continue;
            }
            for tag in tags {
                items.push(Item::Tag(tag.clone()));
                if self.tag_open.as_deref() == Some(tag) {
                    items.push(Item::Examples(tag.clone()));
                }
            }
        }
    }

    /// The focused item, the last one when the list has shrunk under it.
    pub(crate) fn focused_item(&self) -> Option<Item> {
        let items = self.items();
        let last = items.len().checked_sub(1)?;
        items.get(self.item.min(last)).cloned()
    }

    /// Moves to the next item in the focused pane, stopping at the last, or scrolls a pane with no items down a row.
    pub(crate) fn move_down(&mut self) {
        let items = self.items().len();
        if items == 0 {
            self.scroll = self.scroll.saturating_add(1).min(self.scroll_limit());
        } else {
            self.item = self.item.saturating_add(1).min(items.saturating_sub(1));
        }
    }

    /// Moves to the previous item in the focused pane, stopping at the first, or scrolls a pane with no items up a row.
    pub(crate) fn move_up(&mut self) {
        let items = self.items().len();
        if items == 0 {
            self.scroll = self.scroll.min(self.scroll_limit()).saturating_sub(1);
        } else {
            self.item = self.item.min(items.saturating_sub(1)).saturating_sub(1);
        }
    }

    /// How far `pane` is scrolled: the scroll while it has the focus, else none.
    pub(crate) fn scroll_of(&self, pane: Pane) -> u16 {
        if self.focused() == pane {
            self.scroll
        } else {
            0
        }
    }

    /// The most rows the focused pane can hold, so scrolling stops near its end; the draw stops it exactly at the end for the room it has.
    pub(crate) fn scroll_limit(&self) -> u16 {
        let rows = match (self.focused(), self.model.profile()) {
            // Each value can drop under its label.
            (Pane::Profile, _) => 18,
            // The chips, the total and the average, each of which can drop under its label.
            (Pane::Section(Section::Budget), _) => 5,
            // A label, its chips one to a row at most, and a blank row, per group.
            (Pane::Section(Section::Tags), Some(profile)) => [
                profile.tags.len(),
                profile.oracle_tags.len(),
                profile.other_tags.len(),
            ]
            .iter()
            .filter(|n| **n > 0)
            .fold(0_usize, |rows, n| rows.saturating_add(n.saturating_add(2))),
            // The heading, a blank row, and a row per group.
            (Pane::Section(Section::DrawOdds), _) => self
                .model
                .charted()
                .map_or(0, |m| m.draw_odds_buckets().1.len().saturating_add(2)),
            _ => 0,
        };
        u16::try_from(rows).unwrap_or(u16::MAX)
    }

    pub(crate) fn later_turn(&mut self) {
        self.turn = self.turn.saturating_add(1).min(MAX_TURN);
    }

    pub(crate) fn earlier_turn(&mut self) {
        self.turn = self.turn.saturating_sub(1);
    }

    /// The description of `slug` from the catalog.
    pub(crate) fn tag_description(&self, slug: &str) -> Option<&str> {
        self.oracle_tags
            .iter()
            .find(|t| t.slug == slug)
            .and_then(|t| t.description.as_deref())
    }

    /// The More sheet, its items the model's, with Export and Buy off until the deck has cards.
    pub(crate) fn more_menu(&self) -> Menu {
        Menu::new(
            MORE_TITLE,
            MoreItem::ALL
                .iter()
                .map(|item| {
                    let entry = MenuItem::new(item.label()).enabled(self.model.is_enabled(*item));
                    if item.is_danger() {
                        entry.danger()
                    } else {
                        entry
                    }
                })
                .collect(),
        )
    }

    /// The Buy dialog, with links once the deck has loaded.
    pub(crate) fn buy_dialog(&self) -> AlertDialog {
        let dialog = AlertDialog::new(deck_view::Dialog::Buy.title(), self.model.buy_body())
            .with_button(CLOSE, None);
        if self.model.deck().is_some() {
            dialog
                .with_button(TCGPLAYER, Some('t'))
                .with_button(CARD_KINGDOM, Some('k'))
        } else {
            dialog
        }
    }

    /// The Share dialog for whether the deck is shared now.
    pub(crate) fn share_dialog(&self) -> AlertDialog {
        let dialog = AlertDialog::new(deck_view::Dialog::Share.title(), self.model.share_body())
            .with_button(CLOSE, None);
        if self.model.share_token().is_some() {
            dialog
                .with_button(COPY_LINK, None)
                .with_button(STOP_SHARE, None)
        } else {
            dialog.with_button(CREATE_LINK, None)
        }
    }

    /// The dialog the model asked for, built from what it shows now.
    fn dialog_for(&self, dialog: deck_view::Dialog) -> Dialog {
        match dialog {
            deck_view::Dialog::Buy => Dialog::Buy(self.buy_dialog()),
            deck_view::Dialog::Clone => Dialog::Clone(CloneDialog::new(&self.model.clone_name())),
            deck_view::Dialog::Share => Dialog::Share(self.share_dialog()),
            deck_view::Dialog::ClearSkips => Dialog::ClearSkips(clear_skips_dialog()),
            deck_view::Dialog::Delete => Dialog::Delete(delete_dialog()),
        }
    }
}

/// The Clear skips dialog in the app's order, Cancel then the danger Clear, focused on Cancel.
pub(crate) fn clear_skips_dialog() -> AlertDialog {
    AlertDialog::new(deck_view::Dialog::ClearSkips.title(), CLEAR_SKIPS_BODY)
        .with_button(CANCEL, None)
        .with_danger_button(CLEAR, None)
        .with_focus(0)
}

/// The Delete dialog in the app's order, Cancel then the danger Delete, focused on Cancel.
pub(crate) fn delete_dialog() -> AlertDialog {
    AlertDialog::new(deck_view::Dialog::Delete.title(), DELETE_BODY)
        .with_button(CANCEL, None)
        .with_danger_button(DELETE, None)
        .with_focus(0)
}

/// Shows the first-deck hint again, whether or not it has been seen.
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    hint::reopen(cx, OneTimeHint::FirstDeck.key());
}

/// Fetches the deck again, keeping what is open and what shows until the replies land.
pub(crate) fn reload(view: &mut DeckView, cx: &mut Context<'_>) {
    if let Some(deck_id) = view.deck_id() {
        cx.shared.status.clear();
        dispatch(view, Action::Load(deck_id), cx);
    }
}

/// Applies `action` to the model and performs the effects it returns, in order.
pub(crate) fn dispatch(view: &mut DeckView, action: Action, cx: &mut Context<'_>) {
    perform_effects(view, action, cx, /* on_top */ true);
}

/// Applies `action` and performs its effects. A screen covered by another skips the effects that would move the stack, since they would act on the screen over it.
fn perform_effects(view: &mut DeckView, action: Action, cx: &mut Context<'_>, on_top: bool) {
    for effect in view.model.update(action) {
        match effect {
            Effect::FetchProfile { ticket, deck_id } => {
                cx.request(Request::FetchProfile { ticket, deck_id }.into());
            }
            Effect::FetchDeck { ticket, deck_id } => {
                cx.request(Request::FetchDeck { ticket, deck_id }.into());
            }
            // The terminal draws no commander art, so the card is never fetched.
            Effect::FetchCommander { .. } => {}
            Effect::OfferHint(offer) => {
                hint::once(cx, offer.key());
            }
            Effect::SetQuantity {
                ticket,
                deck_id,
                card,
                quantity,
            } => cx.request(
                Request::SetQuantity {
                    ticket,
                    deck_id,
                    card,
                    quantity,
                }
                .into(),
            ),
            Effect::ClearCommander { ticket, deck_id } => {
                cx.request(Request::ClearCommander { ticket, deck_id }.into());
            }
            Effect::RemoveCard {
                ticket,
                deck_id,
                card,
            } => cx.request(
                Request::RemoveCard {
                    ticket,
                    deck_id,
                    card,
                }
                .into(),
            ),
            Effect::CloneDeck {
                ticket,
                deck_id,
                name,
            } => cx.request(
                Request::Clone {
                    ticket,
                    deck_id,
                    name,
                }
                .into(),
            ),
            Effect::ShareDeck { ticket, deck_id } => {
                cx.request(Request::Share { ticket, deck_id }.into());
            }
            Effect::UnshareDeck { ticket, deck_id } => {
                cx.request(Request::Unshare { ticket, deck_id }.into());
            }
            Effect::ClearSkips { ticket, deck_id } => {
                cx.request(Request::ClearSkips { ticket, deck_id }.into());
            }
            Effect::DeleteDeck { ticket, deck_id } => {
                cx.request(Request::Delete { ticket, deck_id }.into());
            }
            Effect::Copy { text, done } => copy(cx, &text, done),
            Effect::OpenDialog(dialog) => view.dialog = Some(view.dialog_for(dialog)),
            Effect::Toast(toast) => cx.show(toast),
            Effect::Navigate(_) if !on_top => {}
            Effect::Navigate(Destination::DeckList) => cx.back(),
            Effect::Navigate(Destination::EditDeck(id)) => screens::deck_form::open_edit(cx, id),
            Effect::Navigate(Destination::DeckCards(id)) => screens::deck_cards::open(cx, id),
            Effect::Navigate(Destination::AddCards(_)) => {
                if let Some(deck) = view.model.deck() {
                    screens::swipe::open(cx, deck);
                }
            }
            Effect::Navigate(Destination::ImportDeck(id)) => screens::deck_import::open(cx, id),
            Effect::Navigate(Destination::ExportDeck(id)) => screens::deck_export::open(cx, id),
            Effect::Navigate(Destination::Deck(id)) => replace_with(cx, id),
        }
    }
}

/// The deck screen's work. Every request but the tag catalog carries its ticket back to the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// Fetches the deck's profile.
    FetchProfile {
        ticket: Ticket,
        deck_id: Uuid,
    },
    /// Fetches the full deck.
    FetchDeck {
        ticket: Ticket,
        deck_id: Uuid,
    },
    /// Fetches the oracle tag catalog for the role tags' definitions.
    LoadOracleTags,
    /// Sets a card's quantity, the copy-limit fix.
    SetQuantity {
        ticket: Ticket,
        deck_id: Uuid,
        card: Uuid,
        quantity: i32,
    },
    /// Clears the commander, the invalid-commander fix.
    ClearCommander {
        ticket: Ticket,
        deck_id: Uuid,
    },
    /// Takes a card out of the deck.
    RemoveCard {
        ticket: Ticket,
        deck_id: Uuid,
        card: Uuid,
    },
    Clone {
        ticket: Ticket,
        deck_id: Uuid,
        name: String,
    },
    Share {
        ticket: Ticket,
        deck_id: Uuid,
    },
    Unshare {
        ticket: Ticket,
        deck_id: Uuid,
    },
    ClearSkips {
        ticket: Ticket,
        deck_id: Uuid,
    },
    Delete {
        ticket: Ticket,
        deck_id: Uuid,
    },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::FetchProfile { .. } => "load_profile",
            Self::FetchDeck { .. } => "load",
            Self::LoadOracleTags => "load_oracle_tags",
            Self::SetQuantity { .. } => "fix_quantity",
            Self::ClearCommander { .. } => "clear_commander",
            Self::RemoveCard { .. } => "remove_card",
            Self::Clone { .. } => "clone",
            Self::Share { .. } => "share",
            Self::Unshare { .. } => "unshare",
            Self::ClearSkips { .. } => "clear_skips",
            Self::Delete { .. } => "delete",
        }
    }

    /// The ticket the request carries, which every one but the tag catalog has.
    #[cfg(test)]
    pub(crate) fn ticket(&self) -> Option<Ticket> {
        match self {
            Self::LoadOracleTags => None,
            Self::FetchProfile { ticket, .. }
            | Self::FetchDeck { ticket, .. }
            | Self::SetQuantity { ticket, .. }
            | Self::ClearCommander { ticket, .. }
            | Self::RemoveCard { ticket, .. }
            | Self::Clone { ticket, .. }
            | Self::Share { ticket, .. }
            | Self::Unshare { ticket, .. }
            | Self::ClearSkips { ticket, .. }
            | Self::Delete { ticket, .. } => Some(*ticket),
        }
    }
}

/// The deck screen's results, each carrying its request's ticket back to the model.
#[derive(Debug)]
pub(crate) enum Update {
    ProfileLoaded {
        ticket: Ticket,
        profile: Box<DeckProfile>,
    },
    DeckLoaded {
        ticket: Ticket,
        deck: Box<Deck>,
    },
    OracleTags(Vec<OracleTag>),
    QuantitySet {
        ticket: Ticket,
    },
    CommanderCleared {
        ticket: Ticket,
    },
    CardRemoved {
        ticket: Ticket,
    },
    Cloned {
        ticket: Ticket,
        deck_id: Uuid,
    },
    Shared {
        ticket: Ticket,
        token: Uuid,
    },
    Unshared {
        ticket: Ticket,
    },
    SkipsCleared {
        ticket: Ticket,
    },
    Deleted {
        ticket: Ticket,
    },
    /// A request failed with this message for an error toast.
    Failed {
        ticket: Ticket,
        error: String,
    },
    /// The clone failed with this message, which also frees the Clone dialog.
    CloneFailed {
        ticket: Ticket,
        error: String,
    },
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Deck(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Deck(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut DeckView> {
    match screen {
        Screen::Deck(view) => Some(view),
        _ => None,
    }
}

/// A new deck view on `deck_id`, its fetches and the tag catalog asked for, and the first-deck hint shown once.
fn opening(cx: &mut Context<'_>, deck_id: Uuid) -> Screen {
    cx.shared.status.clear();
    let mut view = DeckView::default();
    dispatch(&mut view, Action::Load(deck_id), cx);
    cx.request(Request::LoadOracleTags.into());
    Screen::Deck(Box::new(view))
}

/// Opens a deck over the current screen and fetches it, then shows the first-deck hint once.
pub(crate) fn open(cx: &mut Context<'_>, deck_id: Uuid) {
    let screen = opening(cx, deck_id);
    cx.push(screen);
}

/// Swaps the screen on top for this deck, as after creating or cloning one, so going back reaches the list.
pub(crate) fn replace_with(cx: &mut Context<'_>, deck_id: Uuid) {
    let screen = opening(cx, deck_id);
    cx.replace(screen);
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    Some(match request {
        Request::FetchProfile { ticket, deck_id } => reply(
            ticket,
            async {
                client
                    .fetch_deck_profile(deck_id, &account.fresh().await?)
                    .await
            }
            .await,
            |profile| Update::ProfileLoaded {
                ticket,
                profile: Box::new(profile),
            },
        ),
        Request::FetchDeck { ticket, deck_id } => reply(
            ticket,
            async { client.fetch_deck(deck_id, &account.fresh().await?).await }.await,
            |deck| Update::DeckLoaded {
                ticket,
                deck: Box::new(deck),
            },
        ),
        // Definitions only add to the role tags, so a failure shows none.
        Request::LoadOracleTags => {
            return client
                .fetch_oracle_tags()
                .await
                .ok()
                .map(|tags| Update::OracleTags(tags).into());
        }
        Request::SetQuantity {
            ticket,
            deck_id,
            card,
            quantity,
        } => set_quantity(account, ticket, deck_id, card, quantity).await,
        Request::ClearCommander { ticket, deck_id } => {
            clear_commander(account, ticket, deck_id).await
        }
        Request::RemoveCard {
            ticket,
            deck_id,
            card,
        } => remove_card(account, ticket, deck_id, card).await,
        Request::Clone {
            ticket,
            deck_id,
            name,
        } => clone_deck(account, ticket, deck_id, name).await,
        Request::Share { ticket, deck_id } => reply(
            ticket,
            async { client.share_deck(deck_id, &account.fresh().await?).await }.await,
            |share| Update::Shared {
                ticket,
                token: share.share_token,
            },
        ),
        Request::Unshare { ticket, deck_id } => reply(
            ticket,
            async { client.unshare_deck(deck_id, &account.fresh().await?).await }.await,
            |()| Update::Unshared { ticket },
        ),
        Request::ClearSkips { ticket, deck_id } => reply(
            ticket,
            async {
                client
                    .clear_deck_suppressions(deck_id, &account.fresh().await?)
                    .await
            }
            .await,
            |_| Update::SkipsCleared { ticket },
        ),
        Request::Delete { ticket, deck_id } => reply(
            ticket,
            async { client.delete_deck(deck_id, &account.fresh().await?).await }.await,
            |()| Update::Deleted { ticket },
        ),
    })
}

/// Sets a card's quantity, the copy-limit fix.
async fn set_quantity(
    account: &Account,
    ticket: Ticket,
    deck_id: Uuid,
    card: Uuid,
    quantity: i32,
) -> screens::Update {
    let body = HttpPatchDeckCard::new(Some(quantity), None);
    let result = async {
        account
            .client
            .update_deck_card(deck_id, card, &body, &account.fresh().await?)
            .await
    }
    .await;
    reply(ticket, result, |_| Update::QuantitySet { ticket })
}

/// Clears the commander, the invalid-commander fix.
async fn clear_commander(account: &Account, ticket: Ticket, deck_id: Uuid) -> screens::Update {
    let body = Slot::Commander.update_request(None);
    let result = async {
        account
            .client
            .update_deck_profile(deck_id, &body, &account.fresh().await?)
            .await
    }
    .await;
    reply(ticket, result, |_| Update::CommanderCleared { ticket })
}

/// Takes a card out of the deck.
async fn remove_card(
    account: &Account,
    ticket: Ticket,
    deck_id: Uuid,
    card: Uuid,
) -> screens::Update {
    let result = async {
        account
            .client
            .delete_deck_card(deck_id, card, &account.fresh().await?)
            .await
    }
    .await;
    reply(ticket, result, |()| Update::CardRemoved { ticket })
}

/// Clones the deck under `name`. A failure comes back as [`Update::CloneFailed`], which frees the Clone dialog.
async fn clone_deck(
    account: &Account,
    ticket: Ticket,
    deck_id: Uuid,
    name: String,
) -> screens::Update {
    let body = HttpCloneDeck { new_name: name };
    match async {
        account
            .client
            .clone_deck(deck_id, &body, &account.fresh().await?)
            .await
    }
    .await
    {
        Ok(cloned) => Update::Cloned {
            ticket,
            deck_id: cloned.deck_id,
        }
        .into(),
        Err(e) => screens::failure(&e, |error| Update::CloneFailed { ticket, error }.into()),
    }
}

/// The update a request's answer comes back as: `ok`'s on success, else the failure carrying `ticket`.
fn reply<T>(
    ticket: Ticket,
    result: Result<T, zwipe_client::ClientError>,
    ok: impl FnOnce(T) -> Update,
) -> screens::Update {
    match result {
        Ok(value) => ok(value).into(),
        Err(e) => screens::failure(&e, |error| Update::Failed { ticket, error }.into()),
    }
}

/// Hands a reply to the model. A failure raises its toast even when the screen has closed, and a reply to a screen another covers changes it without moving the stack.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let on_top = matches!(nav.top(), Screen::Deck(_));
    let action = match update {
        Update::ProfileLoaded { ticket, profile } => Action::ProfileLoaded { ticket, profile },
        Update::DeckLoaded { ticket, deck } => Action::DeckLoaded { ticket, deck },
        Update::OracleTags(tags) => {
            if let Some(view) = nav.find_mut(pick) {
                view.oracle_tags = tags;
            }
            return;
        }
        Update::QuantitySet { ticket } => Action::QuantitySet { ticket },
        Update::CommanderCleared { ticket } => Action::CommanderCleared { ticket },
        Update::CardRemoved { ticket } => Action::CardRemoved { ticket },
        Update::Cloned { ticket, deck_id } => Action::Cloned { ticket, deck_id },
        Update::Shared { ticket, token } => Action::Shared { ticket, token },
        Update::Unshared { ticket } => Action::Unshared { ticket },
        Update::SkipsCleared { ticket } => Action::SkipsCleared { ticket },
        Update::Deleted { ticket } => Action::Deleted { ticket },
        Update::Failed { ticket, error } => {
            cx.toast(ToastKind::Error, error);
            Action::Failed { ticket }
        }
        Update::CloneFailed { ticket, error } => {
            cx.toast(ToastKind::Error, error);
            if let Some(view) = nav.find_mut(pick)
                && let Some(Dialog::Clone(dialog)) = &mut view.dialog
            {
                dialog.busy = false;
            }
            Action::Failed { ticket }
        }
    };
    if let Some(view) = nav.find_mut(pick) {
        perform_effects(view, action, cx, on_top);
    }
}

/// Copies `text` to the clipboard and raises `done`, or says it could not.
pub(crate) fn copy(cx: &mut Context<'_>, text: &str, done: Toast) {
    if crate::inbound::widgets::clipboard::copy(text).is_ok() {
        cx.show(done);
    } else {
        cx.toast(ToastKind::Error, format!("Could not copy {text}"));
    }
}

/// Answers the deck view's latest profile and deck fetches with `deck`, as the server would.
#[cfg(test)]
pub(crate) fn serve(harness: &mut crate::inbound::event_loop::harness::Harness, deck: Deck) {
    let (mut profile, mut full) = (None, None);
    for request in harness.pending() {
        match request {
            screens::Request::Deck(Request::FetchProfile { ticket, .. }) => profile = Some(*ticket),
            screens::Request::Deck(Request::FetchDeck { ticket, .. }) => full = Some(*ticket),
            _ => {}
        }
    }
    let (Some(profile), Some(full)) = (profile, full) else {
        panic!("no deck fetch went out: {:?}", harness.pending());
    };
    harness.apply(Update::ProfileLoaded {
        ticket: profile,
        profile: Box::new(deck.deck_profile.clone()),
    });
    harness.apply(Update::DeckLoaded {
        ticket: full,
        deck: Box::new(deck),
    });
}
