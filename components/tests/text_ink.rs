//! The ink gate for zwipe's own sheets: the Magic and screen sheets, the
//! app's, the web's and the site's. Text drawn in a tone reads the tone's ink,
//! `var(--ink-<tone>, var(--<tone>))` from orangui's themes, so it reads at
//! 4.5:1 in every theme. A disabled control is exempt, since it is dimmed on
//! purpose, and so is a rule in [`NOT_TEXT`], whose `color` paints a logo
//! rather than letters.

#![expect(
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    reason = "a test: a bad slice or overflow fails it"
)]

/// The tones that carry an `--ink-<tone>` in every theme block.
const TONES: [&str; 14] = [
    "accent-primary",
    "accent-secondary",
    "accent-tertiary",
    "palette-1",
    "palette-2",
    "palette-3",
    "palette-4",
    "palette-5",
    "palette-6",
    "text-muted",
    "color-money",
    "color-success",
    "color-warning",
    "color-error",
];

/// Rules whose `color` paints a logo through `currentColor`, not text.
const NOT_TEXT: [&str; 2] = [
    ".nav-brand:hover .web-nav-logo",
    ".nav-brand:hover .nav-logo",
];

/// Each rule's selector and body, innermost blocks only, comments removed.
fn rules(css: &str) -> Vec<(String, String)> {
    let mut clean = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        clean.push_str(&rest[..start]);
        rest = rest[start..]
            .find("*/")
            .map_or("", |end| &rest[start + end + 2..]);
    }
    clean.push_str(rest);

    let mut out = Vec::new();
    let mut mark = 0;
    let mut selector = None;
    for (i, ch) in clean.char_indices() {
        match ch {
            '{' => {
                selector = Some(
                    clean[mark..i]
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" "),
                );
                mark = i + 1;
            }
            '}' => {
                if let Some(sel) = selector.take() {
                    out.push((sel, clean[mark..i].to_string()));
                }
                mark = i + 1;
            }
            _ => {}
        }
    }
    out
}

/// Whether `selector` styles a disabled control, as opposed to excluding one with `:not(...)`.
fn disabled(selector: &str) -> bool {
    let mut rest = selector;
    let mut outside = String::new();
    while let Some(start) = rest.find(":not(") {
        outside.push_str(&rest[..start]);
        let inner = &rest[start + ":not(".len()..];
        let mut depth = 1_usize;
        let close = inner.char_indices().find_map(|(i, ch)| {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            (depth == 0).then_some(i)
        });
        rest = close.map_or("", |i| &inner[i + 1..]);
    }
    outside.push_str(rest);
    outside.contains("disabled")
}

/// The tones the `color:` declarations in `body` name bare.
fn bare_tones(body: &str) -> Vec<&'static str> {
    body.split(';')
        .filter_map(|decl| {
            let (key, value) = decl.split_once(':')?;
            (key.trim() == "color").then(|| value.trim())
        })
        .filter_map(|value| {
            TONES
                .into_iter()
                .find(|tone| value == format!("var(--{tone})"))
        })
        .collect()
}

#[test]
fn tone_text_reads_its_ink() {
    let sheets = [
        ("magic.css", include_str!("../assets/magic.css")),
        ("filter.css", include_str!("../assets/filter.css")),
        (
            "screens/base.css",
            include_str!("../assets/screens/base.css"),
        ),
        (
            "screens/components.css",
            include_str!("../assets/screens/components.css"),
        ),
        (
            "screens/screens-a.css",
            include_str!("../assets/screens/screens-a.css"),
        ),
        (
            "screens/screens-b.css",
            include_str!("../assets/screens/screens-b.css"),
        ),
        (
            "zwiper main.css",
            include_str!("../../zwiper/assets/main.css"),
        ),
        (
            "zwiper-web web.css",
            include_str!("../../zwiper-web/assets/web.css"),
        ),
        (
            "zite style.css",
            include_str!("../../zite/assets/style.css"),
        ),
        (
            "zite filter-inputs.css",
            include_str!("../../zite/assets/filter-inputs.css"),
        ),
    ];
    let mut offenders = Vec::new();
    for (name, css) in sheets {
        for (selector, body) in rules(css) {
            if NOT_TEXT.contains(&selector.as_str()) || disabled(&selector) {
                continue;
            }
            for tone in bare_tones(&body) {
                offenders.push(format!("{name}: {selector} reads --{tone}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "text drawn in a bare tone (read var(--ink-<tone>, var(--<tone>))):\n  {}",
        offenders.join("\n  ")
    );
}

#[test]
fn the_gate_sees_a_bare_tone_and_passes_an_ink() {
    let css = "/* c */ .x-btn { color: var(--palette-2); }\n@media (hover: hover) { .y-btn:hover { color: var(--ink-palette-1, var(--palette-1)); border-color: var(--palette-1); } }";
    let found: Vec<_> = rules(css)
        .into_iter()
        .flat_map(|(selector, body)| {
            bare_tones(&body)
                .into_iter()
                .map(move |tone| (selector.clone(), tone))
        })
        .collect();
    assert_eq!(found, [(".x-btn".to_string(), "palette-2")]);
    assert!(disabled(".btn:disabled"));
    assert!(!disabled(".btn:not(:disabled, .chip:not(.disabled)):hover"));
}
