//! The app's lifecycle: orangui's [`components::AppLifecycle`], fed by the
//! native shell's window events as well as the page's own visibility.
//!
//! tao reports the return to the foreground as `Resumed` (the scene is about
//! to show) and `Focused(true)` (it is active), and the trip away as
//! `Suspended`. dioxus drops them in its own loop, so they reach the app only
//! through `use_wry_event_handler`.

#[cfg(feature = "desktop")]
use dioxus::desktop as shell;
#[cfg(all(feature = "mobile", not(feature = "desktop")))]
use dioxus::mobile as shell;

/// Keeps the app's lifecycle and shares it with every screen. Call it once,
/// at the root, before anything that reacts to a return.
pub fn use_lifecycle_root() {
    let life = components::use_app_lifecycle_root();
    #[cfg(any(feature = "desktop", feature = "mobile"))]
    {
        use shell::tao::event::Event;
        use shell::{WindowEvent, use_wry_event_handler};
        use_wry_event_handler(move |event, _| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::Resumed | WindowEvent::Focused(true) => life.resumed(),
                    WindowEvent::Suspended => life.suspended(),
                    _ => {}
                }
            }
        });
    }
    #[cfg(not(any(feature = "desktop", feature = "mobile")))]
    let _ = life;
}
