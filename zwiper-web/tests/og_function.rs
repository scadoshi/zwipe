//! The shared deck link's Pages Function (`functions/[owner]/deck/[slug]/[token].js`) names formats,
//! picks which ones lead with a commander and versions the preview image from
//! its own copies of zwipe-core's lists and constants. These keep the copies
//! matched.

use zwipe_core::domain::deck::Format;

const FUNCTION: &str = include_str!("../functions/[owner]/deck/[slug]/[token].js");

/// The text between `start` and the next `end` after it.
fn block(start: &str, end: &str) -> Option<&'static str> {
    let (_, rest) = FUNCTION.split_once(start)?;
    let (inner, _) = rest.split_once(end)?;
    Some(inner)
}

#[test]
fn every_format_has_its_display_name() {
    let names = block("const FORMAT_NAMES = {", "};").expect("FORMAT_NAMES in the function");
    for format in Format::all() {
        let line = format!(
            "  {}: \"{}\",\n",
            format.to_legality_key(),
            format.display_name()
        );
        assert!(names.contains(&line), "FORMAT_NAMES lacks {line:?}");
    }
    assert_eq!(
        names.lines().filter(|l| l.contains(':')).count(),
        Format::all().len(),
        "FORMAT_NAMES holds a format zwipe-core doesn't"
    );
}

#[test]
fn the_commander_formats_match() {
    let set = block("const COMMANDER_FORMATS = new Set([", "]);")
        .expect("COMMANDER_FORMATS in the function");
    for format in Format::all() {
        let quoted = format!("\"{}\"", format.to_legality_key());
        assert_eq!(
            set.contains(&quoted),
            format.has_commander(),
            "COMMANDER_FORMATS disagrees on {quoted}"
        );
    }
}

#[test]
fn the_preview_image_path_matches_the_api() {
    let token = uuid::Uuid::nil();
    let route = zwipe_core::http::paths::shared_deck_preview_route(token);
    let in_function = route.replace(&token.to_string(), "${token}");
    assert!(
        FUNCTION.contains(&format!("`${{apiBase}}{in_function}?v=")),
        "the Function's preview URL differs from {route}"
    );
}

#[test]
fn the_preview_version_carries_the_layout() {
    let layout = zwipe_core::http::paths::SHARE_PREVIEW_LAYOUT;
    assert!(
        FUNCTION.contains(&format!("\nconst PREVIEW_LAYOUT = {layout};\n")),
        "the Function's PREVIEW_LAYOUT differs from zwipe-core's {layout}"
    );
    assert!(
        FUNCTION.contains("preview.png?v=${PREVIEW_LAYOUT}-${Math.floor(changed / 1000)}`"),
        "the Function's preview URL leaves the layout out of v"
    );
}

#[test]
fn the_canonical_link_is_the_app_s_owner_form() {
    assert!(FUNCTION.contains("const SITE = \"https://app.zwipe.net\";"));
    assert!(
        FUNCTION.contains(
            "`${SITE}/${encodeURIComponent(owner)}/deck/${deckSlug(deck.name)}/${lower}`"
        ),
        "the canonical link is the one shared deck shape"
    );
    assert!(!FUNCTION.contains("${SITE}/deck/"), "no bare token link");
}

#[test]
fn the_slug_cap_matches_zwipe_core() {
    let line = format!(
        "const SLUG_MAX = {};",
        zwipe_core::domain::share_link::SLUG_MAX
    );
    assert!(FUNCTION.contains(&line), "the function lacks {line:?}");
}
