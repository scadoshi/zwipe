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
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::deck::{Deck, DeckProfile};
use zwipe_core::http::contracts::deck::HttpDeckCardSearch;
use zwipe_core::http::contracts::deck_card::HttpCreateDeckCard;
use zwipe_core::version::version_at_least;

use super::app::{App, DeckList, LoginField, LoginForm, Screen, SwipeView};
use super::ui;
use crate::domain::swipe::{self, Effect, Stack, Swipe};
use crate::domain::theme::Palette;
use crate::outbound::account::Account;

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
    LoadDecks,
    OpenDeck(Uuid),
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
}

/// A result from a background request.
#[derive(Debug)]
enum Update {
    /// The server's minimum client version is newer than this build.
    TooOld,
    SignedIn(Box<Session>),
    SignInFailed(String),
    Decks(Vec<DeckProfile>),
    DecksFailed(String),
    DeckOpened(Box<Deck>),
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
}

impl EventLoop {
    pub(crate) fn new(account: Arc<Account>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tx,
            rx,
            pending: Vec::new(),
            account,
        }
    }

    /// Runs until the user quits: straight to the decks with a saved session, else to sign-in.
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

            tokio::select! {
                Some(update) = self.rx.recv() => self.apply(app, update),
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

    /// Applies the account's theme and opens the deck list.
    fn signed_in(&mut self, app: &mut App, session: &Session) {
        let preferences = &session.preferences;
        app.palette = Palette::for_account(&preferences.theme, preferences.dark_mode);
        app.username = Some(session.user.username.to_string());
        self.open_decks(app);
    }

    fn open_decks(&mut self, app: &mut App) {
        app.screen = Screen::Decks(DeckList::loading());
        self.pending.push(Request::LoadDecks);
    }

    /// Back to sign-in in the terminal's colors, with `error` on the form.
    fn signed_out(app: &mut App, error: Option<String>) {
        app.palette = Palette::terminal();
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
            Update::Decks(decks) => {
                if let Screen::Decks(list) = &mut app.screen {
                    list.decks = decks;
                    list.loading = false;
                    list.selected = list.selected.min(list.decks.len().saturating_sub(1));
                }
            }
            Update::DecksFailed(error) => {
                if let Screen::Decks(list) = &mut app.screen {
                    list.loading = false;
                    list.error = Some(error);
                }
            }
            Update::DeckOpened(deck) => self.on_deck_opened(app, &deck),
            Update::DeckFailed(error) | Update::CardsFailed(error) => {
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

    /// Builds the deck's card query and fetches the first page.
    fn on_deck_opened(&mut self, app: &mut App, deck: &Deck) {
        let Screen::Swipe(view) = &mut app.screen else {
            return;
        };
        if view.deck.id != deck.deck_profile.id || view.stack.is_some() {
            return;
        }
        view.deck = deck.deck_profile.clone();
        match swipe::query_for(deck) {
            Ok(query) => {
                view.stack = Some(Stack::new(swipe::cards_in(deck)));
                view.query = Some(query);
                self.refill(view);
            }
            Err(e) => view.error = Some(format!("Could not build the card search: {e}")),
        }
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
            Screen::Decks(_) => return self.handle_decks_key(app, key.code),
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

    fn handle_decks_key(&mut self, app: &mut App, code: KeyCode) -> Flow {
        let Screen::Decks(list) = &mut app.screen else {
            return Flow::Continue;
        };
        match code {
            KeyCode::Esc | KeyCode::Char('q') => return Flow::Quit,
            KeyCode::Down | KeyCode::Char('j') => list.next(),
            KeyCode::Up | KeyCode::Char('k') => list.previous(),
            KeyCode::Char('r') => self.open_decks(app),
            KeyCode::Char('O') => {
                "Signing out...".clone_into(&mut app.status);
                self.pending.push(Request::SignOut);
            }
            KeyCode::Enter => {
                if let Some(deck) = list.selected_deck().cloned() {
                    self.pending.push(Request::OpenDeck(deck.id));
                    app.status.clear();
                    app.screen = Screen::Swipe(Box::new(SwipeView::opening(deck)));
                }
            }
            _ => {}
        }
        Flow::Continue
    }

    fn handle_swipe_key(&mut self, app: &mut App, code: KeyCode) {
        let Screen::Swipe(view) = &mut app.screen else {
            return;
        };
        if matches!(code, KeyCode::Esc | KeyCode::Char('q')) {
            app.status.clear();
            self.open_decks(app);
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
            tokio::spawn(async move {
                if let Some(update) = perform(&account, request).await {
                    tx.send(update).ok();
                }
            });
        }
    }
}

/// Runs one request.
async fn perform(account: &Account, request: Request) -> Option<Update> {
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
        Request::LoadDecks => Some(
            match async { client.fetch_deck_profiles(&account.fresh().await?).await }.await {
                Ok(decks) => Update::Decks(decks),
                Err(e) => failure(&e, Update::DecksFailed),
            },
        ),
        Request::OpenDeck(deck_id) => Some(
            match async { client.fetch_deck(deck_id, &account.fresh().await?).await }.await {
                Ok(deck) => Update::DeckOpened(Box::new(deck)),
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
