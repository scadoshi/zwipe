//! A full-bleed rule runs out through its container's side padding to the border and no further:
//! its negative side margin names the container's padding token, so the two move together, and no
//! markup moves a rule with an inline margin of its own.

#![expect(
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    reason = "a test: walks CSS text by byte offsets it finds itself and reads the repo's files"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// The containers' side padding a full-bleed rule may pull out by: the padding tokens, and a card
/// proxy's padding, which scales with the proxy in container units.
const PADDING: [&str; 3] = [
    "calc(-1 * var(--panel-pad))",
    "calc(-1 * var(--card-row-detail-pad-x))",
    "-6cqw",
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

/// The space-separated parts of a shorthand, a parenthesized group kept whole.
fn parts(shorthand: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0_u32;
    let mut part = String::new();
    for c in shorthand.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if c == ' ' && depth == 0 {
            if !part.is_empty() {
                out.push(std::mem::take(&mut part));
            }
        } else {
            part.push(c);
        }
    }
    if !part.is_empty() {
        out.push(part);
    }
    out
}

/// Every side margin `body` sets: the shorthand's sides, the longhands and `margin-inline`.
fn side_margins(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(margin) = value(body, "margin") {
        let parts = parts(&margin);
        match parts.as_slice() {
            [all] => out.push(all.clone()),
            [_, x] | [_, x, _] => out.push(x.clone()),
            [_, right, _, left] => out.extend([right.clone(), left.clone()]),
            _ => {}
        }
    }
    if let Some(inline) = value(body, "margin-inline") {
        out.extend(parts(&inline));
    }
    for property in ["margin-left", "margin-right"] {
        out.extend(value(body, property));
    }
    out
}

/// Whether `selector` styles a rule: its last compound names a class ending in `-rule`, or `hr`.
fn styles_a_rule(selector: &str) -> bool {
    selector.split(',').any(|one| {
        let last = one.split([' ', '>', '+', '~']).next_back().unwrap_or("");
        last == "hr"
            || last
                .split('.')
                .skip(1)
                .any(|class| class.split(':').next().unwrap_or("").ends_with("-rule"))
    })
}

#[test]
fn a_full_bleed_rule_pulls_out_by_its_container_s_padding() {
    let mut off = Vec::new();
    for dir in [
        "components/assets",
        "zwiper/assets",
        "zwiper-web/assets",
        "zite/assets",
    ] {
        let mut sheets = Vec::new();
        files(&root().join(dir), "css", &mut sheets);
        for path in sheets {
            let css = fs::read_to_string(&path).unwrap();
            for (selector, body) in rules(&css) {
                if !styles_a_rule(&selector) {
                    continue;
                }
                for side in side_margins(&body) {
                    if (side.starts_with('-') || side.starts_with("calc(-1 *"))
                        && !PADDING.contains(&side.as_str())
                    {
                        let file = path.strip_prefix(root()).unwrap().display().to_string();
                        off.push(format!("{file}: {selector} {{ margin: {side} }}"));
                    }
                }
            }
        }
    }
    assert!(
        off.is_empty(),
        "full-bleed rules off the padding:\n{}",
        off.join("\n")
    );
}

#[test]
fn no_rule_sets_its_margin_inline() {
    let mut inline = Vec::new();
    let mut sources = Vec::new();
    for dir in ["components/src", "zwiper/src", "zwiper-web/src", "zite/src"] {
        files(&root().join(dir), "rs", &mut sources);
    }
    for path in sources {
        let src = fs::read_to_string(&path).unwrap();
        for line in src.lines() {
            let line = line.trim();
            if line.starts_with("hr {") && line.contains("style:") && line.contains("margin") {
                let file = path.strip_prefix(root()).unwrap().display().to_string();
                inline.push(format!("{file}: {line}"));
            }
        }
    }
    assert!(
        inline.is_empty(),
        "rules with an inline margin:\n{}",
        inline.join("\n")
    );
}

#[test]
fn the_kit_s_panel_rule_runs_to_the_panel_s_border() {
    let kit =
        fs::read_to_string(root().join("zwiper-web/assets/sheets/03-components.css")).unwrap();
    let rules = rules(&kit);
    let body = |selector: &str| {
        rules
            .iter()
            .find(|(head, _)| head == selector)
            .map(|(_, body)| body.clone())
            .unwrap()
    };
    assert_eq!(
        value(&body(".panel-card"), "padding").as_deref(),
        Some("var(--panel-pad)")
    );
    assert_eq!(
        value(&body(".panel-rule"), "margin").as_deref(),
        Some("var(--space-4) calc(-1 * var(--panel-pad))")
    );
}
