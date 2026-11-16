//! Under `prefers-reduced-motion: reduce`, zwipe's own sheets drop their movement: a fold opens
//! and shuts at once, nothing pulses, and nothing slides, drops or eases in on a transform or a
//! layout property. Each sheet answers for its own rules with a reduced-motion rule naming the
//! same selector. An animation the code waits on to end runs for 1ms there, so its
//! `animationend` still comes.

#![expect(
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    reason = "a test: it walks CSS text by byte offsets it found in that text"
)]

use std::collections::{HashMap, HashSet};

/// Every sheet zwipe writes itself, with its name for failure messages.
const SHEETS: [(&str, &str); 11] = [
    ("deck_page.css", include_str!("../assets/deck_page.css")),
    ("filter.css", include_str!("../assets/filter.css")),
    ("guides.css", include_str!("../assets/guides.css")),
    ("magic.css", include_str!("../assets/magic.css")),
    ("base.css", include_str!("../assets/screens/base.css")),
    (
        "components.css",
        include_str!("../assets/screens/components.css"),
    ),
    (
        "screens-a.css",
        include_str!("../assets/screens/screens-a.css"),
    ),
    (
        "screens-b.css",
        include_str!("../assets/screens/screens-b.css"),
    ),
    ("web.css", include_str!("../../zwiper-web/assets/web.css")),
    ("main.css", include_str!("../../zwiper/assets/main.css")),
    ("fonts.css", include_str!("../assets/fonts.css")),
];

/// Keyframes that place a dock as the page scrolls (scroll-driven), which is position rather
/// than motion, and the spinner, whose turning is the message.
const KEPT: [&str; 3] = ["footer-rise", "web-dock-panel-rise", "spin"];

/// Properties whose transition moves something or makes the page lay out again.
const MOVING: [&str; 14] = [
    "transform",
    "translate",
    "rotate",
    "scale",
    "grid-template-rows",
    "height",
    "width",
    "min-height",
    "max-height",
    "top",
    "bottom",
    "left",
    "margin",
    "padding",
];

/// One rule: the at-rules around it, its selectors and its declarations.
struct Rule {
    context: Vec<String>,
    selectors: Vec<String>,
    declarations: String,
}

impl Rule {
    fn reduced(&self) -> bool {
        self.context
            .iter()
            .any(|c| c.contains("prefers-reduced-motion: reduce"))
    }

    fn exempt(&self) -> bool {
        self.context
            .iter()
            .any(|c| c.contains("prefers-reduced-motion: no-preference"))
    }

    fn value(&self, property: &str) -> Option<&str> {
        let prefix = format!("{property}:");
        self.declarations
            .split(';')
            .map(str::trim)
            .find_map(|d| d.strip_prefix(prefix.as_str()))
            .map(str::trim)
    }
}

/// `css` without its comments.
fn uncommented(css: &str) -> String {
    let mut out = String::new();
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        rest = rest[start..]
            .find("*/")
            .map_or("", |end| &rest[start + end + 2..]);
    }
    out.push_str(rest);
    out
}

/// The selectors of a rule's `head`, split at its top-level commas, whitespace collapsed.
fn selectors(head: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut depth = 0_u32;
    let mut start = 0;
    for (at, c) in head.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                found.push(&head[start..at]);
                start = at + 1;
            }
            _ => {}
        }
    }
    found.push(&head[start..]);
    found
        .into_iter()
        .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect()
}

/// Every rule in `css`, with the at-rules it sits in.
fn rules(css: &str) -> Vec<Rule> {
    let css = uncommented(css);
    let mut found = Vec::new();
    let mut context: Vec<String> = Vec::new();
    let mut start = 0;
    for (at, c) in css.char_indices() {
        match c {
            '{' => {
                let head = css[start..at].trim().to_owned();
                context.push(head);
                start = at + 1;
            }
            '}' => {
                let body = css[start..at].trim();
                if let Some(head) = context.pop()
                    && !head.starts_with('@')
                    && !context.iter().any(|c| c.starts_with("@keyframes"))
                {
                    found.push(Rule {
                        context: context.clone(),
                        selectors: selectors(&head),
                        declarations: body.to_owned(),
                    });
                }
                start = at + 1;
            }
            ';' if context.last().is_none_or(|h| h.starts_with('@')) => start = at + 1,
            _ => {}
        }
    }
    found
}

/// The names of the keyframes defined in `css` that move something.
fn moving_keyframes(css: &str) -> HashSet<String> {
    let css = uncommented(css);
    let mut names = HashSet::new();
    for (at, _) in css.match_indices("@keyframes ") {
        let rest = &css[at + 11..];
        let Some(open) = rest.find('{') else { continue };
        let name = rest[..open].trim().to_owned();
        let mut depth = 0;
        let mut end = open;
        for (i, c) in rest[open..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = open + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        let body = &rest[open..end];
        if [
            "transform",
            "translate",
            "rotate",
            "scale",
            "height",
            "margin",
        ]
        .iter()
        .any(|p| body.contains(p))
        {
            names.insert(name);
        }
    }
    names
}

#[test]
fn every_moving_rule_in_zwipe_s_sheets_has_a_reduced_motion_answer() {
    let moving: HashSet<String> = SHEETS
        .iter()
        .flat_map(|(_, css)| moving_keyframes(css))
        .collect();
    let mut missing = Vec::new();
    for (sheet, css) in SHEETS {
        let rules = rules(css);
        let mut answered: HashMap<String, Vec<&Rule>> = HashMap::new();
        for rule in rules.iter().filter(|r| r.reduced()) {
            for selector in &rule.selectors {
                answered.entry(selector.clone()).or_default().push(rule);
            }
        }
        for rule in rules.iter().filter(|r| !r.reduced() && !r.exempt()) {
            let animation = rule.value("animation").filter(|v| *v != "none");
            let animates = animation.is_some_and(|v| {
                !KEPT
                    .iter()
                    .any(|k| v.split_whitespace().any(|w| w.trim_end_matches(',') == *k))
                    && (v.contains("infinite") || v.split([' ', ',']).any(|w| moving.contains(w)))
            });
            let transition = rule.value("transition").filter(|v| *v != "none");
            let transitions = transition.is_some_and(|v| {
                v.split(',').any(|part| {
                    part.split_whitespace()
                        .next()
                        .is_some_and(|property| MOVING.contains(&property))
                })
            });
            for selector in &rule.selectors {
                let answers = answered.get(selector).map_or(&[][..], Vec::as_slice);
                let stills_animation = answers.iter().any(|a| {
                    a.value("animation") == Some("none")
                        || a.value("animation-duration") == Some("1ms")
                });
                let stills_transition = answers
                    .iter()
                    .any(|a| a.value("transition") == Some("none"));
                if animates && !stills_animation {
                    missing.push(format!(
                        "{sheet}: {selector} (animation: {})",
                        animation.unwrap_or_default()
                    ));
                }
                if transitions && !stills_transition {
                    missing.push(format!(
                        "{sheet}: {selector} (transition: {})",
                        transition.unwrap_or_default()
                    ));
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "{} without a reduced-motion answer:\n{}",
        missing.len(),
        missing.join("\n")
    );
}

#[test]
fn a_fold_shuts_and_opens_at_once_under_reduced_motion() {
    let screens = rules(SHEETS[6].1);
    let fold = screens
        .iter()
        .find(|r| r.reduced() && r.selectors.iter().any(|s| s == ".collapsible"))
        .map(|r| r.value("transition"));
    assert_eq!(fold, Some(Some("none")));
}
