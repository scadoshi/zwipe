//! The color map: the five colors on the color pie's pentagon, each corner
//! glowing with its share of the cards and each of the ten color pairs a line
//! as strong as the decks that play both.
//!
//! The geometry and the scaling live in [`ColorMapModel`], a pure value the
//! tests pin; [`ColorMap`] draws it as an inline SVG with the app's mana pips
//! laid over the corners. The look is the `.color-map*` rules in `magic.css`,
//! which read the five colors from `--mana-white` through `--mana-green`.

use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::http::contracts::suggestions::HttpColorMap;

/// The SVG's width, in SVG units.
pub const VIEW_WIDTH: f64 = 240.0;
/// The SVG's height, in SVG units. Shorter than the width because the
/// pentagon's lower corners sit closer to its center than the top one.
pub const VIEW_HEIGHT: f64 = 224.0;
/// The pentagon's center.
const CENTER: Point = Point { x: 120.0, y: 118.0 };
/// From the center to each corner.
const RADIUS: f64 = 80.0;
/// Each corner's angle in degrees, clockwise from the right in SVG's
/// downward y, for W at the top then U, B, R and G clockwise: the standard
/// color pie.
const CORNER_DEGREES: [f64; 5] = [-90.0, -18.0, 54.0, 126.0, 198.0];

/// A corner's glow radius for the fewest cards above 0, just past the pip.
pub const GLOW_MIN_RADIUS: f64 = 16.0;
/// A corner's glow radius for the color with the most cards.
pub const GLOW_MAX_RADIUS: f64 = 34.0;

/// A pair line's stroke width with no decks: the faint guide.
pub const GUIDE_WIDTH: f64 = 1.0;
/// A pair line's stroke width for the pair with the most decks.
pub const LINE_MAX_WIDTH: f64 = 6.0;
/// A pair line's opacity with no decks: the faint guide.
pub const GUIDE_OPACITY: f64 = 0.2;
/// A pair line's opacity for the pair with the most decks.
pub const LINE_MAX_OPACITY: f64 = 0.9;

/// The ten color pairs in WUBRG order: `WU`, `WB`, `WR`, `WG`, `UB`, `UR`,
/// `UG`, `BR`, `BG`, `RG`. [`ColorMapCounts::pairs`] follows it.
pub const PAIRS: [(Color, Color); 10] = [
    (Color::White, Color::Blue),
    (Color::White, Color::Black),
    (Color::White, Color::Red),
    (Color::White, Color::Green),
    (Color::Blue, Color::Black),
    (Color::Blue, Color::Red),
    (Color::Blue, Color::Green),
    (Color::Black, Color::Red),
    (Color::Black, Color::Green),
    (Color::Red, Color::Green),
];

/// The counts a [`ColorMap`] draws.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ColorMapCounts {
    /// Cards whose color identity includes each color, in WUBRG order.
    pub cards: [u32; 5],
    /// Cards with an empty color identity, shown apart from the pentagon.
    pub colorless: u32,
    /// Decks whose identity includes both colors of each pair, in [`PAIRS`]
    /// order.
    pub pairs: [u32; 10],
}

impl From<&HttpColorMap> for ColorMapCounts {
    /// Reads each color and pair by name, so the order sent does not matter
    /// and a color or pair missing from `map` reads 0.
    fn from(map: &HttpColorMap) -> Self {
        let cards = Color::all().map(|color| {
            let short = color.to_short_name();
            map.colors
                .iter()
                .find(|c| c.color == short)
                .map_or(0, |c| c.cards)
        });
        let pairs = PAIRS.map(|(first, second)| {
            let (first, second) = (first.to_short_name(), second.to_short_name());
            map.pairs
                .iter()
                .find(|p| {
                    let [a, b] = &p.colors;
                    (*a == first && *b == second) || (*a == second && *b == first)
                })
                .map_or(0, |p| p.decks)
        });
        Self {
            cards,
            colorless: map.colorless_cards,
            pairs,
        }
    }
}

/// A position in SVG units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// From the left edge.
    pub x: f64,
    /// From the top edge.
    pub y: f64,
}

