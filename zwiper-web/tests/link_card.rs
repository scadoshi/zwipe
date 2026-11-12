//! app.zwipe.net's head names its link card for Open Graph and for X alike,
//! so an unfurl on either side shows the same image and alt text.

const INDEX: &str = include_str!("../index.html");

/// The `content` of the meta tag whose `attr` is `key`, if the head has one.
fn meta(attr: &str, key: &str) -> Option<&'static str> {
    let open = format!(r#"<meta {attr}="{key}" content=""#);
    let (_, rest) = INDEX.split_once(&open)?;
    let (content, _) = rest.split_once('"')?;
    Some(content)
}

#[test]
fn x_s_card_matches_open_graph_s_image_and_alt() {
    let image = meta("property", "og:image").expect("og:image");
    let alt = meta("property", "og:image:alt").expect("og:image:alt");
    assert_eq!(image, "https://app.zwipe.net/assets/og-app.png");
    assert_eq!(meta("name", "twitter:card"), Some("summary_large_image"));
    assert_eq!(meta("name", "twitter:image"), Some(image));
    assert_eq!(meta("name", "twitter:image:alt"), Some(alt));
}
