//! The color map: a field of square tiles, each owned by one of the five
//! colors, so the colors read as territories pushing into each other.
//!
//! The layout lives in [`ColorMapModel`], a pure value the tests pin;
//! [`ColorMap`] draws it as an inline SVG with a key of mana pips and card
//! counts below. The look is the `.color-map*` rules in `magic.css`, and each
//! tile's fill is one of `--mana-white` through `--mana-green`.
//!
//! The model is stable: the same seed and counts always give the same map,
//! and a small change in the counts moves only a few tiles.
//!
//! 1. The active cells are the cells of a fixed square grid nearest its
//!    center, ranked by distance stretched by a smooth seeded noise field.
//!    The ranking does not depend on the tile count, so a larger map is the
//!    smaller one plus a ring of new cells, and its edge comes out ragged and
//!    roughly round.
//! 2. Each color has an anchor out from the center toward its home on the
//!    color pie: W top, then U, B, R and G clockwise. A cell's affinity for a
//!    color is minus its distance to that anchor, measured from the cell
//!    moved by a smooth seeded warp that every color shares, so borders bend
//!    and interlock while each region stays in one piece. A faint noise per
//!    color roughens the borders.
//! 3. Cells go to colors by an optimal assignment: the one with the highest
//!    total affinity that gives every color exactly its quota. A greedy fill
//!    starts it and cycle canceling finishes it. An optimal assignment moves
//!    continuously with its quotas, so a color gaining a tile pushes its
//!    borders out by a tile instead of redrawing the map.

use std::f64::consts::PI;

use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::http::contracts::suggestions::HttpColorMap;

/// The SVG's width, in SVG units. Its height follows the map's rows.
pub const VIEW_WIDTH: f64 = 240.0;
/// Cards per tile.
pub const CARDS_PER_TILE: u32 = 4;
/// The fewest tiles a map with any colored card draws.
pub const MIN_TILES: u32 = 40;
/// The most tiles a map draws.
pub const MAX_TILES: u32 = 520;
/// A map narrower than this many cells keeps the tile size this span gives,
/// so a small collection draws a smaller map; a wider one shrinks its tiles
/// to fit [`VIEW_WIDTH`].
pub const MIN_SPAN: i32 = 16;
/// The gap between tiles, as a share of the tile pitch.
const GAP_SHARE: f64 = 0.12;
/// The tile corner radius, as a share of the tile pitch.
const CORNER_SHARE: f64 = 0.18;

/// The candidate grid runs from `-GRID_REACH` to `GRID_REACH` on both axes,
/// room for [`MAX_TILES`] cells with the edge noise at its widest.
const GRID_REACH: i32 = 18;
/// How far the edge noise stretches a cell's distance from the center, as a
/// share of that distance.
const EDGE_WOBBLE: f64 = 0.12;
/// The edge noise's lattice spacing, in cells.
const EDGE_PERIOD: i32 = 5;
/// Each anchor's distance from the center, as a share of the map's radius.
const ANCHOR_SHARE: f64 = 0.6;
/// How far the warp moves a cell before its distance to each anchor is
/// measured, as a share of the map's radius. Every color sees the same warp,
/// so borders bend and interlock while each region stays in one piece.
const WARP: f64 = 0.3;
/// The warp's lattice spacing, in cells.
const WARP_PERIOD: i32 = 8;
/// How far each color's own noise moves its affinity, as a share of the
/// map's radius.
const BORDER_NOISE: f64 = 0.05;
/// The border noise's lattice spacing, in cells.
const BORDER_PERIOD: i32 = 3;
/// A per-cell jitter, small enough only to break ties.
const TIE_JITTER: f64 = 1e-4;
/// Each color's home on the color pie, in degrees clockwise from the right
/// in SVG's downward y, in WUBRG order: W at the top, then clockwise.
const HOME_DEGREES: [f64; 5] = [-90.0, -18.0, 54.0, 126.0, 198.0];
/// A stop for cycle canceling, far past what a map of [`MAX_TILES`] needs.
const MAX_IMPROVEMENTS: usize = 20_000;

