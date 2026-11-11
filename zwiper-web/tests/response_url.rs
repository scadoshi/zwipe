//! Every fetch `Response` reports a URL. A browser leaves `url` empty on a
//! `Response` that script built (a fetch-wrapping extension, a browser devtools
//! override), and the HTTP client parses it on every reply, so an empty one
//! would throw inside the wasm and end the call. `public/response-url.js`
//! gives such a response the page's URL instead, and it has to run before
//! the app makes its first request.

const INDEX_HTML: &str = include_str!("../index.html");
const SCRIPT: &str = include_str!("../public/response-url.js");

#[test]
fn the_page_loads_the_response_url_script_before_any_other() {
    let first = INDEX_HTML
        .find("<script")
        .unwrap_or_else(|| panic!("index.html has no script"));
    let ours = INDEX_HTML
        .find(r#"<script src="/response-url.js"></script>"#)
        .unwrap_or_else(|| panic!("index.html does not load /response-url.js"));
    assert_eq!(ours, first, "/response-url.js loads first");
    assert!(
        ours < INDEX_HTML.find("</head>").unwrap_or(usize::MAX),
        "in the head"
    );
}

#[test]
fn an_empty_response_url_falls_back_to_the_page() {
    assert!(SCRIPT.contains("Response.prototype"), "{SCRIPT}");
    assert!(SCRIPT.contains(r#""url""#), "{SCRIPT}");
    assert!(
        SCRIPT.contains("|| location.href"),
        "the empty url's fallback: {SCRIPT}"
    );
}
