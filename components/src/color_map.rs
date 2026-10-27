//! The color map: a field of square tiles, each owned by one of the five
//! colors, the colors growing out from the middle like petals and pushing
//! into each other.
//!
//! The layout lives in [`ColorMapModel`], a pure value the tests pin;
//! [`ColorMap`] draws it as an inline SVG at a fixed tile size inside a
//! scrolling viewport, with a key of mana pips and card counts below. The
//! look is the `.color-map*` rules in `magic.css`, and each tile's fill is one
//! of `--mana-white` through `--mana-green`.
//!
//! The model is a seeded growth. Every grid cell has a random step cost from
//! the seed, heavy tailed so growth runs down cheap channels into fingers.
//! Each color starts beside the middle on its side of the color pie (W top,
//! then U, B, R and G clockwise) and all five grow at once, always taking the
//! cheapest free cell next to their own ground, with steps away from their
//! home direction costing more. A color stops at its quota, so the bigger
//! colors keep growing into long petals after the small ones close.
//!
//! The growth reads no other randomness, so the same seed and counts always
//! give the same map. A few more cards for one color let it take a few more
//! cells at the end of its growth, so the map changes by a few tiles instead
//! of redrawing.
//!
//! Each tile also carries a shade and a ring, both read from its distance
//! from the map's middle over the farthest tile's. The shade is one of
//! orangui's four heat levels, full at the middle and fading toward the panel
//! out to the map's edge, the same for every color. The ring is the band of
//! that distance that sets when the tile sweeps in.

use std::collections::{BTreeMap, BTreeSet};

use dioxus::html::geometry::PixelsVector2D;
use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::http::contracts::suggestions::HttpColorMap;

/// The fewest tiles a map with any colored card draws.
pub const MIN_TILES: u32 = 23;
/// The most tiles a map draws: the curve's value at 6,400 colored cards, so
/// the cap and the curve meet with no step.
pub const MAX_TILES: u32 = 295;
/// Tenths of a tile per square root of the colored cards, past
/// [`MIN_TILES`]: 3.4 tiles, about 130 tiles for 1,000 cards and 210 for
/// 3,000. The square root over 6,400 is 80, and `23 + 3.4 * 80` is
/// [`MAX_TILES`].
pub const TILES_PER_ROOT_TENTHS: u32 = 34;
/// One tile's pitch on screen, in CSS pixels: tiles keep this size and a
/// map larger than the viewport scrolls.
pub const PITCH_PX: f64 = 12.0;
/// The gap between tiles, as a share of the pitch.
const GAP_SHARE: f64 = 0.12;
/// The tile corner radius, as a share of the pitch.
const CORNER_SHARE: f64 = 0.18;

/// The grid runs from `-GRID_REACH` to `GRID_REACH` on both axes, room for
/// [`MAX_TILES`] cells in one long petal.
const GRID_REACH: i32 = 40;
/// The grid's side, in cells.
const GRID_SIDE: i32 = 2 * GRID_REACH + 1;
/// The step cost's floor, so no cell is free to take.
const COST_FLOOR: f64 = 0.2;
/// The power on each step's random cost: above 1 makes the costs heavy
/// tailed, so growth runs down cheap channels into fingers.
const COST_POWER: f64 = 2.0;
/// How much more a step pointing away from a color's home costs: a step
/// straight back costs `1 + 2 * HOME_PULL` times one straight out.
const HOME_PULL: f64 = 0.3;
/// How much of a color's step cost is its own rather than shared: each color
/// finds its own cheap channels, so fronts finger into each other.
const OWN_COST: f64 = 0.4;
/// How many times its step cost a color pays to cross a cell another color
/// holds.
const CROSSING_TOLL: f64 = 6.0;
/// Each color's home on the color pie, in degrees clockwise from the right
/// in SVG's downward y, in WUBRG order: W at the top, then clockwise.
const HOME_DEGREES: [f64; 5] = [-90.0, -18.0, 54.0, 126.0, 198.0];
/// Each color's first cell, beside the middle toward its home, in WUBRG
/// order.
const STARTS: [(i32, i32); 5] = [(0, -2), (2, -1), (1, 2), (-1, 2), (-2, -1)];

