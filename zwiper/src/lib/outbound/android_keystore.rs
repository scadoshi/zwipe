//! AES-256-GCM sealing under an Android Keystore key, for the session file.
//!
//! The key lives in the Keystore under a fixed alias and never leaves it: Rust
//! hands plaintext to `javax.crypto.Cipher` over JNI and gets the ciphertext
//! back, and the reverse. The key needs no user authentication, so sealing and
//! opening work with the screen locked. A reinstall drops the key, so a blob
//! written by an earlier install does not open.

use jni::objects::{JByteArray, JObject, JString, JValue};
use jni::{AttachGuard, JavaVM};

/// Keystore alias of the session key. Changing it orphans every sealed file.
const ALIAS: &str = "zwipe-session";
/// GCM nonce length in bytes; `Cipher` picks a fresh one per seal.
const IV_LEN: usize = 12;
/// GCM tag length in bits.
const TAG_BITS: i32 = 128;
/// `KeyProperties.PURPOSE_ENCRYPT | KeyProperties.PURPOSE_DECRYPT`.
const PURPOSES: i32 = 1 | 2;
/// `Cipher.ENCRYPT_MODE` and `Cipher.DECRYPT_MODE`.
const ENCRYPT_MODE: i32 = 1;
const DECRYPT_MODE: i32 = 2;
const TRANSFORMATION: &str = "AES/GCM/NoPadding";

/// Encrypts `plain` and returns `iv || ciphertext || tag`.
pub fn seal(plain: &[u8]) -> anyhow::Result<Vec<u8>> {
    with_env(|env| {
        let key = session_key(env)?;
        let cipher = cipher(env)?;
        // cipher.init(Cipher.ENCRYPT_MODE, key);
        env.call_method(
            &cipher,
            "init",
            "(ILjava/security/Key;)V",
            &[JValue::Int(ENCRYPT_MODE), JValue::Object(&key)],
        )?;
        // byte[] iv = cipher.getIV();
        let iv = env.call_method(&cipher, "getIV", "()[B", &[])?.l()?;
        let iv = env.convert_byte_array(JByteArray::from(iv))?;
        anyhow::ensure!(iv.len() == IV_LEN, "unexpected GCM iv length {}", iv.len());
        // byte[] out = cipher.doFinal(plain);
        let input = env.byte_array_from_slice(plain)?;
        let out = env
            .call_method(&cipher, "doFinal", "([B)[B", &[JValue::Object(&input)])?
            .l()?;
        let out = env.convert_byte_array(JByteArray::from(out))?;
        let mut blob = iv;
        blob.extend_from_slice(&out);
        Ok(blob)
    })
}

/// Decrypts a blob written by [`seal`]. Fails if the key is gone, the blob is
/// truncated, or the tag does not verify.
pub fn open(blob: &[u8]) -> anyhow::Result<Vec<u8>> {
    anyhow::ensure!(
        blob.len() > IV_LEN,
        "sealed blob too short ({} bytes)",
        blob.len()
    );
    let (iv, ciphertext) = blob.split_at(IV_LEN);
    with_env(|env| {
        let key = session_key(env)?;
        let cipher = cipher(env)?;
        // GCMParameterSpec spec = new GCMParameterSpec(128, iv);
        let iv = env.byte_array_from_slice(iv)?;
        let spec = env.new_object(
            "javax/crypto/spec/GCMParameterSpec",
            "(I[B)V",
            &[JValue::Int(TAG_BITS), JValue::Object(&iv)],
        )?;
        // cipher.init(Cipher.DECRYPT_MODE, key, spec);
        env.call_method(
            &cipher,
            "init",
            "(ILjava/security/Key;Ljava/security/spec/AlgorithmParameterSpec;)V",
            &[
                JValue::Int(DECRYPT_MODE),
                JValue::Object(&key),
                JValue::Object(&spec),
            ],
        )?;
        // byte[] out = cipher.doFinal(ciphertext);
        let input = env.byte_array_from_slice(ciphertext)?;
        let out = env
            .call_method(&cipher, "doFinal", "([B)[B", &[JValue::Object(&input)])?
            .l()?;
        Ok(env.convert_byte_array(JByteArray::from(out))?)
    })
}

/// Runs `f` on the app's JavaVM. A Java exception raised inside `f` is
/// cleared and returned as an error carrying its `toString()`, so the VM is
/// left clean for the next JNI call.
fn with_env<T>(f: impl FnOnce(&mut jni::JNIEnv<'_>) -> anyhow::Result<T>) -> anyhow::Result<T> {
    let ctx = ndk_context::android_context();
    // SAFETY: ndk-context guarantees a valid JavaVM pointer for the running
    // app.
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }?;
    let mut env: AttachGuard<'_> = vm.attach_current_thread()?;
    match f(&mut env) {
        Ok(value) => Ok(value),
        Err(e) => {
            if env.exception_check().unwrap_or(false) {
                let message = describe_exception(&mut env);
                Err(anyhow::anyhow!("{e}: {message}"))
            } else {
                Err(e)
            }
        }
    }
}

