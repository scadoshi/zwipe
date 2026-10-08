//! Status colors mean an outcome. Decoration draws from the palette slots. Every
//! status reference in the app's stylesheets must be one of these selectors, and
//! the only Rust files that may write one into an inline style are the ones that
//! draw deck warnings.

use std::{fs, path::Path};
use zwipe_components::status_uses;

const ALLOWED: [&str; 11] = [
    ".alert-dialog-action-danger",
    ".badge-unverified",
    ".badge-verified",
    ".chip-bubble-error",
    ".clear-btn",
    ".input-error, .input-error:focus",
    ".message-error",
    ".message-success",
    ".stat-chip-bad",
    ".text-error",
    ".text-success",
];

const ALLOWED_RS: [&str; 2] = [
    "src/lib/inbound/screens/deck/components/collapsible_section.rs",
    "src/lib/inbound/screens/deck/components/deck_warnings.rs",
];

#[test]
fn status_colors_only_where_they_mean_something() {
    let sheets = [
        include_str!("../assets/main.css"),
        include_str!("../assets/alert-dialog.css"),
    ];
    let offenders: Vec<String> = sheets
        .iter()
        .flat_map(|css| status_uses(css))
        .filter(|u| !ALLOWED.contains(&u.selector.as_str()))
        .map(|u| format!("{} uses {}", u.selector, u.variable))
        .collect();
    assert!(
        offenders.is_empty(),
        "status colors used as decoration (move to a --palette-N slot, or add the selector to ALLOWED if it marks an outcome):\n  {}",
        offenders.join("\n  ")
    );
}

#[test]
fn inline_status_styles_only_in_the_warning_components() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut offenders = Vec::new();
    visit(&root.join("src"), &mut |path| {
        let Ok(text) = fs::read_to_string(path) else {
            return;
        };
        if status_uses(&text).is_empty() {
            return;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        if !ALLOWED_RS.contains(&rel.as_str()) {
            offenders.push(rel);
        }
    });
    assert!(
        offenders.is_empty(),
        "inline status colors outside the warning components (use a --palette-N slot, or add the file to ALLOWED_RS if it marks an outcome):\n  {}",
        offenders.join("\n  ")
    );
}

fn visit(dir: &Path, f: &mut dyn FnMut(&Path)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit(&path, f);
        } else if path.extension().is_some_and(|e| e == "rs") {
            f(&path);
        }
    }
}
