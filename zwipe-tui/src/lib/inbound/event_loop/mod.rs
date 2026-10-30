//! The main loop: keys change the [`App`], background requests feed it, and every pass draws it.
//!
//! Handlers never touch the network or the back stack. They queue a [`Request`] or a [`Navigation`] on their [`Context`]; [`EventLoop::settle`] applies the navigation, and [`EventLoop::dispatch`] spawns each request as a task that answers with an [`Update`].

#[cfg(test)]
pub(crate) mod harness;
#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::mem;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use zwipe_core::domain::auth::models::session::Session;

use super::app::{App, Context, Effects, Navigation};
use super::screens::login::{self, LoginForm};
use super::screens::{self, Request, Screen, Update};
use super::widgets::hint;
use crate::domain::decode;
use crate::outbound::account::Account;
use crate::outbound::theme_file::ThemeFile;

/// Whether the loop keeps going after a key.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Flow {
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
            app.tick(Instant::now());
            terminal.draw(|f| app.draw(f))?;

            let animating = app.animating();
            tokio::select! {
                Some(update) = self.rx.recv() => {
                    if self.apply(app, update) == Flow::Quit {
                        return Ok(());
                    }
                }
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

    /// Checks the version, and opens home with a saved session or sign-in without one.
    fn start(&mut self, app: &mut App, session: Option<&Session>) {
        self.pending.push(Request::CheckVersion);
        let mut cx = Context::new(&mut app.shared);
        match session {
            Some(session) => login::signed_in(&mut cx, session),
            None => cx.reset(Screen::Login(LoginForm::default())),
        }
        let effects = cx.into_effects();
        self.settle(app, effects);
    }

    /// Applies a background result to the app and to the screen it is for.
    fn apply(&mut self, app: &mut App, update: Update) -> Flow {
        let mut cx = Context::new(&mut app.shared);
        screens::apply(&mut app.nav, update, &mut cx);
        let effects = cx.into_effects();
        self.settle(app, effects)
    }

    /// Hands a key to the open hint, else to the screen on top. Ctrl+C quits from anywhere.
    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Flow {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Flow::Quit;
        }
        if app.shared.hint.is_some() {
            if hint::dismisses(key.code) {
                app.shared.hint = None;
            }
            return Flow::Continue;
        }
        let mut cx = Context::new(&mut app.shared);
        app.nav.top_mut().handle_key(key.code, &mut cx);
        let effects = cx.into_effects();
        self.settle(app, effects)
    }

    /// Queues the requests and applies the navigation a handler asked for. A screen uncovered by going back may refresh itself, which can queue more of both.
    fn settle(&mut self, app: &mut App, effects: Effects) -> Flow {
        self.pending.extend(effects.requests);
        let mut quit = effects.quit;
        let mut steps = VecDeque::from(effects.navigation);
        while let Some(step) = steps.pop_front() {
            match step {
                Navigation::Push(screen) => app.nav.push(screen),
                Navigation::Replace(screen) => app.nav.replace(screen),
                Navigation::Reset(screen) => app.nav.reset(screen),
                Navigation::Back => {
                    if app.nav.pop().is_none() {
                        quit = true;
                        continue;
                    }
                    let mut cx = Context::new(&mut app.shared);
                    app.nav.top_mut().on_return(&mut cx);
                    let more = cx.into_effects();
                    self.pending.extend(more.requests);
                    steps.extend(more.navigation);
                    quit |= more.quit;
                }
            }
        }
        if quit { Flow::Quit } else { Flow::Continue }
    }

    fn dispatch(&mut self) {
        for request in mem::take(&mut self.pending) {
            let tx = self.tx.clone();
            let account = Arc::clone(&self.account);
            let theme_file = Arc::clone(&self.theme_file);
            tokio::spawn(async move {
                if let Some(update) = screens::perform(&account, &theme_file, request).await {
                    tx.send(update).ok();
                }
            });
        }
    }
}
