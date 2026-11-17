//! The app's writes run as tasks of the screen that makes them (`spawn`, or `spawn_in_order` for
//! one card's writes in order), never in the root: a root task from `spawn_write` or
//! `spawn_write_in_order` never reaches the server on the phone, so a swipe that saves or
//! dismisses through one is lost with no error.

use std::fs;
use std::path::Path;

/// The root spawns no app file may call.
const ROOT_SPAWNS: [&str; 2] = ["spawn_write(", "spawn_write_in_order("];

#[test]
fn no_app_write_runs_in_the_root() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut offenders = Vec::new();
    visit(&root.join("src"), &mut |path| {
        let Ok(text) = fs::read_to_string(path) else {
            return;
        };
        for (number, line) in text.lines().enumerate() {
            if ROOT_SPAWNS.iter().any(|spawn| line.contains(spawn)) {
                let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
                offenders.push(format!("{rel}:{}", number.saturating_add(1)));
            }
        }
    });
    assert!(
        offenders.is_empty(),
        "writes spawned in the root (use spawn or spawn_in_order):\n  {}",
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
