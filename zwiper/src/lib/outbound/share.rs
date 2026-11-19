//! Hands a text file to the OS share sheet.
//!
//! The file is written into the app's own storage and shared as a file, so Save to Files, Drive,
//! Mail and the rest receive a `.csv` under its name, however big. On iOS it goes in the temporary
//! directory and out through a `UIActivityViewController`. On Android it goes in `cache/shared/`,
//! which the app's `FileProvider` serves (declared by `zcripts/android/file_provider.sh`), and out
//! through an `ACTION_SEND` chooser carrying its `content://` URI with a read grant. Other
//! platforms have no share sheet.

use std::error::Error;
use std::fmt;
#[cfg(any(target_os = "android", test))]
use std::path::{Path, PathBuf};

/// The `FileProvider`'s authority past the application id; the manifest patch declares the same.
#[cfg(any(target_os = "android", test))]
const AUTHORITY_SUFFIX: &str = ".fileprovider";

/// The directory under the app's cache the `FileProvider` serves, and the only one.
#[cfg(any(target_os = "android", test))]
const SHARED_DIR: &str = "shared";

/// Why [`share_file`] couldn't open the share sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShareError {
    /// The platform has no share sheet.
    Unsupported,
    /// Called off the main thread, where `UIKit` must run.
    NotMainThread,
    /// The platform refused or failed, with its own message.
    Platform(String),
}

impl fmt::Display for ShareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported => write!(f, "no share sheet on this platform"),
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
/// [`ShareError`] when the sheet couldn't open: no share sheet on this platform, or the
/// platform's own failure.
pub fn share_file(file_name: &str, text: &str) -> Result<(), ShareError> {
    platform::share(file_name, text)
}

/// The name a shared file is written under: `file_name` with every path separator and leading dot
/// gone, so it stays inside its directory; `collection.csv` when nothing is left.
#[cfg_attr(
    not(any(target_os = "ios", target_os = "android", test)),
    expect(dead_code, reason = "only the mobile platforms write the file")
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

