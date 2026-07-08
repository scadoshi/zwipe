//! Open a URL with the OS default handler (browser, mail app, etc.).
//!
//! Dioxus routes external `<a href>` navigations through `webbrowser::open`, but
//! on iOS (`webbrowser/src/ios.rs`) and Android (`android.rs`) that crate first
//! calls `get_http_url`, which hard-rejects any non-`http(s)` URL with
//! `"not an http url"`. Dioxus ignores the error, so `mailto:` links silently do
//! nothing on mobile. We sidestep the webview and hand the URL straight to the
//! OS: `UIApplication.openURL` on iOS, an `ACTION_VIEW` intent on Android, and
//! `webbrowser` on desktop (where it handles `mailto:` fine).

/// Opens `url` with the OS default handler, logging on failure.
pub fn open(url: &str) {
    if let Err(e) = platform::open(url) {
        tracing::error!("failed to open url {url:?}: {e}");
    }
}

// ============================================================================
// iOS: UIApplication.openURL. Mirrors the `webbrowser` crate's iOS backend,
// minus its `http(s)`-only gate so that `mailto:` is allowed through.
// ============================================================================
#[cfg(target_os = "ios")]
mod platform {
    use objc2::{
        Encode, Encoding, MainThreadMarker, class, msg_send, rc::Retained, runtime::NSObject,
    };
    use objc2_foundation::{NSDictionary, NSString, NSURL};
    use std::ffi::c_void;

    /// Empty stand-in for the Objective-C completion block, so we don't pull in
    /// the `block2` crate just to pass a null block.
    #[repr(transparent)]
    struct FakeBlock(*const c_void);

    // SAFETY: `#[repr(transparent)]` over a pointer — same layout as an optional
    // block reference, which is what `completionHandler:` expects.
    unsafe impl Encode for FakeBlock {
        const ENCODING: Encoding = Encoding::Block;
    }

    pub fn open(url: &str) -> anyhow::Result<()> {
        // `sharedApplication` and `openURL:` must be used on the main thread. A
        // UI tap handler already runs there, so the marker is available.
        let _mtm = MainThreadMarker::new()
            .ok_or_else(|| anyhow::anyhow!("UIApplication must be used on the main thread"))?;

        // app = UIApplication.sharedApplication
        // SAFETY: correct selector/signature; we hold the main-thread marker.
        let app: Option<Retained<NSObject>> =
            unsafe { msg_send![class!(UIApplication), sharedApplication] };
        let app = app.ok_or_else(|| anyhow::anyhow!("UIApplication is NULL"))?;

        let url_string = NSString::from_str(url);
        let url_object = NSURL::URLWithString(&url_string)
            .ok_or_else(|| anyhow::anyhow!("could not build NSURL from {url:?}"))?;
        let options: Retained<NSDictionary<NSObject, NSObject>> = NSDictionary::new();
        let handler = FakeBlock(std::ptr::null());

        // [app openURL:url options:{} completionHandler:nil]
        // SAFETY: correct selector/signature; a null completion block is allowed.
        let _: () = unsafe {
            msg_send![&*app, openURL: &*url_object, options: &*options, completionHandler: handler]
        };
        Ok(())
    }
}

// ============================================================================
// Android: an ACTION_VIEW intent, resolved by the OS to the right app. Same JNI
// + ndk-context pattern as `outbound/session.rs`.
// ============================================================================
#[cfg(target_os = "android")]
mod platform {
    use jni::objects::JObject;

    pub fn open(url: &str) -> anyhow::Result<()> {
        let ctx = ndk_context::android_context();
        // SAFETY: ndk-context guarantees a valid JavaVM and Activity jobject for
        // the running app.
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;
        let context = unsafe { JObject::from_raw(ctx.context().cast()) };

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

        // context.startActivity(intent);
        env.call_method(
            &context,
            "startActivity",
            "(Landroid/content/Intent;)V",
            &[(&intent).into()],
        )?;
        Ok(())
    }
}

// ============================================================================
// Desktop (dev): webbrowser, which handles mailto: on macOS/Windows/Linux.
// ============================================================================
#[cfg(not(any(target_os = "ios", target_os = "android")))]
mod platform {
    pub fn open(url: &str) -> anyhow::Result<()> {
        webbrowser::open(url)?;
        Ok(())
    }
}
