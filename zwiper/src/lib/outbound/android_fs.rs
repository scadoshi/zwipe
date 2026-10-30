//! Android private internal storage path, shared by `session` and `theme_store`.
//!
//! Resolves the app's private files dir (`/data/data/<pkg>/files`) via JNI from
//! the Android `Context` that `ndk-context` exposes. It's sandboxed per-app and
//! survives restarts; both stores drop their JSON here.

use std::path::PathBuf;

use jni::objects::{JObject, JString};

/// Local references `files_dir` creates: the `File` and its path string.
/// A hint to the VM, not a cap.
const LOCAL_FRAME_CAPACITY: i32 = 4;

/// The app's private internal files dir (e.g. `/data/data/<pkg>/files`).
///
/// The JNI calls run in their own local reference frame, so the references
/// they create are freed on return even on an already attached thread.
pub fn files_dir() -> anyhow::Result<PathBuf> {
    let ctx = ndk_context::android_context();
    // SAFETY: ndk-context guarantees a valid JavaVM and Context jobject for the
    // running app.
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let mut env = vm.attach_current_thread()?;
    // SAFETY: the same ndk-context guarantee covers the Context jobject, which
    // stays alive for the running app.
    let context = unsafe { JObject::from_raw(ctx.context().cast()) };

    let path = env.with_local_frame(LOCAL_FRAME_CAPACITY, |env| -> anyhow::Result<String> {
        // File dir = context.getFilesDir();
        let dir = env
            .call_method(&context, "getFilesDir", "()Ljava/io/File;", &[])?
            .l()?;
        // String path = dir.getAbsolutePath();
        let path = env
            .call_method(&dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?
            .l()?;
        Ok(env.get_string(&JString::from(path))?.into())
    })?;
    Ok(PathBuf::from(path))
}
