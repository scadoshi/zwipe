const BACKEND_URL_KEY: &str = "BACKEND_URL";

fn main() {
    dotenvy::dotenv().expect("failed to load .env");
    let backend_url = std::env::var(BACKEND_URL_KEY)
        .expect(format!("{} must be set in .env file", BACKEND_URL_KEY).as_str());
    println!("cargo:rustc-env={}={}", BACKEND_URL_KEY, backend_url);
    println!(
        "cargo:warning=setting {} to {}",
        BACKEND_URL_KEY, backend_url
    );
    println!("cargo:rerun-if-changed=.env");
}
