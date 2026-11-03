//! A loop and an app with no network, for driving screens with keys and results in tests.

use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use reqwest::Url;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::{Deck, Format};
use zwipe_core::http::contracts::deck::HttpDeckCardSearch;
use zwipe_ui::generation::Ticket;

use super::{EventLoop, Flow};
use crate::domain::theme::Palette;
use crate::inbound::app::{App, Context};
use crate::inbound::screens::swipe::SwipeView;
use crate::inbound::screens::{Request, Screen, Update, deck, decks, swipe};
use crate::outbound::account::{Account, SessionFile};
use crate::outbound::theme_file::ThemeFile;
use crate::test_support::{self, deck_profile};

/// A loop and its app, starting on the sign-in screen in the terminal's colors.
#[derive(Debug)]
pub(crate) struct Harness {
    pub(crate) event_loop: EventLoop,
    pub(crate) app: App,
    /// The requests [`Harness::clear`] has cleared, oldest first.
    cleared: Vec<Request>,
}

impl Harness {
    pub(crate) fn new() -> Self {
        let client = ZwipeClient::new(Url::parse("http://127.0.0.1:9").unwrap());
        let file = SessionFile::at(std::env::temp_dir().join("zwiper-tui-tests-no-session.json"));
        let theme_file = ThemeFile::at(std::env::temp_dir().join("zwiper-tui-tests-theme.json"));
        Self {
            event_loop: EventLoop::new(Arc::new(Account::new(client, file)), Arc::new(theme_file)),
            app: App::new(Palette::terminal()),
            cleared: Vec::new(),
        }
    }

    /// A harness with only `screen` open.
    pub(crate) fn on(screen: Screen) -> Self {
        let mut harness = Self::new();
        harness.app.nav.reset(screen);
        harness
    }

    /// Runs `open` the way a handler would, such as a screen's `open`, and settles what it asked for.
    pub(crate) fn open(&mut self, open: impl FnOnce(&mut Context<'_>)) {
        let mut cx = Context::new(&mut self.app.shared);
        open(&mut cx);
        let effects = cx.into_effects();
        self.event_loop.settle(&mut self.app, effects);
    }

    /// Runs `act` on the screen on top with a handler's [`Context`], such as a dispatch no key reaches, and settles what it asked for.
    pub(crate) fn on_top(&mut self, act: impl FnOnce(&mut Screen, &mut Context<'_>)) {
        let App { nav, shared, .. } = &mut self.app;
        let mut cx = Context::new(shared);
        act(nav.top_mut(), &mut cx);
        let effects = cx.into_effects();
        self.event_loop.settle(&mut self.app, effects);
    }

    pub(crate) fn press(&mut self, code: KeyCode) -> Flow {
        self.event_loop
            .handle_key(&mut self.app, KeyEvent::new(code, KeyModifiers::NONE))
    }

    pub(crate) fn press_event(&mut self, key: KeyEvent) -> Flow {
        self.event_loop.handle_key(&mut self.app, key)
    }

    pub(crate) fn type_text(&mut self, text: &str) {
        for c in text.chars() {
            self.press(KeyCode::Char(c));
        }
    }

    pub(crate) fn apply(&mut self, update: impl Into<Update>) -> Flow {
        self.event_loop.apply(&mut self.app, update.into())
    }

    pub(crate) fn start(&mut self) {
        self.event_loop.start(&mut self.app, None);
    }

    /// The requests queued since the last [`Harness::clear`].
    pub(crate) fn pending(&self) -> &[Request] {
        &self.event_loop.pending
    }

    pub(crate) fn clear(&mut self) {
        self.cleared.append(&mut self.event_loop.pending);
    }

    /// Every request queued since the harness began, cleared or not, oldest first.
    pub(crate) fn sent(&self) -> impl DoubleEndedIterator<Item = &Request> {
        self.cleared.iter().chain(&self.event_loop.pending)
    }

    pub(crate) fn screen(&self) -> &Screen {
        self.app.nav.top()
    }

    pub(crate) fn screen_mut(&mut self) -> &mut Screen {
        self.app.nav.top_mut()
    }

    /// Home, then the deck list, then a fresh commander deck, then swiping into it with `cards` served as the first page, with nothing left queued.
    pub(crate) fn swiping(cards: Vec<Card>) -> (Self, Uuid) {
        let mut harness = Self::on(Screen::Home(Box::default()));
        let mut profile = deck_profile("Pile");
        profile.commander_id = Some(Uuid::new_v4());
        profile.format = Some(Format::Commander);
        let deck_id = profile.id;
        harness.open(decks::open);
        let ticket = decks::last_ticket(harness.pending());
        harness.apply(decks::Update::Loaded {
            ticket,
            decks: vec![profile.clone()],
        });
        harness.press(KeyCode::Enter);
        let pile = test_support::deck(profile, Vec::new());
        deck::serve(&mut harness, pile.clone());
        harness.press(KeyCode::Char('s'));
        harness.serve_deck(pile);
        harness.serve(cards);
        harness.clear();
        (harness, deck_id)
    }

    /// The ticket of the newest swipe screen request `pick` accepts, or a panic naming what is queued.
    pub(crate) fn swipe_ticket(&self, pick: impl Fn(&swipe::Request) -> Option<Ticket>) -> Ticket {
        self.pending()
            .iter()
            .rev()
            .find_map(|r| match r {
                Request::Swipe(r) => pick(r),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no such swipe request in {:?}", self.pending()))
    }

    /// Answers the swipe screen's deck fetch with `deck`.
    pub(crate) fn serve_deck(&mut self, deck: Deck) {
        let ticket = self.swipe_ticket(|r| match r {
            swipe::Request::Deck { ticket, .. }
            | swipe::Request::ReloadMaybeboard { ticket, .. } => Some(*ticket),
            _ => None,
        });
        self.apply(swipe::Update::DeckLoaded {
            ticket,
            deck: Box::new(deck),
        });
    }

    /// Answers the swipe screen's newest search from the top with `cards`, synergy applied.
    pub(crate) fn serve(&mut self, cards: Vec<Card>) {
        let ticket = self.swipe_ticket(|r| match r {
            swipe::Request::Search { ticket, .. } => Some(*ticket),
            _ => None,
        });
        self.apply(swipe::Update::Searched {
            ticket,
            page: HttpDeckCardSearch {
                cards,
                synergy_applied: true,
                related_from: None,
            },
        });
    }

    /// The swipe screen's state, or a panic naming the screen on top.
    pub(crate) fn swipe_view(&self) -> &SwipeView {
        match self.screen() {
            Screen::Swipe(view) => view,
            other => panic!("not on the swipe screen: {other:?}"),
        }
    }
}
