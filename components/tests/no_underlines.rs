//! The link gate: no link is underlined, in the app, on app.zwipe.net or on zwipe.net. A text
//! link reads as one by the accent color alone, so no sheet these surfaces load sets an
//! underline. The kit's own sheets carry the same gate in orangui.

const SHEETS: [(&str, &str); 9] = [
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
        "zite style.css",
        include_str!("../../zite/assets/style.css"),
    ),
    (
        "zwiper-web web.css",
        include_str!("../../zwiper-web/assets/web.css"),
    ),
    (
        "zwiper main.css",
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

#[test]
fn no_sheet_underlines_a_link() {
    for (name, sheet) in SHEETS {
        for (number, line) in without_comments(sheet).lines().enumerate() {
            assert!(
                !line.contains("underline"),
                "{name}:{}: {}",
                number.saturating_add(1),
                line.trim()
            );
        }
    }
}
