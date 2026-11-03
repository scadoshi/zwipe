//! Oracle tag examples: real cards that carry one tag, most played first, browsed one at a time.
//!
//! The app's deck-free swipe browse, read only: Left (swipe left) moves to the next card and Down (swipe down) steps back one. Cards come from `search_cards` in pages of [`zwipe_ui::timing::PAGE_SIZE`] by EDHREC rank, fetched again when few are left ahead, up to [`zwipe_ui::timing::EXAMPLES_MAX_CARDS`]. An empty page marks the end.
//!
//! What the browse knows and decides (the cards, the cursor, the paging, the lines in place of a card) is zwipe-ui's [`State`]; this view adds the details sheet and the card's motion. `d` opens the card's details sheet, whose `p` opens its printings read only. Nothing here changes anything on the server.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::time::Instant;

use ratatui::text::Line;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics;
use zwipe_core::domain::user::models::hints::HINT_OTAG_EXAMPLES;
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::oracle_tag_examples::{Action, Destination, Effect, State};
use zwipe_ui::stack::Direction;

use super::Screen;
use crate::domain::theme::Palette;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::card_motion::CardMotion;
use crate::inbound::widgets::sheet::Sheet;
use crate::inbound::widgets::toast::ToastKind;
use crate::inbound::widgets::{card_text, hint};
use crate::outbound::account::Account;

/// The browse: the model plus what only the terminal keeps.
#[derive(Debug)]
pub(crate) struct ExamplesView {
    pub(crate) model: State,
    /// The details sheet over the card, when open.
    pub(crate) details: Option<Sheet>,
    /// The card flying off to the left, or the one coming back after a step back.
    pub(crate) motion: Option<CardMotion<Card>>,
}

impl Default for ExamplesView {
    fn default() -> Self {
        Self {
            model: State::new(),
            details: None,
            motion: None,
        }
    }
}

impl ExamplesView {
    /// Whether a card is in motion, so the loop redraws.
    pub(crate) fn animating(&self) -> bool {
        self.motion
            .as_ref()
            .is_some_and(|m| m.running(Instant::now()))
    }

    /// The card under the one on screen, which a card coming back covers.
    pub(crate) fn under(&self) -> Option<&Card> {
        self.model.window().get(1)
    }

    /// The card on screen, `None` past the last one.
    pub(crate) fn current(&self) -> Option<&Card> {
        self.model.current()
    }

    /// Sends the card on screen off to the left and moves to the next, which may ask for more. Past the last card nothing moves.
    pub(crate) fn next(&mut self, cx: &mut Context<'_>) {
        let Some(card) = self.current().cloned() else {
            return;
        };
        self.motion = Some(CardMotion::exit(Direction::Left, card, Instant::now()));
        dispatch(self, Action::Next, cx);
    }

    /// Opens the details sheet for the card on screen.
    pub(crate) fn open_details(&mut self, palette: &Palette) {
        self.details = self.current().map(|card| details_sheet(palette, card));
    }
}

/// Applies `action` to the model and performs the effects it returns, in order.
pub(crate) fn dispatch(view: &mut ExamplesView, action: Action, cx: &mut Context<'_>) {
    for effect in view.model.update(action) {
        match effect {
            Effect::OfferHint(offer) => {
                hint::once(cx, offer.key());
            }
            Effect::SearchCards { ticket, query } => {
                cx.request(Request::Page { ticket, query }.into());
            }
            Effect::Enter(from) => {
                view.motion = Some(CardMotion::enter(from, Instant::now()));
            }
            Effect::Navigate(Destination::Previous) => cx.back(),
            Effect::Toast(toast) => cx.show(toast),
        }
    }
}

/// The details sheet: the card's text, then its rarity, set, keywords and price, with Printings in its footer.
pub(crate) fn details_sheet(palette: &Palette, card: &Card) -> Sheet {
    let data = &card.scryfall_data;
    let mut body = card_text::lines(palette, data);
    body.push(Line::default());
    body.push(Line::styled(data.rarity.to_long_name(), palette.muted));
    body.push(Line::styled(
        format!("{} ({})", data.set_name, data.set.to_uppercase()),
        palette.muted,
    ));
    if let Some(keywords) = data.keywords.as_ref().filter(|k| !k.is_empty()) {
        body.push(Line::styled(keywords.join(", "), palette.muted));
    }
    if let Some(price) = deck_metrics::card_price(data, PriceCurrency::Usd) {
        body.push(Line::styled(
            PriceCurrency::Usd.format_amount(price),
            palette.muted,
        ));
    }
    Sheet::new(&data.name, body)
        .with_footer("p", "Printings")
        .with_footer("Esc", "Close")
}

/// The examples' work.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    /// One page of the tag's cards, as the model built the search.
    Page {
        ticket: Ticket,
        query: Box<CardQuery>,
    },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Page { .. } => "page",
        }
    }
}

/// The examples' results, each carrying its request's ticket back to the model.
#[derive(Debug)]
pub(crate) enum Update {
    Page { ticket: Ticket, cards: Vec<Card> },
    PageFailed { ticket: Ticket, message: String },
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::OtagExamples(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::OtagExamples(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut ExamplesView> {
    match screen {
        Screen::OtagExamples(view) => Some(view),
        _ => None,
    }
}

/// Opens the examples for `slug`: the model offers the hint and asks for the first page.
pub(crate) fn open(cx: &mut Context<'_>, slug: &str) {
    let mut view = ExamplesView::default();
    dispatch(
        &mut view,
        Action::Load {
            slug: slug.to_owned(),
        },
        cx,
    );
    cx.push(Screen::OtagExamples(Box::new(view)));
}

/// Shows the hint again, for its "?".
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    hint::reopen(cx, HINT_OTAG_EXAMPLES);
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let Request::Page { ticket, query } = request;
    let client = &account.client;
    match async { client.search_cards(&query, &account.fresh().await?).await }.await {
        Ok(cards) => Some(Update::Page { ticket, cards }.into()),
        Err(e) => Some(screens::failure(&e, |message| {
            Update::PageFailed { ticket, message }.into()
        })),
    }
}

/// Hands a page to the model. A failure raises its toast even when the screen has closed.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let action = match update {
        Update::Page { ticket, cards } => Action::Loaded { ticket, cards },
        Update::PageFailed { ticket, message } => {
            cx.toast(ToastKind::Error, message);
            Action::Failed { ticket }
        }
    };
    if let Some(view) = nav.find_mut(pick) {
        dispatch(view, action, cx);
    }
}
