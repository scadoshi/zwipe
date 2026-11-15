//! The hover gate: a hover look applies only where the device hovers. Every `:hover` rule in a
//! sheet zwipe ships (the shared Magic UI sheets, the web app's own sheet and its copies of the
//! kit's, zwipe.net's and the app's) sits inside `@media (hover: hover)`, so a tap on a touch
//! screen does not leave what it touched lit, or paused, until the next tap lands elsewhere.

const SHEETS: [(&str, &str); 23] = [
    (
        "components/assets/deck_page.css",
        include_str!("../assets/deck_page.css"),
    ),
    (
        "components/assets/filter.css",
        include_str!("../assets/filter.css"),
    ),
    (
        "components/assets/fonts.css",
        include_str!("../assets/fonts.css"),
    ),
    (
        "components/assets/guides.css",
        include_str!("../assets/guides.css"),
    ),
    (
        "components/assets/magic.css",
        include_str!("../assets/magic.css"),
    ),
    (
        "components/assets/screens/base.css",
        include_str!("../assets/screens/base.css"),
    ),
    (
        "components/assets/screens/components.css",
        include_str!("../assets/screens/components.css"),
    ),
    (
        "components/assets/screens/screens-a.css",
        include_str!("../assets/screens/screens-a.css"),
    ),
    (
        "components/assets/screens/screens-b.css",
        include_str!("../assets/screens/screens-b.css"),
    ),
    (
        "zwiper-web/assets/web.css",
        include_str!("../../zwiper-web/assets/web.css"),
    ),
    (
        "zwiper-web/assets/sheets/01-fonts.css",
        include_str!("../../zwiper-web/assets/sheets/01-fonts.css"),
    ),
    (
        "zwiper-web/assets/sheets/02-themes.css",
        include_str!("../../zwiper-web/assets/sheets/02-themes.css"),
    ),
    (
        "zwiper-web/assets/sheets/03-components.css",
        include_str!("../../zwiper-web/assets/sheets/03-components.css"),
    ),
    (
        "zwiper-web/assets/sheets/04-nav-fold.css",
        include_str!("../../zwiper-web/assets/sheets/04-nav-fold.css"),
    ),
    (
        "zwiper-web/assets/sheets/05-magic.css",
        include_str!("../../zwiper-web/assets/sheets/05-magic.css"),
    ),
    (
        "zwiper-web/assets/sheets/06-app.css",
        include_str!("../../zwiper-web/assets/sheets/06-app.css"),
    ),
    (
        "zwiper-web/assets/sheets/07-screens.css",
        include_str!("../../zwiper-web/assets/sheets/07-screens.css"),
    ),
    (
        "zwiper-web/assets/sheets/08-toast.css",
        include_str!("../../zwiper-web/assets/sheets/08-toast.css"),
    ),
    (
        "zwiper-web/assets/sheets/09-filter.css",
        include_str!("../../zwiper-web/assets/sheets/09-filter.css"),
    ),
    (
        "zwiper-web/assets/sheets/10-deck-page.css",
        include_str!("../../zwiper-web/assets/sheets/10-deck-page.css"),
    ),
    (
        "zwiper-web/assets/sheets/11-guides.css",
        include_str!("../../zwiper-web/assets/sheets/11-guides.css"),
    ),
    (
        "zite/assets/style.css",
        include_str!("../../zite/assets/style.css"),
    ),
    (
        "zwiper/assets/main.css",
        include_str!("../../zwiper/assets/main.css"),
    ),
];

/// `sheet` with its comments blanked out, line breaks kept so line numbers still match.
fn without_comments(sheet: &str) -> String {
    let mut out = String::with_capacity(sheet.len());
    let mut rest = sheet;
    while let Some(start) = rest.find("/*") {
        let (before, after) = rest.split_at(start);
        out.push_str(before);
        let end = after
            .find("*/")
            .map_or(after.len(), |e| e.saturating_add(2));
        let (comment, tail) = after.split_at(end);
        out.extend(comment.chars().filter(|c| *c == '\n'));
        rest = tail;
    }
    out.push_str(rest);
    out
}

/// Every block's prelude in `sheet` with its line and the preludes of the blocks around it.
fn blocks(sheet: &str) -> Vec<(usize, String, Vec<String>)> {
    let mut found = Vec::new();
    let mut open: Vec<String> = Vec::new();
    let mut prelude = String::new();
    let mut line = 1usize;
    for ch in without_comments(sheet).chars() {
        match ch {
            '{' => {
                let selector = prelude.split_whitespace().collect::<Vec<_>>().join(" ");
                found.push((line, selector.clone(), open.clone()));
                open.push(selector);
                prelude.clear();
            }
            '}' => {
                open.pop();
                prelude.clear();
            }
            ';' => prelude.clear(),
            '\n' => {
                line = line.saturating_add(1);
                prelude.push(' ');
            }
            other => prelude.push(other),
        }
    }
    found
}

#[test]
fn every_hover_look_waits_for_a_device_that_hovers() {
    for (name, sheet) in SHEETS {
        for (line, selector, around) in blocks(sheet) {
            if !selector.contains(":hover") || selector.starts_with('@') {
                continue;
            }
            let gated = around
                .iter()
                .any(|p| p.starts_with("@media") && p.contains("(hover: hover)"));
            assert!(
                gated,
                "{name}:{line}: `{selector}` is outside @media (hover: hover)"
            );
        }
    }
}