/// Salts that keep the noise fields independent.
const EDGE_SALT: u64 = 0x6564_6765;
const EDGE_JITTER_SALT: u64 = 0x6a69_7474;
const BORDER_SALT: u64 = 0x626f_7264;
const WARP_X_SALT: u64 = 0x7761_7278;
const WARP_Y_SALT: u64 = 0x7761_7279;
const TIE_SALT: u64 = 0x7469_6573;

/// The counts a [`ColorMap`] draws.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ColorMapCounts {
    /// Cards whose color identity includes each color, in WUBRG order.
    pub cards: [u32; 5],
    /// Cards with an empty color identity, shown only in the key.
    pub colorless: u32,
}

impl From<&HttpColorMap> for ColorMapCounts {
    /// Reads each color by name, so the order sent does not matter and a
    /// color missing from `map` reads 0.
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

/// One tile of the map.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tile {
    /// The color that owns the tile.
    pub color: Color,
    /// The tile's grid cell, column then row, with the grid's center at 0.
    pub cell: (i32, i32),
    /// The tile's left edge, in SVG units.
    pub x: f64,
    /// The tile's top edge, in SVG units.
    pub y: f64,
}

/// Everything a [`ColorMap`] draws, computed once from its counts and seed.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorMapModel {
    /// Every tile, in grid order (by row, then column).
    pub tiles: Vec<Tile>,
    /// Each tile's side, in SVG units; the same for every tile in the map.
    pub tile_size: f64,
    /// Each tile's corner radius, in SVG units.
    pub corner: f64,
    /// The SVG's height, in SVG units: the map's rows and no more.
    pub height: f64,
    /// What a screen reader hears: each color's share of the colored cards.
    pub label: String,
}

/// splitmix64's finalizer: a strong mix of `z`'s bits.
fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// A seed for [`ColorMap`] from a 128-bit id such as a user's UUID.
#[must_use]
pub fn seed_from_id(id: u128) -> u64 {
    let high = u64::try_from(id >> 64).unwrap_or(0);
    let low = u64::try_from(id & u128::from(u64::MAX)).unwrap_or(0);
    mix(high ^ mix(low))
}

/// A hash of the seed, a salt and a grid point.
fn hash(seed: u64, salt: u64, x: i32, y: i32) -> u64 {
    let point = (u64::from(x.cast_unsigned()) << 32) | u64::from(y.cast_unsigned());
    mix(seed ^ mix(salt ^ mix(point)))
}

/// `h` as a number from 0 up to 1.
fn unit(h: u64) -> f64 {
    f64::from(u32::try_from(h >> 32).unwrap_or(0)) / 4_294_967_296.0
}

/// Smooth value noise from 0 to 1 at cell `(x, y)`: seeded values on a
/// lattice `period` cells apart, blended between with a smoothstep.
fn value_noise(seed: u64, salt: u64, x: i32, y: i32, period: i32) -> f64 {
    let (ix, iy) = (x.div_euclid(period), y.div_euclid(period));
    let span = f64::from(period);
    let smooth = |t: f64| t * t * (3.0 - 2.0 * t);
    let tx = smooth(f64::from(x.rem_euclid(period)) / span);
    let ty = smooth(f64::from(y.rem_euclid(period)) / span);
    let at = |dx: i32, dy: i32| unit(hash(seed, salt, ix + dx, iy + dy));
    let top = at(0, 0) + (at(1, 0) - at(0, 0)) * tx;
    let bottom = at(0, 1) + (at(1, 1) - at(0, 1)) * tx;
    top + (bottom - top) * ty
}

