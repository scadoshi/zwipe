//! zwipe's surfaces take orangui's three shadow levels by what they are: `sm` for what sits on the
//! page, `md` for what floats over it and `lg` for what covers it. No sheet casts a shadow of its
//! own: a raw value would sit outside the scale and miss a light theme's fainter one.

const MAGIC: &str = include_str!("../assets/magic.css");
const FILTER: &str = include_str!("../assets/filter.css");
const WEB: &str = include_str!("../../zwiper-web/assets/web.css");
const ZITE: &str = include_str!("../../zite/assets/style.css");

/// Every sheet that draws a zwipe surface, by name.
const SHEETS: [(&str, &str); 4] = [
    ("magic.css", MAGIC),
    ("filter.css", FILTER),
    ("web.css", WEB),
    ("zite style.css", ZITE),
];

/// The body of the rule in `sheet` whose selector line is `selector` (the top-level one, or else the
/// first nested one), up to the line that closes it, so a brace in a comment doesn't end it.
fn rule(sheet: &'static str, selector: &str) -> &'static str {
    let open = format!("{selector} {{");
    let Some(line) = sheet
        .lines()
        .find(|line| *line == open)
        .or_else(|| sheet.lines().find(|line| line.trim() == open))
    else {
        return "";
    };
    let Some((_, rest)) = sheet.split_once(&format!("\n{line}\n")) else {
        return "";
    };
    let close = rest
        .match_indices('\n')
        .find(|(at, _)| {
            rest.get(at.saturating_add(1)..)
                .is_some_and(|after| after.trim_start().starts_with("}\n"))
        })
        .map_or(rest.len(), |(at, _)| at);
    rest.get(..close).unwrap_or_default()
}

#[test]
fn each_surface_takes_its_level() {
    let surfaces: [(&str, &'static str, &str, &str); 7] = [
        (
            "filter.css",
            FILTER,
            ".sheet-form-drawer > .bottom-sheet.drawer-every-width",
            "lg",
        ),
        ("web.css", WEB, ".screen.web-large-modal", "lg"),
        (
            "zite style.css",
            ZITE,
            ".sd-image-overlay .sd-image-overlay-img img",
            "lg",
        ),
        ("magic.css", MAGIC, ".hover-stack-card img", "md"),
        ("magic.css", MAGIC, ".home-flavor", "sm"),
        ("magic.css", MAGIC, "footer.site-footer", "sm"),
        ("magic.css", MAGIC, "i.ms.ms-cost", "sm"),
    ];
    for (file, sheet, selector, level) in surfaces {
        let body = rule(sheet, selector);
        assert!(
            body.contains(&format!("var(--shadow-{level})")),
            "{file}: {selector} sits at {level}: {body:?}"
        );
    }
}

#[test]
fn no_sheet_casts_a_raw_shadow() {
    for (file, sheet) in SHEETS {
        for line in sheet.lines().map(str::trim) {
            let shadow = line.starts_with("box-shadow:") || line.starts_with("filter: drop-shadow");
            let raw = line.contains("rgba(0, 0, 0") || line.contains("drop-shadow(");
            assert!(!(shadow && raw), "{file}: {line:?} takes a level instead");
        }
    }
}
