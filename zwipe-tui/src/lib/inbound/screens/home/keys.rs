//! Home's keys: open the decks, account or config, switch the counters, refresh, sign out.

use crossterm::event::KeyCode;

use super::{HomeView, Request};
use crate::domain::home::Scope;
use crate::inbound::app::Context;
use crate::inbound::screens::{self, account, config, decks};

pub(crate) fn handle(view: &mut HomeView, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Esc => cx.back(),
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Char('d') | KeyCode::Enter => decks::open(cx),
        KeyCode::Char('a') => account::open(cx),
        KeyCode::Char('c') => config::open(cx),
        KeyCode::Char('s') => {
            view.scope = view.scope.toggled();
            if view.scope == Scope::OnlyMe {
                cx.request(Request::MyMetrics.into());
            }
        }
        KeyCode::Char('r') => view.refresh(cx),
        KeyCode::Char('O') => screens::sign_out(cx, "Signing out..."),
        _ => {}
    }
}
