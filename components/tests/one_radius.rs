//! One radius across the app, the web app and zwipe.net: every container rounds at the kit's
//! panel radius and every chip and button at its chip radius. A box in a container's corner,
//! nearer than the padding, takes the container's radius less the border and its inset, so the
//! two curves match. No sheet and no inline style sets a radius of its own, except the shapes that
//! are no container, chip or button: card art and thumbnails, circles, bars and tracks, mana pips,
//! focus rings and scrollbars.

#![expect(
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    reason = "a test: walks CSS text by byte offsets it finds itself and reads the repo's files"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// The radii a rule may set: the panel's, a chip's, a container's top or bottom corners where it
/// joins another, and `inherit` and `0` for a box that takes its parent's corner or sits flush.
const ALLOWED: [&str; 6] = [
    "var(--panel-radius)",
    "var(--chip-radius)",
    "var(--panel-radius) var(--panel-radius) 0 0",
    "0 0 var(--panel-radius) var(--panel-radius)",
    "inherit",
    "0",
];

/// Whether `radius` is the nested formula: a container's radius token less the 1px border and an
/// inset, floored so it never goes square.
fn nested(radius: &str) -> bool {
    radius
        .strip_prefix("max(0.15rem, calc(var(--")
        .and_then(|rest| rest.split_once(") - 1px - "))
        .is_some_and(|(outer, inset)| {
            ["panel-radius", "dock-radius"].contains(&outer) && inset.ends_with("))")
        })
}

/// Rules whose shape is no container, chip or button, each keeping its own radius.
const OUT_OF_SCOPE: &[&str] = &[
    // Card art: a printed card's own corner, and the card images and thumbnails.
    ".hover-stack-card img",
    ".card-proxy",
    ".cover-flow-card::after",
    ".cover-flow-card img",
    ".cover-flow-ghost",
    ".deck-head-card .deck-head-card-image img",
    ".card-stack-card",
    ".skeleton-bar.deck-stacks-ghost-card",
    ".deck-featured-image",
    ".deck-featured-card .deck-featured-image img",
    ".skeleton-bar.web-ghost-card",
    ".sd-image-overlay .sd-image-overlay-img img",
    ".flippable-card-wrapper img",
    ".home-deck-tile-art",
    ".carousel-card-image",
    ".card-image",
    ".skeleton-image",
    ".card-list-results .skeleton-image",
    ".card-row-art",
    ".card-proxy.card-row-art",
    ".skeleton-card-thumb",
    // A card proxy's chips scale with the proxy, in em.
    ".card-proxy .detail-chip, .card-proxy .stat-chip, .card-proxy .chip, .card-proxy .card-row-price",
    // Circles and pills.
    ".filter-dot",
    ".spinner",
    ".rounded-full",
    ".skeleton-bar.web-ghost-pip",
    // Bars and tracks.
    ".chart-track",
    ".chart-fill",
    ".skeleton-bar.chart-track",
    ".data-bar",
    ".data-bar-fill",
    ".skeleton-bar.data-bar-ghost",
    ".color-bar-track",
    ".skeleton-bar.color-bar-track",
    ".color-bar-pip-ghost, .skeleton-bar.color-bar-pip-ghost",
    ".activity-ghost-bar",
    ".skeleton-bar",
    ".skeleton-export-textarea",
    // Mana pips.
    "i.ms.ms-cost",
    ".identity-pips i.ms.ms-cost",
    ".card-row-colors i.ms.ms-cost",
    // Inline code in zwipe.net's prose.
    "code",
    // Focus rings and scrollbars.
    ".chip-remove:focus-visible",
    ".card-row-compact:focus-visible::after",
    ".card-group-header:focus-visible, .card-group-toggle:focus-visible, .deck-fold-toggle:focus-visible",
    ":is(.privacy-content a, .privacy-content .privacy-link, .about-credits a):focus-visible",
    "*::-webkit-scrollbar-thumb",
    // The store icon tile.
    ".app-icon-tile",
];

/// The radii an inline style may set: the two tokens, and the deck charts' bars, rounded on top.
const INLINE_ALLOWED: [&str; 3] = [
    "border-radius:var(--panel-radius)",
    "border-radius:var(--chip-radius)",
    "border-radius:0.15rem 0.15rem 0 0",
];

