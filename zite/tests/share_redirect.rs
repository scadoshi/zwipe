//! A shared deck lives on the web app: zwipe.net's shared deck links answer with a permanent
//! redirect there, so links phones built before the move still make keep opening.

const FUNCTION: &str = include_str!("../functions/deck/[token].js");
const OWNER_FORM: &str = include_str!("../functions/[owner]/deck/[slug]/[token].js");

#[test]
fn every_shared_deck_link_moves_to_the_web_app_for_good() {
    assert!(
        FUNCTION.contains("const APP = \"https://app.zwipe.net\";"),
        "{FUNCTION}"
    );
    assert!(FUNCTION.contains("Response.redirect(`${APP}${path}${url.search}`, 301)"));
    assert!(
        OWNER_FORM.contains(r#"export { onRequestGet } from "../../../deck/[token].js";"#),
        "{OWNER_FORM}"
    );
}
