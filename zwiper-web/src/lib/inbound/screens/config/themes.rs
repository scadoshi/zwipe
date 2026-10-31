//! The Themes sheet, a side panel on a wide window so the preview repaints
//! the page beside it.

use components::{TOAST_QUICK, ThemeConfig, ThemeSheet};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::http::contracts::user::HttpUpdatePreferences;

use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::sheet::{SheetForm, SheetFrame};

/// The kit's theme sheet, saving a kept pick to the account. A click
/// previews a row, the arrow keys move the preview (see the root's keys),
/// Enter is `Save` and Esc is `Back`. A failed save toasts and leaves the
/// preview applied, as the app does; a discarded pick wipes back out and
/// then says `Theme unchanged`.
#[component]
pub(super) fn ThemesSheet(open: Signal<bool>) -> Element {
    let mut theme: Signal<ThemeConfig> = use_context();
    let toast = use_toast();
    let authed = use_authed();

    let save = move |picked: ThemeConfig| {
        let request = HttpUpdatePreferences {
            theme: Some(picked.name),
            dark_mode: Some(picked.is_dark),
            exclude_universes_beyond: None,
            universes_beyond_exceptions: None,
            pip_style: None,
        };
        spawn(async move {
            if let Some(preferences) = authed
                .run("update_preferences", |c, s| async move {
                    c.update_preferences(request, &s).await
                })
                .await
            {
                theme.set(components::theme_from_preferences(&preferences));
                toast.success(
                    "Theme saved".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    };

    rsx! {
        div { class: "web-theme-sheet",
            SheetFrame { form: SheetForm::SidePanel,
                ThemeSheet {
                    open,
                    theme,
                    on_save: save,
                    on_unchanged: move |()| {
                        toast.info(
                            "Theme unchanged".to_string(),
                            ToastOptions::default().duration(TOAST_QUICK),
                        );
                    },
                }
            }
        }
    }
}
