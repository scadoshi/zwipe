//! Edge back-swipe / hardware-back navigation.
//!
//! zwiper on mobile is native Rust driving a WebView; the router runs on an
//! in-memory history stack, so the OS back intent (iOS left-edge swipe, Android
//! hardware/gesture back) does not reach it on its own. This layout wraps every
//! route and routes that intent into the router's `go_back()`.
//!
//! **iOS.** We attach our own `UIScreenEdgePanGestureRecognizer` to the
//! WKWebView and drive `go_back()` directly. The obvious alternative,
//! WKWebView's built-in `allowsBackForwardNavigationGestures` fed by a
//! `history.pushState` trap, *works* but is slow: the swipe triggers WKWebView's
//! native page transition toward the (empty, same-document) trap entry, which
//! fails to animate ("interruptibleAnimatorForTransition returned nil"), and
//! `popstate` only fires after that transition settles, ~3s vs. the instant
//! on-screen Back. Our recognizer skips WebView history entirely: on gesture
//! start it sends on a channel and the router navigates immediately.
//!
//! **Android (future).** The generated `MainActivity` gets patched post-bundle
//! to route the hardware back into the same funnel; root-screen exit lands with
//! that work. Not wired yet.
//!
//! All native-gated, so the yet-to-ship zwiper web build keeps the browser's
//! real back button untouched.

use dioxus::prelude::*;

use crate::inbound::router::Router;

/// Layout wrapping all routes: installs the OS-back bridge, then renders the
/// active route via `Outlet`. Being a layout puts it inside the router context,
/// so `use_navigator()` resolves here.
#[component]
pub fn BackHandlerLayout() -> Element {
    #[cfg(all(target_os = "ios", feature = "mobile"))]
    {
        let nav = use_navigator();
        use_effect(move || {
            // Drain edge-gesture signals into router back-navigation. At a root
            // screen `can_go_back()` is false and we intentionally no-op.
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
            spawn(async move {
                while rx.recv().await.is_some() {
                    if nav.can_go_back() {
                        nav.go_back();
                    }
                }
            });
            ios::install_edge_back(tx);
        });
    }

    // Android: the patched MainActivity intercepts the OS back (gesture + button)
    // and dispatches a `zwipe:back` DOM event; we route it to the router, and
    // exit the app (finish the Activity) from a root screen. Inert until that
    // native patch is in place (see zcripts/android/back_handler patch).
    #[cfg(all(target_os = "android", feature = "mobile"))]
    {
        let nav = use_navigator();
        use_future(move || async move {
            let mut eval =
                document::eval("window.addEventListener('zwipe:back', () => dioxus.send(1));");
            while eval.recv::<i32>().await.is_ok() {
                if nav.can_go_back() {
                    nav.go_back();
                } else {
                    android::finish_activity();
                }
            }
        });
    }

    rsx! {
        Outlet::<Router> {}
    }
}

/// iOS left-edge back gesture: a `UIScreenEdgePanGestureRecognizer` on the
/// WKWebView whose target forwards `go_back` requests over a channel.
#[cfg(all(target_os = "ios", feature = "mobile"))]
mod ios {
    use dioxus::mobile::wry::WebViewExtIOS;
    use objc2::rc::{Allocated, Retained};
    use objc2::runtime::{AnyObject, NSObject};
    use objc2::{AnyThread, DefinedClass, MainThreadMarker, class, define_class, msg_send, sel};
    use tokio::sync::mpsc::UnboundedSender;

    /// `UIGestureRecognizerStateBegan` — fire once as the pan starts, so the
    /// nav feels as immediate as tapping Back.
    const STATE_BEGAN: isize = 1;
    /// `UIRectEdgeLeft` (1 << 1) — recognize only pans from the left screen edge.
    const EDGE_LEFT: usize = 2;

    /// Rust state carried by the Obj-C target object.
    struct Ivars {
        tx: UnboundedSender<()>,
    }

    define_class!(
        // SAFETY: NSObject has no subclassing requirements; no Drop impl.
        #[unsafe(super(NSObject))]
        #[name = "ZwipeEdgeBackTarget"]
        #[ivars = Ivars]
        struct EdgeBackTarget;

        impl EdgeBackTarget {
            /// Target-action for the edge pan. `sender` is the recognizer.
            #[unsafe(method(handleEdgePan:))]
            fn handle_edge_pan(&self, sender: &AnyObject) {
                // SAFETY: `state` is a valid selector on UIGestureRecognizer.
                let state: isize = unsafe { msg_send![sender, state] };
                if state == STATE_BEGAN {
                    let _ = self.ivars().tx.send(());
                }
            }
        }
    );

    impl EdgeBackTarget {
        fn new(tx: UnboundedSender<()>) -> Retained<Self> {
            let this = Self::alloc().set_ivars(Ivars { tx });
            // SAFETY: designated initializer for our NSObject subclass.
            unsafe { msg_send![super(this), init] }
        }
    }

    /// Attach the edge-back recognizer to the app's WKWebView. Must run on the
    /// main thread (UI effects do).
    pub fn install_edge_back(tx: UnboundedSender<()>) {
        if MainThreadMarker::new().is_none() {
            tracing::error!("edge-back gesture must be installed on the main thread; skipping");
            return;
        }
        let webview = dioxus::mobile::window().webview.webview();
        let target = EdgeBackTarget::new(tx);
        // SAFETY: standard UIKit gesture setup with correct selectors/types,
        // performed on the main thread.
        unsafe {
            let alloc: Allocated<AnyObject> =
                msg_send![class!(UIScreenEdgePanGestureRecognizer), alloc];
            let recognizer: Retained<AnyObject> =
                msg_send![alloc, initWithTarget: &*target, action: sel!(handleEdgePan:)];
            let _: () = msg_send![&*recognizer, setEdges: EDGE_LEFT];
            let _: () = msg_send![&*webview, addGestureRecognizer: &*recognizer];
        }
        // A gesture recognizer keeps only a weak reference to its target; leak
        // ours so it outlives this call (one target per app launch).
        std::mem::forget(target);
    }
}

/// Android app exit: finish the Activity when back is pressed at a root screen.
/// Same JNI + ndk-context pattern as `outbound/open_url.rs`.
#[cfg(all(target_os = "android", feature = "mobile"))]
mod android {
    use jni::objects::JObject;

    /// Finish the current Activity (exit the app), logging on failure.
    pub fn finish_activity() {
        if let Err(e) = try_finish() {
            tracing::error!("edge-back: failed to finish activity: {e}");
        }
    }

    fn try_finish() -> anyhow::Result<()> {
        let ctx = ndk_context::android_context();
        // SAFETY: ndk-context guarantees a valid JavaVM and Activity jobject.
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;
        let activity = unsafe { JObject::from_raw(ctx.context().cast()) };
        // activity.finish();
        env.call_method(&activity, "finish", "()V", &[])?;
        Ok(())
    }
}
