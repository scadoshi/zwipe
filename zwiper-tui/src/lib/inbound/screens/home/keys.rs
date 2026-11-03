//! Home's keys: the action bar's Decks, Account and Config, the hero's scope and Activity, the color map's Playstyle and "?", and the featured card's Image and Buy. Presses the model has an action for go through [`dispatch`].

use crossterm::event::KeyCode;
use zwipe_ui::screens::home::{Action, BUY_BODY, BUY_TITLE, CLOSE, Flavor};

use super::{HomeView, Overlay, dispatch, toggle_scope};
use crate::inbound::app::Context;
use crate::inbound::screens::{self, image_preview};
use crate::inbound::widgets::alert::{AlertChoice, AlertDialog};
use crate::inbound::widgets::{browser, help, info};

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
                    buy(view, cx, i);
                }
                Some(AlertChoice::Dismissed) => view.overlay = None,
                None => {}
            },
        }
        return;
    }
    match code {
        KeyCode::Esc => cx.back(),
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Char('d') | KeyCode::Enter => dispatch(view, Action::Decks, cx),
        KeyCode::Char('a') => dispatch(view, Action::Account, cx),
        KeyCode::Char('c') => dispatch(view, Action::Config, cx),
        KeyCode::Char('v') => dispatch(view, Action::Activity, cx),
        KeyCode::Char('p') => dispatch(view, Action::Playstyle, cx),
        KeyCode::Char('s') => toggle_scope(view, cx),
        KeyCode::Char('?') => view.overlay = Some(Overlay::Info(color_map_topic())),
        KeyCode::Char('i') => {
            if let Flavor::Quote(_) = view.model.flavor()
                && let Some(card) = view.model.flavor_card()
            {
                image_preview::open(cx, card.clone(), 0);
            }
        }
        KeyCode::Char('b') => {
            if let Flavor::Quote(quote) = view.model.flavor()
                && quote.price.is_some()
            {
                view.overlay = Some(Overlay::Buy(buy_dialog(view)));
            }
        }
        KeyCode::Char('r') => view.refresh(cx),
        KeyCode::Char('O') => screens::sign_out(cx, "Signing out..."),
        _ => {}
    }
}

/// The Buy dialog: Close, then the model's retailer links, `t` and `k`.
pub(crate) fn buy_dialog(view: &HomeView) -> AlertDialog {
    let keys = ['t', 'k'];
    view.model.buy_links().iter().zip(keys).fold(
        AlertDialog::new(BUY_TITLE, BUY_BODY).with_button(CLOSE, None),
        |dialog, (link, key)| dialog.with_button(link.label, Some(key)),
    )
}

/// Opens the retailer behind the Buy dialog's button `index`, if it is one.
fn buy(view: &HomeView, cx: &mut Context<'_>, index: usize) {
    let link = index
        .checked_sub(1)
        .and_then(|at| view.model.buy_links().into_iter().nth(at));
    if let Some(link) = link {
        browser::open(cx, &link.url);
    }
}
