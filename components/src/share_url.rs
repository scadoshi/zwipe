//! A shared deck's public link, shown in the Share dialog, and that dialog's actions once the
//! deck is shared.

use dioxus::prelude::*;
use zwipe_ui::screens::deck_view::{CLOSE, COPY_LINK, STOP_SHARE};

use crate::DialogAction;

/// `url` in an accent chip centered under the dialog's text, hugging the
/// link's width. It stays on one line while the dialog fits it and otherwise
/// breaks at any character. A tap selects all of it, and so does Tab, which
/// stops on it, so it can be copied by hand (Ctrl+C) as well as with Copy
/// link.
#[component]
pub fn ShareUrl(url: String) -> Element {
    rsx! {
        div { class: "share-url",
            span {
                class: "stat-chip share-url-chip",
                tabindex: "0",
                "aria-label": "Share link",
                onfocus: move |_| {
                    document::eval(SELECT_LINK_JS);
                },
                "{url}"
            }
        }
    }
}

/// Selects the whole link in the focused chip.
const SELECT_LINK_JS: &str = "(() => { const chip = document.activeElement; if (!chip || !chip.matches('.share-url-chip')) return; const range = document.createRange(); range.selectNodeContents(chip); const sel = window.getSelection(); sel.removeAllRanges(); sel.addRange(range); })();";

/// The Share dialog's actions while the deck is shared, in order: Stop sharing (`on_stop`) first,
/// set apart from the rest by the kit's dialog sheet, then Close and Copy link (`on_copy`), where
/// focus starts. Tab from Copy link never lands on the action that can't be undone.
#[must_use]
pub fn shared_link_actions(on_copy: Callback<()>, on_stop: Callback<()>) -> Vec<DialogAction> {
    vec![
        DialogAction::Confirm {
            label: STOP_SHARE.into(),
            danger: true,
            on_confirm: on_stop,
        },
        DialogAction::Dismiss(CLOSE.into()),
        DialogAction::Confirm {
            label: COPY_LINK.into(),
            danger: false,
            on_confirm: on_copy,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The actions' labels and whether each is a danger, built inside a runtime as a callback
    /// needs.
    fn labels() -> Vec<(String, bool)> {
        let dom = VirtualDom::new(|| rsx! {});
        dom.in_runtime(|| {
            dom.in_scope(ScopeId::ROOT, || {
                shared_link_actions(Callback::new(|()| {}), Callback::new(|()| {}))
                    .into_iter()
                    .map(|action| match action {
                        DialogAction::Dismiss(label) => (label, false),
                        DialogAction::Confirm { label, danger, .. } => (label, danger),
                        _ => (String::new(), false),
                    })
                    .collect()
            })
        })
    }

    #[test]
    fn the_link_is_a_tab_stop_that_selects_itself() {
        let html =
            dioxus_ssr::render_element(rsx! { ShareUrl { url: "https://x.test/d".to_owned() } });
        assert!(html.contains("tabindex=\"0\""), "{html}");
        assert!(html.contains("aria-label=\"Share link\""), "{html}");
        assert!(
            SELECT_LINK_JS.contains("range.selectNodeContents(chip)"),
            "{SELECT_LINK_JS}"
        );
    }

    #[test]
    fn stop_sharing_leads_the_row_so_tab_from_copy_link_never_lands_on_it() {
        assert_eq!(
            labels(),
            vec![
                (STOP_SHARE.to_owned(), true),
                (CLOSE.to_owned(), false),
                (COPY_LINK.to_owned(), false),
            ]
        );
    }
}
