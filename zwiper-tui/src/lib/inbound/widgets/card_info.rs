//! A card as the card screens show it: the five-line info block under a swiped card, the details body of the eye sheet and an expanded row, and the text card that stands in for an image.
//!
//! Faces are shown one at a time, as the app does; the caller keeps which face is up.

use std::{iter, mem};

use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Padding, Paragraph, Wrap};
use zwipe_core::domain::card::{Card, card_role};
use zwipe_ui::card::{self, Face};

use super::measure;
use crate::domain::theme::Palette;

/// The info block: name, prices, set, release date and artist. A missing price or artist line stays as a blank line so the block keeps its height.
pub(crate) fn info_lines(palette: &Palette, card: &Card) -> Vec<Line<'static>> {
    let data = &card.scryfall_data;
    let artist = data
        .artist
        .as_deref()
        .filter(|a| !a.is_empty())
        .map(|a| format!("Artist: {a}"))
        .unwrap_or_default();
    vec![
        Line::from(data.name.clone()).style(palette.base).bold(),
        Line::from(prices(card)).style(palette.muted),
        Line::from(format!("Set: {}", data.set_name)).style(palette.muted),
        Line::from(format!("Released: {}", data.released_at)).style(palette.muted),
        Line::from(artist).style(palette.muted),
    ]
}

/// "Prices:" then each of USD, EUR and TIX the card has, joined by " |"; empty when it has none.
pub(crate) fn prices(card: &Card) -> String {
    let prices = &card.scryfall_data.prices;
    let parts: Vec<String> = [
        prices.usd.as_ref().map(|usd| format!(" ${usd}")),
        prices.eur.as_ref().map(|eur| format!(" €{eur}")),
        prices.tix.as_ref().map(|tix| format!(" {tix} TIX")),
    ]
    .into_iter()
    .flatten()
    .collect();
    if parts.is_empty() {
        return String::new();
    }
    format!("Prices:{}", parts.join(" |"))
}

/// How many faces the card shows, at least one.
pub(crate) fn face_count(card: &Card) -> usize {
    card::faces(&card.scryfall_data).len().max(1)
}

/// The face at `face`, wrapping around.
fn face_at(card: &Card, face: usize) -> Option<Face> {
    let faces = card::faces(&card.scryfall_data);
    let at = face.checked_rem(faces.len())?;
    faces.into_iter().nth(at)
}

/// A face's mana cost in mana colors.
fn cost_spans(palette: &Palette, face: &Face) -> Vec<Span<'static>> {
    face.mana_cost
        .iter()
        .map(|symbol| {
            let style = symbol
                .color
                .and_then(|i| palette.mana.get(i).copied())
                .map_or(palette.muted, Style::bold);
            Span::styled(format!("{{{}}}", symbol.text), style)
        })
        .collect()
}

fn chip(palette: &Palette, text: &str) -> Span<'static> {
    Span::styled(format!(" {text} "), palette.muted.reversed())
}

/// The details body: cost, type and rarity and set chips, rules text, stats, keywords, and the card roles with their oracle tags.
pub(crate) fn details_lines(palette: &Palette, card: &Card, face: usize) -> Vec<Line<'static>> {
    let data = &card.scryfall_data;
    let mut lines = Vec::new();
    let Some(face) = face_at(card, face) else {
        return lines;
    };
    let cost = cost_spans(palette, &face);
    if !cost.is_empty() {
        lines.push(Line::from(cost));
    }
    let mut chips = Vec::new();
    if !face.type_line.is_empty() {
        chips.push(chip(palette, &face.type_line));
        chips.push(Span::raw(" "));
    }
    chips.push(chip(palette, &data.rarity.to_long_name()));
    chips.push(Span::raw(" "));
    chips.push(chip(palette, &data.set_name));
    lines.push(Line::from(chips));
    if !face.rules.is_empty() {
        lines.push(Line::default());
        lines.extend(face.rules.iter().map(|r| Line::from(r.clone())));
    }
    if let Some(stats) = &face.stats {
        lines.push(Line::default());
        lines.push(Line::from(vec![chip(palette, stats)]));
    }
    let keywords: Vec<String> = data.keywords.clone().unwrap_or_default();
    if !keywords.is_empty() {
        lines.push(Line::default());
        lines.push(Line::from("Keywords").style(palette.muted.bold()));
        let mut spans = Vec::new();
        for keyword in keywords {
            spans.push(chip(palette, &keyword));
            spans.push(Span::raw(" "));
        }
        lines.push(Line::from(spans));
    }
    let profile = &card.card_profile;
    if !profile.card_roles.is_empty() || !profile.other_oracle_tags.is_empty() {
        lines.push(Line::default());
        lines.push(Line::from("Card roles").style(palette.muted.bold()));
        for role in &profile.card_roles {
            let tags = profile
                .oracle_tags_by_role
                .get(role)
                .map(|t| t.join(", "))
                .unwrap_or_default();
            lines.push(role_line(palette, &card_role::role_label(role), &tags));
        }
        if !profile.other_oracle_tags.is_empty() {
            lines.push(role_line(
                palette,
                "Other tags",
                &profile.other_oracle_tags.join(", "),
            ));
        }
    }
    lines
}

