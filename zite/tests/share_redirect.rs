//! A shared deck lives on the web app: zwipe.net's owner and slug link answers with a permanent
//! redirect there, so a pasted zwipe.net link keeps opening. The bare token form has no page.

use std::path::Path;

const OWNER_FORM: &str = include_str!("../functions/[owner]/deck/[slug]/[token].js");

#[test]
fn an_owner_and_slug_link_moves_to_the_same_path_on_the_web_app_for_good() {
    assert!(
        OWNER_FORM.contains("const APP = \"https://app.zwipe.net\";"),
        "{OWNER_FORM}"
    );
    assert!(
        OWNER_FORM.contains("Response.redirect(`${APP}${url.pathname}${url.search}`, 301)"),
        "{OWNER_FORM}"
    );
    assert!(
        OWNER_FORM.contains("export async function onRequest("),
        "every method redirects, HEAD included"
    );
}

#[test]
fn the_bare_token_link_has_no_function() {
    let bare = Path::new(env!("CARGO_MANIFEST_DIR")).join("functions/deck");
    assert!(!bare.exists(), "{} is gone", bare.display());
    assert!(
        !OWNER_FORM.contains("PLACEHOLDER") && !OWNER_FORM.contains("params.token"),
        "no bare token form left in the owner form: {OWNER_FORM}"
    );
}
