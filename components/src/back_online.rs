//! A hook that tells a screen when the device's connection comes back.

use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

/// Gives each listener its own slot on `window`, so one screen's drop never
/// stops another's.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Calls `on_online` each time the browser, or the app's web view, fires
/// `online`: the connection is back. A screen whose load failed passes its
/// retry, so it recovers without a tap. The listener goes with the
/// component.
pub fn use_back_online(on_online: Callback<()>) {
    let id = use_hook(|| NEXT_ID.fetch_add(1, Ordering::Relaxed));
    use_hook(move || {
        spawn(async move {
            let mut events = document::eval(&ONLINE_JS.replace("__ID__", &id.to_string()));
            while let Ok(true) = events.recv::<bool>().await {
                on_online.call(());
            }
        });
    });
    use_drop(move || {
        document::eval(&ONLINE_DROP_JS.replace("__ID__", &id.to_string()));
    });
}

/// Sends `true` on each `online` event.
const ONLINE_JS: &str = r"
const onOnline = () => dioxus.send(true);
window['__zwipeOnline__ID__'] = onOnline;
window.addEventListener('online', onOnline);
";

/// Stops one listener.
const ONLINE_DROP_JS: &str = r"
const held = window['__zwipeOnline__ID__'];
if (held) window.removeEventListener('online', held);
delete window['__zwipeOnline__ID__'];
";