/// The card's oracle tags in the order the details list them: each role's tags, then the other tags, each once.
pub(crate) fn oracle_tags(card: &Card) -> Vec<String> {
    let profile = &card.card_profile;
    let mut tags: Vec<String> = Vec::new();
    let by_role = profile
        .card_roles
        .iter()
        .filter_map(|role| profile.oracle_tags_by_role.get(role))
        .flatten();
    for tag in by_role.chain(&profile.other_oracle_tags) {
        if !tags.contains(tag) {
            tags.push(tag.clone());
        }
    }
    tags
}

/// One role chip and its oracle tags. A role with no tags is a grey chip, as the app draws a chip with nothing to open.
fn role_line(palette: &Palette, label: &str, tags: &str) -> Line<'static> {
    let chip = if tags.is_empty() {
        palette.muted
    } else {
        palette.accent
    };
    let mut spans = vec![Span::styled(format!(" {label} "), chip.reversed())];
    if !tags.is_empty() {
        spans.push(Span::styled(format!("  {tags}"), palette.muted));
    }
    Line::from(spans)
}

/// The text card's parts, top to bottom.
#[derive(Clone)]
struct ProxyParts {
    /// Name and cost, the face mark, and the type line.
    head: Vec<Line<'static>>,
    rules: Vec<Line<'static>>,
    flavor: Option<Line<'static>>,
    stats: Option<Line<'static>>,
}

impl ProxyParts {
    /// The parts as lines, a blank line before each part after the head.
    fn lines(self) -> Vec<Line<'static>> {
        let Self {
            head,
            rules,
            flavor,
            stats,
        } = self;
        let mut lines = head;
        if !rules.is_empty() {
            lines.push(Line::default());
            lines.extend(rules);
        }
        for line in [flavor, stats].into_iter().flatten() {
            lines.push(Line::default());
            lines.push(line);
        }
        lines
    }
}

/// The text card's parts for `face`, or `None` for a face the card lacks.
fn proxy_parts(palette: &Palette, card: &Card, face: usize) -> Option<ProxyParts> {
    let shown = face_at(card, face)?;
    let mut title = vec![Span::raw(shown.name.clone()).bold()];
    let cost = cost_spans(palette, &shown);
    if !cost.is_empty() {
        title.push(Span::raw("  "));
        title.extend(cost);
    }
    let mut head = vec![Line::from(title)];
    if face_count(card) > 1 {
        let count = face_count(card);
        let at = face.checked_rem(count).unwrap_or(0);
        head.push(
            Line::from(format!("Face {} of {count}, t flips", at.saturating_add(1)))
                .style(palette.muted),
        );
    }
    if !shown.type_line.is_empty() {
        head.push(Line::from(shown.type_line.clone()).italic());
    }
    let rules = shown.rules.iter().map(|r| Line::from(r.clone())).collect();
    let flavor = card
        .scryfall_data
        .card_faces
        .as_ref()
        .and_then(|faces| faces.get(face.checked_rem(face_count(card))?))
        .and_then(|f| f.flavor_text.clone())
        .or_else(|| card.scryfall_data.flavor_text.clone())
        .map(|flavor| Line::from(flavor).style(palette.muted).italic());
    let stats = shown
        .stats
        .as_ref()
        .map(|stats| Line::from(stats.clone()).bold().right_aligned());
    Some(ProxyParts {
        head,
        rules,
        flavor,
        stats,
    })
}

/// The text card that stands in for an image: name and cost, type line, rules, flavor text, stats, and a Flip mark on a card with more faces.
pub(crate) fn proxy_lines(palette: &Palette, card: &Card, face: usize) -> Vec<Line<'static>> {
    proxy_parts(palette, card, face).map_or_else(Vec::new, ProxyParts::lines)
}

