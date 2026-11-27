//! zwipe's sheets take their spacing from orangui's scale, `--space-1` to `--space-7` (0.25 to 3rem
//! on a 4px base): a padding, margin or gap names a step by its token. Off the scale on purpose:
//! hairlines (0.15rem and under), `em` padding that follows its own text, and the listed lengths
//! that make room for something rather than space things apart or follow a size that isn't
//! spacing. This holds for a value built in `calc()` or `var()` too, for the custom properties
//! that carry a gap into one, and for a padding, margin or gap set inline in the Rust code.

use std::fs;
use std::path::{Path, PathBuf};

/// Lengths off the scale on purpose: (the sheet's path from the workspace root, the
/// declaration's property, the length, why).
const ALLOWED: [(&str, &str, &str, &str); 18] = [
    (
        "components/assets/screens/base.css",
        "--content-pad-bottom",
        "5rem",
        "a screen's room for its dock under the content",
    ),
    (
        "components/assets/screens/screens-b.css",
        "--content-pad-bottom",
        "5rem",
        "a picker's room for its dock under the chips",
    ),
    (
        "zwiper-web/assets/web.css",
        "--content-pad-bottom",
        "5rem",
        "a screen's room for its dock under the content",
    ),
    (
        "zwiper-web/assets/web.css",
        "--content-pad-bottom",
        "8.6rem",
        "a screen's room for its two stacked docks under the content",
    ),
    (
        "components/assets/magic.css",
        "margin-top",
        "0.85rem",
        "the color count ghost centers on a line of the count's 0.85rem text",
    ),
    (
        "components/assets/magic.css",
        "margin-top",
        "0.65rem",
        "the color count ghost's own 0.65rem height",
    ),
    (
        "components/assets/magic.css",
        "margin-bottom",
        "0.85rem",
        "the color count ghost centers on a line of the count's 0.85rem text",
    ),
    (
        "components/assets/magic.css",
        "margin-bottom",
        "0.65rem",
        "the color count ghost's own 0.65rem height",
    ),
    (
        "components/assets/magic.css",
        "margin-bottom",
        "0.375rem",
        "a phone's last card action row: the action reach's inset, so the reach stays inside a clipping row",
    ),
    (
        "components/assets/deck_page.css",
        "padding-bottom",
        "5rem",
        "the deck page's room for its dock",
    ),
    (
        "zwiper-web/assets/web.css",
        "padding",
        "5rem",
        "the card page's room for its dock under the columns",
    ),
    (
        "components/assets/magic.css",
        "padding-inline",
        "3.6rem",
        "Printings' title bar's room for its close button on both sides",
    ),
    (
        "components/assets/screens/base.css",
        "margin-bottom",
        "2.75rem",
        "a stacked toast's overlap with the one before, as the kit's",
    ),
    (
        "zwiper-web/assets/web.css",
        "padding-bottom",
        "5rem",
        "a screen's room for its dock",
    ),
    (
        "zwiper-web/assets/web.css",
        "padding-bottom",
        "8.6rem",
        "a screen's room for its two stacked docks",
    ),
    (
        "zwiper-web/assets/web.css",
        "padding",
        "6rem",
        "a screen's room under its last panel for the dock",
    ),
    (
        "zwiper-web/assets/web.css",
        "padding-top",
        "2.75rem",
        "the swipe card's room for the confirmation chip over the stack",
    ),
    (
        "zwiper/assets/main.css",
        "padding",
        "6rem",
        "a screen's room under its last panel for the dock",
    ),
];

/// The custom properties that carry a gap into a rule.
const SPACING_TOKENS: [&str; 10] = [
    "--drawer-float-gap",
    "--deck-list-gap",
    "--deck-row-inset",
    "--flow-step-inset",
    "--content-pad-bottom",
    "--heat-gutter",
    "--row-gap",
    "--pair-gap",
    "--web-sticky-top",
    "--stack-head-inset",
];

