//! A loop and an app with no network, for driving screens with keys and results in tests.

use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use reqwest::Url;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::Format;
use zwipe_core::http::contracts::deck::HttpDeckCardSearch;

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
}

impl Harness {
    pub(crate) fn new() -> Self {
        let client = ZwipeClient::new(Url::parse("http://127.0.0.1:9").unwrap());
        let file = SessionFile::at(std::env::temp_dir().join("zwipe-tui-tests-no-session.json"));
        let theme_file = ThemeFile::at(std::env::temp_dir().join("zwipe-tui-tests-theme.json"));
        Self {
            event_loop: EventLoop::new(Arc::new(Account::new(client, file)), Arc::new(theme_file)),
            app: App::new(Palette::terminal()),
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
        self.event_loop.pending.clear();
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
        harness.apply(decks::Update::Loaded(vec![profile.clone()]));
        harness.press(KeyCode::Enter);
        harness.apply(deck::Update::Loaded(Box::new(test_support::deck(
            profile,
            Vec::new(),
        ))));
        harness.press(KeyCode::Char('s'));
        harness.serve(cards);
        harness.clear();
        (harness, deck_id)
    }

    /// Answers the swipe screen's open search with `cards`, synergy applied.
    pub(crate) fn serve(&mut self, cards: Vec<Card>) {
        let view = self.swipe_view();
        let update = swipe::Update::Searched {
            deck_id: view.deck.id,
            generation: view.generation,
            filter: Box::new(view.filter.clone()),
            page: swipe::Page::First,
            result: Ok(HttpDeckCardSearch {
                cards,
                synergy_applied: true,
                related_from: None,
            }),
        };
        self.apply(update);
    }

    /// The swipe screen's state, or a panic naming the screen on top.
    pub(crate) fn swipe_view(&self) -> &SwipeView {
        match self.screen() {
            Screen::Swipe(view) => view,
            other => panic!("not on the swipe screen: {other:?}"),
        }
    }
}
