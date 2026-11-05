//! Bakes `BACKEND_URL` into the build from the process env or this crate's
//! `.env`, failing the build on a malformed one. Unset, the build keeps the
//! site default in `config.rs`.

#![expect(
    clippy::panic,
    reason = "a build script fails the build by panicking on a bad variable"
)]

const BACKEND_URL_KEY: &str = "BACKEND_URL";

fn main() {
    // No .env is fine: the variable may come from the process env, or not at all.
    dotenvy::dotenv().ok();

    if let Ok(backend_url) = std::env::var(BACKEND_URL_KEY) {
        let rest = backend_url
            .strip_prefix("https://")
            .or_else(|| backend_url.strip_prefix("http://"))
            .unwrap_or_else(|| {
                panic!("{BACKEND_URL_KEY} must start with http:// or https://, got {backend_url:?}")
            });
        assert!(
            !rest.is_empty() && !rest.starts_with('/'),
            "{BACKEND_URL_KEY} has no host: {backend_url:?}"
        );
        // A release build sends bearer tokens to this URL from every browser.
        let release = std::env::var("PROFILE").is_ok_and(|p| p == "release");
        assert!(
            !release || backend_url.starts_with("https://"),
            "{BACKEND_URL_KEY} must be https:// in a release build, got {backend_url:?}"
        );
        println!("cargo:rustc-env={BACKEND_URL_KEY}={backend_url}");
        println!("cargo:info=setting {BACKEND_URL_KEY} to {backend_url}");
    }

    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-env-changed={BACKEND_URL_KEY}");
}