/// The bands of distance from the map's middle the entrance sweeps in, one
/// after another.
pub const SWEEP_RINGS: u32 = 7;
/// The wait between one ring's sweep and the next, in milliseconds. With
/// orangui's 0.4s `heat-sweep`, the last ring lands about 700ms after mount.
pub const SWEEP_STEP_MS: u32 = 50;
/// The counts a [`ColorMapGhost`] grows from: about 40 tiles, enough to
/// branch like a real map while staying a small cluster.
const GHOST_COUNTS: ColorMapCounts = ColorMapCounts {
    cards: [8, 6, 6, 5, 5],
    colorless: 0,
};
/// The seed a [`ColorMapGhost`] grows from, so every skeleton has the same
/// shape.
const GHOST_SEED: u64 = 0x5eed_c0de;
/// The full shade, a color's own mana fill.
pub const FULL_SHADE: u8 = 4;
/// Each shade's share of the mana fill over the panel, faintest first, the
/// way orangui's `.heat-1` through `.heat-4` mix the accent.
const SHADE_PERCENTS: [u8; 4] = [30, 55, 80, 100];

/// The most passes [`close_pockets`] makes. One pass fills every pocket in
/// the tested maps; the rest are a guard for a pocket a pass leaves.
const POCKET_PASSES: u32 = 4;

/// Salt for the step costs.
const COST_SALT: u64 = 0x636f_7374;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tile {
    /// The color that owns the tile.
    pub color: Color,
    /// The tile's grid cell, x then y, with the grid's middle at 0.
    pub cell: (i32, i32),
    /// The tile's column from the map's left edge.
    pub column: i32,
    /// The tile's row from the map's top edge.
    pub row: i32,
    /// The tile's shade, 1 (faintest) to [`FULL_SHADE`]: full at the map's
    /// middle, fainter toward its far edge, whatever the tile's color.
    pub shade: u8,
    /// The tile's band of distance from the map's middle, 0 at the middle
    /// to `SWEEP_RINGS - 1` at the farthest tile.
    pub ring: u32,
}

/// Everything a [`ColorMap`] draws, computed once from its counts and seed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorMapModel {
    /// Every tile, by row and then column.
    pub tiles: Vec<Tile>,
    /// The map's width, in tiles.
    pub columns: i32,
    /// The map's height, in tiles.
    pub rows: i32,
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

/// A hash of the seed, a salt and a grid cell.
fn hash(seed: u64, salt: u64, (x, y): (i32, i32)) -> u64 {
    let point = (u64::from(x.cast_unsigned()) << 32) | u64::from(y.cast_unsigned());
    mix(seed ^ mix(salt ^ mix(point)))
}

/// `h` as a number above 0 and at most 1.
fn unit(h: u64) -> f64 {
    (f64::from(u32::try_from(h >> 32).unwrap_or(0)) + 1.0) / 4_294_967_296.0
}

/// The map's tile count for `colored` cards: [`MIN_TILES`] plus
/// [`TILES_PER_ROOT_TENTHS`] tenths per square root of the cards, rounded
/// down, at most
/// [`MAX_TILES`]; none without colored cards. The square root keeps a big
/// collection from spreading the map far.
#[must_use]
pub fn tile_total(colored: u64) -> u32 {
    if colored == 0 {
        return 0;
    }
    let per_root = u64::from(TILES_PER_ROOT_TENTHS);
    let grown = (per_root * per_root).saturating_mul(colored).isqrt() / 10;
    u32::try_from(grown)
        .unwrap_or(MAX_TILES)
        .saturating_add(MIN_TILES)
        .min(MAX_TILES)
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

/// A cell's index in the grid, or `None` off the grid.
fn index((x, y): (i32, i32)) -> Option<usize> {
    if x.abs() > GRID_REACH || y.abs() > GRID_REACH {
        return None;
    }
    usize::try_from((y + GRID_REACH) * GRID_SIDE + (x + GRID_REACH)).ok()
}

/// The four cells beside `(x, y)`.
fn neighbors((x, y): (i32, i32)) -> [(i32, i32); 4] {
    [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
}

/// The seed's random cost for taking `cell`, heavy tailed above
/// [`COST_FLOOR`].
fn step_cost(seed: u64, salt: u64, cell: (i32, i32)) -> f64 {
    COST_FLOOR + (-unit(hash(seed, salt, cell)).ln()).powf(COST_POWER)
}

/// What taking `cell` costs `color`: the cell's step cost, more the further
/// the cell points away from the color's home.
fn color_cost(seed: u64, color: usize, cell: (i32, i32)) -> f64 {
    let home = HOME_DEGREES.get(color).copied().unwrap_or(0.0).to_radians();
    let angle = f64::from(cell.1).atan2(f64::from(cell.0));
    let shared = step_cost(seed, COST_SALT, cell);
    let own = step_cost(
        seed,
        COST_SALT ^ mix(u64::try_from(color).unwrap_or(0) + 1),
        cell,
    );
    ((1.0 - OWN_COST) * shared + OWN_COST * own) * (1.0 + HOME_PULL * (1.0 - (angle - home).cos()))
}

/// A frontier entry for [`grow`]: cheapest first, ties by cell then color.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Claim {
    cost: f64,
    cell: (i32, i32),
    color: usize,
}

impl Eq for Claim {}

impl Ord for Claim {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost
            .total_cmp(&self.cost)
            .then_with(|| other.cell.cmp(&self.cell))
            .then_with(|| other.color.cmp(&self.color))
    }
}

