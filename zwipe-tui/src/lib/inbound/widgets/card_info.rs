//! A card as the card screens show it: the five-line info block under a swiped card, the details body of the eye sheet and an expanded row, and the text card that stands in for an image.
//!
//! Faces are shown one at a time, as the app does; the caller keeps which face is up.

use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::card_role::role_label;

use crate::domain::card::{self, Face};
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
    let count = faces.len();
    if count == 0 {
        return None;
    }
    faces.into_iter().nth(face % count)
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

fn chip(palette: &Palette, text: String) -> Span<'static> {
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
        chips.push(chip(palette, face.type_line.clone()));
        chips.push(Span::raw(" "));
    }
    chips.push(chip(palette, data.rarity.to_long_name()));
    chips.push(Span::raw(" "));
    chips.push(chip(palette, data.set_name.clone()));
    lines.push(Line::from(chips));
    if !face.rules.is_empty() {
        lines.push(Line::default());
        lines.extend(face.rules.iter().map(|r| Line::from(r.clone())));
    }
    if let Some(stats) = &face.stats {
        lines.push(Line::default());
        lines.push(Line::from(vec![chip(palette, stats.clone())]));
    }
    let keywords: Vec<String> = data.keywords.clone().unwrap_or_default();
    if !keywords.is_empty() {
        lines.push(Line::default());
        lines.push(Line::from("Keywords").style(palette.muted.bold()));
        let mut spans = Vec::new();
        for keyword in keywords {
            spans.push(chip(palette, keyword));
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
            lines.push(role_line(palette, role_label(role), &tags));
        }
        if !profile.other_oracle_tags.is_empty() {
            lines.push(role_line(
                palette,
                "Other tags".to_owned(),
                &profile.other_oracle_tags.join(", "),
            ));
        }
    }
    lines
}

fn role_line(palette: &Palette, label: String, tags: &str) -> Line<'static> {
    let mut spans = vec![Span::styled(
        format!(" {label} "),
        palette.accent.reversed(),
    )];
    if !tags.is_empty() {
        spans.push(Span::styled(format!("  {tags}"), palette.muted));
    }
    Line::from(spans)
}

/// The text card that stands in for an image: name and cost, type line, rules, flavor text, stats, and a Flip mark on a card with more faces.
pub(crate) fn proxy_lines(palette: &Palette, card: &Card, face: usize) -> Vec<Line<'static>> {
    let Some(shown) = face_at(card, face) else {
        return Vec::new();
    };
    let mut title = vec![Span::raw(shown.name.clone()).bold()];
    let cost = cost_spans(palette, &shown);
    if !cost.is_empty() {
        title.push(Span::raw("  "));
        title.extend(cost);
    }
    let mut lines = vec![Line::from(title)];
    if face_count(card) > 1 {
        let count = face_count(card);
        let at = face % count;
        lines.push(
            Line::from(format!("Face {} of {count}, t flips", at.saturating_add(1)))
                .style(palette.muted),
        );
    }
    if !shown.type_line.is_empty() {
        lines.push(Line::from(shown.type_line.clone()).italic());
    }
    if !shown.rules.is_empty() {
        lines.push(Line::default());
        lines.extend(shown.rules.iter().map(|r| Line::from(r.clone())));
    }
    let flavor = card
        .scryfall_data
        .card_faces
        .as_ref()
        .and_then(|faces| faces.get(face % face_count(card)))
        .and_then(|f| f.flavor_text.clone())
        .or_else(|| card.scryfall_data.flavor_text.clone());
    if let Some(flavor) = flavor {
        lines.push(Line::default());
        lines.push(Line::from(flavor).style(palette.muted).italic());
    }
    if let Some(stats) = &shown.stats {
        lines.push(Line::default());
        lines.push(Line::from(stats.clone()).bold().right_aligned());
    }
    lines
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
