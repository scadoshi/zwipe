//! Hands a text file to the OS share sheet.
//!
//! On iOS the file is written to the app's temporary directory and shared as that file through a
//! `UIActivityViewController`, so Save to Files, Mail and AirDrop keep its name. On Android it goes
//! out as an `ACTION_SEND` chooser with the text itself as `EXTRA_TEXT` and the name as
//! `EXTRA_SUBJECT`: sharing a file there takes a `FileProvider`, which the app doesn't declare. Other
//! platforms have no share sheet.

use std::error::Error;
use std::fmt;

/// The largest text Android's share carries. An intent's extras cross a Binder transaction capped
/// near 1 MiB for the whole process, and a Java string is two bytes a character, so half that
/// leaves room for the rest of the intent.
#[cfg_attr(
    not(any(target_os = "android", test)),
    expect(dead_code, reason = "only Android shares the text itself")
)]
const MAX_SHARED_TEXT_BYTES: usize = 400 * 1024;

/// Why [`share_file`] couldn't open the share sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShareError {
    /// The platform has no share sheet.
    Unsupported,
    /// The text is past what the platform's share can carry.
    TooBig,
    /// Called off the main thread, where `UIKit` must run.
    NotMainThread,
    /// The platform refused or failed, with its own message.
    Platform(String),
}

impl fmt::Display for ShareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported => write!(f, "no share sheet on this platform"),
            Self::TooBig => write!(f, "too big to share"),
            Self::NotMainThread => write!(f, "the share sheet opens only from the main thread"),
            Self::Platform(message) => write!(f, "could not share: {message}"),
        }
    }
}

impl Error for ShareError {}

/// Opens the share sheet with `text` as a file named `file_name`.
///
/// # Errors
///
/// [`ShareError`] when the sheet couldn't open: no share sheet on this platform, text past
/// Android's cap, or the platform's own failure.
pub fn share_file(file_name: &str, text: &str) -> Result<(), ShareError> {
    platform::share(file_name, text)
}

/// Whether Android's share can carry `text`: at most [`MAX_SHARED_TEXT_BYTES`].
#[cfg_attr(
    not(any(target_os = "android", test)),
    expect(dead_code, reason = "only Android shares the text itself")
)]
fn fits_android(text: &str) -> bool {
    text.len() <= MAX_SHARED_TEXT_BYTES
}

/// The name a shared file is written under: `file_name` with every path separator and leading dot
/// gone, so it stays inside the temporary directory; `collection.csv` when nothing is left.
#[cfg_attr(
    not(any(target_os = "ios", test)),
    expect(dead_code, reason = "only iOS writes the file")
)]
fn safe_file_name(file_name: &str) -> String {
    let cleaned: String = file_name
        .chars()
        .filter(|c| !matches!(c, '/' | '\\' | '\0'))
        .collect();
    let trimmed = cleaned.trim_start_matches('.').trim();
    if trimmed.is_empty() {
        "collection.csv".to_owned()
    } else {
        trimmed.to_owned()
    }
}

#[cfg(target_os = "ios")]
mod platform {
    use std::ffi::c_void;
    use std::{env, fs};

    use objc2::rc::{Allocated, Retained};
    use objc2::runtime::{AnyClass, AnyObject};
    use objc2::{Encode, Encoding, MainThreadMarker, msg_send};
    use objc2_foundation::{NSArray, NSString, NSURL};

    use super::ShareError;

    /// A null completion block, so the crate needs no block dependency.
    #[repr(transparent)]
    struct NullBlock(*const c_void);

    // SAFETY: `#[repr(transparent)]` over a pointer, the layout of an optional block reference,
    // which `completion:` takes.
    unsafe impl Encode for NullBlock {
        const ENCODING: Encoding = Encoding::Block;
    }

    pub(super) fn share(file_name: &str, text: &str) -> Result<(), ShareError> {
        MainThreadMarker::new().ok_or(ShareError::NotMainThread)?;
        let path = env::temp_dir().join(super::safe_file_name(file_name));
        fs::write(&path, text).map_err(|e| ShareError::Platform(e.to_string()))?;
        let path = path
            .to_str()
            .ok_or_else(|| ShareError::Platform("temporary path is not utf-8".into()))?;
        let url = NSURL::fileURLWithPath(&NSString::from_str(path));
        present(&url).map_err(|e| ShareError::Platform(e.into()))
    }

