//! Android private internal storage path, shared by `session` and `theme_store`.
//!
//! Resolves the app's private files dir (`/data/data/<pkg>/files`) via JNI from
//! the Android `Context` that `ndk-context` exposes. It's sandboxed per-app and
//! survives restarts; both stores drop their JSON here.

use jni::objects::{JObject, JString};
use std::path::PathBuf;

/// The app's private internal files dir (e.g. `/data/data/<pkg>/files`).
pub fn files_dir() -> anyhow::Result<PathBuf> {
    let ctx = ndk_context::android_context();
    // SAFETY: ndk-context guarantees a valid JavaVM and Context jobject for the
    // running app.
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let mut env = vm.attach_current_thread()?;
    let context = unsafe { JObject::from_raw(ctx.context().cast()) };

    // File dir = context.getFilesDir();
    let dir = env
        .call_method(&context, "getFilesDir", "()Ljava/io/File;", &[])?
        .l()?;
    // String path = dir.getAbsolutePath();
    let path = env
        .call_method(&dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?
        .l()?;
    let path: String = env.get_string(&JString::from(path))?.into();
    Ok(PathBuf::from(path))
}