/// Clears the pending Java exception and returns its `toString()`.
fn describe_exception(env: &mut jni::JNIEnv<'_>) -> String {
    let Ok(throwable) = env.exception_occurred() else {
        return "unknown java exception".into();
    };
    let _ = env.exception_clear();
    env.call_method(&throwable, "toString", "()Ljava/lang/String;", &[])
        .and_then(|v| v.l())
        .and_then(|s| env.get_string(&JString::from(s)).map(String::from))
        .unwrap_or_else(|_| "unknown java exception".into())
}

/// `Cipher.getInstance("AES/GCM/NoPadding")`.
fn cipher<'a>(env: &mut jni::JNIEnv<'a>) -> anyhow::Result<JObject<'a>> {
    let transformation = env.new_string(TRANSFORMATION)?;
    Ok(env
        .call_static_method(
            "javax/crypto/Cipher",
            "getInstance",
            "(Ljava/lang/String;)Ljavax/crypto/Cipher;",
            &[JValue::Object(&transformation)],
        )?
        .l()?)
}

/// The Keystore key under [`ALIAS`], generated on first use.
fn session_key<'a>(env: &mut jni::JNIEnv<'a>) -> anyhow::Result<JObject<'a>> {
    // KeyStore store = KeyStore.getInstance("AndroidKeyStore");
    let provider = env.new_string("AndroidKeyStore")?;
    let store = env
        .call_static_method(
            "java/security/KeyStore",
            "getInstance",
            "(Ljava/lang/String;)Ljava/security/KeyStore;",
            &[JValue::Object(&provider)],
        )?
        .l()?;
    // store.load(null);
    env.call_method(
        &store,
        "load",
        "(Ljava/security/KeyStore$LoadStoreParameter;)V",
        &[JValue::Object(&JObject::null())],
    )?;
    // Key key = store.getKey(ALIAS, null);
    let alias = env.new_string(ALIAS)?;
    let key = env
        .call_method(
            &store,
            "getKey",
            "(Ljava/lang/String;[C)Ljava/security/Key;",
            &[JValue::Object(&alias), JValue::Object(&JObject::null())],
        )?
        .l()?;
    if !key.is_null() {
        return Ok(key);
    }
    tracing::info!("android keystore: generating session key {ALIAS}");
    generate_key(env, &alias)
}

/// Generates the AES-256 GCM key under `alias` and returns it.
fn generate_key<'a>(env: &mut jni::JNIEnv<'a>, alias: &JString<'_>) -> anyhow::Result<JObject<'a>> {
    const BUILDER: &str = "android/security/keystore/KeyGenParameterSpec$Builder";
    const BUILDER_SIG: &str = "Landroid/security/keystore/KeyGenParameterSpec$Builder;";

    // KeyGenParameterSpec.Builder builder =
    //     new KeyGenParameterSpec.Builder(alias, PURPOSE_ENCRYPT | PURPOSE_DECRYPT);
    let builder = env.new_object(
        BUILDER,
        "(Ljava/lang/String;I)V",
        &[JValue::Object(alias), JValue::Int(PURPOSES)],
    )?;
    // builder.setBlockModes("GCM").setEncryptionPaddings("NoPadding").setKeySize(256);
    let gcm = string_array(env, "GCM")?;
    env.call_method(
        &builder,
        "setBlockModes",
        format!("([Ljava/lang/String;){BUILDER_SIG}"),
        &[JValue::Object(&gcm)],
    )?;
    let no_padding = string_array(env, "NoPadding")?;
    env.call_method(
        &builder,
        "setEncryptionPaddings",
        format!("([Ljava/lang/String;){BUILDER_SIG}"),
        &[JValue::Object(&no_padding)],
    )?;
    env.call_method(
        &builder,
        "setKeySize",
        format!("(I){BUILDER_SIG}"),
        &[JValue::Int(256)],
    )?;
    // KeyGenParameterSpec spec = builder.build();
    let spec = env
        .call_method(
            &builder,
            "build",
            "()Landroid/security/keystore/KeyGenParameterSpec;",
            &[],
        )?
        .l()?;
    // KeyGenerator generator = KeyGenerator.getInstance("AES", "AndroidKeyStore");
    let algorithm = env.new_string("AES")?;
    let provider = env.new_string("AndroidKeyStore")?;
    let generator = env
        .call_static_method(
            "javax/crypto/KeyGenerator",
            "getInstance",
            "(Ljava/lang/String;Ljava/lang/String;)Ljavax/crypto/KeyGenerator;",
            &[JValue::Object(&algorithm), JValue::Object(&provider)],
        )?
        .l()?;
    // generator.init(spec);
    env.call_method(
        &generator,
        "init",
        "(Ljava/security/spec/AlgorithmParameterSpec;)V",
        &[JValue::Object(&spec)],
    )?;
    // return generator.generateKey();
    Ok(env
        .call_method(&generator, "generateKey", "()Ljavax/crypto/SecretKey;", &[])?
        .l()?)
}

/// A one-element `String[]`, for the builder's varargs setters.
fn string_array<'a>(env: &mut jni::JNIEnv<'a>, value: &str) -> anyhow::Result<JObject<'a>> {
    let value = env.new_string(value)?;
    let array = env.new_object_array(1, "java/lang/String", &value)?;
    Ok(array.into())
}
