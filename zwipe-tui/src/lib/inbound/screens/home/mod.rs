//! The home screen: the logo, the counters, the color map and the featured flavor card.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;

use super::{Screen, failure};
use crate::domain::decode;
use crate::domain::home::{Figures, Scope};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::outbound::account::Account;

/// The home screen's state.
#[derive(Debug, Default)]
pub(crate) struct HomeView {
    pub(crate) scope: Scope,
    pub(crate) everybody: Option<Figures>,
    pub(crate) mine: Option<Figures>,
    pub(crate) play_style: Option<HttpPlayStyleSummary>,
    pub(crate) play_style_failed: bool,
    /// The card the server features this hour. Shown only when it has flavor text.
    pub(crate) flavor: Option<Card>,
    /// When the logo started decoding in. `None` draws it settled.
    pub(crate) logo_started: Option<Instant>,
}

impl HomeView {
    /// A home screen whose logo starts decoding in now.
    pub(crate) fn opening() -> Self {
        Self {
            logo_started: Some(Instant::now()),
            ..Self::default()
        }
    }

    /// How far the logo's entrance has run; settled when it never started.
    pub(crate) fn logo_elapsed(&self) -> Duration {
        self.logo_started.map_or(decode::DURATION, |t| t.elapsed())
    }

    /// Whether the logo is still decoding in.
    pub(crate) fn animating(&self) -> bool {
        self.logo_elapsed() < decode::DURATION
    }

    /// The counters for the scope on screen, `None` until they arrive.
    pub(crate) fn figures(&self) -> Option<Figures> {
        match self.scope {
            Scope::Everybody => self.everybody,
            Scope::OnlyMe => self.mine,
        }
    }

    /// Starts over in the same scope: the logo decodes again and every panel is fetched again.
    pub(crate) fn refresh(&mut self, cx: &mut Context<'_>) {
        let scope = self.scope;
        *self = Self {
            scope,
            ..Self::opening()
        };
        fetch_panels(cx);
        if scope == Scope::OnlyMe {
            cx.request(Request::MyMetrics.into());
        }
    }
}

/// The home screen's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    /// Everybody's counters.
    PublicMetrics,
    /// The user's own counters.
    MyMetrics,
    /// The color map.
    PlayStyle,
    /// This hour's featured flavor card.
    Flavor,
}

/// The home screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    PublicMetrics(Figures),
    MyMetrics(Figures),
    PlayStyle(Box<HttpPlayStyleSummary>),
    PlayStyleFailed,
    Flavor(Box<Card>),
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Home(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Home(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut HomeView> {
    match screen {
        Screen::Home(view) => Some(view),
        _ => None,
    }
}

/// Starts over from home and fetches its counters, color map and featured card.
pub(crate) fn open(cx: &mut Context<'_>) {
    cx.reset(Screen::Home(Box::new(HomeView::opening())));
    fetch_panels(cx);
}

fn fetch_panels(cx: &mut Context<'_>) {
    for request in [Request::PublicMetrics, Request::PlayStyle, Request::Flavor] {
        cx.request(request.into());
    }
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let update = match request {
        Request::PublicMetrics => client
            .public_metrics()
            .await
            .ok()
            .map(|metrics| Update::PublicMetrics(Figures::from(&metrics)))?,
        Request::MyMetrics => async { client.my_metrics(&account.fresh().await?).await }
            .await
            .ok()
            .map(|counters| Update::MyMetrics(Figures::from(&counters)))?,
        Request::PlayStyle => {
            match async { client.play_style_summary(&account.fresh().await?).await }.await {
                Ok(summary) => Update::PlayStyle(Box::new(summary)),
                Err(e) => return Some(failure(&e, |_| Update::PlayStyleFailed.into())),
            }
        }
        Request::Flavor => client
            .featured_flavor()
            .await
            .ok()
            .map(|card| Update::Flavor(Box::new(card)))?,
    };
    Some(update.into())
}

pub(crate) fn apply(nav: &mut Nav, update: Update, _cx: &mut Context<'_>) {
    let Some(view) = nav.find_mut(pick) else {
        return;
    };
    match update {
        Update::PublicMetrics(figures) => view.everybody = Some(figures),
        Update::MyMetrics(figures) => view.mine = Some(figures),
        Update::PlayStyle(summary) => {
            view.play_style = Some(*summary);
            view.play_style_failed = false;
        }
        Update::PlayStyleFailed => view.play_style_failed = true,
        Update::Flavor(card) => view.flavor = Some(*card),
    }
}
