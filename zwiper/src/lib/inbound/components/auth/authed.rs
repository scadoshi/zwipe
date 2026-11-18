//! Authed-call facade: the one place every screen-initiated authed request
//! refreshes the session, reports failures to telemetry, and toasts.
//!
//! Screens go through this instead of calling `ensure_fresh`, `report_error`,
//! and the toast themselves, so every screen handles a failed authed call the
//! same way.
//!
//! Not for everything: `session_upkeep`, `signal_logout`, the catalog cache,
//! and the hint recorder have shapes that don't fit a screen-scoped facade
//! and keep calling `ensure_fresh` directly. The facade is for screens.

use std::sync::Mutex;

use chrono::Utc;
use components::{TOAST_NORMAL, ToastOptions, Toasts, use_toast};
use dioxus::prelude::*;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_ui::toast::RepeatGate;
use zwipe_ui::write_repeat::{Answer, Unconfirmed, Verdict, WriteKey, Written};

use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::components::telemetry::vocabulary::{Screen, component};

/// Whether a failed call raises its error toast. Every failure is logged and
/// reported to telemetry either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Toasting {
    /// The user sees the failure.
    Toast,
    /// Telemetry and log only, for work the user never waits on.
    Quiet,
}

/// Everything the ceremony needs, in one Copy handle so closures and spawns
/// capture it freely.
#[derive(Clone, Copy)]
pub struct Authed {
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    usage_buffer: Signal<UsageBuffer>,
    toast: Toasts,
    screen: Screen,
    /// The session's writes whose answer was lost, kept by the app; `None` outside it.
    unconfirmed: Option<Signal<Unconfirmed>>,
}

// By hand: `Toasts` has no `Debug`.
impl std::fmt::Debug for Authed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Authed")
            .field("session", &self.session)
            .field("client", &self.client)
            .field("usage_buffer", &self.usage_buffer)
            .field("screen", &self.screen)
            .finish_non_exhaustive()
    }
}

/// Reads the app contexts once and pins the telemetry screen for every call
/// made through the handle. One call per screen component, next to the other
/// hooks.
pub fn use_authed(screen: Screen) -> Authed {
    Authed {
        session: use_context(),
        client: use_context(),
        usage_buffer: use_context(),
        toast: use_toast(),
        screen,
        unconfirmed: try_use_context(),
    }
}

impl Authed {
    /// This handle with its calls reported under `screen`, for a sheet above the screens
    /// that learns its host when it opens.
    #[must_use]
    pub fn at(mut self, screen: Screen) -> Self {
        self.screen = screen;
        self
    }
}

impl Authed {
    /// The uniform path: refresh, run, and on any failure report + toast,
    /// yielding `None`. `op` is the telemetry action name, part of the
    /// server-side `client_errors` dedupe key, so it stays fixed per site.
    pub async fn run<T, Fut>(
        &self,
        op: &'static str,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Option<T>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        self.execute(component::NONE, op, Toasting::Toast, f)
            .await
            .ok()
    }

    /// [`Self::run`] with a component breadcrumb, for dialogs and sheets that
    /// report their host screen plus their own component name.
    pub async fn run_at<T, Fut>(
        &self,
        component: &'static str,
        op: &'static str,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Option<T>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        self.execute(component, op, Toasting::Toast, f).await.ok()
    }

    /// Same reporting as [`Self::run`], but hands the `Err` back instead of
    /// swallowing it, for optimistic-update sites that must revert their
    /// state on failure.
    ///
    /// # Errors
    ///
    /// Returns the refresh's or the call's [`ClientError`].
    pub async fn try_run<T, Fut>(
        &self,
        op: &'static str,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Result<T, ClientError>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        self.execute(component::NONE, op, Toasting::Toast, f).await
    }

    /// Telemetry and log only, no toast: for work the user never initiated
    /// and never waits on (prefetch, fire-and-forget flushes). When in doubt
    /// use [`Self::run`] instead; a user who hits a dead end deserves a
    /// warning.
    pub async fn run_quiet<T, Fut>(
        &self,
        op: &'static str,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Option<T>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        self.execute(component::NONE, op, Toasting::Quiet, f)
            .await
            .ok()
    }

    /// [`Self::run_quiet`] handing the `Err` back, for sites that act on
    /// the kind of failure without toasting it.
    ///
    /// # Errors
    ///
    /// Returns the refresh's or the call's [`ClientError`].
    pub async fn try_run_quiet<T, Fut>(
        &self,
        op: &'static str,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Result<T, ClientError>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        self.execute(component::NONE, op, Toasting::Quiet, f).await
    }

