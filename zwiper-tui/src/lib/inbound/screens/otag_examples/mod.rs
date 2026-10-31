//! Oracle tag examples: real cards that carry one tag, most played first, browsed one at a time.
//!
//! The app's deck-free swipe browse, read only: Left (swipe left) moves to the next card and Down (swipe down) steps back one. Cards come from `search_cards` in pages of [`PAGE_LIMIT`] by EDHREC rank, fetched again when few are left ahead, up to [`MAX_CARDS`]. An empty page marks the end.
//!
//! `d` opens the card's details sheet, whose `p` opens its printings read only. Nothing here changes anything on the server.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::time::Instant;

use ratatui::text::Line;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::error::InvalidCardCriteria;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::card_filter::{CardQuery, CardSortKey};
use zwipe_core::domain::deck::deck_metrics;
use zwipe_core::domain::user::models::hints::HINT_OTAG_EXAMPLES;

use super::Screen;
use crate::domain::stack::Direction;
use crate::domain::theme::Palette;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::card_motion::CardMotion;
use crate::inbound::widgets::sheet::Sheet;
use crate::inbound::widgets::toast::ToastKind;
use crate::inbound::widgets::{card_text, hint};
use crate::outbound::account::Account;

/// Cards per page, as the app's `PAGE_LIMIT`.
pub(crate) const PAGE_LIMIT: u32 = 25;

/// The next page loads once the cursor is this close to the end, as the app's `LOAD_MORE_THRESHOLD`.
pub(crate) const LOAD_MORE_THRESHOLD: usize = 5;

/// The browse stops paging at this many cards, as the app's `MAX_CARDS_IN_STACK`.
pub(crate) const MAX_CARDS: usize = 500;

/// The screen's title.
pub(crate) const TITLE: &str = "Examples";

/// Said past the last card.
pub(crate) const END: &str = "No more cards. Swipe down to go back.";

/// Said when the tag has no cards at all.
pub(crate) const EMPTY: &str = "No example cards for this tag yet.";

/// Said when stepping back from the first card.
pub(crate) const AT_FIRST: &str = "Already at the first card";

/// The browse: the tag, the cards fetched so far, the cursor, and the paging state.
#[derive(Debug)]
pub(crate) struct ExamplesView {
    pub(crate) slug: String,
    pub(crate) cards: Vec<Card>,
    /// The card on screen; equal to the count past the last card.
    pub(crate) index: usize,
    /// Where the next page starts.
    pub(crate) offset: u32,
    /// A page is on its way.
    pub(crate) loading: bool,
    /// An empty page came back or the cap was reached: no more pages.
    pub(crate) exhausted: bool,
    /// The details sheet over the card, when open.
    pub(crate) details: Option<Sheet>,
    /// The card flying off to the left, or the one coming back after a step back.
    pub(crate) motion: Option<CardMotion<Card>>,
}

impl ExamplesView {
    fn new(slug: &str) -> Self {
        Self {
            slug: slug.to_owned(),
            cards: Vec::new(),
            index: 0,
            offset: 0,
            loading: false,
            exhausted: false,
            details: None,
            motion: None,
        }
    }

    /// Whether a card is in motion, so the loop redraws.
    pub(crate) fn animating(&self) -> bool {
        self.motion
            .as_ref()
            .is_some_and(|m| m.running(Instant::now()))
    }

    /// The card under the one on screen, which a card coming back covers.
    pub(crate) fn under(&self) -> Option<&Card> {
        self.cards.get(self.index.checked_add(1)?)
    }

    /// The card on screen, `None` past the last one.
    pub(crate) fn current(&self) -> Option<&Card> {
        self.cards.get(self.index)
    }

    /// Asks for the next page unless one is on its way, the end was reached, or the browse holds [`MAX_CARDS`].
    pub(crate) fn load_more(&mut self, cx: &mut Context<'_>) {
        if self.loading || self.exhausted {
            return;
        }
        if self.cards.len() >= MAX_CARDS {
            self.exhausted = true;
            return;
        }
        self.loading = true;
        cx.request(
            Request::Page {
                slug: self.slug.clone(),
                offset: self.offset,
            }
            .into(),
        );
    }