/// The map's tile count for `colored` cards: one tile per
/// [`CARDS_PER_TILE`] cards, rounded, from [`MIN_TILES`] to [`MAX_TILES`];
/// none without colored cards.
#[must_use]
pub fn tile_total(colored: u64) -> u32 {
    if colored == 0 {
        return 0;
    }
    let tiles = (colored + u64::from(CARDS_PER_TILE / 2)) / u64::from(CARDS_PER_TILE);
    u32::try_from(tiles)
        .unwrap_or(MAX_TILES)
        .clamp(MIN_TILES, MAX_TILES)
}

/// Each color's tiles out of `total`, in proportion to `cards` by largest
/// remainder, so they sum to `total`. Ties for a leftover tile go in WUBRG
/// order.
#[must_use]
pub fn quotas(cards: [u32; 5], total: u32) -> [u32; 5] {
    let sum: u64 = cards.iter().map(|&n| u64::from(n)).sum();
    if sum == 0 {
        return [0; 5];
    }
    let share = |n: u32| u64::from(total) * u64::from(n);
    let mut quotas = cards.map(|n| u32::try_from(share(n) / sum).unwrap_or(0));
    let given: u32 = quotas.iter().sum();
    let mut order: Vec<usize> = (0..5).collect();
    order.sort_by_key(|&i| {
        let remainder = cards.get(i).map_or(0, |&n| share(n) % sum);
        (std::cmp::Reverse(remainder), i)
    });
    let leftover = usize::try_from(total.saturating_sub(given)).unwrap_or(0);
    for i in order.into_iter().take(leftover) {
        if let Some(q) = quotas.get_mut(i) {
            *q += 1;
        }
    }
    quotas
}

/// The `total` cells nearest the grid's center, ranked by distance stretched
/// by the seed's edge noise, in rank order. The ranking is fixed per seed,
/// so a larger `total` keeps every cell a smaller one picks.
fn active_cells(seed: u64, total: usize) -> Vec<(i32, i32)> {
    let mut ranked: Vec<(f64, (i32, i32))> = (-GRID_REACH..=GRID_REACH)
        .flat_map(|y| (-GRID_REACH..=GRID_REACH).map(move |x| (x, y)))
        .map(|(x, y)| {
            let distance = f64::from(x).hypot(f64::from(y));
            let wobble = 2.0 * value_noise(seed, EDGE_SALT, x, y, EDGE_PERIOD) - 1.0;
            let jitter = 0.35 * unit(hash(seed, EDGE_JITTER_SALT, x, y));
            (distance * (1.0 + EDGE_WOBBLE * wobble) + jitter, (x, y))
        })
        .collect();
    ranked.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    ranked
        .into_iter()
        .take(total)
        .map(|(_, cell)| cell)
        .collect()
}

/// Each cell's affinity for each color: minus the distance from the warped
/// cell to the color's anchor, plus the color's border noise, both as shares
/// of the map's radius.
fn affinities(seed: u64, cells: &[(i32, i32)], radius: f64) -> Vec<[f64; 5]> {
    let anchors = HOME_DEGREES.map(|degrees| {
        let radians = f64::to_radians(degrees);
        (
            ANCHOR_SHARE * radius * radians.cos(),
            ANCHOR_SHARE * radius * radians.sin(),
        )
    });
    cells
        .iter()
        .map(|&(x, y)| {
            let warp = |salt: u64| {
                WARP * radius * (2.0 * value_noise(seed, salt, x, y, WARP_PERIOD) - 1.0)
            };
            let (wx, wy) = (
                f64::from(x) + warp(WARP_X_SALT),
                f64::from(y) + warp(WARP_Y_SALT),
            );
            let mut row = [0.0; 5];
            for ((color, slot), (ax, ay)) in (0_u64..).zip(row.iter_mut()).zip(anchors) {
                let reach = (wx - ax).hypot(wy - ay) / radius;
                let salt = BORDER_SALT ^ mix(color);
                let noise = 2.0 * value_noise(seed, salt, x, y, BORDER_PERIOD) - 1.0;
                let tie = TIE_JITTER * unit(hash(seed, TIE_SALT ^ color, x, y));
                *slot = -reach + BORDER_NOISE * noise + tie;
            }
            row
        })
        .collect()
}

