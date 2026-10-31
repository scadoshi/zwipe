//! A card as lines of text: each face's name and cost, type line, rules and stats, for screens that show a card without its image.

use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use zwipe_core::domain::card::scryfall_data::ScryfallData;

use crate::domain::card::{self, Face};
use crate::domain::theme::Palette;

/// One face as lines: the name and cost, the type line, the rules, the stats.
pub(crate) fn face_lines(palette: &Palette, face: &Face) -> Vec<Line<'static>> {
    let mut title = vec![Span::raw(face.name.clone()).bold()];
    if !face.mana_cost.is_empty() {
        title.push(Span::raw("  "));
        title.extend(face.mana_cost.iter().map(|symbol| {
            let style = symbol
                .color
                .and_then(|i| palette.mana.get(i).copied())
                .map_or(palette.muted, ratatui::style::Style::bold);
            Span::styled(format!("{{{}}}", symbol.text), style)
        }));
    }
    let mut lines = vec![Line::from(title)];
    if !face.type_line.is_empty() {
        lines.push(Line::from(face.type_line.clone()).italic());
    }
    lines.extend(face.rules.iter().map(|rule| Line::from(rule.clone())));
    if let Some(stats) = &face.stats {
        lines.push(Line::from(stats.clone()).bold());
    }
    lines
}

/// Every face of `data`, front first, split by a muted `//`.
pub(crate) fn lines(palette: &Palette, data: &ScryfallData) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for (i, face) in card::faces(data).iter().enumerate() {
        if i > 0 {
            lines.push(Line::styled("//", palette.muted));
        }
        lines.extend(face_lines(palette, face));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;

    #[test]
    fn a_card_reads_name_type_and_rules() {
        let mut card = test_support::card("Llanowar Elves");
        card.scryfall_data.type_line = Some("Creature — Elf Druid".to_owned());
        card.scryfall_data.oracle_text = Some("{T}: Add {G}.".to_owned());
        let text: Vec<String> = lines(&Palette::terminal(), &card.scryfall_data)
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(text[0], "Llanowar Elves");
        assert!(text.contains(&"Creature — Elf Druid".to_owned()));
        assert!(text.iter().any(|l| l.contains("Add {G}")));
        assert!(!text.contains(&"//".to_owned()), "one face, no split");
    }
}