    /// Moves to the next card, possibly past the last, and asks for more when few are left ahead.
    pub(crate) fn next(&mut self, cx: &mut Context<'_>) {
        let total = self.cards.len();
        let Some(card) = self.current().cloned() else {
            return;
        };
        self.motion = Some(CardMotion::exit(Direction::Left, card, Instant::now()));
        self.index = self.index.saturating_add(1);
        if self.index.saturating_add(1) >= total.saturating_sub(LOAD_MORE_THRESHOLD) {
            self.load_more(cx);
        }
    }

    /// Steps back one card, or says it is already at the first.
    pub(crate) fn back_one(&mut self, cx: &mut Context<'_>) {
        match self.index.checked_sub(1) {
            Some(index) => {
                self.index = index;
                self.motion = Some(CardMotion::enter(Direction::Left, Instant::now()));
            }
            None => cx.toast(ToastKind::Info, AT_FIRST.to_owned()),
        }
    }

    /// Opens the details sheet for the card on screen.
    pub(crate) fn open_details(&mut self, palette: &Palette) {
        self.details = self.current().map(|card| details_sheet(palette, card));
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// One page of the tag's cards from `offset`.
    Page { slug: String, offset: u32 },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Page { .. } => "page",
        }
    }
}

/// The examples' results.
#[derive(Debug)]
pub(crate) enum Update {
    Page {
        slug: String,
        offset: u32,
        cards: Vec<Card>,
    },
    PageFailed {
        slug: String,
        message: String,
    },
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

/// The search for one page of `slug`'s cards, as the app builds it.
///
/// # Errors
///
/// Returns [`InvalidCardCriteria`] when the criteria contradict each other, which these never do.
pub(crate) fn query_for(slug: &str, offset: u32) -> Result<CardQuery, InvalidCardCriteria> {
    CardQueryBuilder::new()
        .set_oracle_tags_contains_any(vec![slug.to_owned()])
        .set_is_token(false)
        .set_sort(CardSortKey::EdhrecRank)
        .set_limit(PAGE_LIMIT)
        .set_offset(offset)
        .build()
}

/// Opens the examples for `slug` and asks for the first page.
pub(crate) fn open(cx: &mut Context<'_>, slug: &str) {
    let mut view = ExamplesView::new(slug);
    view.load_more(cx);
    cx.push(Screen::OtagExamples(Box::new(view)));
    hint::once(cx, HINT_OTAG_EXAMPLES);
}

/// Shows the hint again, for its "?".
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    hint::reopen(cx, HINT_OTAG_EXAMPLES);
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let Request::Page { slug, offset } = request;
    let Ok(query) = query_for(&slug, offset) else {
        // A query that cannot be built has no cards, which ends the browse.
        return Some(
            Update::Page {
                slug,
                offset,
                cards: Vec::new(),
            }
            .into(),
        );
    };
    let client = &account.client;
    match async { client.search_cards(&query, &account.fresh().await?).await }.await {
        Ok(cards) => Some(
            Update::Page {
                slug,
                offset,
                cards,
            }
            .into(),
        ),
        Err(e) => Some(screens::failure(&e, |message| {
            Update::PageFailed { slug, message }.into()
        })),
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    match update {
        Update::Page {
            slug,
            offset,
            cards,
        } => {
            let Some(view) = nav.find_mut(pick) else {
                return;
            };
            if view.slug != slug || view.offset != offset || !view.loading {
                return;
            }
            view.loading = false;
            if cards.is_empty() {
                view.exhausted = true;
            } else {
                view.offset = offset.saturating_add(PAGE_LIMIT);
                view.cards.extend(cards);
            }
        }
        Update::PageFailed { slug, message } => {
            let Some(view) = nav.find_mut(pick) else {
                return;
            };
            if view.slug != slug {
                return;
            }
            view.loading = false;
            cx.toast(ToastKind::Error, message);
        }
    }
}
