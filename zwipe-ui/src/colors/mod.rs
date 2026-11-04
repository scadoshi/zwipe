//! Color counts across the user's decks and the bars that draw them: one per color in WUBRG order, then colorless, each with its mana pip and card count.
//!
//! Home's Colors panel and Playstyle's Colors section draw the same [`Bar`]s from the same [`ColorCounts`], so the two never disagree.

#[cfg(test)]
mod tests;

use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::http::contracts::suggestions::HttpColorMap;

/// The counts the bars draw.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ColorCounts {
    /// Cards whose color identity includes each color, in WUBRG order.
    pub cards: [u32; 5],
    /// Cards with an empty color identity.
    pub colorless: u32,
}

impl From<&HttpColorMap> for ColorCounts {
    /// Reads each color by name, so the order sent does not matter and a color missing from `map` reads 0.
    fn from(map: &HttpColorMap) -> Self {
        let cards = Color::all().map(|color| {
            let short = color.to_short_name();
            map.colors
                .iter()
                .find(|c| c.color == short)
                .map_or(0, |c| c.cards)
        });
        Self {
            cards,
            colorless: map.colorless_cards,
        }
    }
}

/// One color bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bar {
    /// The bar's color, `None` for the colorless bar.
    pub color: Option<Color>,
    /// Cards whose color identity includes the color, or that have none.
    pub count: u32,
    /// The bar's fill as a share of the largest count, 0 to 100.
    pub pct: u32,
}

/// The bars for `counts`: one per color in WUBRG order, then colorless, the largest count at 100.
#[must_use]
pub fn bars(counts: &ColorCounts) -> Vec<Bar> {
    let max = counts
        .cards
        .iter()
        .copied()
        .chain([counts.colorless])
        .max()
        .unwrap_or(0)
        .max(1);
    let bar = |color: Option<Color>, count: u32| Bar {
        color,
        count,
        pct: count.saturating_mul(100).checked_div(max).unwrap_or(0),
    };
    Color::all()
        .into_iter()
        .zip(counts.cards)
        .map(|(color, count)| bar(Some(color), count))
        .chain([bar(None, counts.colorless)])
        .collect()
}

/// Whether the counts have anything to draw: a colored or colorless card.
#[must_use]
pub fn has_data(counts: &ColorCounts) -> bool {
    counts.colorless > 0 || counts.cards.iter().any(|n| *n > 0)
}
