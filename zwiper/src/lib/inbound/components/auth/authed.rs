//! Authed-call facade: the one place every screen-initiated authed request
//! refreshes the session, reports failures to telemetry, and toasts.
//!
//! Replaces the per-call-site ceremony (`ensure_fresh` + hand-rolled
//! `report_error` + toast) that every screen used to copy, and that some
//! screens inevitably got wrong: the "some screens toast, some swallow"
//! inconsistency this kills at the root
//! (context/plans/authed_error_handler.md).
//!
//! Not for everything: `session_upkeep`, `signal_logout`, the catalog cache,
//! and the hint recorder have shapes that don't fit a screen-scoped facade
//! and keep calling `ensure_fresh` directly. The facade is for screens.

use std::time::Duration;

use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, Toasts, use_toast};
use zwipe_core::domain::auth::models::session::Session;

use crate::{
    inbound::components::{
        auth::ensure_session::EnsureFresh,
        telemetry::{
            usage_buffer::UsageBuffer,
            vocabulary::{Screen, component},
        },
    },
    outbound::client::{ZwipeClient, error::ClientError},
};

/// Everything the ceremony needs, in one Copy handle so closures and spawns
/// capture it freely.
#[derive(Clone, Copy)]
pub struct Authed {
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    usage_buffer: Signal<UsageBuffer>,
    toast: Toasts,
    screen: Screen,
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
    }
}

impl Authed {
    /// The uniform path: refresh, run, and on any failure report + toast,
    /// yielding `None`. `op` is the telemetry action name and MUST match the
    /// string the call site used before conversion, so the server-side
    /// `client_errors` dedupe keys stay continuous.
    pub async fn run<T, Fut>(
        &self,
        op: &'static str,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Option<T>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        self.execute(component::NONE, op, false, f).await.ok()
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
        self.execute(component, op, false, f).await.ok()
    }

    /// Same reporting as [`Self::run`], but hands the `Err` back instead of
    /// swallowing it, for optimistic-update sites that must revert their
    /// state on failure.
    pub async fn try_run<T, Fut>(
        &self,
        op: &'static str,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Result<T, ClientError>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        self.execute(component::NONE, op, false, f).await
    }

    /// Telemetry and log only, no toast: for work the user never initiated
    /// and never waits on (prefetch, fire-and-forget flushes). When in doubt
    /// use [`Self::run`] instead; a user who hits a dead end deserves a
    /// warning (owner call, 2026-09-02).
    pub async fn run_quiet<T, Fut>(
        &self,
        op: &'static str,
        f: impl FnOnce(ZwipeClient, Session) -> Fut,
    ) -> Option<T>
    where
        Fut: Future<Output = Result<T, ClientError>>,
    {
        self.execute(component::NONE, op, true, f).await.ok()
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
        self.execute(component, op, true, f).await.ok()
    }

    async fn execute<T, Fut>(
        &self,
        component: &'static str,
        op: &'static str,
        quiet: bool,
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
                self.report(component, op, &e, quiet);
                return Err(e);
            }
        };
        match f((self.client)(), session).await {
            Ok(value) => Ok(value),
            Err(e) => {
                self.report(component, op, &e, quiet);
                Err(e)
            }
        }
    }

    fn report(&self, component: &'static str, op: &'static str, e: &ClientError, quiet: bool) {
        tracing::warn!("{op} failed: {e}");
        self.usage_buffer
            .peek()
            .report_error(self.screen.as_str(), component, op, e);
        if !quiet {
            self.toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
        }
    }
}
