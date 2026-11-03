//! The color map: a field of square tiles, each owned by one of the five
//! colors, the colors growing out from the middle like petals and pushing
//! into each other.
//!
//! The layout is zwipe-ui's [`ColorMapModel`], a pure value its tests pin, so
//! every client draws the same shape for the same player. [`ColorMap`] draws
//! it as an inline SVG at a fixed tile size inside a scrolling viewport, with
//! a key of mana pips and card counts below. The look is the `.color-map*`
//! rules in `magic.css`, and each tile's fill is one of `--mana-white`
//! through `--mana-green`, mixed over the panel at the tile's shade.

use dioxus::html::geometry::PixelsVector2D;
use dioxus::prelude::*;
use orangui::SplashGate;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_ui::color_map::{
    ColorMapCounts, ColorMapModel, SWEEP_STEP_MS, Tile, ghost_model, shade_percent,
};

/// One tile's pitch on screen, in CSS pixels: tiles keep this size and a
/// map larger than the viewport scrolls.
const PITCH_PX: f64 = 12.0;
/// The gap between tiles, as a share of the pitch.
const GAP_SHARE: f64 = 0.12;
/// The tile corner radius, as a share of the pitch.
const CORNER_SHARE: f64 = 0.18;

/// A tile's fill: its color's mana fill mixed over the panel background at
/// its shade's share, the full fill at the full shade.
fn tile_fill(tile: &Tile) -> String {
    let var = mana_var(tile.color);
    let percent = shade_percent(tile.shade);
    if percent >= 100 {
        format!("var({var})")
    } else {
        format!("color-mix(in srgb, var({var}) {percent}%, var(--bg-primary))")
    }
}

/// The CSS custom property holding `color`'s mana fill.
#[must_use]
pub(crate) fn mana_var(color: Color) -> &'static str {
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
/// [`seed_from_id`](zwipe_ui::color_map::seed_from_id) of their id.
///
/// The tiles sweep in from the map's middle out, a ring at a time, in
/// orangui's heat grid sweep, held under a splash until its [`SplashGate`]
/// opens, and every color fades from full at the map's
/// middle toward the panel at its edges. The key's pips stay at full color.
///
/// Tiles keep their 12px pitch at any size. The map runs the full width of its
/// container and up to 14rem tall, centered when it fits and otherwise
/// scrolling, opened on the map's middle. It scrolls in two nested boxes,
/// up and down in orangui's `scroll-fade-y` and across in its
/// `scroll-fade-x`, so an edge with more map past it fades out. Under a
/// rule, like a panel's action bar, a key of six chips gives every color's pip and card count, zero included, then the colorless
/// cards, which the map leaves out. A host's `key_action` sits at the key's
/// end, wrapping with its chips.
#[component]
pub fn ColorMap(
    counts: ColorMapCounts,
    seed: u64,
    /// A control drawn after the key's chips, on their lines.
    #[props(default)]
    key_action: Option<Element>,
) -> Element {
    let model = ColorMapModel::new(&counts, seed);
    let width = f64::from(model.columns) * PITCH_PX;
    let height = f64::from(model.rows) * PITCH_PX;
    let view_box = format!("0 0 {} {}", model.columns, model.rows);
    let (inset, size, corner) = (GAP_SHARE / 2.0, 1.0 - GAP_SHARE, CORNER_SHARE);
    let key = Color::all().into_iter().zip(counts.cards);
    let colorless = counts.colorless;
    // The sweep holds under a splash and runs as it leaves.
    let held = try_consume_context::<SplashGate>().is_some_and(|gate| !gate.0());
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
                            class: if held { "color-map-svg held" } else { "color-map-svg" },
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
                {key_action}
            }
        }
    }
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
    use super::*;

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
}
