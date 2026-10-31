use zwipe_core::http::contracts::suggestions::HttpColorCards;

use super::*;

const SEED: u64 = 0x5eed_c0de_1234_5678;
const SEEDS: [u64; 6] = [SEED, 1, 2, 3, 42, 99];

#[test]
fn index_rejects_cells_at_the_edge_of_i32() {
    assert_eq!(index((i32::MIN, 0)), None);
    assert_eq!(index((0, i32::MIN)), None);
    assert_eq!(index((i32::MAX, i32::MAX)), None);
    assert_eq!(index((-GRID_REACH, -GRID_REACH)), Some(0));
}

#[test]
fn neighbors_skip_cells_past_the_edge_of_i32() {
    let corner: Vec<(i32, i32)> = neighbors((i32::MAX, i32::MIN)).collect();
    assert_eq!(
        corner,
        vec![(i32::MAX - 1, i32::MIN), (i32::MAX, i32::MIN + 1)]
    );
    assert_eq!(neighbors((0, 0)).count(), 4);
}

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
    let mut seen: BTreeSet<(i32, i32)> = BTreeSet::new();
    let mut largest = [0_usize; 5];
    for (&start, &color) in &cells {
        if seen.contains(&start) {
            continue;
        }
        let mut stack = vec![start];
        seen.insert(start);
        let mut size = 0;
        while let Some(cell) = stack.pop() {
            size += 1;
            for next in neighbors(cell) {
                if cells.get(&next) == Some(&color) && !seen.contains(&next) {
                    seen.insert(next);
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
    assert!(
        model.tiles.iter().all(|t| {
            (0..model.columns).contains(&t.column) && (0..model.rows).contains(&t.row)
        })
    );
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
        (f64::from(model.columns) - 1.0) / 2.0,
        (f64::from(model.rows) - 1.0) / 2.0,
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
fn shades_mix_over_the_panel_by_share() {
    assert_eq!(shade_percent(4), 100);
    assert_eq!(shade_percent(3), 80);
    assert_eq!(shade_percent(2), 55);
    assert_eq!(shade_percent(1), 30);
    assert_eq!(shade_percent(0), 100);
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

/// The app's own `ColorMapModel` output for fixed inputs, printed from `components::color_map` on main: one line per case, each tile as its color letter, shade and ring, rows split by `|`, then the label.
const GOLDEN: &str = include_str!("golden.txt");

fn golden_grid(model: &ColorMapModel) -> String {
    let mut rows = Vec::new();
    for row in 0..model.rows {
        let mut line = String::new();
        for column in 0..model.columns {
            match model
                .tiles
                .iter()
                .find(|t| t.row == row && t.column == column)
            {
                Some(t) => {
                    let letter = t.color.to_short_name();
                    let (shade, ring) = (t.shade, t.ring);
                    line.push_str(&letter);
                    line.push_str(&shade.to_string());
                    line.push_str(&ring.to_string());
                }
                None => line.push_str("..."),
            }
        }
        rows.push(line);
    }
    format!(
        "{}x{}|{}|{}",
        model.columns,
        model.rows,
        rows.join("|"),
        model.label
    )
}

#[test]
fn the_map_matches_the_apps_tiles_shades_and_rings() {
    let cases: [(&str, [u32; 5], u32, u64); 5] = [
        (
            "mixed",
            [30, 12, 0, 8, 50],
            4,
            seed_from_id(0x0123_4567_89ab_cdef_0011_2233_4455_6677),
        ),
        ("live", [108, 76, 43, 195, 0], 140, seed_from_id(1)),
        ("single", [1, 0, 0, 0, 0], 0, 7),
        ("huge", [2_000, 1_500, 1_200, 1_000, 900], 9, 42),
        ("empty", [0, 0, 0, 0, 0], 5, 3),
    ];
    let mut ours: Vec<String> = cases
        .iter()
        .map(|&(name, cards, colorless, seed)| {
            let model = ColorMapModel::new(&ColorMapCounts { cards, colorless }, seed);
            format!("GOLDEN {name} {seed} {}", golden_grid(&model))
        })
        .collect();
    ours.push(format!("GOLDEN ghost 0 {}", golden_grid(&ghost_model())));
    let app: Vec<&str> = GOLDEN.lines().collect();
    assert_eq!(app.len(), ours.len());
    for (app, ours) in app.iter().zip(&ours) {
        assert_eq!(*app, ours.as_str());
    }
}