/// The workspace root.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Every file under `dir` with extension `ext`, skipping generated sheets and build output.
fn files(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if path.is_dir() {
            if !matches!(
                name.as_str(),
                "target" | "sheets" | "public" | "dist" | "tests"
            ) {
                files(&path, ext, out);
            }
        } else if path.extension().is_some_and(|e| e == ext) {
            out.push(path);
        }
    }
}

/// `css` without its comments, the same length so offsets still line up.
fn without_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find("*/")
            .map_or(rest.len(), |e| start + e + 2);
        out.extend(
            rest[start..end]
                .chars()
                .map(|c| if c == '\n' { '\n' } else { ' ' }),
        );
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// Every style rule in `css` as its selector and its declarations, at-rules looked inside.
fn rules(css: &str) -> Vec<(String, String)> {
    let css = without_comments(css);
    let mut out = Vec::new();
    let mut heads: Vec<String> = Vec::new();
    let mut start = 0;
    for (at, c) in css.char_indices() {
        match c {
            '{' => {
                heads.push(css[start..at].trim().to_owned());
                start = at + 1;
            }
            '}' => {
                if let Some(head) = heads.pop()
                    && !head.starts_with('@')
                {
                    let head = head.split_whitespace().collect::<Vec<_>>().join(" ");
                    out.push((head, css[start..at].to_owned()));
                }
                start = at + 1;
            }
            _ => {}
        }
    }
    out
}

/// The value `body` gives `property`, if it sets it.
fn value(body: &str, property: &str) -> Option<String> {
    body.split(';').find_map(|decl| {
        let (name, value) = decl.split_once(':')?;
        (name.trim() == property).then(|| value.split_whitespace().collect::<Vec<_>>().join(" "))
    })
}

/// Every zwipe sheet: the shared components', the app's, the web app's and zwipe.net's.
fn sheets() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in [
        "components/assets",
        "zwiper/assets",
        "zwiper-web/assets",
        "zite/assets",
    ] {
        files(&root().join(dir), "css", &mut out);
    }
    out.sort();
    out
}

#[test]
fn no_sheet_sets_a_radius_of_its_own() {
    let mut own = Vec::new();
    for path in sheets() {
        let css = fs::read_to_string(&path).unwrap();
        for (selector, body) in rules(&css) {
            let Some(radius) = value(&body, "border-radius") else {
                continue;
            };
            if !ALLOWED.contains(&radius.as_str())
                && !nested(&radius)
                && !OUT_OF_SCOPE.contains(&selector.as_str())
            {
                let file = path.strip_prefix(root()).unwrap().display().to_string();
                own.push(format!("{file}: {selector} {{ border-radius: {radius} }}"));
            }
        }
    }
    assert!(
        own.is_empty(),
        "radii outside the rule:\n{}",
        own.join("\n")
    );
}

#[test]
fn no_inline_style_sets_a_radius_of_its_own() {
    let mut own = Vec::new();
    let mut sources = Vec::new();
    for dir in ["components/src", "zwiper/src", "zwiper-web/src", "zite/src"] {
        files(&root().join(dir), "rs", &mut sources);
    }
    for path in sources {
        let src = fs::read_to_string(&path).unwrap();
        for (at, _) in src.match_indices("border-radius:") {
            let style: String = src[at..]
                .chars()
                .take_while(|c| *c != ';' && *c != '"')
                .collect();
            if !INLINE_ALLOWED.contains(&style.as_str()) {
                let file = path.strip_prefix(root()).unwrap().display().to_string();
                own.push(format!("{file}: {style}"));
            }
        }
    }
    assert!(
        own.is_empty(),
        "inline radii outside the rule:\n{}",
        own.join("\n")
    );
}

#[test]
fn no_sheet_names_the_kit_s_old_corner_tokens() {
    for path in sheets() {
        let css = fs::read_to_string(&path).unwrap();
        for token in [
            "--radius-readonly",
            "--radius-control",
            "--corner-radius",
            "--deck-row-radius",
        ] {
            assert!(
                !css.contains(token),
                "{} still names {token}",
                path.display()
            );
        }
    }
}
