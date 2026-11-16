use components::Changelog as ChangelogView;
use dioxus::prelude::*;
use zwipe_ui::changelog::ChangelogCache;

use crate::components::{PageMeta, sleep_ms};
use crate::{Footer, Nav, api};

/// One frame for the selected month's cards to mount before the scroll.
const MOUNT_WAIT_MS: u32 = 50;

/// How many releases the list shows before its "Show more" button, and how
/// many more each click reveals.
const PAGE_SIZE: usize = 5;

/// Reads the URL fragment on the client, as text without its `#`.
const READ_HASH_JS: &str = "dioxus.send(decodeURIComponent(window.location.hash.slice(1)))";

#[component]
pub fn Changelog() -> Element {
    // The release the URL fragment names (`/about/changelog#v1-12-1`). Read after
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

    // The compiled copy, which the prerender draws in full; once hydrated, the browser asks
    // the server and swaps its copy in only when it differs.
    let mut cache = use_signal(ChangelogCache::default);
    use_future(move || async move {
        if !cfg!(target_arch = "wasm32") {
            return;
        }
        let fetched = api::client().fetch_changelog().await.ok();
        let next = cache.peek().after_fetch(fetched);
        if let Some(next) = next {
            cache.set(next);
        }
    });
    let data = cache.read().shown().clone();

    rsx! {
        PageMeta {
            title: "Changelog",
            description: "Every Zwipe release, newest first: what changed in each version of the app.",
            path: "/about/changelog/",
        }
        Nav {}
        main { id: "content", class: "page content-enter",
            ChangelogView { data, anchor: anchor(), link_versions: true, page_size: PAGE_SIZE }
        }
        Footer {}
    }
}
