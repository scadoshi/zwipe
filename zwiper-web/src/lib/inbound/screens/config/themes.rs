//! The Themes sheet, a side panel on a wide window so the preview repaints
//! the page beside it, and the dark mode toggle. The main menu's theme pair
//! draws both, so the menu and the Theme row save through one path. Signed
//! out, a pick is kept on this device (the root stores every theme change)
//! and a profile's theme takes over at login.

use components::{SheetFrame, SheetWide, TOAST_NORMAL, TOAST_QUICK, ThemeConfig, ThemeSheet};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::user::HttpUpdateSettings;

use crate::domain::settings::{self, RequestSeq};
use crate::inbound::chrome::authed::use_authed;

/// Flips Dark/Light at once and saves it, saying what it did; a failure
/// puts back the mode from before the click, and a reply overtaken by a
/// newer click is ignored.
pub(crate) fn use_toggle_dark_mode() -> Callback<()> {
    let mut theme: Signal<ThemeConfig> = use_context();
    let session: Signal<Option<Session>> = use_context();
    let authed = use_authed();
    let toast = use_toast();
    let mut dark_seq = use_signal(RequestSeq::default);
    use_callback(move |()| {
        let prev = theme.peek().clone();
        let next = ThemeConfig {
            name: prev.name.clone(),
            is_dark: !prev.is_dark,
        };
        theme.set(next.clone());
        if !signed_in(session) {
            toast.success(
                dark_mode_note(next.is_dark).to_string(),
                ToastOptions::default().duration(TOAST_NORMAL),
            );
            return;
        }
        let mine = dark_seq.write().begin();
        let request = HttpUpdateSettings {
            theme: Some(next.name),
            dark_mode: Some(next.is_dark),
            exclude_universes_beyond: None,
            universes_beyond_exceptions: None,
            pip_style: None,
        };
        spawn(async move {
            let reply = authed
                .try_run("update_preferences", |c, s| async move {
                    c.update_settings(request, &s).await
                })
                .await
                .ok();
            let saved_dark = reply.as_ref().map(|r| r.dark_mode);
            let saved = reply.map(|r| components::theme_from_settings(&r));
            let Some(shown) = settings::settle(*dark_seq.peek(), mine, saved, prev) else {
                return;
            };
            theme.set(shown);
            if let Some(dark) = saved_dark {
                toast.success(
                    dark_mode_note(dark).to_string(),
                    ToastOptions::default().duration(TOAST_NORMAL),
                );
            }
        });
    })
}

/// The kit's theme sheet, saving a kept pick to the account. A click
/// previews a row, the arrow keys move the preview (see the root's keys),
/// Enter is `Save` and Esc is `Back`. A failed save toasts and leaves the
/// preview applied, as the app does; a discarded pick wipes back out and
/// then says `Theme unchanged`.
#[component]
pub(crate) fn ThemesSheet(open: Signal<bool>) -> Element {
    let mut theme: Signal<ThemeConfig> = use_context();
    let session: Signal<Option<Session>> = use_context();
    let toast = use_toast();
    let authed = use_authed();

    let save = move |picked: ThemeConfig| {
        if !signed_in(session) {
            theme.set(picked);
            toast.success(
                "Theme saved".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
            return;
        }
        let request = HttpUpdateSettings {
            theme: Some(picked.name),
            dark_mode: Some(picked.is_dark),
            exclude_universes_beyond: None,
            universes_beyond_exceptions: None,
            pip_style: None,
        };
        spawn(async move {
            if let Some(settings) = authed
                .run("update_preferences", |c, s| async move {
                    c.update_settings(request, &s).await
                })
                .await
            {
                theme.set(components::theme_from_settings(&settings));
                toast.success(
                    "Theme saved".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    };

    rsx! {
        div { class: "web-theme-sheet",
            SheetFrame { wide: SheetWide::Drawer,
                ThemeSheet { open, theme, on_save: save }
            }
        }
    }
}

/// Whether a live session is on, so a theme pick saves to the profile.
fn signed_in(session: Signal<Option<Session>>) -> bool {
    session.peek().as_ref().is_some_and(|s| !s.is_expired())
}

/// What the toast says after the mode flips.
fn dark_mode_note(dark: bool) -> &'static str {
    if dark {
        "Dark mode on"
    } else {
        "Dark mode off"
    }
}