/// Each cell's color index, the assignment with the highest total affinity
/// that gives each color exactly its quota. `quotas` sums to the cell count.
fn assign(scores: &[[f64; 5]], quotas: [u32; 5]) -> Vec<usize> {
    // Greedy start: every (cell, color) pair by affinity, best first, each
    // cell to the first color in that order that has room left.
    let mut pairs: Vec<(usize, usize)> = (0..scores.len())
        .flat_map(|cell| (0..5).map(move |color| (cell, color)))
        .filter(|&(_, color)| quotas.get(color).is_some_and(|&q| q > 0))
        .collect();
    let score = |cell: usize, color: usize| {
        scores
            .get(cell)
            .and_then(|row| row.get(color))
            .copied()
            .unwrap_or(f64::NEG_INFINITY)
    };
    pairs.sort_by(|a, b| score(b.0, b.1).total_cmp(&score(a.0, a.1)).then(a.cmp(b)));
    let mut room = quotas;
    let mut owner: Vec<Option<usize>> = vec![None; scores.len()];
    for (cell, color) in pairs {
        let (Some(slot), Some(left)) = (owner.get_mut(cell), room.get_mut(color)) else {
            continue;
        };
        if slot.is_none() && *left > 0 {
            *slot = Some(color);
            *left -= 1;
        }
    }
    let mut owner: Vec<usize> = owner.into_iter().map(|o| o.unwrap_or(0)).collect();

    // Cycle canceling: while moving one cell along each edge of some cycle
    // of colors raises the total affinity, move them. Quotas hold because
    // every color on a cycle gives one cell and takes one.
    for _ in 0..MAX_IMPROVEMENTS {
        let Some(moves) = best_cycle(scores, &owner) else {
            break;
        };
        for (cell, to) in moves {
            if let Some(slot) = owner.get_mut(cell) {
                *slot = to;
            }
        }
    }
    owner
}

/// The cycle of colors whose best moves raise the total affinity most, as
/// (cell, new color) moves, or `None` when no cycle raises it.
fn best_cycle(scores: &[[f64; 5]], owner: &[usize]) -> Option<Vec<(usize, usize)>> {
    // best[from][to]: the cell of `from` that gains most by moving to `to`.
    let mut best: [[Option<(f64, usize)>; 5]; 5] = [[None; 5]; 5];
    for (cell, (row, &from)) in scores.iter().zip(owner).enumerate() {
        let Some(own) = row.get(from) else { continue };
        let Some(edges) = best.get_mut(from) else {
            continue;
        };
        for (to, (edge, theirs)) in edges.iter_mut().zip(row).enumerate() {
            let gain = theirs - own;
            if to != from && edge.is_none_or(|(g, _)| gain > g) {
                *edge = Some((gain, cell));
            }
        }
    }
    let edge = |from: usize, to: usize| {
        best.get(from)
            .and_then(|row| row.get(to))
            .copied()
            .flatten()
    };

    let mut top: Option<(f64, Vec<usize>)> = None;
    let mut path = Vec::with_capacity(5);
    for start in 0..5 {
        path.clear();
        path.push(start);
        extend_cycles(&edge, &mut path, 0.0, &mut top);
    }
    let (gain, cycle) = top?;
    if gain <= 1e-12 {
        return None;
    }
    let moves = cycle
        .iter()
        .zip(cycle.iter().cycle().skip(1))
        .filter_map(|(&from, &to)| edge(from, to).map(|(_, cell)| (cell, to)))
        .collect();
    Some(moves)
}