impl Point {
    /// The point as percentages of the view box, for laying HTML over it.
    #[must_use]
    pub fn percent(self) -> (f64, f64) {
        (self.x / VIEW_WIDTH * 100.0, self.y / VIEW_HEIGHT * 100.0)
    }
}

/// One color's corner of the pentagon.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corner {
    /// The corner's color.
    pub color: Color,
    /// Where the corner sits.
    pub at: Point,
    /// Its cards against the color with the most, 0 to 1.
    pub strength: f64,
    /// The glow's radius: area grows with `strength`, from
    /// [`GLOW_MIN_RADIUS`] to [`GLOW_MAX_RADIUS`]; [`GLOW_MIN_RADIUS`] with
    /// no cards, when the glow is not drawn and only its outline shows.
    pub glow_radius: f64,
}

impl Corner {
    /// Whether the corner's color has any cards, and so a glow.
    #[must_use]
    pub fn glows(&self) -> bool {
        self.strength > 0.0
    }
}

/// One color pair's line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PairLine {
    /// The pair in WUBRG order.
    pub colors: (Color, Color),
    /// The first color's corner.
    pub from: Point,
    /// The second color's corner.
    pub to: Point,
    /// Neighbors on the pie, a rim edge; otherwise an enemy pair, a line of
    /// the inner star.
    pub allied: bool,
    /// Its decks against the pair with the most, 0 to 1.
    pub strength: f64,
    /// From [`GUIDE_WIDTH`] to [`LINE_MAX_WIDTH`] with `strength`.
    pub width: f64,
    /// From [`GUIDE_OPACITY`] to [`LINE_MAX_OPACITY`] with `strength`.
    pub opacity: f64,
}

/// Everything a [`ColorMap`] draws, computed once from its counts.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorMapModel {
    /// The five corners in WUBRG order.
    pub corners: Vec<Corner>,
    /// The ten pair lines, weakest first so the strongest draw on top.
    pub lines: Vec<PairLine>,
    /// Cards with an empty color identity.
    pub colorless: u32,
    /// What a screen reader hears: the color with the most cards and the
    /// pair with the most decks.
    pub label: String,
}

/// `count` against `max`, 0 to 1; 0 when `max` is 0.
fn ratio(count: u32, max: u32) -> f64 {
    if max == 0 {
        0.0
    } else {
        f64::from(count) / f64::from(max)
    }
}

/// The corner of `color`.
#[must_use]
pub fn corner_point(color: Color) -> Point {
    let degrees = Color::all()
        .into_iter()
        .zip(CORNER_DEGREES)
        .find(|(c, _)| *c == color)
        .map_or(0.0, |(_, degrees)| degrees);
    let radians = degrees.to_radians();
    Point {
        x: CENTER.x + RADIUS * radians.cos(),
        y: CENTER.y + RADIUS * radians.sin(),
    }
}

/// Whether `first` and `second` sit side by side on the color pie.
fn allied(first: Color, second: Color) -> bool {
    let place = |color: Color| Color::all().iter().position(|c| *c == color);
    match (place(first), place(second)) {
        (Some(a), Some(b)) => matches!(a.abs_diff(b), 1 | 4),
        _ => false,
    }
}

/// The first of `items` with the largest count above 0, earlier winning ties.
fn leader<T: Copy>(items: impl IntoIterator<Item = (T, u32)>) -> Option<(T, u32)> {
    items
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .fold(None, |best, (item, count)| match best {
            Some((_, top)) if top >= count => best,
            _ => Some((item, count)),
        })
}

/// "1 card" or "`n` cards", and the same for any noun.
fn counted(n: u32, noun: &str) -> String {
    if n == 1 {
        format!("1 {noun}")
    } else {
        format!("{n} {noun}s")
    }
}