/// The rows `lines` take wrapped to `width` cells, as the text card wraps them.
fn wrapped_rows(lines: &[Line<'static>], width: usize) -> usize {
    Paragraph::new(lines.to_vec())
        .wrap(Wrap { trim: true })
        .line_count(u16::try_from(width).unwrap_or(u16::MAX))
}

/// The text card's lines fitted to `rows` rows of `width` cells. A card too tall gives up its flavor text first, then the end of its rules, which close on an ellipsis; the name, the type line and the stats always stay.
fn fitted_proxy_lines(
    palette: &Palette,
    card: &Card,
    face: usize,
    width: usize,
    rows: usize,
) -> Vec<Line<'static>> {
    let Some(mut parts) = proxy_parts(palette, card, face) else {
        return Vec::new();
    };
    let all = parts.clone().lines();
    if wrapped_rows(&all, width) <= rows {
        return all;
    }
    parts.flavor = None;
    let stats_rows = if parts.stats.is_some() { 2 } else { 0 };
    let room = rows
        .saturating_sub(wrapped_rows(&parts.head, width))
        .saturating_sub(stats_rows)
        // The blank line between the head and the rules.
        .saturating_sub(1);
    let wrapped = wrap(mem::take(&mut parts.rules), width, 0);
    let cut = wrapped.len() > room;
    let mut rules: Vec<Line<'static>> = wrapped.into_iter().take(room).collect();
    if cut && let Some(last) = rules.pop() {
        let text = format!("{last}\u{2026}");
        rules.push(Line::from(measure::ellipsize(&text, width)).style(last.style));
    }
    parts.rules = rules;
    parts.lines()
}

/// The bordered text card of [`proxy_lines`], at most `max_width` wide, and the rect it takes centered in `area`. A card taller than `area` is fitted to it, keeping its stats line.
pub(crate) fn proxy_card(
    palette: &Palette,
    card: &Card,
    face: usize,
    border: Style,
    area: Rect,
    max_width: u16,
) -> (Paragraph<'static>, Rect) {
    let width = area.width.min(max_width);
    let inner = usize::from(width.saturating_sub(4));
    let rows = usize::from(area.height.saturating_sub(2));
    let text = Paragraph::new(fitted_proxy_lines(palette, card, face, inner, rows))
        .wrap(Wrap { trim: true })
        .block(
            crate::inbound::widgets::panel::bordered()
                .border_style(border)
                .padding(Padding::horizontal(1)),
        );
    let height = u16::try_from(text.line_count(width.saturating_sub(4))).unwrap_or(u16::MAX);
    let rect = area.centered(
        Constraint::Length(width),
        Constraint::Length(height.min(area.height)),
    );
    (text, rect)
}

/// The bullet that starts a list line in hints, popups and the privacy policy.
pub(crate) const BULLET: &str = "• ";

/// `bullet` and `spans` wrapped to `width` cells with each continuation hung under the text after the bullet. A reversed span, a key chip, never breaks inside.
pub(crate) fn bulleted(
    bullet: Span<'static>,
    spans: Vec<Span<'static>>,
    width: usize,
) -> Vec<Line<'static>> {
    let indent = bullet.width();
    let glued = spans.into_iter().map(|mut span| {
        if span.style.add_modifier.contains(Modifier::REVERSED) {
            span.content = span.content.replace(' ', "\u{a0}").into();
        }
        span
    });
    let line = Line::from(iter::once(bullet).chain(glued).collect::<Vec<_>>());
    wrap(vec![line], width, indent)
}

