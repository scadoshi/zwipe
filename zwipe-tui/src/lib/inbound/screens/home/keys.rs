//! Home's keys: the action bar's Decks, Account and Config, the hero's scope and Activity, the color map's Playstyle and "?", and the featured card's Image and Buy.

use crossterm::event::KeyCode;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics;

use super::{HomeView, Overlay};
use crate::domain::home;
use crate::inbound::app::Context;
use crate::inbound::screens::{self, account, config, decks, image_preview};
use crate::inbound::widgets::alert::{AlertChoice, AlertDialog};
use crate::inbound::widgets::{browser, info};

/// The Buy dialog's title.
pub(crate) const BUY_TITLE: &str = "Buy card";

/// The Buy dialog's body.
pub(crate) const BUY_BODY: &str = "Buy this card from a retailer.";

/// The color map's "?" topic.
pub(crate) fn color_map_topic() -> info::InfoPopup {
    info::InfoPopup::new(
        "Color map",
        &[
            "Each tile stands for a share of the colored cards across your decks",
            "Every color grows out from the middle, and the color with the most cards claims the most ground",
            "The shape is generated per player",
            "Colorless cards are in the key below",
        ],
    )
}

pub(crate) fn handle(view: &mut HomeView, code: KeyCode, cx: &mut Context<'_>) {
    if let Some(overlay) = &mut view.overlay {
        match overlay {
            Overlay::Info(_) => {
                if info::closes_on(code) {
                    view.overlay = None;
                }
            }
            Overlay::Buy(dialog) => match dialog.handle_key(code) {
                Some(AlertChoice::Pressed(i)) => {
                    view.overlay = None;
                    buy(cx, i);
                }
                Some(AlertChoice::Dismissed) => view.overlay = None,
                None => {}
            },
        }
        return;
    }
    match code {
        KeyCode::Esc => cx.back(),
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Char('d') | KeyCode::Enter => decks::open(cx),
        KeyCode::Char('a') => account::open(cx),
        KeyCode::Char('c') => config::open(cx),
        KeyCode::Char('v') => screens::activity::open(cx),
        KeyCode::Char('p') => screens::playstyle::open(cx),
        KeyCode::Char('s') => view.toggle_scope(cx),
        KeyCode::Char('?') => view.overlay = Some(Overlay::Info(color_map_topic())),
        KeyCode::Char('i') => {
            if let Some(card) = featured(cx) {
                image_preview::open(cx, card, 0);
            }
        }
        KeyCode::Char('b') => {
            let priced = featured(cx).is_some_and(|card| {
                deck_metrics::card_price(&card.scryfall_data, PriceCurrency::Usd).is_some()
            });
            if priced {
                view.overlay = Some(Overlay::Buy(buy_dialog()));
            }
        }
        KeyCode::Char('r') => view.refresh(cx),
        KeyCode::Char('O') => screens::sign_out(cx, "Signing out..."),
        _ => {}
    }
}

/// The featured card when it has flavor text, the only time its panel shows.
fn featured(cx: &Context<'_>) -> Option<zwipe_core::domain::card::Card> {
    cx.shared
        .caches
        .flavor
        .as_ref()
        .map(|f| &f.card)
        .filter(|card| has_flavor(card))
        .cloned()
}

/// Whether a card has flavor text to quote.
pub(crate) fn has_flavor(card: &zwipe_core::domain::card::Card) -> bool {
    card.scryfall_data
        .flavor_text
        .as_deref()
        .is_some_and(|t| !t.trim().is_empty())
}

/// The Buy dialog: Close, then the two retailers.
pub(crate) fn buy_dialog() -> AlertDialog {
    AlertDialog::new(BUY_TITLE, BUY_BODY)
        .with_button("Close", None)
        .with_button("TCGplayer \u{2197}", Some('t'))
        .with_button("Card Kingdom \u{2197}", Some('k'))
}

/// Opens the retailer behind the Buy dialog's button `index`, if it is one.
fn buy(cx: &mut Context<'_>, index: usize) {
    let Some(card) = featured(cx) else {
        return;
    };
    let data = &card.scryfall_data;
    let url = match index {
        1 => home::tcgplayer_card_url(&data.name, data.tcgplayer_id),
        2 => home::cardkingdom_card_url(&data.name),
        _ => return,
    };
    browser::open(cx, &url);
}