impl ColorMapModel {
    /// Lays out `counts`: each corner's glow scaled against the color with
    /// the most cards, each line against the pair with the most decks.
    #[must_use]
    pub fn new(counts: &ColorMapCounts) -> Self {
        let max_cards = counts.cards.iter().copied().max().unwrap_or(0);
        let corners = Color::all()
            .into_iter()
            .zip(counts.cards)
            .map(|(color, cards)| {
                let strength = ratio(cards, max_cards);
                let min_area = GLOW_MIN_RADIUS * GLOW_MIN_RADIUS;
                let max_area = GLOW_MAX_RADIUS * GLOW_MAX_RADIUS;
                Corner {
                    color,
                    at: corner_point(color),
                    strength,
                    glow_radius: (min_area + (max_area - min_area) * strength).sqrt(),
                }
            })
            .collect();

        let max_decks = counts.pairs.iter().copied().max().unwrap_or(0);
        let mut lines: Vec<PairLine> = PAIRS
            .into_iter()
            .zip(counts.pairs)
            .map(|((first, second), decks)| {
                let strength = ratio(decks, max_decks);
                PairLine {
                    colors: (first, second),
                    from: corner_point(first),
                    to: corner_point(second),
                    allied: allied(first, second),
                    strength,
                    width: GUIDE_WIDTH + (LINE_MAX_WIDTH - GUIDE_WIDTH) * strength,
                    opacity: GUIDE_OPACITY + (LINE_MAX_OPACITY - GUIDE_OPACITY) * strength,
                }
            })
            .collect();
        lines.sort_by(|a, b| a.strength.total_cmp(&b.strength));

        let top_color = leader(Color::all().into_iter().zip(counts.cards));
        let top_pair = leader(PAIRS.into_iter().zip(counts.pairs));
        let label = match (top_color, top_pair) {
            (None, _) => "Color map. No colored cards yet".to_string(),
            (Some((color, cards)), None) => format!(
                "Color map. {} leads with {}",
                color.to_long_name(),
                counted(cards, "card"),
            ),
            (Some((color, cards)), Some(((first, second), decks))) => format!(
                "Color map. {} leads with {}. {} and {} lead the pairs with {}",
                color.to_long_name(),
                counted(cards, "card"),
                first.to_long_name(),
                second.to_long_name().to_lowercase(),
                counted(decks, "deck"),
            ),
        };

        Self {
            corners,
            lines,
            colorless: counts.colorless,
            label,
        }
    }
}

/// The CSS custom property holding `color`'s mana fill.
fn mana_var(color: Color) -> &'static str {
    match color {
        Color::White => "--mana-white",
        Color::Blue => "--mana-blue",
        Color::Black => "--mana-black",
        Color::Red => "--mana-red",
        Color::Green => "--mana-green",
    }
}

