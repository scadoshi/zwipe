//! The main loop: keys change the [`App`], background requests feed it, and every pass draws it.
//!
//! Handlers never touch the network. They queue a [`Request`]; [`EventLoop::dispatch`] spawns each as a task that answers with an [`Update`].

use std::sync::Arc;
use std::{fmt, mem};

use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use uuid::Uuid;
use zwipe_client::ClientError;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::deck::{Deck, DeckProfile};
use zwipe_core::http::contracts::deck::HttpDeckCardSearch;
use zwipe_core::http::contracts::deck_card::HttpCreateDeckCard;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;
use zwipe_core::version::version_at_least;

use super::app::{
    App, DeckList, DeckView, HomeView, LoginField, LoginForm, Screen, Step, SwipeView,
};
use super::ui;
use crate::domain::decode;
use crate::domain::home::{Figures, Scope};
use crate::domain::swipe::{self, Effect, Stack, Swipe};
use crate::domain::theme::Palette;
use crate::outbound::account::Account;
use crate::outbound::theme_file::{RememberedTheme, ThemeFile};

/// This build's version, which the server's minimum client version is checked against.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Shown when the commander's synergy picks run out and the server moves on to cards like the deck.
const RELATED_NOTICE: &str = "That's every synergy pick. Showing cards like your deck.";

/// A password on its way to the server, never shown in `Debug` output.
#[derive(Clone, PartialEq, Eq)]
struct Password(String);

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Password(<redacted>)")
    }
}

/// Work a handler asks for.
#[derive(Debug, Clone, PartialEq)]
enum Request {
    /// Compares this build with the server's minimum client version.
    CheckVersion,
    SignIn {
        identifier: String,
        password: Password,
    },
    /// Everybody's counters.
    PublicMetrics,
    /// The user's own counters.
    MyMetrics,
    /// The color map.
    PlayStyle,
    /// This hour's featured flavor card.
    Flavor,
    LoadDecks,
    LoadDeck(Uuid),
    FetchCards {
        deck_id: Uuid,
        query: Box<CardQuery>,
    },
    /// Sends a swipe or an undo.
    Send {
        deck_id: Uuid,
        effect: Effect,
    },
    SignOut,
    /// Saves the account's theme for drawing before the next sign-in.
    RememberTheme(RememberedTheme),
}

/// A result from a background request.
#[derive(Debug)]
enum Update {
    /// The server's minimum client version is newer than this build.
    TooOld,
    SignedIn(Box<Session>),
    SignInFailed(String),
    PublicMetrics(Figures),
    MyMetrics(Figures),
    PlayStyle(Box<HttpPlayStyleSummary>),
    PlayStyleFailed,
    Flavor(Box<Card>),
    Decks(Vec<DeckProfile>),
    DecksFailed(String),
    DeckLoaded(Box<Deck>),
    DeckFailed(String),
    Cards {
        deck_id: Uuid,
        page: HttpDeckCardSearch,
    },
    CardsFailed(String),
    Status(String),
    /// The server no longer accepts the session.
    SessionEnded,
    SignedOut,
}

#[derive(Debug, PartialEq, Eq)]
enum Flow {
    Continue,
    Quit,
}

/// Owns the request queue and the channel results come back on.
#[derive(Debug)]
pub(crate) struct EventLoop {
    tx: UnboundedSender<Update>,
    rx: UnboundedReceiver<Update>,
    pending: Vec<Request>,
    account: Arc<Account>,
    theme_file: Arc<ThemeFile>,
}

