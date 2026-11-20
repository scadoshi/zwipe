//! The end of a list that loads more as it is reached: a marker under the list, and
//! [`use_list_end`], which calls the screen when the marker scrolls near the view and when the
//! keyboard's End is pressed on the page, so End reaches the next page rather than stopping at the
//! last card drawn.

use dioxus::prelude::*;
use uuid::Uuid;

use crate::keys::{STACK_DETAILS_CLASS, cursor_guard_js};

/// The marker's element id, under the list it ends.
pub const LIST_END_ID: &str = "list-end";

/// How near the view, in pixels, the marker counts as reached, so the next page is under way
/// before the last row shows.
const REACH_PX: u32 = 600;

/// Watches the marker and the End key: a marker that comes within [`REACH_PX`] of the view, or
/// End under the cursor keys' guard, sends once. A marker drawn again is watched again.
const LIST_END_JS: &str = r"
const watch = () => {
    const end = document.getElementById('__ID__');
    if (!end || end === window['__zwipeListEndAt__KEY__']) return;
    window['__zwipeListEndAt__KEY__'] = end;
    const seen = window['__zwipeListEndSeen__KEY__'];
    if (seen) seen.disconnect();
    const observer = new IntersectionObserver((hits) => {
        if (hits.some((hit) => hit.isIntersecting)) dioxus.send(true);
    }, { rootMargin: '__REACH__px 0px' });
    observer.observe(end);
    window['__zwipeListEndSeen__KEY__'] = observer;
};
watch();
window['__zwipeListEndTick__KEY__'] = setInterval(watch, 500);
const onKey = (e) => {
    if (e.key !== 'End' || !__GUARD__) return;
    dioxus.send(true);
};
window['__zwipeListEndKey__KEY__'] = onKey;
document.addEventListener('keydown', onKey);
";

/// Stops watching.
const LIST_END_DROP_JS: &str = r"
clearInterval(window['__zwipeListEndTick__KEY__']);
const seen = window['__zwipeListEndSeen__KEY__'];
if (seen) seen.disconnect();
document.removeEventListener('keydown', window['__zwipeListEndKey__KEY__']);
delete window['__zwipeListEndSeen__KEY__'];
delete window['__zwipeListEndAt__KEY__'];
delete window['__zwipeListEndKey__KEY__'];
";

/// The marker at the end of a list that [`use_list_end`] watches.
#[component]
pub fn ListEnd() -> Element {
    rsx! {
        div { id: LIST_END_ID, class: "list-end", "aria-hidden": "true" }
    }
}

/// Calls `on_end` each time the [`ListEnd`] marker scrolls near the view, and each time End is
/// pressed while the page's keys reach it (no text field, dialog or sheet but the details drawer
/// has them). The screen decides whether there is more to load.
pub fn use_list_end(on_end: Callback<()>) {
    let key = use_hook(Uuid::new_v4).simple().to_string();
    let watch_key = key.clone();
    use_hook(move || {
        let script = LIST_END_JS
            .replace("__ID__", LIST_END_ID)
            .replace("__KEY__", &watch_key)
            .replace("__REACH__", &REACH_PX.to_string())
            .replace("__GUARD__", &cursor_guard_js(STACK_DETAILS_CLASS));
        spawn(async move {
            let mut ends = document::eval(&script);
            while ends.recv::<bool>().await.is_ok() {
                on_end.call(());
            }
        });
    });
    use_drop(move || {
        document::eval(&LIST_END_DROP_JS.replace("__KEY__", &key));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_marker_is_watched_ahead_of_the_view_and_again_when_redrawn() {
        assert!(LIST_END_JS.contains("rootMargin: '__REACH__px 0px'"));
        assert!(
            LIST_END_JS.contains("end === window['__zwipeListEndAt__KEY__']"),
            "the same marker is watched once"
        );
        assert!(
            LIST_END_JS.contains("setInterval(watch, 500)"),
            "a marker drawn again is found again"
        );
    }

    #[test]
    fn end_reaches_the_next_page_under_the_cursor_keys_guard() {
        assert!(LIST_END_JS.contains("if (e.key !== 'End' || !__GUARD__) return;"));
        for drop in [
            "clearInterval(",
            "seen.disconnect()",
            "removeEventListener('keydown'",
        ] {
            assert!(LIST_END_DROP_JS.contains(drop), "{drop}");
        }
    }
}