/// Walks every simple cycle that starts at `path`'s first color and only
/// visits higher colors after it, keeping the one with the largest gain.
fn extend_cycles(
    edge: &impl Fn(usize, usize) -> Option<(f64, usize)>,
    path: &mut Vec<usize>,
    gain: f64,
    top: &mut Option<(f64, Vec<usize>)>,
) {
    let (Some(&start), Some(&last)) = (path.first(), path.last()) else {
        return;
    };
    if path.len() > 1
        && let Some((back, _)) = edge(last, start)
    {
        let total = gain + back;
        if top.as_ref().is_none_or(|(g, _)| total > *g) {
            *top = Some((total, path.clone()));
        }
    }
    for next in (start + 1)..5 {
        if path.contains(&next) {
            continue;
        }
        let Some((step, _)) = edge(last, next) else {
            continue;
        };
        path.push(next);
        extend_cycles(edge, path, gain + step, top);
        path.pop();
    }
}

/// Each colored share, largest first with WUBRG breaking ties, as "Blue 31
/// percent, red 29 percent", then the colorless count.
fn label(counts: &ColorMapCounts) -> String {
    let sum: u64 = counts.cards.iter().map(|&n| u64::from(n)).sum();
    let mut shares: Vec<(Color, u32)> = Color::all()
        .into_iter()
        .zip(counts.cards)
        .filter(|(_, n)| *n > 0)
        .collect();
    shares.sort_by_key(|&(_, n)| std::cmp::Reverse(n));
    let parts: Vec<String> = shares
        .into_iter()
        .enumerate()
        .map(|(i, (color, n))| {
            let percent = (u64::from(n) * 100 + sum / 2) / sum.max(1);
            let name = color.to_long_name();
            let name = if i == 0 { name } else { name.to_lowercase() };
            format!("{name} {percent} percent")
        })
        .collect();
    let mut label = if parts.is_empty() {
        "Color map. No colored cards yet".to_string()
    } else {
        format!("Color map. {}", parts.join(", "))
    };
    if counts.colorless > 0 {
        label.push_str(&format!(". Colorless {}", counted(counts.colorless)));
    }
    label
}

/// "1 card" or "`n` cards".
fn counted(n: u32) -> String {
    if n == 1 {
        "1 card".to_string()
    } else {
        format!("{n} cards")
    }
}

impl ColorMapModel {
    /// Lays out `counts` for `seed`: one tile per few colored cards, shared
    /// out by color in proportion, the same map for the same inputs.
    #[must_use]
    pub fn new(counts: &ColorMapCounts, seed: u64) -> Self {
        let colored: u64 = counts.cards.iter().map(|&n| u64::from(n)).sum();
        let total = tile_total(colored);
        let quotas = quotas(counts.cards, total);
        let mut cells = active_cells(seed, usize::try_from(total).unwrap_or(0));
        cells.sort_by_key(|&(x, y)| (y, x));
        let radius = (f64::from(total) / PI).sqrt().max(1.0);
        let scores = affinities(seed, &cells, radius);
        let owner = assign(&scores, quotas);

        let bounds = |pick: fn(&(i32, i32)) -> i32| {
            let low = cells.iter().map(pick).min().unwrap_or(0);
            let high = cells.iter().map(pick).max().unwrap_or(0);
            (low, high - low + 1)
        };
        let (min_x, span_x) = bounds(|c| c.0);
        let (min_y, span_y) = bounds(|c| c.1);
        let pitch = VIEW_WIDTH / f64::from(span_x.max(span_y).max(MIN_SPAN));
        let gap = pitch * GAP_SHARE;
        let offset_x = (VIEW_WIDTH - f64::from(span_x) * pitch) / 2.0;
        let palette = Color::all();
        let tiles = cells
            .iter()
            .zip(owner)
            .filter_map(|(&(x, y), color)| {
                Some(Tile {
                    color: *palette.get(color)?,
                    cell: (x, y),
                    x: offset_x + f64::from(x - min_x) * pitch + gap / 2.0,
                    y: f64::from(y - min_y) * pitch + gap / 2.0,
                })
            })
            .collect();
        Self {
            tiles,
            tile_size: pitch - gap,
            corner: pitch * CORNER_SHARE,
            height: if cells.is_empty() {
                0.0
            } else {
                f64::from(span_y) * pitch
            },
            label: label(counts),
        }
    }
}

