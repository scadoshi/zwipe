//! Every weight a sheet names is a face zwipe ships: `JetBrains Mono` in 400, 500 and 700. A browser
//! draws 600 with the 700 face, so a rule says 700 when it means bold.

use std::fs;
use std::path::{Path, PathBuf};

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

#[test]
fn no_sheet_asks_for_a_weight_that_draws_as_another() {
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
        let css = fs::read_to_string(&path).unwrap_or_default();
        for (number, line) in css.lines().enumerate() {
            let squeezed: String = line.split_whitespace().collect();
            assert!(
                !squeezed.contains("font-weight:600"),
                "{}:{}: 600 draws with the 700 face; say 700",
                path.display(),
                number.saturating_add(1)
            );
        }
    }
}
