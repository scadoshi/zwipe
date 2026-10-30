use components::{Changelog as ChangelogContent, Panel};
use dioxus::prelude::*;

use crate::components::{PageMeta, sleep_ms};
use crate::{Footer, Nav};

/// One frame for the selected month's cards to mount before the scroll.
const MOUNT_WAIT_MS: u32 = 50;

/// Reads the URL fragment on the client, as text without its `#`.
const READ_HASH_JS: &str = "dioxus.send(decodeURIComponent(window.location.hash.slice(1)))";

#[component]
pub fn Changelog() -> Element {
    // The release the URL fragment names (`/changelog#v1-12-1`). Read after
    // hydration, since the prerendered page cannot know it; the changelog then
    // selects that release's chips and the card is scrolled into view.
    let mut anchor = use_signal(|| None::<String>);
    use_effect(move || {
        spawn(async move {
            let mut read = document::eval(READ_HASH_JS);
            let Ok(fragment) = read.recv::<String>().await else {
                return;
            };
            if fragment.is_empty() {
                return;
            }
            anchor.set(Some(fragment.clone()));
            sleep_ms(MOUNT_WAIT_MS).await;
            let _ = document::eval(&format!(
                "document.getElementById({fragment:?})?.scrollIntoView({{ block: 'start' }});"
            ))
            .await;
        });
    });

    rsx! {
        PageMeta {
            title: "Changelog",
            description: "Every Zwipe release, newest first: what changed in each version of the app.",
            path: "/changelog",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "page-header",
                Panel { title: "Changelog", title_h1: true,
                    p { class: "tagline", "Every release, newest first." }
                }
            }
            ChangelogContent { anchor: anchor(), link_versions: true }
        }
        Footer {}
    }
}
