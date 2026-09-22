//! Turns off iOS's shake-to-undo prompt.
//!
//! iOS reads a shake as "Undo Typing" and raises a system alert. Zwipe's own
//! undo is a downward swipe on the add stack, so the alert is both an
//! interruption and a second, conflicting meaning for undo in an app built
//! around gestures. Nothing here wants the system behavior: the only text
//! entry is short form fields.

/// Disables the system shake-to-undo prompt for the rest of the process.
///
/// Call from the root component's first render, not `main`:
/// `sharedApplication` is nil until the app object exists, so calling it
/// earlier silently does nothing.
///
/// Fails soft. A missing shake suppression is not worth a panic, so a missing
/// main-thread marker or app object just returns.
#[cfg(target_os = "ios")]
pub fn disable() {
    use objc2::{MainThreadMarker, class, msg_send, rc::Retained, runtime::NSObject};

    // `sharedApplication` must be used on the main thread. A component's
    // render already runs there, so the marker is available.
    let Some(_mtm) = MainThreadMarker::new() else {
        return;
    };

    // app = UIApplication.sharedApplication
    // SAFETY: correct selector and signature; we hold the main-thread marker.
    let app: Option<Retained<NSObject>> =
        unsafe { msg_send![class!(UIApplication), sharedApplication] };
    let Some(app) = app else {
        return;
    };

    // [app setApplicationSupportsShakeToEdit:NO]
    // SAFETY: the setter takes a single BOOL and returns nothing.
    let _: () = unsafe { msg_send![&*app, setApplicationSupportsShakeToEdit: false] };
}

/// No-op off iOS: no other platform turns a shake into an undo prompt.
#[cfg(not(target_os = "ios"))]
pub fn disable() {}
