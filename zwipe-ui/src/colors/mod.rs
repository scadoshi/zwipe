//! Color counts and the bars that draw them: one per color in WUBRG order, then colorless, each with its mana pip and card count.
//!
//! Home's Colors panel, Playstyle's Colors section and a deck's Colors chart draw the same [`Bar`]s from the same [`ColorCounts`], so the three never disagree: the largest count fills its bar and any other nonzero count draws at least a sliver.

#[cfg(test)]
mod tests;

use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::http::contracts::suggestions::HttpColorMap;

/// The smallest fill of a bar with a nonzero count, in percent, so a small count still shows.
const BAR_SLIVER_PCT: u32 = 4;

/// The counts the bars draw.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ColorCounts {
    /// Cards whose color identity includes each color, in WUBRG order.
    pub cards: [u32; 5],
    /// Cards with an empty color identity.
    pub colorless: u32,
}

impl ColorCounts {
    /// The bars: one per color in WUBRG order, then colorless, the largest count at 100 and any other nonzero count at least [`BAR_SLIVER_PCT`].
    #[must_use]
    pub fn bars(&self) -> Vec<Bar> {
        let max = self
            .cards
            .iter()
            .copied()
            .chain([self.colorless])
            .max()
            .unwrap_or(0)
            .max(1);
        let bar = |color: Option<Color>, count: u32| Bar {
            color,
            count,
            pct: bar_pct(count, max),
        };
        Color::all()
            .into_iter()
            .zip(self.cards)
            .map(|(color, count)| bar(Some(color), count))
            .chain([bar(None, self.colorless)])
            .collect()
    }

    /// Whether the counts have anything to draw: a colored or colorless card.
    #[must_use]
    pub fn has_cards(&self) -> bool {
        self.colorless > 0 || self.cards.iter().any(|n| *n > 0)
    }
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

impl From<&DeckMetrics> for ColorCounts {
    /// A deck's color buckets; a count past `u32::MAX` saturates.
    fn from(metrics: &DeckMetrics) -> Self {
        let (cards, colorless) = metrics.color_buckets();
        Self {
            cards: cards.map(saturated),
            colorless: saturated(colorless),
        }
    }
}

/// `count` as `u32`, `u32::MAX` past it.
fn saturated(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// `count` as a share of `max` (at least 1), 0 to 100: a nonzero count reads at least [`BAR_SLIVER_PCT`].
fn bar_pct(count: u32, max: u32) -> u32 {
    if count == 0 {
        return 0;
    }
    count
        .saturating_mul(100)
        .checked_div(max)
        .unwrap_or(0)
        .max(BAR_SLIVER_PCT)
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