/// Breaks lines wider than `width` between words, keeping each word's style; a continuation starts with `indent` spaces.
pub(crate) fn wrap(lines: Vec<Line<'static>>, width: usize, indent: usize) -> Vec<Line<'static>> {
    let width = width.max(indent.saturating_add(1));
    let mut out = Vec::new();
    for line in lines {
        if line.width() <= width {
            out.push(line);
            continue;
        }
        let mut current: Vec<Span<'static>> = Vec::new();
        let mut used = 0usize;
        for span in line.spans {
            let style = span.style;
            for (i, word) in span.content.split(' ').enumerate() {
                let piece = if i == 0 {
                    word.to_owned()
                } else {
                    format!(" {word}")
                };
                let len = piece.chars().count();
                if used > indent && used.saturating_add(len) > width {
                    out.push(Line::from(std::mem::take(&mut current)));
                    current.push(Span::raw(" ".repeat(indent)));
                    let trimmed = piece.trim_start().to_owned();
                    used = indent.saturating_add(trimmed.chars().count());
                    current.push(Span::styled(trimmed, style));
                    continue;
                }
                used = used.saturating_add(len);
                current.push(Span::styled(piece, style));
            }
        }
        out.push(Line::from(current));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::card;

    fn text(lines: &[Line<'_>]) -> Vec<String> {
        lines.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn prices_join_what_the_card_has() {
        let mut c = card("Bolt");
        c.scryfall_data.prices.usd = Some("1.23".to_owned());
        c.scryfall_data.prices.eur = Some("1.10".to_owned());
        c.scryfall_data.prices.tix = Some("0.05".to_owned());
        assert_eq!(prices(&c), "Prices: $1.23 | €1.10 | 0.05 TIX");
        c.scryfall_data.prices.usd = None;
        assert_eq!(prices(&c), "Prices: €1.10 | 0.05 TIX");
        c.scryfall_data.prices.eur = None;
        c.scryfall_data.prices.tix = None;
        assert_eq!(prices(&c), "");
    }

    #[test]
    fn a_role_with_no_tags_is_a_grey_chip() {
        let palette = Palette::for_account("gruvbox", true);
        let bare = role_line(&palette, "Ramp", "");
        assert_eq!(bare.to_string(), " Ramp ");
        assert_eq!(bare.spans[0].style, palette.muted.reversed());
        let tagged = role_line(&palette, "Ramp", "mana rock");
        assert_eq!(tagged.to_string(), " Ramp   mana rock");
        assert_eq!(tagged.spans[0].style, palette.accent.reversed());
    }

    #[test]
    fn a_short_card_gives_up_its_flavor_then_its_rules_but_keeps_its_stats() {
        let mut c = card("Bear");
        c.scryfall_data.type_line = Some("Creature — Bear".to_owned());
        c.scryfall_data.oracle_text =
            Some("one two three four five six seven eight nine ten".to_owned());
        c.scryfall_data.flavor_text = Some("Growl.".to_owned());
        c.scryfall_data.power = Some("2".to_owned());
        c.scryfall_data.toughness = Some("2".to_owned());
        let palette = Palette::terminal();
        let all = text(&fitted_proxy_lines(&palette, &c, 0, 20, 50));
        assert!(
            all.contains(&"Growl.".to_owned()),
            "room for everything: {all:?}"
        );
        let no_flavor = text(&fitted_proxy_lines(&palette, &c, 0, 20, 8));
        assert!(!no_flavor.contains(&"Growl.".to_owned()), "{no_flavor:?}");
        assert!(
            no_flavor.iter().any(|l| l.ends_with("ten")),
            "the rules stay whole: {no_flavor:?}"
        );
        let cut = text(&fitted_proxy_lines(&palette, &c, 0, 20, 6));
        assert_eq!(cut.len(), 6, "{cut:?}");
        assert!(
            cut[3].ends_with('…'),
            "the rules close on an ellipsis: {cut:?}"
        );
        assert_eq!(cut.last().map(String::as_str), Some("2/2"));
    }

    #[test]
    fn long_lines_wrap_between_words_with_an_indent() {
        let lines = wrap(vec![Line::from("    one two three four")], 13, 4);
        assert_eq!(text(&lines), ["    one two", "    three", "    four"]);
        assert_eq!(wrap(vec![Line::from("short")], 13, 4).len(), 1);
    }

    #[test]
    fn the_info_block_keeps_five_lines() {
        let mut c = card("Bolt");
        c.scryfall_data.artist = None;
        let lines = text(&info_lines(&Palette::terminal(), &c));
        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0], "Bolt");
        assert!(lines[2].starts_with("Set: "));
        assert!(lines[3].starts_with("Released: "));
        assert_eq!(lines[4], "");
    }

    #[test]
    fn details_and_proxy_show_one_face() {
        let mut c = card("Bolt");
        c.scryfall_data.type_line = Some("Instant".to_owned());
        c.scryfall_data.oracle_text = Some("Deal 3.".to_owned());
        c.scryfall_data.mana_cost = Some("{R}".to_owned());
        c.scryfall_data.keywords = Some(vec!["Haste".to_owned()]);
        let details = text(&details_lines(&Palette::terminal(), &c, 0)).join("\n");
        assert!(details.contains("{R}"));
        assert!(details.contains("Instant"));
        assert!(details.contains("Deal 3."));
        assert!(details.contains("Keywords"));
        let proxy = text(&proxy_lines(&Palette::terminal(), &c, 5)).join("\n");
        assert!(proxy.contains("Bolt"));
        assert!(!proxy.contains("flips"), "one face");
        assert_eq!(face_count(&c), 1);
    }
}
