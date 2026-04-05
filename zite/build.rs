#![allow(clippy::expect_used)]

fn main() {
    // Copy shared themes into assets so asset!() can find them
    std::fs::copy("../shared/themes.css", "assets/themes.css")
        .expect("failed to copy shared/themes.css into zite/assets/");
    println!("cargo:rerun-if-changed=../shared/themes.css");
}