    /// Presents a `UIActivityViewController` sharing `url` over the topmost view controller of the
    /// key window. On iPad the sheet is a popover, anchored to that controller's view.
    fn present(url: &NSURL) -> Result<(), &'static str> {
        let app_class = AnyClass::get(c"UIApplication").ok_or("no UIApplication")?;
        // SAFETY: `sharedApplication` takes no arguments and returns the process's
        // UIApplication, which lives as long as the process.
        let app: *mut AnyObject = unsafe { msg_send![app_class, sharedApplication] };
        // SAFETY: the pointer is either null or the live shared application.
        let app = unsafe { app.as_ref() }.ok_or("no UIApplication")?;
        // SAFETY: `keyWindow` is a nullable UIWindow getter, read on the main thread.
        let window: Option<Retained<AnyObject>> = unsafe { msg_send![app, keyWindow] };
        let window = window.ok_or("no key window")?;
        // SAFETY: `rootViewController` is a nullable UIViewController getter.
        let root: Option<Retained<AnyObject>> = unsafe { msg_send![&*window, rootViewController] };
        let mut presenter = root.ok_or("no root view controller")?;
        // A controller already presenting something can't present again, so walk up to the one
        // on top.
        loop {
            // SAFETY: `presentedViewController` is a nullable getter.
            let on_top: Option<Retained<AnyObject>> =
                unsafe { msg_send![&*presenter, presentedViewController] };
            match on_top {
                Some(next) => presenter = next,
                None => break,
            }
        }

        let items: Retained<NSArray<NSURL>> = NSArray::from_slice(&[url]);
        let class =
            AnyClass::get(c"UIActivityViewController").ok_or("no UIActivityViewController")?;
        // SAFETY: `alloc` on a class object returns an uninitialized instance.
        let allocated: Allocated<AnyObject> = unsafe { msg_send![class, alloc] };
        let no_activities: Option<&AnyObject> = None;
        // SAFETY: `initWithActivityItems:applicationActivities:` takes an NSArray of items and a
        // nullable NSArray of custom activities.
        let sheet: Option<Retained<AnyObject>> = unsafe {
            msg_send![allocated, initWithActivityItems: &*items, applicationActivities: no_activities]
        };
        let sheet = sheet.ok_or("UIActivityViewController init returned nil")?;

        // SAFETY: `popoverPresentationController` is a nullable getter, non-null on iPad.
        let popover: Option<Retained<AnyObject>> =
            unsafe { msg_send![&*sheet, popoverPresentationController] };
        if let Some(popover) = popover {
            // SAFETY: `view` is the presenter's UIView getter.
            let view: Option<Retained<AnyObject>> = unsafe { msg_send![&*presenter, view] };
            // SAFETY: `setSourceView:` takes a nullable UIView.
            unsafe {
                let () = msg_send![&*popover, setSourceView: view.as_deref()];
            }
        }

        // SAFETY: a null completion block is allowed, and this runs on the main thread.
        unsafe {
            let () = msg_send![
                &*presenter,
                presentViewController: &*sheet,
                animated: true,
                completion: NullBlock(std::ptr::null()),
            ];
        }
        Ok(())
    }
}

#[cfg(target_os = "android")]
mod platform {
    use jni::JNIEnv;
    use jni::objects::JObject;

    use super::ShareError;

    /// Local references `share` creates: the action, the type, the intent, the two extra keys and
    /// their values, the chooser, the `Activity` class and what `putExtra` and `addFlags` return.
    /// A hint to the VM, not a cap.
    const LOCAL_FRAME_CAPACITY: i32 = 16;

    /// `Intent.FLAG_ACTIVITY_NEW_TASK`, which `startActivity` requires when the context is not an
    /// `Activity`.
    const FLAG_ACTIVITY_NEW_TASK: i32 = 0x1000_0000;

