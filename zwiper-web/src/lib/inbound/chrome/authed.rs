//! The authed-call facade: the one way a screen makes an authed request.
//!
//! It refreshes the session, runs the call, and on a failure logs it and
//! raises the app's error toast, so every screen handles a failed call the
//! same way. The app also reports each failure to its telemetry; the web
//! build has none, so the log line is the whole report.

use std::fmt;

use components::TOAST_NORMAL;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, Toasts, use_toast};
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;

use crate::inbound::session::ensure_fresh::ensure_fresh;

/// The contexts a call needs, in one `Copy` handle that closures and spawned
/// tasks capture freely.
#[derive(Clone, Copy)]
pub struct Authed {
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    toast: Toasts,
}

impl fmt::Debug for Authed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Authed")
            .field("session", &self.session)
            .field("client", &self.client)
            .finish_non_exhaustive()
    }
}

/// Reads the root contexts once. Call it beside a screen's other hooks.
pub fn use_authed() -> Authed {
    Authed {
        session: use_context(),
        client: use_context(),
        toast: use_toast(),
    }
}

/// Whether a failure raises the error toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Report {
    /// Log and toast `to_user_message()` for 5 s.
    Toast,
    /// Log only, for work the user never waits on.
    Quiet,
}

impl Authed {
    /// Refreshes the session, runs `f`, and on any failure logs it and
    /// toasts the error, yielding `None`. `op` names the call in the log.
    pub async fn run<T, Fut>(
        &self,
        op: &'static str,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Option<T>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        self.execute(op, Report::Toast, f).await.ok()
    }

    /// [`run`](Self::run) that hands the error back as well, for an
    /// optimistic change that reverts on failure.
    ///
    /// # Errors
    ///
    /// The refresh's or the call's [`ClientError`], already logged and
    /// toasted.
    pub async fn try_run<T, Fut>(
        &self,
        op: &'static str,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Result<T, ClientError>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        self.execute(op, Report::Toast, f).await
    }

    /// [`run`](Self::run) with no toast, for loads the user never asked for
    /// and never waits on.
    pub async fn run_quiet<T, Fut>(
        &self,
        op: &'static str,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Option<T>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        self.execute(op, Report::Quiet, f).await.ok()
    }

    async fn execute<T, Fut>(
        &self,
        op: &'static str,
        report: Report,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Result<T, ClientError>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        // A rejected refresh has already cleared the session, and the auth
        // gate sends the user to Login; the caller still hears of it.
        let session = match ensure_fresh(self.session, self.client).await {
            Ok(session) => session,
            Err(e) => {
                self.report(op, &e, report);
                return Err(e);
            }
        };
        let http = self.client.peek().clone();
        f(http, session)
            .await
            .inspect_err(|e| self.report(op, e, report))
    }

    fn report(&self, op: &'static str, e: &ClientError, report: Report) {
        tracing::warn!("{op} failed: {e}");
        if report == Report::Toast {
            self.toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(TOAST_NORMAL),
            );
        }
    }
}
