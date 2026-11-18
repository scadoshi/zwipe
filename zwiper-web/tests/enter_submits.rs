//! Enter in a field submits every form on the site, by one rule: the form's `onsubmit` runs the
//! screen's submit, `EnterSubmit` opens the form as its default button, ahead of any visible
//! button a screen holds disabled, and `novalidate` leaves the checks to the screen.

#![expect(
    clippy::string_slice,
    clippy::unwrap_used,
    reason = "a test: reads its own source tree and slices the text it has just searched"
)]

use std::fs;
use std::path::Path;

/// Every `.rs` file under `dir`, with its text.
fn sources(dir: &Path, out: &mut Vec<(String, String)>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push((
                path.display().to_string(),
                fs::read_to_string(&path).unwrap(),
            ));
        }
    }
}

#[test]
fn every_form_opens_with_the_enter_submit() {
    let mut files = Vec::new();
    sources(Path::new("src"), &mut files);
    let mut forms = 0;
    for (path, text) in &files {
        for (at, _) in text.match_indices("onsubmit: move |event| {") {
            forms += 1;
            let head = &text[text[..at].rfind("form {").unwrap()..at];
            assert!(
                head.contains("novalidate: true,"),
                "{path}: the screen's own checks run, not the browser's, which would block Enter on an invalid email: {head}"
            );
            let indent = &text[text[..at].rfind('\n').unwrap() + 1..at];
            let rest = &text[at..];
            let close = format!("\n{indent}}},\n");
            let after = rest
                .split_once(close.as_str())
                .map_or("", |(_, after)| after);
            assert!(
                after.trim_start().starts_with("EnterSubmit {}"),
                "{path}: a form's first child is EnterSubmit: {}",
                &after[..after.len().min(200)]
            );
        }
        assert!(
            !text.contains("r#type: \"submit\"") || path.ends_with("enter_submit.rs"),
            "{path}: the one submit button is EnterSubmit"
        );
    }
    assert!(
        forms == 8,
        "the deck form, Import, the three sign-in screens and the account sheets: {forms}"
    );
}

#[test]
fn the_archidekt_link_sits_in_a_form() {
    let import = include_str!("../src/lib/inbound/screens/deck/import.rs");
    let form = import.split_once("form {").map_or("", |(_, rest)| rest);
    let body = form
        .split_once("div { class: \"web-import-results\"")
        .map_or("", |(body, _)| body);
    assert!(
        body.contains("id: \"import-url\""),
        "the link field is in the form: {body}"
    );
    assert!(
        body.contains("if !disabled {\n"),
        "an empty link sends nothing: {body}"
    );
}