    /// [`Self::try_run_quiet`] with a component breadcrumb.
    ///
    /// # Errors
    ///
    /// Returns the refresh's or the call's [`ClientError`].
    pub async fn try_run_quiet_at<T, Fut>(
        &self,
        component: &'static str,
        op: &'static str,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Result<T, ClientError>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        self.execute(component, op, Toasting::Quiet, f).await
    }

    /// [`Self::run_quiet`] with a component breadcrumb.
    pub async fn run_quiet_at<T, Fut>(
        &self,
        component: &'static str,
        op: &'static str,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Option<T>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        self.execute(component, op, Toasting::Quiet, f).await.ok()
    }

    async fn execute<T, Fut>(
        &self,
        component: &'static str,
        op: &'static str,
        toasting: Toasting,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Result<T, ClientError>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        // Refresh failure: auth rejections have already cleared the session
        // (the AuthGate redirects to login); transient errors left it alone
        // so the next attempt retries. Either way the site is told.
        let session = match self.session.ensure_fresh(self.client).await {
            Ok(session) => session,
            Err(e) => {
                self.report(component, op, &e, toasting);
                return Err(e);
            }
        };
        match f((self.client)(), session).await {
            Ok(value) => Ok(value),
            Err(e) => {
                self.report(component, op, &e, toasting);
                Err(e)
            }
        }
    }

    /// Runs a write `f` that acts on `key`, as [`run`](Self::run) does, except that the answer a
    /// repeat gives once an earlier, lost try landed (an add's 409, a card removal's 422, a deck deletion's 404) reads as
    /// [`Written::LandedEarlier`] with no error toast. A network error is remembered for the
    /// session, so the next try of the same write can tell.
    pub async fn run_write<T, Fut>(
        &self,
        op: &'static str,
        key: WriteKey,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Written<T>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        self.run_write_at(component::NONE, op, key, f).await
    }

    /// [`Self::run_write`] with a component breadcrumb, as [`Self::run_at`] is to [`Self::run`].
    pub async fn run_write_at<T, Fut>(
        &self,
        component: &'static str,
        op: &'static str,
        key: WriteKey,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Written<T>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        let result = self.execute(component, op, Toasting::Quiet, f).await;
        match (self.settle(key, &result), result) {
            (Verdict::LandedEarlier, _) => Written::LandedEarlier,
            (Verdict::Done | Verdict::Failed, Ok(value)) => Written::Done(value),
            (Verdict::Done | Verdict::Failed, Err(e)) => {
                self.toast_error(&e);
                Written::Failed
            }
        }
    }

    /// How a write on `key` that ended in `result` reads once a lost earlier try is counted, for
    /// a caller that makes its own failure message; [`run_write`](Self::run_write) uses it.
    pub fn settle<T>(&self, key: WriteKey, result: &Result<T, ClientError>) -> Verdict {
        let answer = answer_of(result);
        match self.unconfirmed {
            Some(mut unconfirmed) => unconfirmed.write().settle(key, answer),
            None if result.is_ok() => Verdict::Done,
            None => Verdict::Failed,
        }
    }

    fn report(
        &self,
        component: &'static str,
        op: &'static str,
        e: &ClientError,
        toasting: Toasting,
    ) {
        tracing::warn!("{op} failed: {e}");
        self.usage_buffer
            .peek()
            .report_error(self.screen.as_str(), component, op, e);
        if toasting == Toasting::Toast {
            self.toast_error(e);
        }
    }

    /// Raises the error toast for `e`, unless one saying the same is up.
    fn toast_error(&self, e: &ClientError) {
        let message = e.to_user_message();
        if admit_error_toast(&message) {
            self.toast
                .error(message, ToastOptions::default().duration(TOAST_NORMAL));
        }
    }
}

/// How a write's `result` ended, for [`Unconfirmed::settle`]: a call made outside [`Authed`]
/// settles through it too.
pub(crate) fn answer_of<T>(result: &Result<T, ClientError>) -> Answer {
    match result {
        Ok(_) => Answer::Done,
        Err(ClientError::Network(_)) => Answer::Network,
        Err(e) => e.status().map_or(Answer::Failed, Answer::Status),
    }
}

/// The error toasts up across every screen, so a failure retried shows once.
static ERROR_TOASTS: Mutex<RepeatGate> = Mutex::new(RepeatGate::new());

/// Whether an error toast saying `message` shows: not while one with the same
/// words is still up. A poisoned gate lets the toast through.
fn admit_error_toast(message: &str) -> bool {
    ERROR_TOASTS.lock().map_or(true, |mut gate| {
        gate.admit(message, Utc::now(), TOAST_NORMAL)
    })
}
