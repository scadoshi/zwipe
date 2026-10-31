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
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::metrics::AnonymousEventKind;

use super::app::{App, Context, Effects, Navigation, session};
use super::screens::{self, Request, Update, login};
use super::widgets::help;
use super::widgets::splash::Splash;
use crate::outbound::account::Account;
use crate::outbound::theme_file::ThemeFile;

/// With this set, a debug build panics on F12, to try the crash report and the terminal restore by hand. Release builds have no such key.
#[cfg(debug_assertions)]
const CRASH_VAR: &str = "ZWIPER_TUI_DEBUG_CRASH";

/// Panics on F12 when [`CRASH_VAR`] is set, in debug builds only.
#[cfg(debug_assertions)]
fn crash_on_request(code: KeyCode) {
    let requested = code == KeyCode::F(12) && std::env::var_os(CRASH_VAR).is_some();
    assert!(!requested, "debug crash requested with F12");
}

/// How often the loop redraws while something moves: the rolling counters' 16 ms tick, which the slower animations land on too.
const FRAME: Duration = Duration::from_millis(16);

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
        let splash = Splash::new(Instant::now());
        app.shared.gate = splash.gate();
        app.shared.splash = Some(splash);
        self.start(app, session.as_ref());
        let mut events = EventStream::new();
        // The first tick fires at once: the launch's refresh and version check.
        let mut upkeep = session::ticker(session::UPKEEP_INTERVAL);
        let mut flush = session::ticker(session::FLUSH_INTERVAL);

        loop {
            self.dispatch();
            app.tick(Instant::now());
            terminal.draw(|f| app.draw(f))?;

            let animating = app.animating() || app.entering(Instant::now());
            tokio::select! {
                Some(update) = self.rx.recv() => {
                    if self.apply(app, update) == Flow::Quit {
                        return Ok(());
                    }
                }
                () = tokio::time::sleep(FRAME), if animating => {}
                _ = upkeep.tick() => self.pending.push(Request::Upkeep),
                _ = flush.tick() => self.pending.push(Request::FlushUsage),
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

    /// Fetches the changelog, sends a crash report a previous run left, and opens home with a saved session or sign-in without one, telling the server anonymously that the app opened.
    fn start(&mut self, app: &mut App, session: Option<&Session>) {
        self.pending.push(Request::FetchChangelog);
        self.pending.push(Request::SendCrashReport);
        let mut cx = Context::new(&mut app.shared);
        if let Some(session) = session {
            login::signed_in(&mut cx, session);
        } else {
            cx.request(Request::Record(AnonymousEventKind::AppOpened));
            let login = login::screen(&mut cx);
            cx.reset(login);
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

    /// Hands a key to the open dialog, else to the screen on top. Ctrl+C quits from anywhere, the update notice takes every other key but quitting, Ctrl+T expands or collapses the toasts, and F1 opens Help & feedback.
    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> Flow {
        let control = key.modifiers.contains(KeyModifiers::CONTROL);
        if control && key.code == KeyCode::Char('c') {
            return Flow::Quit;
        }
        #[cfg(debug_assertions)]
        crash_on_request(key.code);
        if app.shared.update_required {
            return if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                Flow::Quit
            } else {
                Flow::Continue
            };
        }
        if app.shared.splash.is_some() {
            app.shared.skip_splash(Instant::now());
            return Flow::Continue;
        }
        if control && key.code == KeyCode::Char('t') {
            app.shared.toasts.toggle_expanded();
            return Flow::Continue;
        }
        let mut cx = Context::new(&mut app.shared);
        if cx.shared.help.is_some() {
            help::handle_key(&mut cx, key.code);
            let effects = cx.into_effects();
            return self.settle(app, effects);
        }
        if key.code == KeyCode::F(1) {
            help::open(&mut cx);
            return Flow::Continue;
        }
        if let Some(open) = &mut cx.shared.hint {
            if open.handle_key(key.code) {
                cx.shared.hint = None;
            }
            return Flow::Continue;
        }
        app.nav.top_mut().handle_key(key.code, &mut cx);
        let effects = cx.into_effects();
        self.settle(app, effects)
    }

    /// Queues the requests and applies the navigation a handler asked for. A screen uncovered by going back may refresh itself, which can queue more of both.
    fn settle(&mut self, app: &mut App, effects: Effects) -> Flow {
        self.pending.extend(effects.requests);
        if !effects.navigation.is_empty() {
            app.screen_changed(Instant::now());
        }
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