/// The color pie as a heat map. Corners W at the top, then U, B, R and G
/// clockwise, each with the app's mana pip over a soft glow in its color
/// whose area grows with the color's cards. All ten pairs are lines (allied
/// pairs the rim, enemy pairs the inner star) in the theme's text color,
/// wider and stronger with the decks that play both colors; a pair with none
/// stays a faint guide so the pentagon always reads. Each glow also carries a
/// thin muted outline, so a corner reads even when its mana color matches
/// the background. Below, a chip counts the colorless cards when there are
/// any. Scales to its container up to about 15.5rem wide, centered.
#[component]
pub fn ColorMap(counts: ColorMapCounts) -> Element {
    let model = ColorMapModel::new(&counts);
    let view_box = format!("0 0 {VIEW_WIDTH} {VIEW_HEIGHT}");
    let colorless = model.colorless;
    rsx! {
        div { class: "color-map",
            div {
                class: "color-map-plot",
                role: "img",
                aria_label: "{model.label}",
                svg {
                    class: "color-map-svg",
                    view_box: "{view_box}",
                    "aria-hidden": "true",
                    defs {
                        for corner in model.corners.iter() {
                            radialGradient {
                                key: "{corner.color.to_short_name()}",
                                id: "color-map-glow-{corner.color.to_short_name()}",
                                stop {
                                    offset: "0%",
                                    style: "stop-color:var({mana_var(corner.color)});stop-opacity:0.95;",
                                }
                                stop {
                                    offset: "55%",
                                    style: "stop-color:var({mana_var(corner.color)});stop-opacity:0.55;",
                                }
                                stop {
                                    offset: "100%",
                                    style: "stop-color:var({mana_var(corner.color)});stop-opacity:0;",
                                }
                            }
                        }
                    }
                    for line in model.lines.iter() {
                        line {
                            key: "{line.colors.0.to_short_name()}{line.colors.1.to_short_name()}",
                            class: if line.allied { "color-map-line color-map-rim" } else { "color-map-line color-map-star" },
                            x1: "{line.from.x:.2}",
                            y1: "{line.from.y:.2}",
                            x2: "{line.to.x:.2}",
                            y2: "{line.to.y:.2}",
                            stroke_width: "{line.width:.2}",
                            stroke_opacity: "{line.opacity:.2}",
                        }
                    }
                    for corner in model.corners.iter() {
                        g { key: "{corner.color.to_short_name()}",
                            if corner.glows() {
                                circle {
                                    class: "color-map-glow",
                                    cx: "{corner.at.x:.2}",
                                    cy: "{corner.at.y:.2}",
                                    r: "{corner.glow_radius:.2}",
                                    fill: "url(#color-map-glow-{corner.color.to_short_name()})",
                                }
                            }
                            circle {
                                class: "color-map-outline",
                                cx: "{corner.at.x:.2}",
                                cy: "{corner.at.y:.2}",
                                r: "{corner.glow_radius:.2}",
                            }
                        }
                    }
                }
                for corner in model.corners.iter() {
                    {
                        let (left, top) = corner.at.percent();
                        let short = corner.color.to_short_name();
                        let class = format!("ms ms-{} ms-cost", short.to_lowercase());
                        rsx! {
                            span {
                                key: "{short}",
                                class: "identity-pips color-map-pip",
                                style: "left:{left:.3}%;top:{top:.3}%;",
                                i { class: "{class}" }
                            }
                        }
                    }
                }
            }
            if colorless > 0 {
                span { class: "stat-chip color-map-colorless",
                    i { class: "ms ms-c ms-cost" }
                    "Colorless {counted(colorless, \"card\")}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::http::contracts::suggestions::{HttpColorCards, HttpColorPair};

    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn line(model: &ColorMapModel, first: Color, second: Color) -> PairLine {
        *model
            .lines
            .iter()
            .find(|l| l.colors == (first, second))
            .unwrap()
    }

    #[test]
    fn corners_run_clockwise_from_white_at_the_top() {
        let w = corner_point(Color::White);
        let u = corner_point(Color::Blue);
        let b = corner_point(Color::Black);
        let r = corner_point(Color::Red);
        let g = corner_point(Color::Green);
        assert!(
            close(w.x, CENTER.x) && close(w.y, CENTER.y - RADIUS),
            "W top"
        );
        assert!(u.x > CENTER.x && u.y < CENTER.y, "U upper right");
        assert!(b.x > CENTER.x && b.y > CENTER.y, "B lower right");
        assert!(r.x < CENTER.x && r.y > CENTER.y, "R lower left");
        assert!(g.x < CENTER.x && g.y < CENTER.y, "G upper left");
        assert!(
            close(u.y, g.y) && close(b.y, r.y),
            "mirrored across the middle"
        );
        assert!(close(u.x - CENTER.x, CENTER.x - g.x));
        for p in [w, u, b, r, g] {
            let reach = GLOW_MAX_RADIUS;
            assert!(
                p.x - reach >= 0.0
                    && p.x + reach <= VIEW_WIDTH
                    && p.y - reach >= 0.0
                    && p.y + reach <= VIEW_HEIGHT,
                "the widest glow at {p:?} stays inside the view box"
            );
        }
    }

    #[test]
    fn allied_pairs_are_the_rim_and_enemy_pairs_the_star() {
        let model = ColorMapModel::new(&ColorMapCounts::default());
        let rim: Vec<(Color, Color)> = PAIRS
            .into_iter()
            .filter(|(a, b)| line(&model, *a, *b).allied)
            .collect();
        assert_eq!(
            rim,
            vec![
                (Color::White, Color::Blue),
                (Color::White, Color::Green),
                (Color::Blue, Color::Black),
                (Color::Black, Color::Red),
                (Color::Red, Color::Green),
            ]
        );
    }

    #[test]
    fn a_pair_with_no_decks_stays_a_faint_guide() {
        let counts = ColorMapCounts {
            pairs: [4, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ..ColorMapCounts::default()
        };
        let model = ColorMapModel::new(&counts);
        let guide = line(&model, Color::Red, Color::Green);
        assert!(close(guide.width, GUIDE_WIDTH) && close(guide.opacity, GUIDE_OPACITY));
        assert!(guide.opacity > 0.0, "a guide is never invisible");
        assert_eq!(model.lines.len(), 10, "every pair draws");

        let empty = ColorMapModel::new(&ColorMapCounts::default());
        assert!(
            empty
                .lines
                .iter()
                .all(|l| close(l.width, GUIDE_WIDTH) && close(l.opacity, GUIDE_OPACITY)),
            "with no decks at all, every line is a guide"
        );
    }

    #[test]
    fn the_strongest_pair_and_corner_are_full() {
        let counts = ColorMapCounts {
            cards: [10, 0, 5, 40, 20],
            colorless: 0,
            pairs: [0, 0, 6, 0, 0, 0, 0, 3, 0, 0],
        };
        let model = ColorMapModel::new(&counts);
        let boros = line(&model, Color::White, Color::Red);
        assert!(close(boros.strength, 1.0));
        assert!(close(boros.width, LINE_MAX_WIDTH) && close(boros.opacity, LINE_MAX_OPACITY));
        let rakdos = line(&model, Color::Black, Color::Red);
        assert!(close(rakdos.strength, 0.5), "half the top pair's decks");
        assert_eq!(
            model.lines.last().map(|l| l.colors),
            Some((Color::White, Color::Red)),
            "the strongest line draws last, on top"
        );

        let strengths: Vec<f64> = model.corners.iter().map(|c| c.strength).collect();
        assert_eq!(strengths, vec![0.25, 0.0, 0.125, 1.0, 0.5]);
        let red = model.corners.get(3).unwrap();
        assert!(close(red.glow_radius, GLOW_MAX_RADIUS));
        let blue = model.corners.get(1).unwrap();
        assert!(!blue.glows() && close(blue.glow_radius, GLOW_MIN_RADIUS));
        let green = model.corners.get(4).unwrap();
        let area = |r: f64| r * r - GLOW_MIN_RADIUS * GLOW_MIN_RADIUS;
        assert!(
            close(area(green.glow_radius), area(GLOW_MAX_RADIUS) / 2.0),
            "glow area past the pip grows with the share"
        );
    }

    #[test]
    fn the_label_names_the_top_color_and_pair() {
        let counts = ColorMapCounts {
            cards: [10, 0, 5, 40, 20],
            colorless: 3,
            pairs: [0, 0, 6, 0, 0, 0, 0, 6, 0, 0],
        };
        assert_eq!(
            ColorMapModel::new(&counts).label,
            "Color map. Red leads with 40 cards. White and red lead the pairs with 6 decks",
            "WUBRG order breaks the tie"
        );
        let mono = ColorMapCounts {
            cards: [0, 1, 0, 0, 0],
            ..ColorMapCounts::default()
        };
        assert_eq!(
            ColorMapModel::new(&mono).label,
            "Color map. Blue leads with 1 card"
        );
        assert_eq!(
            ColorMapModel::new(&ColorMapCounts::default()).label,
            "Color map. No colored cards yet"
        );
    }

    #[test]
    fn counts_read_the_contract_by_name() {
        let map = HttpColorMap {
            colors: vec![
                HttpColorCards {
                    color: "R".to_string(),
                    cards: 9,
                },
                HttpColorCards {
                    color: "W".to_string(),
                    cards: 4,
                },
            ],
            colorless_cards: 2,
            pairs: vec![HttpColorPair {
                colors: ["G".to_string(), "U".to_string()],
                decks: 3,
            }],
        };
        assert_eq!(
            ColorMapCounts::from(&map),
            ColorMapCounts {
                cards: [4, 0, 0, 9, 0],
                colorless: 2,
                pairs: [0, 0, 0, 0, 0, 0, 3, 0, 0, 0],
            },
            "missing entries read 0 and a reversed pair still matches"
        );
    }
}