impl PartialOrd for Claim {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Grows every color with a quota from its start at once, each always
/// taking the cheapest free cell beside its own ground, until each holds
/// exactly its quota. Returns each claimed cell with its color index.
fn grow(seed: u64, quotas: [u32; 5]) -> Vec<((i32, i32), usize)> {
    let side = usize::try_from(GRID_SIDE).unwrap_or(0);
    let mut owner: Vec<Option<usize>> = vec![None; side * side];
    let mut room = quotas;
    let mut claimed = Vec::new();
    let mut frontier = std::collections::BinaryHeap::new();
    for (color, (&start, &quota)) in STARTS.iter().zip(&quotas).enumerate() {
        if quota > 0 {
            frontier.push(Claim {
                cost: 0.0,
                cell: start,
                color,
            });
        }
    }
    let free = |owner: &[Option<usize>], cell| {
        index(cell)
            .and_then(|i| owner.get(i))
            .is_some_and(Option::is_none)
    };
    let mut visited = vec![[false; 5]; side * side];
    while let Some(Claim { cost, cell, color }) = frontier.pop() {
        let Some(left) = room.get_mut(color) else {
            continue;
        };
        let Some(seen) = index(cell)
            .and_then(|i| visited.get_mut(i))
            .and_then(|row| row.get_mut(color))
        else {
            continue;
        };
        if *left == 0 || *seen {
            continue;
        }
        *seen = true;
        // Another color's cell is crossed, not taken, at a steep price, so a
        // color walled in breaks out nearby instead of stopping.
        let toll = if free(&owner, cell) {
            if let Some(slot) = index(cell).and_then(|i| owner.get_mut(i)) {
                *slot = Some(color);
            }
            *left -= 1;
            claimed.push((cell, color));
            1.0
        } else {
            CROSSING_TOLL
        };
        for next in neighbors(cell) {
            let unseen = index(next)
                .and_then(|i| visited.get(i))
                .and_then(|row| row.get(color))
                .is_some_and(|seen| !seen);
            if unseen {
                frontier.push(Claim {
                    cost: cost + toll * color_cost(seed, color, next),
                    cell: next,
                    color,
                });
            }
        }
    }

    // A color walled in by the others before its quota fills the free cells
    // inside its own ground first, then the free cells beside the map
    // nearest its ground, so every quota is met.
    for (color, left) in room.iter_mut().enumerate() {
        while *left > 0 {
            let own: Vec<(i32, i32)> = claimed
                .iter()
                .filter(|&&(_, c)| c == color)
                .map(|&(cell, _)| cell)
                .collect();
            let near_own = |cell: (i32, i32)| {
                own.iter()
                    .map(|&(x, y)| (x - cell.0).abs() + (y - cell.1).abs())
                    .min()
                    .unwrap_or(0)
            };
            let cheapest = claimed
                .iter()
                .flat_map(|&(cell, _)| neighbors(cell))
                .filter(|&cell| free(&owner, cell))
                .min_by(|&a, &b| {
                    near_own(a)
                        .cmp(&near_own(b))
                        .then(color_cost(seed, color, a).total_cmp(&color_cost(seed, color, b)))
                        .then(a.cmp(&b))
                });
            let Some(cell) = cheapest else { break };
            if let Some(slot) = index(cell).and_then(|i| owner.get_mut(i)) {
                *slot = Some(color);
            }
            *left -= 1;
            claimed.push((cell, color));
        }
    }
    claimed
}

/// The empty cells around a map's tiles, split by whether they reach the
/// outside.
#[derive(Debug)]
struct EmptyCells {
    /// Empty cells in the tiles' bounds, one cell out on every side, that a
    /// walk over empty cells from that frame's edge reaches.
    open: BTreeSet<(i32, i32)>,
    /// Empty cells inside the bounds the walk never reaches, in row order.
    walled: Vec<(i32, i32)>,
}

/// Sorts the empty cells around `owner`'s tiles into open and walled.
fn empty_cells(owner: &BTreeMap<(i32, i32), usize>) -> EmptyCells {
    let mut open = BTreeSet::new();
    let (Some(low_x), Some(high_x), Some(low_y), Some(high_y)) = (
        owner.keys().map(|c| c.0).min(),
        owner.keys().map(|c| c.0).max(),
        owner.keys().map(|c| c.1).min(),
        owner.keys().map(|c| c.1).max(),
    ) else {
        return EmptyCells {
            open,
            walled: Vec::new(),
        };
    };
    let (frame_x, frame_y) = ((low_x - 1)..=(high_x + 1), (low_y - 1)..=(high_y + 1));
    let mut stack: Vec<(i32, i32)> = frame_x
        .clone()
        .flat_map(|x| [(x, low_y - 1), (x, high_y + 1)])
        .chain(
            frame_y
                .clone()
                .flat_map(|y| [(low_x - 1, y), (high_x + 1, y)]),
        )
        .collect();
    while let Some(cell) = stack.pop() {
        let inside = frame_x.contains(&cell.0) && frame_y.contains(&cell.1);
        if !inside || owner.contains_key(&cell) || !open.insert(cell) {
            continue;
        }
        stack.extend(neighbors(cell));
    }
    let walled = (low_y..=high_y)
        .flat_map(|y| (low_x..=high_x).map(move |x| (x, y)))
        .filter(|cell| !owner.contains_key(cell) && !open.contains(cell))
        .collect();
    EmptyCells { open, walled }
}

/// Fills the pockets the growth leaves: empty cells walled in by tiles, which
/// read as holes in a color's ground. Each pocket cell, in row order, takes
/// the color most of its neighbors hold (ties in WUBRG order), and that color
/// gives up one tile on its outer edge to pay for it, the one with the most
/// open sides and then the farthest from the middle, so every quota stays
/// exact and the branching outline stays. A color with no outer tile leaves
/// the pocket for the next color around it. Stops when no pocket is left or
/// after [`POCKET_PASSES`] passes, the same result for the same tiles.
fn close_pockets(claimed: &mut Vec<((i32, i32), usize)>) {
    let mut owner: BTreeMap<(i32, i32), usize> = claimed.iter().copied().collect();
    for _ in 0..POCKET_PASSES {
        let EmptyCells {
            mut open,
            walled: pockets,
        } = empty_cells(&owner);
        if pockets.is_empty() {
            break;
        }
        for pocket in pockets {
            let mut around = [0_u32; 5];
            for next in neighbors(pocket) {
                if let Some(slot) = owner.get(&next).and_then(|&c| around.get_mut(c)) {
                    *slot += 1;
                }
            }
            let mut colors: Vec<usize> = (0..5).filter(|&c| around.get(c) > Some(&0)).collect();
            colors.sort_by_key(|&c| (std::cmp::Reverse(around.get(c).copied()), c));
            let open_sides =
                |cell: (i32, i32)| neighbors(cell).iter().filter(|n| open.contains(n)).count();
            let donor = colors.into_iter().find_map(|color| {
                owner
                    .iter()
                    .filter(|&(&cell, &c)| c == color && open_sides(cell) > 0)
                    .map(|(&cell, _)| cell)
                    .max_by(|&a, &b| {
                        let far = |(x, y): (i32, i32)| x * x + y * y;
                        open_sides(a)
                            .cmp(&open_sides(b))
                            .then(far(a).cmp(&far(b)))
                            .then(b.cmp(&a))
                    })
                    .map(|cell| (cell, color))
            });
            let Some((donor, color)) = donor else {
                continue;
            };
            owner.remove(&donor);
            open.insert(donor);
            owner.insert(pocket, color);
        }
    }
    *claimed = owner.into_iter().collect();
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

/// How many of `steps` evenly spaced marks above 0 `ratio` reaches, where
/// `ratio` runs from 0 to 1: 0 for a ratio under the first mark, `steps - 1`
/// at 1.
fn band(ratio: f64, steps: u32) -> u32 {
    let reached = (1..steps)
        .filter(|&k| ratio * f64::from(steps) >= f64::from(k))
        .count();
    u32::try_from(reached).unwrap_or(0)
}

/// The straight-line distance between two cells.
fn distance((ax, ay): (f64, f64), (bx, by): (f64, f64)) -> f64 {
    (ax - bx).hypot(ay - by)
}

/// Sets each tile's shade and ring from one measure: its distance from the
/// middle of the map's `columns` by `rows` box over the farthest tile's. The
/// shade fades from [`FULL_SHADE`] at the middle to 1 at the farthest tiles
/// in quarters of that ratio, and the ring is its band of [`SWEEP_RINGS`].
fn shade_and_ring(tiles: &mut [Tile], columns: i32, rows: i32) {
    let middle = (f64::from(columns - 1) / 2.0, f64::from(rows - 1) / 2.0);
    let spot = |t: &Tile| distance((f64::from(t.column), f64::from(t.row)), middle);
    let reach = tiles.iter().map(spot).fold(0.0, f64::max);
    let steps = u32::from(FULL_SHADE);
    for tile in tiles.iter_mut() {
        let ratio = if reach > 0.0 { spot(tile) / reach } else { 0.0 };
        let faded = u8::try_from(band(ratio, steps)).unwrap_or(0);
        tile.shade = FULL_SHADE.saturating_sub(faded);
        tile.ring = band(ratio, SWEEP_RINGS);
    }
}

/// A tile's fill: its color's mana fill mixed over the panel background at
/// its shade's share, the full fill at [`FULL_SHADE`].
#[must_use]
pub fn tile_fill(tile: &Tile) -> String {
    let var = mana_var(tile.color);
    let percent = usize::from(tile.shade)
        .checked_sub(1)
        .and_then(|i| SHADE_PERCENTS.get(i))
        .copied()
        .unwrap_or(100);
    if percent >= 100 {
        format!("var({var})")
    } else {
        format!("color-mix(in srgb, var({var}) {percent}%, var(--bg-primary))")
    }
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
    /// Lays out `counts` for `seed`: tiles growing slowly with the colored
    /// cards, shared out by color in proportion, the same map for the same
    /// inputs.
    #[must_use]
    pub fn new(counts: &ColorMapCounts, seed: u64) -> Self {
        let colored: u64 = counts.cards.iter().map(|&n| u64::from(n)).sum();
        let quotas = quotas(counts.cards, tile_total(colored));
        let mut claimed = grow(seed, quotas);
        close_pockets(&mut claimed);
        claimed.sort_by_key(|&((x, y), _)| (y, x));

        let bounds = |pick: fn(&((i32, i32), usize)) -> i32| {
            let low = claimed.iter().map(pick).min().unwrap_or(0);
            let high = claimed.iter().map(pick).max().unwrap_or(-1);
            (low, high - low + 1)
        };
        let (min_x, columns) = bounds(|c| c.0.0);
        let (min_y, rows) = bounds(|c| c.0.1);
        let palette = Color::all();
        let mut tiles: Vec<Tile> = claimed
            .into_iter()
            .filter_map(|(cell, color)| {
                Some(Tile {
                    color: *palette.get(color)?,
                    cell,
                    column: cell.0 - min_x,
                    row: cell.1 - min_y,
                    shade: FULL_SHADE,
                    ring: 0,
                })
            })
            .collect();
        shade_and_ring(&mut tiles, columns, rows);
        Self {
            tiles,
            columns,
            rows,
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

/// Scrolls `viewport` so the middle of what it holds sits in its middle.
async fn center(viewport: std::rc::Rc<MountedData>) {
    let (Ok(content), Ok(frame)) = (
        viewport.get_scroll_size().await,
        viewport.get_client_rect().await,
    ) else {
        return;
    };
    let x = ((content.width - frame.size.width) / 2.0).max(0.0);
    let y = ((content.height - frame.size.height) / 2.0).max(0.0);
    // A renderer that cannot scroll leaves the map at its top left.
    viewport
        .scroll(PixelsVector2D::new(x, y), ScrollBehavior::Instant)
        .await
        .ok();
}

/// The color territory map. Square tiles, each filled with one color's mana
/// fill, the tile count growing slowly with the cards and each color's share
/// in proportion to its cards, every color growing out from its home on the
/// color pie (W at the top, then U, B, R and G clockwise). `seed` fixes the
/// map's shape: pass the same one for the same player, such as
/// [`seed_from_id`] of their id.
///
/// The tiles sweep in from the map's middle out, a ring at a time, in
/// orangui's heat grid sweep, and every color fades from full at the map's
/// middle toward the panel at its edges. The key's pips stay at full color.
///
/// Tiles keep [`PITCH_PX`] at any size. The map runs the full width of its
/// container and up to 14rem tall, centered when it fits and otherwise
/// scrolling, opened on the map's middle. It scrolls in two nested boxes,
/// up and down in orangui's `scroll-fade-y` and across in its
/// `scroll-fade-x`, so an edge with more map past it fades out. Under it, a
/// key of six chips gives every color's pip and card count, zero included, then the colorless
/// cards, which the map leaves out.
#[component]
pub fn ColorMap(counts: ColorMapCounts, seed: u64) -> Element {
    let model = ColorMapModel::new(&counts, seed);
    let width = f64::from(model.columns) * PITCH_PX;
    let height = f64::from(model.rows) * PITCH_PX;
    let view_box = format!("0 0 {} {}", model.columns, model.rows);
    let (inset, size, corner) = (GAP_SHARE / 2.0, 1.0 - GAP_SHARE, CORNER_SHARE);
    let key = Color::all().into_iter().zip(counts.cards);
    let colorless = counts.colorless;
    rsx! {
        div { class: "color-map",
            if !model.tiles.is_empty() {
                div {
                    class: "color-map-viewport scroll-fade-y",
                    role: "img",
                    aria_label: "{model.label}",
                    onmounted: move |event| {
                        spawn(center(event.data()));
                    },
                    div {
                        class: "color-map-across scroll-fade-x",
                        onmounted: move |event| {
                            spawn(center(event.data()));
                        },
                        svg {
                            class: "color-map-svg",
                            view_box: "{view_box}",
                            width: "{width}",
                            height: "{height}",
                            "aria-hidden": "true",
                            for tile in model.tiles.iter() {
                                rect {
                                    key: "{tile.cell.0},{tile.cell.1}",
                                    class: "color-map-tile",
                                    x: "{f64::from(tile.column) + inset:.2}",
                                    y: "{f64::from(tile.row) + inset:.2}",
                                    width: "{size:.2}",
                                    height: "{size:.2}",
                                    rx: "{corner:.2}",
                                    style: "fill:{tile_fill(tile)};animation-delay:{tile.ring * SWEEP_STEP_MS}ms;",
                                }
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
                span { class: "stat-chip color-map-chip",
                    i { class: "ms ms-c ms-cost" }
                    "Colorless {colorless}"
                }
            }
        }
    }
}

/// The map a [`ColorMapGhost`] draws: `GHOST_COUNTS` grown from
/// `GHOST_SEED`, so it branches out to jagged tips like a real map.
#[must_use]
pub fn ghost_model() -> ColorMapModel {
    ColorMapModel::new(&GHOST_COUNTS, GHOST_SEED)
}

/// A [`ColorMap`]'s viewport while its counts load: a small map of ghost
/// tiles (`ghost_model`) at the map's pitch, in the middle of a box as tall
/// as a map of about 2,200 colored cards, so the swap to the real map stays
/// in place. The tiles grow out ring by ring to the map's jagged tips and
/// fold back in, tips first, on a loop until the data lands, dimmer toward
/// the edge.
/// The look is the app's `.color-map-ghost*` skeleton rules; it carries no
/// key, which the caller ghosts as chips.
#[component]
pub fn ColorMapGhost() -> Element {
    let model = ghost_model();
    let (width, height) = (
        f64::from(model.columns) * PITCH_PX,
        f64::from(model.rows) * PITCH_PX,
    );
    let (inset, size, corner) = (GAP_SHARE / 2.0, 1.0 - GAP_SHARE, CORNER_SHARE);
    rsx! {
        div { class: "color-map-viewport color-map-ghost",
            svg {
                class: "color-map-svg",
                view_box: "0 0 {model.columns} {model.rows}",
                width: "{width}",
                height: "{height}",
                "aria-hidden": "true",
                for tile in model.tiles {
                    rect {
                        key: "{tile.column},{tile.row}",
                        class: "color-map-ghost-tile color-map-ghost-ring-{tile.ring}",
                        x: "{f64::from(tile.column) + inset:.2}",
                        y: "{f64::from(tile.row) + inset:.2}",
                        width: "{size:.2}",
                        height: "{size:.2}",
                        rx: "{corner:.2}",
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::http::contracts::suggestions::HttpColorCards;

    use super::*;

    const SEED: u64 = 0x5eed_c0de_1234_5678;
    const SEEDS: [u64; 6] = [SEED, 1, 2, 3, 42, 99];

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

    /// Each color's largest 4-connected region, in tiles, in WUBRG order.
    fn largest_regions(model: &ColorMapModel) -> [usize; 5] {
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
            while let Some(cell) = stack.pop() {
                size += 1;
                for next in neighbors(cell) {
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
        largest
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
    fn tiles_grow_slowly_with_cards_within_the_clamp() {
        assert_eq!(tile_total(0), 0);
        assert_eq!(tile_total(1), 26);
        assert_eq!(tile_total(100), 57);
        assert_eq!(tile_total(720), 114);
        assert_eq!(tile_total(900), 125);
        assert_eq!(tile_total(1000), 130);
        assert_eq!(tile_total(3000), 209);
        assert_eq!(tile_total(6399), 294);
        assert_eq!(tile_total(6400), MAX_TILES);
        assert_eq!(tile_total(10_000), MAX_TILES);
        assert_eq!(tile_total(u64::MAX), MAX_TILES);
        let small = ColorMapCounts {
            cards: [3, 0, 0, 1, 0],
            colorless: 0,
        };
        assert_eq!(ColorMapModel::new(&small, SEED).tiles.len(), 29);
        let huge = ColorMapCounts {
            cards: [9000, 9000, 9000, 9000, 9000],
            colorless: 0,
        };
        assert_eq!(ColorMapModel::new(&huge, SEED).tiles.len(), 295);
    }

    #[test]
    fn quotas_are_exact_by_largest_remainder() {
        assert_eq!(quotas([1, 1, 1, 0, 0], 40), [14, 13, 13, 0, 0]);
        assert_eq!(quotas([180, 200, 70, 190, 80], 180), [45, 50, 18, 47, 20]);
        for seed in SEEDS {
            for cards in [
                sample().cards,
                [30, 12, 0, 44, 8],
                [0, 0, 0, 1, 0],
                [500, 3, 2, 1, 900],
                [9000, 9000, 9000, 9000, 9000],
            ] {
                let counts = ColorMapCounts {
                    cards,
                    colorless: 0,
                };
                let model = ColorMapModel::new(&counts, seed);
                let total = tile_total(cards.iter().map(|&n| u64::from(n)).sum());
                assert_eq!(
                    tally(&model),
                    quotas(cards, total),
                    "cards {cards:?}, seed {seed}"
                );
                assert_eq!(quotas(cards, total).iter().sum::<u32>(), total);
            }
        }
    }

    #[test]
    fn each_color_holds_mostly_together() {
        let (mut together, mut all) = (0, 0);
        for seed in SEEDS {
            for cards in [
                sample().cards,
                [30, 12, 0, 44, 8],
                [100, 100, 100, 100, 100],
                [2000, 1500, 900, 300, 100],
            ] {
                let model = ColorMapModel::new(
                    &ColorMapCounts {
                        cards,
                        colorless: 0,
                    },
                    seed,
                );
                for ((tiles, largest), color) in tally(&model)
                    .into_iter()
                    .zip(largest_regions(&model))
                    .zip(Color::all())
                {
                    let tiles = usize::try_from(tiles).unwrap();
                    assert!(
                        largest * 5 >= tiles * 3,
                        "{color:?} keeps {largest} of {tiles} together, seed {seed}, cards {cards:?}"
                    );
                    together += largest;
                    all += tiles;
                }
            }
        }
        assert!(
            together * 100 >= all * 95,
            "{together} of {all} tiles sit in their color's main region"
        );
    }

    #[test]
    fn a_small_change_moves_a_few_tiles() {
        for seed in SEEDS {
            for i in 0..5 {
                let before = by_cell(&ColorMapModel::new(&sample(), seed));
                let mut grown = sample();
                *grown.cards.get_mut(i).unwrap() += 5;
                let after = by_cell(&ColorMapModel::new(&grown, seed));
                let changed = before
                    .iter()
                    .filter(|(cell, c)| after.get(cell) != Some(c))
                    .count()
                    + after
                        .keys()
                        .filter(|cell| !before.contains_key(cell))
                        .count();
                // A tile stands for more cards on the smaller map, so five more
                // cards can move a few more of them.
                assert!(
                    changed <= 16,
                    "{changed} tiles changed for seed {seed}, color {i}"
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
        for seed in SEEDS {
            let model = ColorMapModel::new(&counts, seed);
            let center = |color: Color| {
                let own: Vec<&Tile> = model.tiles.iter().filter(|t| t.color == color).collect();
                let n = f64::from(u32::try_from(own.len()).unwrap());
                let x: f64 = own.iter().map(|t| f64::from(t.cell.0)).sum::<f64>() / n;
                let y: f64 = own.iter().map(|t| f64::from(t.cell.1)).sum::<f64>() / n;
                (x, y)
            };
            for (color, degrees) in Color::all().into_iter().zip(HOME_DEGREES) {
                let (x, y) = center(color);
                let (dx, dy) = (degrees.to_radians().cos(), degrees.to_radians().sin());
                assert!(x * dx + y * dy > 1.0, "{color:?} leans home, seed {seed}");
            }
        }
    }

    #[test]
    fn tiles_fill_the_map_bounds() {
        let model = ColorMapModel::new(&sample(), SEED);
        assert!(model.tiles.iter().all(|t| {
            (0..model.columns).contains(&t.column) && (0..model.rows).contains(&t.row)
        }));
        assert!(model.tiles.iter().any(|t| t.column == 0));
        assert!(model.tiles.iter().any(|t| t.row == model.rows - 1));
        let empty = ColorMapModel::new(
            &ColorMapCounts {
                cards: [0; 5],
                colorless: 4,
            },
            SEED,
        );
        assert!(empty.tiles.is_empty(), "colorless never takes a tile");
        assert_eq!((empty.columns, empty.rows), (0, 0));
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

    /// A sample of counts, small to capped, for the shade and ring tests.
    fn spread() -> Vec<ColorMapCounts> {
        [
            sample().cards,
            [30, 12, 0, 44, 8],
            [0, 0, 0, 1, 0],
            [500, 3, 2, 1, 900],
            [9000, 9000, 9000, 9000, 9000],
        ]
        .into_iter()
        .map(|cards| ColorMapCounts {
            cards,
            colorless: 0,
        })
        .collect()
    }

    /// Each tile's distance from the middle of `model`'s box, with its
    /// shade, nearest first.
    fn shades_by_distance(model: &ColorMapModel) -> Vec<(f64, u8)> {
        let middle = (
            f64::from(model.columns - 1) / 2.0,
            f64::from(model.rows - 1) / 2.0,
        );
        let mut by_distance: Vec<(f64, u8)> = model
            .tiles
            .iter()
            .map(|t| {
                let spot = (f64::from(t.column), f64::from(t.row));
                (distance(spot, middle), t.shade)
            })
            .collect();
        by_distance.sort_by(|a, b| a.0.total_cmp(&b.0));
        by_distance
    }

    #[test]
    fn the_middle_is_full_and_the_far_edge_faintest() {
        for seed in SEEDS {
            for counts in spread() {
                let model = ColorMapModel::new(&counts, seed);
                let by_distance = shades_by_distance(&model);
                let (nearest, _) = by_distance.first().copied().unwrap();
                let (farthest, _) = by_distance.last().copied().unwrap();
                for &(d, shade) in &by_distance {
                    if d <= nearest {
                        assert_eq!(shade, FULL_SHADE, "middle, seed {seed}, {counts:?}");
                    }
                    if d >= farthest && farthest > nearest {
                        assert_eq!(shade, 1, "far edge, seed {seed}, {counts:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn shades_never_rise_with_distance_from_the_middle() {
        for seed in SEEDS {
            for counts in spread() {
                let model = ColorMapModel::new(&counts, seed);
                assert!(
                    shades_by_distance(&model).is_sorted_by(|a, b| a.1 >= b.1),
                    "fades outward, seed {seed}, {counts:?}"
                );
            }
        }
    }

    #[test]
    fn the_ghost_grows_like_a_small_map() {
        let ghost = ghost_model();
        assert_eq!(ghost, ghost_model(), "the same shape every time");
        assert!(
            (30..=50).contains(&ghost.tiles.len()),
            "{} tiles",
            ghost.tiles.len()
        );
        assert!(
            ghost.tiles.iter().map(|tile| tile.ring).min() <= Some(1),
            "grows from near the middle"
        );
        assert_eq!(
            ghost.tiles.iter().map(|tile| tile.ring).max(),
            Some(SWEEP_RINGS - 1),
            "tips on the last ring"
        );
    }

    #[test]
    fn no_empty_cell_is_walled_in() {
        for seed in SEEDS {
            for counts in spread() {
                let model = ColorMapModel::new(&counts, seed);
                let owner: BTreeMap<(i32, i32), usize> =
                    model.tiles.iter().map(|t| (t.cell, 0)).collect();
                let walled = empty_cells(&owner).walled;
                assert_eq!(walled, [], "seed {seed}, cards {:?}", counts.cards);
            }
        }
    }

    #[test]
    fn every_tile_has_a_shade_and_a_ring() {
        for seed in SEEDS {
            for counts in spread() {
                let model = ColorMapModel::new(&counts, seed);
                assert!(
                    model
                        .tiles
                        .iter()
                        .all(|t| (1..=FULL_SHADE).contains(&t.shade))
                );
                assert!(model.tiles.iter().all(|t| t.ring < SWEEP_RINGS));
                assert_eq!(
                    model.tiles.iter().map(|t| t.ring).max(),
                    Some(SWEEP_RINGS - 1),
                    "the farthest tile sweeps last, cards {:?}",
                    counts.cards
                );
                assert_eq!(model, ColorMapModel::new(&counts, seed));
            }
        }
    }

    #[test]
    fn fills_mix_over_the_panel_by_shade() {
        let tile = |shade| Tile {
            color: Color::Red,
            cell: (0, 0),
            column: 0,
            row: 0,
            shade,
            ring: 0,
        };
        assert_eq!(tile_fill(&tile(4)), "var(--mana-red)");
        assert_eq!(
            tile_fill(&tile(3)),
            "color-mix(in srgb, var(--mana-red) 80%, var(--bg-primary))"
        );
        assert_eq!(
            tile_fill(&tile(1)),
            "color-mix(in srgb, var(--mana-red) 30%, var(--bg-primary))"
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
