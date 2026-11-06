//! The chart and Colors bar pieces Home, Activity and Playstyle share.

use zwipe_core::http::contracts::suggestions::HttpColorMap;
use zwipe_ui::activity::Bar;
use zwipe_ui::colors::ColorCounts;

/// zwipe-ui's chart bars as the kit's chart draws them.
pub fn chart_bars(bars: Vec<Bar>) -> Vec<components::Bar> {
    bars.into_iter()
        .map(|bar| components::Bar {
            key: bar.key,
            label: bar.label,
            count: bar.count,
            pct: bar.pct,
            revealable: bar.revealable,
        })
        .collect()
}

/// The Colors bars for `map`, when it has a colored or colorless card on
/// it (zwipe-ui's rule): one per color in WUBRG order, then colorless.
pub fn color_bars(map: &HttpColorMap) -> Option<Vec<zwipe_ui::colors::Bar>> {
    let counts = ColorCounts::from(map);
    counts.has_cards().then(|| counts.bars())
}