/// Where Android writes a shared file named `file_name`: under `cache` in [`SHARED_DIR`], the
/// directory the `FileProvider` serves.
#[cfg(any(target_os = "android", test))]
fn shared_path(cache: &Path, file_name: &str) -> PathBuf {
    cache.join(SHARED_DIR).join(safe_file_name(file_name))
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
    use std::fs;
    use std::path::PathBuf;

    use jni::JNIEnv;
    use jni::objects::{JClass, JObject, JValue};

    use super::ShareError;

    /// Local references `share` creates, the class loader and the intents among them. A hint to
    /// the VM, not a cap.
    const LOCAL_FRAME_CAPACITY: i32 = 32;

    /// `Intent.FLAG_ACTIVITY_NEW_TASK`, which `startActivity` requires when the context is not an
    /// `Activity`.
    const FLAG_ACTIVITY_NEW_TASK: i32 = 0x1000_0000;

    /// `Intent.FLAG_GRANT_READ_URI_PERMISSION`: the app the user picks may read the file.
    const FLAG_GRANT_READ_URI_PERMISSION: i32 = 0x0000_0001;

    pub(super) fn share(file_name: &str, text: &str) -> Result<(), ShareError> {
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
        result
    }

    fn platform_error(e: &jni::errors::Error) -> ShareError {
        ShareError::Platform(e.to_string())
    }

    impl From<jni::errors::Error> for ShareError {
        fn from(e: jni::errors::Error) -> Self {
            platform_error(&e)
        }
    }

    fn fire(
        env: &mut JNIEnv<'_>,
        context: &JObject<'_>,
        file_name: &str,
        text: &str,
    ) -> Result<(), ShareError> {
        env.with_local_frame(LOCAL_FRAME_CAPACITY, |env| -> Result<(), ShareError> {
            let path = write_shared(env, context, file_name, text)?;
            let uri = content_uri(env, context, &path)?;
            open_chooser(env, context, &uri, &super::safe_file_name(file_name))?;
            Ok(())
        })
    }

    /// Writes `text` into the shared directory under the app's cache and returns its path.
    fn write_shared(
        env: &mut JNIEnv<'_>,
        context: &JObject<'_>,
        file_name: &str,
        text: &str,
    ) -> Result<PathBuf, ShareError> {
        // String cache = context.getCacheDir().getAbsolutePath();
        let cache_dir = env
            .call_method(context, "getCacheDir", "()Ljava/io/File;", &[])?
            .l()?;
        let cache = env
            .call_method(&cache_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?
            .l()?;
        let cache: String = env.get_string(&cache.into())?.into();
        let path = super::shared_path(std::path::Path::new(&cache), file_name);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| ShareError::Platform(e.to_string()))?;
        }
        fs::write(&path, text).map_err(|e| ShareError::Platform(e.to_string()))?;
        Ok(path)
    }

    /// The `FileProvider`'s `content://` URI for `path`. The class is the app's own (androidx),
    /// which a native thread's `FindClass` can't see, so it loads through the context's class
    /// loader.
    fn content_uri<'local>(
        env: &mut JNIEnv<'local>,
        context: &JObject<'_>,
        path: &std::path::Path,
    ) -> jni::errors::Result<JObject<'local>> {
        let loader = env
            .call_method(context, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?
            .l()?;
        let name = env.new_string("androidx.core.content.FileProvider")?;
        let provider: JClass<'_> = env
            .call_method(
                &loader,
                "loadClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
                &[(&name).into()],
            )?
            .l()?
            .into();
        // String authority = context.getPackageName() + ".fileprovider";
        let package = env
            .call_method(context, "getPackageName", "()Ljava/lang/String;", &[])?
            .l()?;
        let package: String = env.get_string(&package.into())?.into();
        let authority = env.new_string(format!("{package}{}", super::AUTHORITY_SUFFIX))?;
        let path = env.new_string(path.to_string_lossy())?;
        let file = env.new_object("java/io/File", "(Ljava/lang/String;)V", &[(&path).into()])?;
        env.call_static_method(
            &provider,
            "getUriForFile",
            "(Landroid/content/Context;Ljava/lang/String;Ljava/io/File;)Landroid/net/Uri;",
            &[context.into(), (&authority).into(), (&file).into()],
        )?
        .l()
    }

    /// Starts the chooser for an `ACTION_SEND` of `uri` as `text/csv`, named `file_name`, with a
    /// read grant for the app picked.
    fn open_chooser(
        env: &mut JNIEnv<'_>,
        context: &JObject<'_>,
        uri: &JObject<'_>,
        file_name: &str,
    ) -> jni::errors::Result<()> {
        let intent_class = "android/content/Intent";
        let action_send = env
            .get_static_field(intent_class, "ACTION_SEND", "Ljava/lang/String;")?
            .l()?;
        let send = env.new_object(
            intent_class,
            "(Ljava/lang/String;)V",
            &[(&action_send).into()],
        )?;
        let mime = env.new_string("text/csv")?;
        env.call_method(
            &send,
            "setType",
            "(Ljava/lang/String;)Landroid/content/Intent;",
            &[(&mime).into()],
        )?;
        let stream = env
            .get_static_field(intent_class, "EXTRA_STREAM", "Ljava/lang/String;")?
            .l()?;
        env.call_method(
            &send,
            "putExtra",
            "(Ljava/lang/String;Landroid/os/Parcelable;)Landroid/content/Intent;",
            &[(&stream).into(), uri.into()],
        )?;
        let subject = env
            .get_static_field(intent_class, "EXTRA_SUBJECT", "Ljava/lang/String;")?
            .l()?;
        let name = env.new_string(file_name)?;
        env.call_method(
            &send,
            "putExtra",
            "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
            &[(&subject).into(), (&name).into()],
        )?;
        // The grant reaches the picked app through the intent's clip data.
        let clip = env
            .call_static_method(
                "android/content/ClipData",
                "newRawUri",
                "(Ljava/lang/CharSequence;Landroid/net/Uri;)Landroid/content/ClipData;",
                &[(&name).into(), uri.into()],
            )?
            .l()?;
        env.call_method(
            &send,
            "setClipData",
            "(Landroid/content/ClipData;)V",
            &[(&clip).into()],
        )?;
        env.call_method(
            &send,
            "addFlags",
            "(I)Landroid/content/Intent;",
            &[JValue::Int(FLAG_GRANT_READ_URI_PERMISSION)],
        )?;

        // Intent chooser = Intent.createChooser(send, null);
        let no_title = JObject::null();
        let chooser = env
            .call_static_method(
                intent_class,
                "createChooser",
                "(Landroid/content/Intent;Ljava/lang/CharSequence;)Landroid/content/Intent;",
                &[(&send).into(), (&no_title).into()],
            )?
            .l()?;
        let mut flags = FLAG_GRANT_READ_URI_PERMISSION;
        // An Application context can start an activity only into a new task.
        if !env.is_instance_of(context, "android/app/Activity")? {
            flags |= FLAG_ACTIVITY_NEW_TASK;
        }
        env.call_method(
            &chooser,
            "addFlags",
            "(I)Landroid/content/Intent;",
            &[JValue::Int(flags)],
        )?;
        env.call_method(
            context,
            "startActivity",
            "(Landroid/content/Intent;)V",
            &[(&chooser).into()],
        )?;
        Ok(())
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
    fn a_shared_file_lands_in_the_directory_the_provider_serves() {
        let path = shared_path(Path::new("/data/user/0/app/cache"), "zwipe-collection.csv");
        assert_eq!(
            path,
            Path::new("/data/user/0/app/cache/shared/zwipe-collection.csv")
        );
        assert_eq!(
            shared_path(Path::new("/c"), "../x.csv"),
            Path::new("/c/shared/x.csv"),
            "the name can't climb out"
        );
    }

    #[test]
    fn the_manifest_patch_serves_what_android_shares() {
        let script = include_str!("../../../../zcripts/android/file_provider.sh");
        assert!(
            script.contains(&format!("${{applicationId}}{AUTHORITY_SUFFIX}")),
            "the provider's authority"
        );
        assert!(
            script.contains(&format!(
                "<cache-path name=\"{SHARED_DIR}\" path=\"{SHARED_DIR}/\""
            )),
            "the directory it serves"
        );
    }
}