/// The CSS custom property holding `color`'s mana fill.
#[must_use]
pub fn mana_var(color: Color) -> &'static str {
    match color {
        Color::White => "--mana-white",
        Color::Blue => "--mana-blue",
        Color::Black => "--mana-black",
        Color::Red => "--mana-red",
        Color::Green => "--mana-green",
    }
}

/// The color territory map. Square tiles, each filled with one color's mana
/// fill, one tile per few cards and each color's share in proportion to its
/// cards, every color growing out from its home on the color pie (W at the
/// top, then U, B, R and G clockwise). `seed` fixes the map's shape: pass the
/// same one for the same player, such as [`seed_from_id`] of their id. Under
/// the map, a key of chips gives each color's pip and card count, then the
/// colorless cards, which the map leaves out. Scales to its container up to
/// about 15.5rem wide, centered.
#[component]
pub fn ColorMap(counts: ColorMapCounts, seed: u64) -> Element {
    let model = ColorMapModel::new(&counts, seed);
    let view_box = format!("0 0 {VIEW_WIDTH} {:.2}", model.height);
    let size = format!("{:.2}", model.tile_size);
    let corner = format!("{:.2}", model.corner);
    let key = Color::all()
        .into_iter()
        .zip(counts.cards)
        .filter(|(_, n)| *n > 0);
    let colorless = counts.colorless;
    rsx! {
        div { class: "color-map",
            if !model.tiles.is_empty() {
                div {
                    class: "color-map-plot",
                    role: "img",
                    aria_label: "{model.label}",
                    svg {
                        class: "color-map-svg",
                        view_box: "{view_box}",
                        "aria-hidden": "true",
                        for tile in model.tiles.iter() {
                            rect {
                                key: "{tile.cell.0},{tile.cell.1}",
                                x: "{tile.x:.2}",
                                y: "{tile.y:.2}",
                                width: "{size}",
                                height: "{size}",
                                rx: "{corner}",
                                style: "fill:var({mana_var(tile.color)});",
                            }
                        }
                    }
                }
            }
            div { class: "color-map-key",
                for (color, cards) in key {
                    span {
                        key: "{color.to_short_name()}",
                        class: "stat-chip color-map-chip",
                        i { class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost" }
                        "{cards}"
                    }
                }
                if colorless > 0 {
                    span { class: "stat-chip color-map-chip",
                        i { class: "ms ms-c ms-cost" }
                        "Colorless {colorless}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use zwipe_core::http::contracts::suggestions::HttpColorCards;

    use super::*;

    const SEED: u64 = 0x5eed_c0de_1234_5678;

    fn sample() -> ColorMapCounts {
        ColorMapCounts {
            cards: [180, 200, 70, 190, 80],
            colorless: 189,
        }
    }

    fn by_cell(model: &ColorMapModel) -> BTreeMap<(i32, i32), Color> {
        model.tiles.iter().map(|t| (t.cell, t.color)).collect()
    }

    fn tally(model: &ColorMapModel) -> [u32; 5] {
        Color::all().map(|color| {
            u32::try_from(model.tiles.iter().filter(|t| t.color == color).count()).unwrap()
        })
    }

    /// Tiles outside their color's largest 4-connected region.
    fn strays(model: &ColorMapModel) -> usize {
        let cells = by_cell(model);
        let mut seen: BTreeMap<(i32, i32), ()> = BTreeMap::new();
        let mut largest = [0_usize; 5];
        for (&start, &color) in &cells {
            if seen.contains_key(&start) {
                continue;
            }
            let mut stack = vec![start];
            seen.insert(start, ());
            let mut size = 0;
            while let Some((x, y)) = stack.pop() {
                size += 1;
                for next in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                    if cells.get(&next) == Some(&color) && !seen.contains_key(&next) {
                        seen.insert(next, ());
                        stack.push(next);
                    }
                }
            }
            let i = Color::all().iter().position(|c| *c == color).unwrap();
            let best = largest.get_mut(i).unwrap();
            *best = (*best).max(size);
        }
        tally(model)
            .into_iter()
            .zip(largest)
            .map(|(n, best)| usize::try_from(n).unwrap() - best)
            .sum()
    }

    #[test]
    fn the_same_seed_and_counts_draw_the_same_map() {
        assert_eq!(
            ColorMapModel::new(&sample(), SEED),
            ColorMapModel::new(&sample(), SEED)
        );
        assert_ne!(
            by_cell(&ColorMapModel::new(&sample(), SEED)),
            by_cell(&ColorMapModel::new(&sample(), SEED + 1)),
            "another seed draws another shape"
        );
    }

    #[test]
    fn tiles_follow_cards_within_the_clamp() {
        assert_eq!(tile_total(0), 0);
        assert_eq!(tile_total(1), MIN_TILES);
        assert_eq!(tile_total(720), 180);
        assert_eq!(tile_total(722), 181, "rounds half up");
        assert_eq!(tile_total(1_000_000), MAX_TILES);
        let small = ColorMapCounts {
            cards: [3, 0, 0, 1, 0],
            colorless: 0,
        };
        assert_eq!(ColorMapModel::new(&small, SEED).tiles.len(), 40);
        let huge = ColorMapCounts {
            cards: [9000, 9000, 9000, 9000, 9000],
            colorless: 0,
        };
        assert_eq!(ColorMapModel::new(&huge, SEED).tiles.len(), 520);
    }

    #[test]
    fn quotas_are_exact_by_largest_remainder() {
        assert_eq!(quotas([1, 1, 1, 0, 0], 40), [14, 13, 13, 0, 0]);
        assert_eq!(quotas([180, 200, 70, 190, 80], 180), [45, 50, 18, 47, 20]);
        for (cards, seed) in [
            (sample().cards, SEED),
            ([30, 12, 0, 44, 8], 7),
            ([0, 0, 0, 1, 0], 9),
            ([500, 3, 2, 1, 900], 11),
        ] {
            let counts = ColorMapCounts {
                cards,
                colorless: 0,
            };
            let model = ColorMapModel::new(&counts, seed);
            let total = tile_total(cards.iter().map(|&n| u64::from(n)).sum());
            assert_eq!(tally(&model), quotas(cards, total), "cards {cards:?}");
            assert_eq!(quotas(cards, total).iter().sum::<u32>(), total);
        }
    }

    #[test]
    fn colors_hold_together() {
        for seed in [SEED, 1, 2, 3, 42, 99] {
            for cards in [
                sample().cards,
                [30, 12, 0, 44, 8],
                [100, 100, 100, 100, 100],
            ] {
                let model = ColorMapModel::new(
                    &ColorMapCounts {
                        cards,
                        colorless: 0,
                    },
                    seed,
                );
                let strays = strays(&model);
                assert!(
                    strays * 50 <= model.tiles.len().max(150),
                    "{strays} stray tiles of {} for seed {seed}, cards {cards:?}",
                    model.tiles.len()
                );
            }
        }
    }

    #[test]
    fn a_small_change_moves_a_few_tiles() {
        for seed in [SEED, 1, 2, 3, 42, 99] {
            for i in 0..5 {
                let before = ColorMapModel::new(&sample(), seed);
                let mut grown = sample();
                *grown.cards.get_mut(i).unwrap() += 5;
                let after = ColorMapModel::new(&grown, seed);
                let (before, after) = (by_cell(&before), by_cell(&after));
                assert!(
                    before.keys().all(|cell| after.contains_key(cell)),
                    "a larger map keeps every cell"
                );
                let moved = before
                    .iter()
                    .filter(|(cell, c)| after.get(cell) != Some(c))
                    .count();
                assert!(
                    moved <= 12,
                    "{moved} tiles moved for seed {seed}, color {i}"
                );
            }
        }
    }

    #[test]
    fn colors_lean_toward_their_home_on_the_pie() {
        let counts = ColorMapCounts {
            cards: [100; 5],
            colorless: 0,
        };
        for seed in [SEED, 1, 2] {
            let model = ColorMapModel::new(&counts, seed);
            let center = |color: Color| {
                let own: Vec<&Tile> = model.tiles.iter().filter(|t| t.color == color).collect();
                let n = f64::from(u32::try_from(own.len()).unwrap());
                let x: f64 = own.iter().map(|t| f64::from(t.cell.0)).sum::<f64>() / n;
                let y: f64 = own.iter().map(|t| f64::from(t.cell.1)).sum::<f64>() / n;
                (x, y)
            };
            let (w, u, b, r, g) = (
                center(Color::White),
                center(Color::Blue),
                center(Color::Black),
                center(Color::Red),
                center(Color::Green),
            );
            assert!(w.1 < -2.0, "W top, seed {seed}");
            assert!(u.0 > 2.0 && u.1 < 0.0, "U upper right, seed {seed}");
            assert!(b.0 > 0.0 && b.1 > 2.0, "B lower right, seed {seed}");
            assert!(r.0 < 0.0 && r.1 > 2.0, "R lower left, seed {seed}");
            assert!(g.0 < -2.0 && g.1 < 0.0, "G upper left, seed {seed}");
        }
    }

    #[test]
    fn tiles_share_one_size_and_fit_the_view() {
        let small = ColorMapModel::new(
            &ColorMapCounts {
                cards: [30, 12, 0, 44, 8],
                colorless: 26,
            },
            SEED,
        );
        let large = ColorMapModel::new(
            &ColorMapCounts {
                cards: sample().cards.map(|n| n * 3),
                colorless: 0,
            },
            SEED,
        );
        assert!(
            small.tile_size > large.tile_size,
            "a big map shrinks its tiles"
        );
        for model in [&small, &large] {
            for tile in &model.tiles {
                assert!(tile.x >= 0.0 && tile.x + model.tile_size <= VIEW_WIDTH);
                assert!(tile.y >= 0.0 && tile.y + model.tile_size <= model.height);
            }
        }
        let empty = ColorMapModel::new(
            &ColorMapCounts {
                cards: [0; 5],
                colorless: 4,
            },
            SEED,
        );
        assert!(empty.tiles.is_empty(), "colorless never takes a tile");
    }

    #[test]
    fn the_label_gives_each_share() {
        assert_eq!(
            ColorMapModel::new(&sample(), SEED).label,
            "Color map. Blue 28 percent, red 26 percent, white 25 percent, green 11 percent, black 10 percent. Colorless 189 cards"
        );
        let mono = ColorMapCounts {
            cards: [0, 1, 0, 0, 0],
            colorless: 1,
        };
        assert_eq!(
            ColorMapModel::new(&mono, SEED).label,
            "Color map. Blue 100 percent. Colorless 1 card"
        );
        assert_eq!(
            ColorMapModel::new(&ColorMapCounts::default(), SEED).label,
            "Color map. No colored cards yet"
        );
    }

    #[test]
    fn seeds_differ_by_id() {
        assert_eq!(seed_from_id(7), seed_from_id(7));
        assert_ne!(seed_from_id(7), seed_from_id(8));
        assert_ne!(seed_from_id(1 << 64), seed_from_id(1));
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
            pairs: Vec::new(),
        };
        assert_eq!(
            ColorMapCounts::from(&map),
            ColorMapCounts {
                cards: [4, 0, 0, 9, 0],
                colorless: 2,
            },
            "missing colors read 0"
        );
    }
}