impl EventLoop {
    pub(crate) fn new(account: Arc<Account>, theme_file: Arc<ThemeFile>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tx,
            rx,
            pending: Vec::new(),
            account,
            theme_file,
        }
    }

    /// Runs until the user quits: straight home with a saved session, else to sign-in.
    ///
    /// # Errors
    ///
    /// Fails when the terminal cannot be drawn to or read from.
    pub(crate) async fn run(mut self, terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
        let session = self.account.session().await;
        self.start(app, session.as_ref());
        let mut events = EventStream::new();

        loop {
            self.dispatch();
            terminal.draw(|f| ui::draw(f, app))?;

            let animating = app.animating();
            tokio::select! {
                Some(update) = self.rx.recv() => self.apply(app, update),
                () = tokio::time::sleep(decode::TICK), if animating => {}
                event = events.next() => match event {
                    Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                        if self.handle_key(app, key) == Flow::Quit {
                            return Ok(());
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => return Err(e.into()),
                    None => return Ok(()),
                },
            }
        }
    }

    fn start(&mut self, app: &mut App, session: Option<&Session>) {
        self.pending.push(Request::CheckVersion);
        match session {
            Some(session) => self.signed_in(app, session),
            None => app.screen = Screen::Login(LoginForm::default()),
        }
    }

    /// Applies the account's theme, remembers it for the next launch, and goes home.
    fn signed_in(&mut self, app: &mut App, session: &Session) {
        let preferences = &session.preferences;
        app.palette = Palette::for_account(&preferences.theme, preferences.dark_mode);
        self.pending.push(Request::RememberTheme(RememberedTheme {
            theme: preferences.theme.clone(),
            dark_mode: preferences.dark_mode,
        }));
        let username = session.user.username.to_string();
        app.status = format!("Hello, {username}!");
        app.username = Some(username);
        self.open_home(app);
    }

    /// Opens home and fetches its counters, color map and featured card.
    fn open_home(&mut self, app: &mut App) {
        app.screen = Screen::Home(Box::new(HomeView::opening()));
        self.pending
            .extend([Request::PublicMetrics, Request::PlayStyle, Request::Flavor]);
    }

    fn open_decks(&mut self, app: &mut App) {
        app.status.clear();
        app.screen = Screen::Decks(DeckList::loading());
        self.pending.push(Request::LoadDecks);
    }

    fn open_deck(&mut self, app: &mut App, profile: DeckProfile) {
        app.status.clear();
        self.pending.push(Request::LoadDeck(profile.id));
        app.screen = Screen::Deck(Box::new(DeckView::opening(profile)));
    }

    /// Back to sign-in, still in the account's theme, with `error` on the form.
    fn signed_out(app: &mut App, error: Option<String>) {
        app.username = None;
        app.status.clear();
        app.screen = Screen::Login(LoginForm {
            error,
            ..LoginForm::default()
        });
    }

    fn apply(&mut self, app: &mut App, update: Update) {
        match update {
            Update::TooOld => app.screen = Screen::UpdateRequired,
            Update::SignedIn(session) => {
                if matches!(app.screen, Screen::Login(_)) {
                    self.signed_in(app, &session);
                }
            }
            Update::SignInFailed(error) => {
                if let Screen::Login(form) = &mut app.screen {
                    form.busy = false;
                    form.error = Some(error);
                }
            }
            Update::PublicMetrics(figures) => {
                if let Screen::Home(home) = &mut app.screen {
                    home.everybody = Some(figures);
                }
            }
            Update::MyMetrics(figures) => {
                if let Screen::Home(home) = &mut app.screen {
                    home.mine = Some(figures);
                }
            }
            Update::PlayStyle(summary) => {
                if let Screen::Home(home) = &mut app.screen {
                    home.play_style = Some(*summary);
                    home.play_style_failed = false;
                }
            }
            Update::PlayStyleFailed => {
                if let Screen::Home(home) = &mut app.screen {
                    home.play_style_failed = true;
                }
            }
            Update::Flavor(card) => {
                if let Screen::Home(home) = &mut app.screen {
                    home.flavor = Some(*card);
                }
            }
            Update::Decks(decks) => {
                if let Screen::Decks(list) = &mut app.screen {
                    list.set_decks(decks);
                }
            }
            Update::DecksFailed(error) => {
                if let Screen::Decks(list) = &mut app.screen {
                    list.loading = false;
                    list.error = Some(error);
                }
            }
            Update::DeckLoaded(deck) => {
                if let Screen::Deck(view) = &mut app.screen
                    && view.profile.id == deck.deck_profile.id
                {
                    view.load(*deck);
                }
            }
            Update::DeckFailed(error) => {
                if let Screen::Deck(view) = &mut app.screen {
                    view.error = Some(error);
                }
            }
            Update::CardsFailed(error) => {
                if let Screen::Swipe(view) = &mut app.screen {
                    view.fetching = false;
                    view.error = Some(error);
                }
            }
            Update::Cards { deck_id, page } => {
                if let Screen::Swipe(view) = &mut app.screen
                    && view.deck.id == deck_id
                    && let Some(stack) = &mut view.stack
                {
                    stack.push_page(page);
                    view.fetching = false;
                    self.refill(view);
                }
            }
            Update::Status(status) => app.status = status,
            Update::SessionEnded => {
                Self::signed_out(app, Some("Your session ended. Sign in again.".to_owned()));
                self.pending.push(Request::SignOut);
            }
            Update::SignedOut => Self::signed_out(app, None),
        }
    }

    /// Starts swiping into the loaded deck: builds its card query and fetches the first page.
    fn start_swipe(&mut self, app: &mut App, deck: &Deck) {
        let mut view = SwipeView::opening(deck.deck_profile.clone());
        match swipe::query_for(deck) {
            Ok(query) => {
                view.stack = Some(Stack::new(swipe::cards_in(deck)));
                view.query = Some(query);
                self.refill(&mut view);
            }
            Err(e) => view.error = Some(format!("Could not build the card search: {e}")),
        }
        app.status.clear();
        app.screen = Screen::Swipe(Box::new(view));
    }

    /// Fetches the next page when the stack is running low and no fetch is out.
    fn refill(&mut self, view: &mut SwipeView) {
        let (Some(stack), Some(query)) = (&view.stack, &view.query) else {
            return;
        };
        if !view.fetching && stack.needs_more() {
            view.fetching = true;
            self.pending.push(Request::FetchCards {
                deck_id: view.deck.id,
                query: Box::new(query.clone()),
            });
        }
    }

    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Flow {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Flow::Quit;
        }
        match &mut app.screen {
            Screen::Login(form) => return self.handle_login_key(form, key.code),
            Screen::Home(_) => return self.handle_home_key(app, key.code),
            Screen::Decks(_) => return self.handle_decks_key(app, key.code),
            Screen::Deck(_) => return self.handle_deck_key(app, key.code),
            Screen::Swipe(_) => self.handle_swipe_key(app, key.code),
            Screen::UpdateRequired => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                    return Flow::Quit;
                }
            }
        }
        Flow::Continue
    }

    fn handle_login_key(&mut self, form: &mut LoginForm, code: KeyCode) -> Flow {
        match code {
            KeyCode::Esc => return Flow::Quit,
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Up | KeyCode::Down => {
                form.focus = form.focus.other();
            }
            KeyCode::Enter if form.focus == LoginField::Identifier => {
                form.focus = LoginField::Password;
            }
            KeyCode::Enter => self.submit(form),
            KeyCode::Backspace => {
                form.focused_mut().pop();
            }
            KeyCode::Char(c) => form.focused_mut().push(c),
            _ => {}
        }
        Flow::Continue
    }

    fn submit(&mut self, form: &mut LoginForm) {
        if form.busy {
            return;
        }
        let identifier = form.identifier.trim();
        if identifier.is_empty() || form.password.is_empty() {
            form.error = Some("Enter your username or email and your password.".to_owned());
            return;
        }
        form.busy = true;
        form.error = None;
        self.pending.push(Request::SignIn {
            identifier: identifier.to_owned(),
            password: Password(form.password.clone()),
        });
    }

    fn handle_home_key(&mut self, app: &mut App, code: KeyCode) -> Flow {
        let Screen::Home(home) = &mut app.screen else {
            return Flow::Continue;
        };
        match code {
            KeyCode::Esc | KeyCode::Char('q') => return Flow::Quit,
            KeyCode::Char('d') | KeyCode::Enter => self.open_decks(app),
            KeyCode::Char('s') => {
                home.scope = home.scope.toggled();
                if home.scope == Scope::OnlyMe {
                    self.pending.push(Request::MyMetrics);
                }
            }
            KeyCode::Char('r') => {
                let scope = home.scope;
                self.open_home(app);
                if let Screen::Home(home) = &mut app.screen {
                    home.scope = scope;
                }
                if scope == Scope::OnlyMe {
                    self.pending.push(Request::MyMetrics);
                }
            }
            KeyCode::Char('O') => self.sign_out(app),
            _ => {}
        }
        Flow::Continue
    }

    fn handle_decks_key(&mut self, app: &mut App, code: KeyCode) -> Flow {
        let Screen::Decks(list) = &mut app.screen else {
            return Flow::Continue;
        };
        match code {
            KeyCode::Char('q') => return Flow::Quit,
            KeyCode::Esc => self.open_home(app),
            KeyCode::Down | KeyCode::Char('j') => list.next(),
            KeyCode::Up | KeyCode::Char('k') => list.previous(),
            KeyCode::Char('r') => self.open_decks(app),
            KeyCode::Char('O') => self.sign_out(app),
            KeyCode::Enter => {
                if let Some(profile) = list.selected_deck().cloned() {
                    self.open_deck(app, profile);
                }
            }
            _ => {}
        }
        Flow::Continue
    }

    fn handle_deck_key(&mut self, app: &mut App, code: KeyCode) -> Flow {
        let Screen::Deck(view) = &mut app.screen else {
            return Flow::Continue;
        };
        match code {
            KeyCode::Char('q') => return Flow::Quit,
            KeyCode::Esc => self.open_decks(app),
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => view.step_section(Step::Next),
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                view.step_section(Step::Previous);
            }
            KeyCode::Char('c') => view.next_currency(),
            KeyCode::Char('+' | '=') => view.later_turn(),
            KeyCode::Char('-') => view.earlier_turn(),
            KeyCode::Char('r') => {
                let profile = view.profile.clone();
                self.open_deck(app, profile);
            }
            KeyCode::Char('s') => {
                if let Some(deck) = view.deck.take() {
                    self.start_swipe(app, &deck);
                }
            }
            _ => {}
        }
        Flow::Continue
    }

    fn sign_out(&mut self, app: &mut App) {
        "Signing out...".clone_into(&mut app.status);
        self.pending.push(Request::SignOut);
    }

    fn handle_swipe_key(&mut self, app: &mut App, code: KeyCode) {
        let Screen::Swipe(view) = &mut app.screen else {
            return;
        };
        if matches!(code, KeyCode::Esc | KeyCode::Char('q')) {
            let profile = view.deck.clone();
            self.open_deck(app, profile);
            return;
        }
        let Some(stack) = &mut view.stack else {
            return;
        };
        let swiped = match code {
            KeyCode::Right | KeyCode::Char('l') => Some(Swipe::Add),
            KeyCode::Left | KeyCode::Char('h') => Some(Swipe::Pass),
            KeyCode::Up | KeyCode::Char('m') => Some(Swipe::Maybe),
            _ => None,
        };
        if let Some(swiped) = swiped {
            if stack.current().is_none() {
                return;
            }
            if swiped == Swipe::Add {
                view.added = view.added.saturating_add(1);
            }
            if let Some(effect) = stack.swipe(swiped) {
                self.pending.push(Request::Send {
                    deck_id: view.deck.id,
                    effect,
                });
            }
        } else if matches!(
            code,
            KeyCode::Down | KeyCode::Char('u') | KeyCode::Backspace
        ) {
            let Some((undone, effect)) = stack.undo() else {
                return;
            };
            if undone == Swipe::Add {
                view.added = view.added.saturating_sub(1);
            }
            if let Some(effect) = effect {
                self.pending.push(Request::Send {
                    deck_id: view.deck.id,
                    effect,
                });
            }
        } else {
            return;
        }

        app.status = if stack.at_related_start() {
            RELATED_NOTICE.to_owned()
        } else {
            String::new()
        };
        self.refill(view);
    }

    fn dispatch(&mut self) {
        for request in mem::take(&mut self.pending) {
            let tx = self.tx.clone();
            let account = Arc::clone(&self.account);
            let theme_file = Arc::clone(&self.theme_file);
            tokio::spawn(async move {
                if let Some(update) = perform(&account, &theme_file, request).await {
                    tx.send(update).ok();
                }
            });
        }
    }
}

