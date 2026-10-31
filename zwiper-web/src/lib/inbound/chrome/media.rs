//! Whether the window is at least a width, for the few places where a wide
//! window changes what a screen does and not only how it looks, such as the
//! filter docked open beside Add.

use dioxus::prelude::*;
use uuid::Uuid;

/// The window's width at which Add and Remove dock the filter as a third
/// column, `web-layout.md`'s three-panel width.
pub const THREE_COLUMNS_PX: u32 = 1280;

/// True while the window is at least `px` wide, kept current as it resizes.
/// Off the browser it stays false.
pub fn use_min_width(px: u32) -> Signal<bool> {
    let mut wide = use_signal(|| false);
    let id = use_hook(Uuid::new_v4);
    use_hook(move || {
        let script = MIN_WIDTH_JS
            .replace("__PX__", &px.to_string())
            .replace("__ID__", &id.to_string());
        spawn(async move {
            let mut changes = document::eval(&script);
            while let Ok(matches) = changes.recv::<bool>().await {
                wide.set(matches);
            }
        });
    });
    use_drop(move || {
        document::eval(&MIN_WIDTH_DROP_JS.replace("__ID__", &id.to_string()));
    });
    wide
}

/// Sends whether the window matches, then again on each change.
const MIN_WIDTH_JS: &str = r"
const query = window.matchMedia('(min-width: __PX__px)');
const onChange = (e) => dioxus.send(e.matches);
window['__zwipeWidth__ID__'] = [query, onChange];
dioxus.send(query.matches);
query.addEventListener('change', onChange);
";

/// Stops one listener.
const MIN_WIDTH_DROP_JS: &str = r"
const held = window['__zwipeWidth__ID__'];
if (held) held[0].removeEventListener('change', held[1]);
delete window['__zwipeWidth__ID__'];
";