    pub(super) fn share(file_name: &str, text: &str) -> Result<(), ShareError> {
        if !super::fits_android(text) {
            return Err(ShareError::TooBig);
        }
        let ctx = ndk_context::android_context();
        // SAFETY: ndk-context guarantees a valid JavaVM and Context jobject for the running app.
        let vm =
            unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.map_err(|e| platform_error(&e))?;
        let mut env = vm.attach_current_thread().map_err(|e| platform_error(&e))?;
        // SAFETY: the same ndk-context guarantee covers the Context jobject, which stays alive
        // for the running app. It may be the Application rather than an Activity.
        let context = unsafe { JObject::from_raw(ctx.context().cast()) };
        let result = fire(&mut env, &context, file_name, text);
        // A failed call leaves its Java exception pending, and the next JNI call anywhere in the
        // process aborts on it.
        if result.is_err()
            && let Err(e) = env.exception_clear()
        {
            tracing::error!("could not clear a java exception: {e}");
        }
        result.map_err(|e| platform_error(&e))
    }

    fn platform_error(e: &jni::errors::Error) -> ShareError {
        ShareError::Platform(e.to_string())
    }

    fn fire(
        env: &mut JNIEnv<'_>,
        context: &JObject<'_>,
        file_name: &str,
        text: &str,
    ) -> jni::errors::Result<()> {
        env.with_local_frame(LOCAL_FRAME_CAPACITY, |env| -> jni::errors::Result<()> {
            // Intent send = new Intent(Intent.ACTION_SEND).setType("text/plain");
            let action_send = env
                .get_static_field(
                    "android/content/Intent",
                    "ACTION_SEND",
                    "Ljava/lang/String;",
                )?
                .l()?;
            let send = env.new_object(
                "android/content/Intent",
                "(Ljava/lang/String;)V",
                &[(&action_send).into()],
            )?;
            let mime = env.new_string("text/plain")?;
            env.call_method(
                &send,
                "setType",
                "(Ljava/lang/String;)Landroid/content/Intent;",
                &[(&mime).into()],
            )?;
            for (field, value) in [("EXTRA_TEXT", text), ("EXTRA_SUBJECT", file_name)] {
                let key = env
                    .get_static_field("android/content/Intent", field, "Ljava/lang/String;")?
                    .l()?;
                let value = env.new_string(value)?;
                env.call_method(
                    &send,
                    "putExtra",
                    "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
                    &[(&key).into(), (&value).into()],
                )?;
            }

            // Intent chooser = Intent.createChooser(send, null);
            let no_title = JObject::null();
            let chooser = env
                .call_static_method(
                    "android/content/Intent",
                    "createChooser",
                    "(Landroid/content/Intent;Ljava/lang/CharSequence;)Landroid/content/Intent;",
                    &[(&send).into(), (&no_title).into()],
                )?
                .l()?;

            // An Application context can start an activity only into a new task.
            if !env.is_instance_of(context, "android/app/Activity")? {
                env.call_method(
                    &chooser,
                    "addFlags",
                    "(I)Landroid/content/Intent;",
                    &[FLAG_ACTIVITY_NEW_TASK.into()],
                )?;
            }
            env.call_method(
                context,
                "startActivity",
                "(Landroid/content/Intent;)V",
                &[(&chooser).into()],
            )?;
            Ok(())
        })
    }
}

#[cfg(not(any(target_os = "ios", target_os = "android")))]
mod platform {
    use super::ShareError;

    pub(super) fn share(_file_name: &str, _text: &str) -> Result<(), ShareError> {
        Err(ShareError::Unsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shared_file_stays_in_the_temporary_directory() {
        assert_eq!(
            safe_file_name("zwipe-collection.csv"),
            "zwipe-collection.csv"
        );
        assert_eq!(safe_file_name("../../etc/passwd"), "etcpasswd");
        assert_eq!(safe_file_name("..."), "collection.csv");
        assert_eq!(safe_file_name(""), "collection.csv");
    }

    #[test]
    fn android_carries_text_up_to_its_cap() {
        assert!(fits_android(&"x".repeat(MAX_SHARED_TEXT_BYTES)));
        assert!(!fits_android(&"x".repeat(MAX_SHARED_TEXT_BYTES + 1)));
    }
}
