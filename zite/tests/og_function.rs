//! The share page's Pages Function (`functions/deck/[token].js`) names formats
//! and picks which ones lead with a commander from its own copies of
//! zwipe-core's lists. These keep the copies matched.

use zwipe_core::domain::deck::Format;

const FUNCTION: &str = include_str!("../functions/deck/[token].js");

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
