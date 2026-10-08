//! Finds every status-color reference in a stylesheet, with the selector it
//! sits under, so a consumer's test can hold them to an allowlist. The status
//! variables (`--color-success`, `--color-warning`, `--color-error` and their
//! `--border-*` twins) mean an outcome; decoration draws from `--palette-N`
//! instead. See `context/plans/palette.md`.

/// One reference to a status variable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusUse {
    /// The rule's selector, whitespace collapsed, comments removed.
    pub selector: String,
    /// The variable, e.g. `--color-error`.
    pub variable: String,
}

const STATUS_VARS: [&str; 6] = [
    "--color-success",
    "--color-warning",
    "--color-error",
    "--border-success",
    "--border-warning",
    "--border-error",
];

/// Every `var(--status)` reference in `css`, in order, each with the selector of
/// the rule it appears in. A reference inside an `@media` block reports the
/// rule's own selector, not the query.
pub fn status_uses(css: &str) -> Vec<StatusUse> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = css.get(from..).and_then(|rest| rest.find("var(--")) {
        let at = from + rel;
        let Some(tail) = css.get(at + 4..) else { break };
        let Some(close) = tail.find(')') else { break };
        let Some(name) = tail.get(..close) else { break };
        if STATUS_VARS.contains(&name) {
            out.push(StatusUse {
                selector: selector_before(css, at),
                variable: name.to_string(),
            });
        }
        from = at + 4 + close;
    }
    out
}

/// The selector of the rule enclosing byte offset `at`: the text between the
/// rule's `{` and whichever of `{` or `}` precedes it.
fn selector_before(css: &str, at: usize) -> String {
    let before = css.get(..at).unwrap_or("");
    let Some(open) = before.rfind('{') else {
        return String::new();
    };
    let head = before.get(..open).unwrap_or("");
    let prev = head.rfind(['{', '}']).map_or(0, |i| i + 1);
    let raw = head.get(prev..).unwrap_or("");
    strip_comments(raw)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn strip_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("/*") {
        out.push_str(rest.get(..start).unwrap_or(""));
        match rest.get(start + 2..).and_then(|r| r.find("*/")) {
            Some(end) => rest = rest.get(start + 2 + end + 2..).unwrap_or(""),
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_uses_with_their_selectors() {
        let css = "/* note */\n.a, .b {\n  color: var(--color-error);\n}\n@media (min-width: 1px) {\n  .c:hover { border: 1px solid var(--border-success); color: var(--palette-1); }\n}\n";
        let uses = status_uses(css);
        assert_eq!(uses.len(), 2);
        assert_eq!(uses.first().map(|u| u.selector.as_str()), Some(".a, .b"));
        assert_eq!(
            uses.first().map(|u| u.variable.as_str()),
            Some("--color-error")
        );
        assert_eq!(uses.get(1).map(|u| u.selector.as_str()), Some(".c:hover"));
    }

    #[test]
    fn ignores_palette_and_accent_variables() {
        assert!(
            status_uses(".x { color: var(--palette-3); background: var(--accent-primary); }")
                .is_empty()
        );
    }
}
