//! Open a URL in the in-app browser or with the OS default handler.
//!
//! Dioxus routes external `<a href>` navigations through `webbrowser::open`, but
//! on iOS (`webbrowser/src/ios.rs`) and Android (`android.rs`) that crate first
//! calls `get_http_url`, which hard-rejects any non-`http(s)` URL with
//! `"not an http url"`. Dioxus ignores the error, so `mailto:` links silently do
//! nothing on mobile. We sidestep the webview and hand the URL to the platform.
//!
//! On mobile, `http` and `https` URLs open in the in-app browser: an
//! `SFSafariViewController` sheet on iOS, a Custom Tab on Android. Every other
//! scheme goes to the OS: `UIApplication.openURL` on iOS, an `ACTION_VIEW`
//! intent on Android. Desktop hands everything to `webbrowser`, which handles
//! `mailto:` fine there.

/// Opens `url` in the in-app browser or with the OS default handler, logging on
/// failure.
pub fn open(url: &str) {
    if let Err(e) = platform::open(url) {
        tracing::error!("failed to open url {url:?}: {e}");
    }
}

/// Whether `url` belongs in the in-app browser, which only loads web pages: its
/// scheme is `http` or `https`, in any case.
#[cfg_attr(
    not(any(target_os = "ios", target_os = "android", test)),
    expect(dead_code, reason = "desktop opens every URL with webbrowser")
)]
fn opens_in_app(url: &str) -> bool {
    url.split_once(':').is_some_and(|(scheme, _)| {
        scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
    })
}

// iOS: SFSafariViewController for web pages, UIApplication.openURL for the rest.
// The openURL path mirrors the `webbrowser` crate's iOS backend, minus its
// `http(s)`-only gate so that `mailto:` is allowed through.
#[cfg(target_os = "ios")]
mod platform {
    use std::ffi::c_void;

    use objc2::rc::{Allocated, Retained};
    use objc2::runtime::{AnyClass, NSObject};
    use objc2::{Encode, Encoding, MainThreadMarker, class, msg_send};
    use objc2_foundation::{NSDictionary, NSString, NSURL};

    // Links SafariServices, which holds SFSafariViewController. The class is
    // looked up by name at runtime, so no symbols are declared.
    #[link(name = "SafariServices", kind = "framework")]
    unsafe extern "C" {}

    /// Empty stand-in for the Objective-C completion block, so we don't pull in
    /// the `block2` crate just to pass a null block.
    #[repr(transparent)]
    struct FakeBlock(*const c_void);

    // SAFETY: `#[repr(transparent)]` over a pointer, same layout as an optional
    // block reference, which is what `completionHandler:` and `completion:`
    // expect.
    unsafe impl Encode for FakeBlock {
        const ENCODING: Encoding = Encoding::Block;
    }

    pub(crate) fn open(url: &str) -> anyhow::Result<()> {
        // UIKit must be used on the main thread. A UI tap handler already runs
        // there, so the marker is available.
        let _mtm = MainThreadMarker::new()
            .ok_or_else(|| anyhow::anyhow!("UIKit must be used on the main thread"))?;

        // app = UIApplication.sharedApplication
        // SAFETY: correct selector/signature; we hold the main-thread marker.
        let app: Option<Retained<NSObject>> =
            unsafe { msg_send![class!(UIApplication), sharedApplication] };
        let app = app.ok_or_else(|| anyhow::anyhow!("UIApplication is NULL"))?;

        let url_string = NSString::from_str(url);
        let url_object = NSURL::URLWithString(&url_string)
            .ok_or_else(|| anyhow::anyhow!("could not build NSURL from {url:?}"))?;

        if super::opens_in_app(url) {
            match present_safari(&app, &url_object) {
                Ok(()) => return Ok(()),
                Err(e) => tracing::warn!("in-app browser unavailable, opening externally: {e}"),
            }
        }
        open_external(&app, &url_object);
        Ok(())
    }

    /// Presents an `SFSafariViewController` for `url` over the topmost view
    /// controller of the key window. Must run on the main thread with an
    /// `http(s)` URL; the controller raises an exception for any other scheme.
    fn present_safari(app: &NSObject, url: &NSURL) -> anyhow::Result<()> {
        // window = app.keyWindow
        // SAFETY: correct selector/signature; the caller is on the main thread.
        let window: Option<Retained<NSObject>> = unsafe { msg_send![app, keyWindow] };
        let window = window.ok_or_else(|| anyhow::anyhow!("no key window"))?;

        // presenter = window.rootViewController
        // SAFETY: correct selector/signature; the caller is on the main thread.
        let root: Option<Retained<NSObject>> = unsafe { msg_send![&*window, rootViewController] };
        let mut presenter =
            root.ok_or_else(|| anyhow::anyhow!("key window has no root view controller"))?;

        // A controller already presenting something can't present again, so
        // walk up to the one on top.
        loop {
            // SAFETY: correct selector/signature; the caller is on the main thread.
            let on_top: Option<Retained<NSObject>> =
                unsafe { msg_send![&*presenter, presentedViewController] };
            match on_top {
                Some(next) => presenter = next,
                None => break,
            }
        }

        let safari_class = AnyClass::get(c"SFSafariViewController")
            .ok_or_else(|| anyhow::anyhow!("SFSafariViewController is not available"))?;

        // safari = [[SFSafariViewController alloc] initWithURL:url]
        // SAFETY: correct selector/signature for a class object.
        let allocated: Allocated<NSObject> = unsafe { msg_send![safari_class, alloc] };
        // SAFETY: correct selector/signature; `opens_in_app` limited the URL to
        // http(s), the schemes `initWithURL:` accepts.
        let safari: Option<Retained<NSObject>> = unsafe { msg_send![allocated, initWithURL: url] };
        let safari =
            safari.ok_or_else(|| anyhow::anyhow!("SFSafariViewController init returned NULL"))?;

        // [presenter presentViewController:safari animated:YES completion:nil]
        // SAFETY: correct selector/signature; a null completion block is
        // allowed and the caller is on the main thread.
        unsafe {
            let () = msg_send![
                &*presenter,
                presentViewController: &*safari,
                animated: true,
                completion: FakeBlock(std::ptr::null())
            ];
        }
        Ok(())
    }

