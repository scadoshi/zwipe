//! The color map's layout, ported from the app's `components::color_map`: a field of square tiles, each owned by one of the five colors, the colors growing out from the middle like petals and pushing into each other.
//!
//! [`ColorMapModel`] is a pure value: the same counts and seed always give the same tiles. The growth, the pocket filling, the shades and the rings follow the app line for line, so a terminal draws the same shape the phone does for the same player.

use std::collections::{BTreeMap, BTreeSet};

use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::http::contracts::suggestions::HttpColorMap;

/// The fewest tiles a map with any colored card draws.
pub const MIN_TILES: u32 = 23;
/// The most tiles a map draws: the curve's value at 6,400 colored cards, so the cap and the curve meet with no step.
pub const MAX_TILES: u32 = 295;
/// Tenths of a tile per square root of the colored cards, past [`MIN_TILES`]: 3.4 tiles, about 130 tiles for 1,000 cards and 210 for 3,000. The square root over 6,400 is 80, and `23 + 3.4 * 80` is [`MAX_TILES`].
pub const TILES_PER_ROOT_TENTHS: u32 = 34;

/// The grid runs from `-GRID_REACH` to `GRID_REACH` on both axes, room for [`MAX_TILES`] cells in one long petal.
const GRID_REACH: i32 = 40;
/// The grid's side, in cells.
const GRID_SIDE: i32 = 2 * GRID_REACH + 1;
/// The step cost's floor, so no cell is free to take.
const COST_FLOOR: f64 = 0.2;
/// The power on each step's random cost: above 1 makes the costs heavy tailed, so growth runs down cheap channels into fingers.
const COST_POWER: f64 = 2.0;
/// How much more a step pointing away from a color's home costs: a step straight back costs `1 + 2 * HOME_PULL` times one straight out.
const HOME_PULL: f64 = 0.3;
/// How much of a color's step cost is its own rather than shared: each color finds its own cheap channels, so fronts finger into each other.
const OWN_COST: f64 = 0.4;
/// How many times its step cost a color pays to cross a cell another color holds.
const CROSSING_TOLL: f64 = 6.0;
/// Each color's home on the color pie, in degrees clockwise from the right in SVG's downward y, in WUBRG order: W at the top, then clockwise.
const HOME_DEGREES: [f64; 5] = [-90.0, -18.0, 54.0, 126.0, 198.0];
/// Each color's first cell, beside the middle toward its home, in WUBRG order.
const STARTS: [(i32, i32); 5] = [(0, -2), (2, -1), (1, 2), (-1, 2), (-2, -1)];

/// The bands of distance from the map's middle the entrance sweeps in, one after another.
pub const SWEEP_RINGS: u32 = 7;
/// The wait between one ring's sweep and the next, in milliseconds. With orangui's 0.4s `heat-sweep`, the last ring lands about 700ms after mount.
pub const SWEEP_STEP_MS: u32 = 50;
/// The counts the ghost map grows from: about 40 tiles, enough to branch like a real map while staying a small cluster.
const GHOST_COUNTS: ColorMapCounts = ColorMapCounts {
    cards: [8, 6, 6, 5, 5],
    colorless: 0,
};
/// The seed the ghost map grows from, so every skeleton has the same shape.
const GHOST_SEED: u64 = 0x5eed_c0de;
/// The full shade, a color's own mana fill.
pub const FULL_SHADE: u8 = 4;
/// Each shade's share of the mana fill over the panel, faintest first, the way orangui's `.heat-1` through `.heat-4` mix the accent.
const SHADE_PERCENTS: [u8; 4] = [30, 55, 80, 100];

/// The most passes [`close_pockets`] makes. One pass fills every pocket in the tested maps; the rest are a guard for a pocket a pass leaves.
const POCKET_PASSES: u32 = 4;

/// Salt for the step costs.
const COST_SALT: u64 = 0x636f_7374;

/// The counts a the map draws.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ColorMapCounts {
    /// Cards whose color identity includes each color, in WUBRG order.
    pub cards: [u32; 5],
    /// Cards with an empty color identity, shown only in the key.
    pub colorless: u32,
}

impl From<&HttpColorMap> for ColorMapCounts {
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
    /// The tile's shade, 1 (faintest) to [`FULL_SHADE`]: full at the map's middle, fainter toward its far edge, whatever the tile's color.
    pub shade: u8,
    /// The tile's band of distance from the map's middle, 0 at the middle to `SWEEP_RINGS - 1` at the farthest tile.
    pub ring: u32,
}

/// Everything a the map draws, computed once from its counts and seed.
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

/// A seed for the map from a 128-bit id such as a user's UUID.
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

/// The map's tile count for `colored` cards: [`MIN_TILES`] plus [`TILES_PER_ROOT_TENTHS`] tenths per square root of the cards, rounded down, at most [`MAX_TILES`]; none without colored cards. The square root keeps a big collection from spreading the map far.
#[must_use]
pub fn tile_total(colored: u64) -> u32 {
    if colored == 0 {
        return 0;
    }
    let per_root = u64::from(TILES_PER_ROOT_TENTHS);
    // 34 squared, nowhere near saturating.
    let per_root_squared = per_root.saturating_mul(per_root);
    let grown = per_root_squared.saturating_mul(colored).isqrt() / 10;
    u32::try_from(grown)
        .unwrap_or(MAX_TILES)
        .saturating_add(MIN_TILES)
        .min(MAX_TILES)
}

