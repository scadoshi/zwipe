//! Renders MTG oracle text, replacing `{symbol}` tokens (mana, tap, energy, …)
//! with [Mana-font](https://mana.andrewgioia.com) glyphs.

use dioxus::prelude::*;

/// A parsed slice of oracle text.
enum Segment {
    /// Literal text.
    Text(String),
    /// A Mana-font class suffix: `u`, `tap`, `wu`, `2w`.
    Symbol(String),
}

/// Splits oracle text into literal runs and `{...}` symbols.
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

/// Scryfall symbol body to Mana-font class suffix: lowercase, slashes dropped
/// (`W/U` -> `wu`), tap and untap special-cased.
fn symbol_class(sym: &str) -> String {
    let s = sym.to_ascii_lowercase().replace('/', "");
    match s.as_str() {
        "t" => "tap".to_string(),
        "q" => "untap".to_string(),
        _ => s,
    }
}

/// Oracle text with symbols rendered as glyphs.
#[component]
pub fn OracleText(text: String, class: String) -> Element {
    // Scryfall's single newline between abilities reads cramped.
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
