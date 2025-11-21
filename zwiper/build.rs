#![allow(clippy::expect_used, clippy::panic)]

const BACKEND_URL_KEY: &str = "BACKEND_URL";
const RUST_LOG_KEY: &str = "RUST_LOG";
const RUST_BACKTRACE_KEY: &str = "RUST_BACKTRACE";

fn main() {
    dotenvy::dotenv().expect("failed to load .env");

    let backend_url = std::env::var(BACKEND_URL_KEY)
        .unwrap_or_else(|_| panic!("{} must be set in .env file", BACKEND_URL_KEY));
    let rust_log = std::env::var(RUST_LOG_KEY)
        .unwrap_or_else(|_| panic!("{} must be set in .env file", RUST_LOG_KEY));
    let rust_backtrace = std::env::var(RUST_BACKTRACE_KEY)
        .unwrap_or_else(|_| panic!("{} must be set in .env file", RUST_BACKTRACE_KEY));

    println!("cargo:rustc-env={}={}", BACKEND_URL_KEY, backend_url);
    println!("cargo:info=setting {} to {}", BACKEND_URL_KEY, backend_url);

    println!("cargo:rustc-env={}={}", RUST_LOG_KEY, rust_log);
    println!("cargo:info=setting {} to {}", RUST_LOG_KEY, rust_log);

    println!("cargo:rustc-env={}={}", RUST_BACKTRACE_KEY, rust_backtrace);
    println!(
        "cargo:info=setting {} to {}",
        RUST_BACKTRACE_KEY, rust_backtrace
    );

    println!("cargo:rerun-if-changed=.env");
}