/// Runs one request.
async fn perform(account: &Account, theme_file: &ThemeFile, request: Request) -> Option<Update> {
    let client = &account.client;
    match request {
        Request::CheckVersion => {
            let minimum = client.fetch_min_client_version().await.ok()?;
            (!version_at_least(VERSION, &minimum.min_version)).then_some(Update::TooOld)
        }
        Request::SignIn {
            identifier,
            password,
        } => Some(match account.sign_in(&identifier, &password.0).await {
            Ok(session) => Update::SignedIn(Box::new(session)),
            Err(e) => Update::SignInFailed(e.to_user_message()),
        }),
        Request::SignOut => {
            account.sign_out().await;
            Some(Update::SignedOut)
        }
        Request::RememberTheme(theme) => {
            // Only the next launch's first screen depends on it, so a failed write is not worth a message.
            theme_file.save(&theme).ok();
            None
        }
        Request::PublicMetrics => client
            .public_metrics()
            .await
            .ok()
            .map(|metrics| Update::PublicMetrics(Figures::from(&metrics))),
        Request::MyMetrics => async { client.my_metrics(&account.fresh().await?).await }
            .await
            .ok()
            .map(|counters| Update::MyMetrics(Figures::from(&counters))),
        Request::PlayStyle => Some(
            match async { client.play_style_summary(&account.fresh().await?).await }.await {
                Ok(summary) => Update::PlayStyle(Box::new(summary)),
                Err(e) => failure(&e, |_| Update::PlayStyleFailed),
            },
        ),
        Request::Flavor => client
            .featured_flavor()
            .await
            .ok()
            .map(|card| Update::Flavor(Box::new(card))),
        Request::LoadDecks => Some(
            match async { client.fetch_deck_profiles(&account.fresh().await?).await }.await {
                Ok(decks) => Update::Decks(decks),
                Err(e) => failure(&e, Update::DecksFailed),
            },
        ),
        Request::LoadDeck(deck_id) => Some(
            match async { client.fetch_deck(deck_id, &account.fresh().await?).await }.await {
                Ok(deck) => Update::DeckLoaded(Box::new(deck)),
                Err(e) => failure(&e, Update::DeckFailed),
            },
        ),
        Request::FetchCards { deck_id, query } => Some(
            match async {
                let session = account.fresh().await?;
                client.search_deck_cards(deck_id, &query, &session).await
            }
            .await
            {
                Ok(page) => Update::Cards { deck_id, page },
                Err(e) => failure(&e, Update::CardsFailed),
            },
        ),
        Request::Send { deck_id, effect } => send(account, deck_id, effect).await.err().map(|e| {
            failure(&e, |m| {
                Update::Status(format!("That swipe did not save: {m}"))
            })
        }),
    }
}

