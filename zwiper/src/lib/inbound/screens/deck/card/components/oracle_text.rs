//! Renders MTG oracle text, replacing `{symbol}` tokens (mana, tap, energy, …)
//! with [Mana-font](https://mana.andrewgioia.com) glyphs.

use dioxus::prelude::*;

/// A parsed slice of oracle text.
enum Segment {
    /// Literal text (newlines preserved by the container's `white-space`).
    Text(String),
    /// A Mana-font class suffix, e.g. `u`, `tap`, `e`, `wu`, `2w`.
    Symbol(String),
}

/// Splits oracle text into literal runs and `{...}` symbol tokens.
fn parse(text: &str) -> Vec<Segment> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        buf.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        if let Some(close) = after.find('}') {
            if !buf.is_empty() {
                out.push(Segment::Text(std::mem::take(&mut buf)));
            }
            out.push(Segment::Symbol(symbol_class(&after[..close])));
            rest = &after[close + 1..];
        } else {
            // Unterminated brace: keep the remainder as literal text.
            buf.push_str(&rest[open..]);
            rest = "";
        }
    }
    buf.push_str(rest);
    if !buf.is_empty() {
        out.push(Segment::Text(buf));
    }
    out
}

/// Maps a Scryfall symbol body (no braces) to a Mana-font class suffix:
/// lowercase, slashes dropped (`W/U` -> `wu`), with the tap/untap specials.
fn symbol_class(sym: &str) -> String {
    let s = sym.to_ascii_lowercase().replace('/', "");
    match s.as_str() {
        "t" => "tap".to_string(),
        "q" => "untap".to_string(),
        _ => s,
    }
}

/// Oracle text with mana/tap/energy/etc. symbols rendered as glyphs.
#[component]
pub(crate) fn OracleText(text: String, class: String) -> Element {
    // Scryfall separates abilities with a single newline, which reads cramped.
    // Double them so each ability gets a blank line between it and the next.
    let text = text.replace('\n', "\n\n");
    rsx! {
        p { class: "{class}",
            for (i, seg) in parse(&text).into_iter().enumerate() {
                match seg {
                    Segment::Text(t) => rsx! { span { key: "{i}", "{t}" } },
                    Segment::Symbol(c) => rsx! { i { key: "{i}", class: "ms ms-{c} ms-cost ms-shadow oracle-sym" } },
                }
            }
        }
    }
}