/// Every `.css` file under `dir`.
fn sheets(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for path in entries.filter_map(Result::ok).map(|entry| entry.path()) {
        if path.is_dir() {
            sheets(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "css") {
            out.push(path);
        }
    }
}

/// Every spacing declaration in `css`: its property and its value.
fn declarations(css: &str) -> Vec<(&str, &str)> {
    css.split(';')
        .filter_map(|chunk| {
            let line = chunk
                .rsplit(['{', '}', '\n'])
                .next()
                .unwrap_or_default()
                .trim();
            let (property, value) = line.split_once(':')?;
            let property = property.trim();
            let value = value.trim();
            let spacing = ["padding", "margin", "gap", "row-gap", "column-gap"]
                .iter()
                .any(|p| property == *p || property.starts_with(&format!("{p}-")))
                || SPACING_TOKENS.contains(&property);
            spacing.then_some((property, value))
        })
        .collect()
}

/// Every rem length written out in `value`, without its sign: a number right before `rem`.
fn rem_lengths(value: &str) -> Vec<String> {
    value
        .match_indices("rem")
        .filter_map(|(at, _)| {
            let before = value.get(..at)?;
            let start = before
                .rfind(|c: char| !(c.is_ascii_digit() || c == '.'))
                .map_or(0, |i| i.saturating_add(1));
            let number = before.get(start..)?;
            (!number.is_empty()).then(|| format!("{number}rem"))
        })
        .collect()
}

/// Every `.rs` file under `dir`.
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for path in entries.filter_map(Result::ok).map(|entry| entry.path()) {
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn every_spacing_length_names_a_step_or_is_listed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut found = Vec::new();
    for dir in [
        "components/assets",
        "zite/assets",
        "zwiper/assets",
        "zwiper-web/assets",
    ] {
        sheets(&root.join(dir), &mut found);
    }
    assert!(found.len() > 10, "the sheets were found: {found:?}");
    for path in found {
        let name = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if name.contains("/sheets/") || name.ends_with("fonts.css") {
            continue;
        }
        let css = fs::read_to_string(&path).unwrap_or_default();
        for (property, value) in declarations(&css) {
            for length in rem_lengths(value) {
                let rem: f64 = length.trim_end_matches("rem").parse().unwrap_or(f64::NAN);
                let allowed = ALLOWED
                    .iter()
                    .any(|(f, p, l, _)| *f == name && *p == property && *l == length);
                assert!(
                    rem <= 0.15 || allowed,
                    "{name}: `{property}: {value}` writes {length}; name a step (var(--space-N)) or list it with a reason"
                );
            }
        }
    }
}

#[test]
fn the_stacks_column_gap_is_the_one_the_columns_are_counted_with() {
    let css = include_str!("../assets/deck_page.css");
    for selector in [".deck-stacks", ".deck-stacks.deck-stacks-columns"] {
        let body = css
            .split_once(&format!("\n{selector} {{\n"))
            .and_then(|(_, rest)| rest.split_once("\n}"))
            .map_or("", |(body, _)| body);
        assert!(
            body.contains("var(--space-4);") && body.contains("gap:"),
            "{selector} spaces its columns by --space-4: {body:?}"
        );
    }
    assert_eq!(
        zwipe_ui::deck_cards::columns::STACK_GAP_CENTIREM,
        100,
        "--space-4 is 1rem, a hundred hundredths"
    );
}

#[test]
fn every_inline_spacing_length_in_the_rust_names_a_step() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut found = Vec::new();
    for dir in ["components/src", "zwiper-web/src", "zwiper/src", "zite/src"] {
        sources(&root.join(dir), &mut found);
    }
    assert!(found.len() > 50, "the sources were found");
    for path in found {
        let code = fs::read_to_string(&path).unwrap_or_default();
        for literal in code.split('"').skip(1).step_by(2) {
            for (property, value) in declarations(literal) {
                for length in rem_lengths(value) {
                    let rem: f64 = length.trim_end_matches("rem").parse().unwrap_or(f64::NAN);
                    assert!(
                        rem <= 0.15,
                        "{}: inline `{property}: {value}` writes {length}; name a step (var(--space-N))",
                        path.display()
                    );
                }
            }
        }
    }
}

/// The kit dock's height as zwipe would write it to keep room for the dock.
const DOCK: &str = "2.4rem + 2 * var(--space-1) + 2px";

#[test]
fn the_floating_drawer_keeps_no_room_for_the_docks() {
    let filter = include_str!("../assets/filter.css");
    let web = include_str!("../../zwiper-web/assets/web.css");
    for (sheet, css) in [("filter.css", filter), ("web.css", web)] {
        let drawer_bottoms = css
            .split(".bottom-sheet.drawer-every-width {")
            .skip(1)
            .filter_map(|rule| rule.split('}').next())
            .flat_map(str::lines)
            .filter(|line| line.trim().starts_with("bottom:"));
        for bottom in drawer_bottoms {
            assert!(
                !bottom.contains(DOCK),
                "{sheet}: the drawer runs over the docks to the bottom edge: {bottom}"
            );
        }
    }
}