/// Sends one swipe or undo.
async fn send(account: &Account, deck_id: Uuid, effect: Effect) -> Result<(), ClientError> {
    let client = &account.client;
    let session = account.fresh().await?;
    match effect {
        Effect::Add {
            scryfall_data_id,
            oracle_id,
            maybeboard,
        } => {
            let request = HttpCreateDeckCard {
                scryfall_data_id: scryfall_data_id.to_string(),
                oracle_id: oracle_id.map(|id| id.to_string()).unwrap_or_default(),
                quantity: 1,
                board: maybeboard.then(|| "maybeboard".to_owned()),
            };
            client
                .create_deck_card(deck_id, &request, &session)
                .await
                .map(drop)
        }
        Effect::Skip { oracle_id } => client.skip_deck_card(deck_id, oracle_id, &session).await,
        Effect::Unskip { oracle_id } => client.unskip_deck_card(deck_id, oracle_id, &session).await,
        Effect::Remove { scryfall_data_id } => {
            client
                .delete_deck_card(deck_id, scryfall_data_id, &session)
                .await
        }
    }
}

/// An update for a failed request: the session ending when the server rejects it, else `report` with a message fit to show.
fn failure(error: &ClientError, report: impl FnOnce(String) -> Update) -> Update {
    match error {
        ClientError::Unauthorized(_) => Update::SessionEnded,
        other => report(other.to_user_message()),
    }
}

#[cfg(test)]
mod tests;