/// Each color's tiles out of `total`, in proportion to `cards` by largest remainder, so they sum to `total`. Ties for a leftover tile go in WUBRG order.
#[must_use]
pub fn quotas(cards: [u32; 5], total: u32) -> [u32; 5] {
    let sum: u64 = cards.iter().map(|&n| u64::from(n)).sum();
    if sum == 0 {
        return [0; 5];
    }
    // A u32 times a u32 always fits a u64, so this never saturates.
    let share = |n: u32| u64::from(total).saturating_mul(u64::from(n));
    let mut quotas = cards.map(|n| {
        share(n)
            .checked_div(sum)
            .and_then(|q| u32::try_from(q).ok())
            .unwrap_or(0)
    });
    let given: u32 = quotas.iter().sum();
    let mut order: Vec<usize> = (0..5).collect();
    order.sort_by_key(|&i| {
        let remainder = cards
            .get(i)
            .and_then(|&n| share(n).checked_rem(sum))
            .unwrap_or(0);
        (std::cmp::Reverse(remainder), i)
    });
    let leftover = usize::try_from(total.saturating_sub(given)).unwrap_or(0);
    for i in order.into_iter().take(leftover) {
        if let Some(q) = quotas.get_mut(i) {
            // At most `total`, a u32, after the leftover tile.
            *q = q.saturating_add(1);
        }
    }
    quotas
}

/// A cell's index in the grid, or `None` off the grid.
fn index((x, y): (i32, i32)) -> Option<usize> {
    let column = x.checked_add(GRID_REACH)?;
    let row = y.checked_add(GRID_REACH)?;
    if !(0..GRID_SIDE).contains(&column) || !(0..GRID_SIDE).contains(&row) {
        return None;
    }
    usize::try_from(row.checked_mul(GRID_SIDE)?.checked_add(column)?).ok()
}

/// The four cells beside `(x, y)`, less any past the edge of `i32`.
fn neighbors((x, y): (i32, i32)) -> impl Iterator<Item = (i32, i32)> {
    [
        (x.checked_add(1), Some(y)),
        (x.checked_sub(1), Some(y)),
        (Some(x), y.checked_add(1)),
        (Some(x), y.checked_sub(1)),
    ]
    .into_iter()
    .filter_map(|(x, y)| Some((x?, y?)))
}

/// The seed's random cost for taking `cell`, heavy tailed above [`COST_FLOOR`].
fn step_cost(seed: u64, salt: u64, cell: (i32, i32)) -> f64 {
    COST_FLOOR + (-unit(hash(seed, salt, cell)).ln()).powf(COST_POWER)
}