    /// Hands `url` to the OS with `UIApplication.openURL`.
    fn open_external(app: &NSObject, url: &NSURL) {
        let options: Retained<NSDictionary<NSObject, NSObject>> = NSDictionary::new();
        let handler = FakeBlock(std::ptr::null());

        // [app openURL:url options:{} completionHandler:nil]
        // SAFETY: correct selector/signature; a null completion block is allowed.
        let _: () =
            unsafe { msg_send![app, openURL: url, options: &*options, completionHandler: handler] };
    }
}

// Android: an ACTION_VIEW intent, resolved by the OS to the right app. Web pages
// carry the Custom Tabs session extra, so a browser that supports it opens as a
// tab over the app. Same JNI + ndk-context pattern as `outbound/session.rs`.
#[cfg(target_os = "android")]
mod platform {
    use jni::JNIEnv;
    use jni::objects::JObject;

    /// Local references `open` creates: the URL string, the `Uri`, the action
    /// string, the `Intent`, and for web pages the `Bundle`, its key and the
    /// `Intent` that `putExtras` returns. A hint to the VM, not a cap.
    const LOCAL_FRAME_CAPACITY: i32 = 12;

    /// Intent extra that marks an `ACTION_VIEW` as a Custom Tabs request. The
    /// value is a `Bundle` holding a session binder, which may be null.
    const CUSTOM_TABS_SESSION: &str = "android.support.customtabs.extra.SESSION";

    /// Fires the intent from its own local reference frame, so the references
    /// it creates are freed on return even on an already attached thread.
    pub(crate) fn open(url: &str) -> anyhow::Result<()> {
        let ctx = ndk_context::android_context();
        // SAFETY: ndk-context guarantees a valid JavaVM and Activity jobject for
        // the running app.
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;
        // SAFETY: the same ndk-context guarantee covers the Activity jobject,
        // which stays alive for the running app.
        let context = unsafe { JObject::from_raw(ctx.context().cast()) };

        env.with_local_frame(LOCAL_FRAME_CAPACITY, |env| -> anyhow::Result<()> {
            // Uri uri = Uri.parse(url);
            let url_jstring = env.new_string(url)?;
            let uri = env
                .call_static_method(
                    "android/net/Uri",
                    "parse",
                    "(Ljava/lang/String;)Landroid/net/Uri;",
                    &[(&url_jstring).into()],
                )?
                .l()?;

            // Intent intent = new Intent(Intent.ACTION_VIEW, uri);
            let action_view = env
                .get_static_field(
                    "android/content/Intent",
                    "ACTION_VIEW",
                    "Ljava/lang/String;",
                )?
                .l()?;
            let intent = env.new_object(
                "android/content/Intent",
                "(Ljava/lang/String;Landroid/net/Uri;)V",
                &[(&action_view).into(), (&uri).into()],
            )?;

            if super::opens_in_app(url)
                && let Err(e) = add_custom_tabs_session(env, &intent)
            {
                tracing::warn!("custom tabs extra failed, opening a plain intent: {e}");
                // A failed JNI call leaves its Java exception pending, and
                // the next call would fail on it.
                env.exception_clear()?;
            }

            // context.startActivity(intent);
            env.call_method(
                &context,
                "startActivity",
                "(Landroid/content/Intent;)V",
                &[(&intent).into()],
            )?;
            Ok(())
        })
    }

    /// Adds the Custom Tabs session extra to `intent`. `putExtras` is the last
    /// call, so on error the intent is left as it was.
    fn add_custom_tabs_session(env: &mut JNIEnv<'_>, intent: &JObject<'_>) -> anyhow::Result<()> {
        // Bundle extras = new Bundle();
        let extras = env.new_object("android/os/Bundle", "()V", &[])?;

        // extras.putBinder(CUSTOM_TABS_SESSION, null);
        let key = env.new_string(CUSTOM_TABS_SESSION)?;
        env.call_method(
            &extras,
            "putBinder",
            "(Ljava/lang/String;Landroid/os/IBinder;)V",
            &[(&key).into(), (&JObject::null()).into()],
        )?;

        // intent.putExtras(extras);
        env.call_method(
            intent,
            "putExtras",
            "(Landroid/os/Bundle;)Landroid/content/Intent;",
            &[(&extras).into()],
        )?;
        Ok(())
    }
}

// Desktop (dev): webbrowser, which handles mailto: on macOS/Windows/Linux.
#[cfg(not(any(target_os = "ios", target_os = "android")))]
mod platform {
    pub(crate) fn open(url: &str) -> anyhow::Result<()> {
        webbrowser::open(url)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_schemes_open_in_app() {
        for url in [
            "https://zwipe.net/guides/filtering",
            "http://example.com",
            "HTTPS://ZWIPE.NET",
            "Http://example.com/path?q=1",
        ] {
            assert!(opens_in_app(url), "input {url:?}");
        }
    }

    #[test]
    fn other_schemes_go_to_the_os() {
        for url in [
            "mailto:support@zwipe.net",
            "tel:+15555550100",
            "sms:+15555550100",
            "httpx://example.com",
            "ftp://example.com",
            "https",
            "",
            "zwipe.net/https:",
        ] {
            assert!(!opens_in_app(url), "input {url:?}");
        }
    }
}