/// What taking `cell` costs `color`: the cell's step cost, more the further the cell points away from the color's home.
fn color_cost(seed: u64, color: usize, cell: (i32, i32)) -> f64 {
    let home = HOME_DEGREES.get(color).copied().unwrap_or(0.0).to_radians();
    let angle = f64::from(cell.1).atan2(f64::from(cell.0));
    let shared = step_cost(seed, COST_SALT, cell);
    let own = step_cost(
        seed,
        // `color` is a WUBRG index, 0 to 4, so this never saturates.
        COST_SALT ^ mix(u64::try_from(color).unwrap_or(0).saturating_add(1)),
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

/// Grows every color with a quota from its start at once, each always taking the cheapest free cell beside its own ground, until each holds exactly its quota. Returns each claimed cell with its color index.
fn grow(seed: u64, quotas: [u32; 5]) -> Vec<((i32, i32), usize)> {
    let side = usize::try_from(GRID_SIDE).unwrap_or(0);
    // GRID_SIDE is 81, so this never saturates.
    let cells = side.saturating_mul(side);
    let mut owner: Vec<Option<usize>> = vec![None; cells];
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
    let mut visited = vec![[false; 5]; cells];
    while let Some(Claim { cost, cell, color }) = frontier.pop() {
        let Some(left) = room.get_mut(color) else {
            continue;
        };
        let Some(was_visited) = index(cell)
            .and_then(|i| visited.get_mut(i))
            .and_then(|row| row.get_mut(color))
        else {
            continue;
        };
        let Some(rest) = left.checked_sub(1) else {
            continue;
        };
        if *was_visited {
            continue;
        }
        *was_visited = true;
        // Another color's cell is crossed, not taken, at a steep price, so a color walled in breaks out nearby instead of stopping.
        let toll = if free(&owner, cell) {
            if let Some(slot) = index(cell).and_then(|i| owner.get_mut(i)) {
                *slot = Some(color);
            }
            *left = rest;
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

    // A color walled in by the others before its quota fills the free cells inside its own ground first, then the free cells beside the map nearest its ground, so every quota is met.
    for (color, left) in room.iter_mut().enumerate() {
        while let Some(rest) = left.checked_sub(1) {
            let own: Vec<(i32, i32)> = claimed
                .iter()
                .filter(|&&(_, c)| c == color)
                .map(|&(cell, _)| cell)
                .collect();
            let near_own = |cell: (i32, i32)| {
                own.iter()
                    .map(|&(x, y)| x.abs_diff(cell.0).saturating_add(y.abs_diff(cell.1)))
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
            *left = rest;
            claimed.push((cell, color));
        }
    }
    claimed
}

/// The empty cells around a map's tiles, split by whether they reach the outside.
#[derive(Debug)]
struct EmptyCells {
    /// Empty cells in the tiles' bounds, one cell out on every side, that a walk over empty cells from that frame's edge reaches.
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
    // The frame one cell out; tiles at the edge of `i32` have none.
    let (Some(left), Some(right), Some(top), Some(bottom)) = (
        low_x.checked_sub(1),
        high_x.checked_add(1),
        low_y.checked_sub(1),
        high_y.checked_add(1),
    ) else {
        return EmptyCells {
            open,
            walled: Vec::new(),
        };
    };
    let (frame_x, frame_y) = (left..=right, top..=bottom);
    let mut stack: Vec<(i32, i32)> = frame_x
        .clone()
        .flat_map(|x| [(x, top), (x, bottom)])
        .chain(frame_y.clone().flat_map(|y| [(left, y), (right, y)]))
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

/// Fills the pockets the growth leaves: empty cells walled in by tiles, which read as holes in a color's ground. Each pocket cell, in row order, takes the color most of its neighbors hold (ties in WUBRG order), and that color gives up one tile on its outer edge to pay for it, the one with the most open sides and then the farthest from the middle, so every quota stays exact and the branching outline stays. A color with no outer tile leaves the pocket for the next color around it. Stops when no pocket is left or after [`POCKET_PASSES`] passes, the same result for the same tiles.
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
            let around: [usize; 5] = std::array::from_fn(|c| {
                neighbors(pocket)
                    .filter(|next| owner.get(next) == Some(&c))
                    .count()
            });
            let mut colors: Vec<usize> = (0..5).filter(|&c| around.get(c) > Some(&0)).collect();
            colors.sort_by_key(|&c| (std::cmp::Reverse(around.get(c).copied()), c));
            let open_sides =
                |cell: (i32, i32)| neighbors(cell).filter(|n| open.contains(n)).count();
            let donor = colors.into_iter().find_map(|color| {
                owner
                    .iter()
                    .filter(|&(&cell, &c)| c == color && open_sides(cell) > 0)
                    .map(|(&cell, _)| cell)
                    .max_by(|&a, &b| {
                        let far =
                            |(x, y): (i32, i32)| x.checked_mul(x)?.checked_add(y.checked_mul(y)?);
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

/// Each colored share, largest first with WUBRG breaking ties, as "Blue 31 percent, red 29 percent", then the colorless count.
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
            let percent = u64::from(n)
                .checked_mul(100)
                .and_then(|scaled| scaled.checked_add(sum / 2))
                .and_then(|scaled| scaled.checked_div(sum))
                .unwrap_or(0);
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
        label.push_str(". Colorless ");
        label.push_str(&counted(counts.colorless));
    }
    label
}

/// How many of `steps` evenly spaced marks above 0 `ratio` reaches, where `ratio` runs from 0 to 1: 0 for a ratio under the first mark, `steps - 1` at 1.
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

/// Sets each tile's shade and ring from one measure: its distance from the middle of the map's `columns` by `rows` box over the farthest tile's. The shade fades from [`FULL_SHADE`] at the middle to 1 at the farthest tiles in quarters of that ratio, and the ring is its band of [`SWEEP_RINGS`].
fn shade_and_ring(tiles: &mut [Tile], columns: i32, rows: i32) {
    let middle = (
        (f64::from(columns) - 1.0) / 2.0,
        (f64::from(rows) - 1.0) / 2.0,
    );
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

/// A tile's share of its mana color over the panel background, in percent: the full color at [`FULL_SHADE`], fainter for lower shades.
#[must_use]
pub fn shade_percent(shade: u8) -> u8 {
    usize::from(shade)
        .checked_sub(1)
        .and_then(|i| SHADE_PERCENTS.get(i))
        .copied()
        .unwrap_or(100)
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
    /// Lays out `counts` for `seed`: tiles growing slowly with the colored cards, shared out by color in proportion, the same map for the same inputs.
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
            let span = high
                .checked_sub(low)
                .and_then(|d| d.checked_add(1))
                .unwrap_or(0);
            (low, span)
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
                    column: cell.0.checked_sub(min_x)?,
                    row: cell.1.checked_sub(min_y)?,
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

/// The map a loading skeleton draws: a small map grown from a fixed seed, so it branches out to jagged tips like a real one.
#[must_use]
pub fn ghost_model() -> ColorMapModel {
    ColorMapModel::new(&GHOST_COUNTS, GHOST_SEED)
}

#[cfg(test)]
mod tests;
