//! The printing picker's cover flow, shared by the app and the web: the
//! selected printing large and facing the viewer, the others stacked behind
//! it on either side, each further one smaller, more overlapped, darker and
//! turned toward the middle, every card gliding to its new place as the
//! selection moves. An arrow button at each side steps one printing, and
//! so do a sideways wheel or trackpad scroll and a drag or swipe across the
//! stage; a tap on a side card brings it to the middle. The host keeps the
//! selection in `index`, so its own keys and Save read the same one.

use std::collections::HashMap;

use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use crate::flippable_card_image::{image_seen, mark_image_seen};
use crate::icons::{ARROW_LEFT, ARROW_RIGHT, DockIcon};

/// How far a wheel or trackpad scrolls, in pixels, to move one printing.
const WHEEL_STEP_PX: f64 = 60.0;

/// How far a drag or swipe runs, in pixels, to move one printing.
const DRAG_STEP_PX: f64 = 50.0;

/// One wheel line, in pixels: a mouse wheel that scrolls three lines a
/// notch moves one printing a notch, as a pixel wheel's notch does.
const WHEEL_LINE_PX: f64 = 20.0;

/// One wheel page, in pixels.
const WHEEL_PAGE_PX: f64 = 400.0;

/// Which way a step through the printings goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// To the printing before.
    Back,
    /// To the printing after.
    Forward,
}

/// A wheel movement along its longer axis, in pixels, whatever unit it
/// came in.
fn wheel_px(delta: &WheelDelta) -> f64 {
    let scale = match delta {
        WheelDelta::Pixels(_) => 1.0,
        WheelDelta::Lines(_) => WHEEL_LINE_PX,
        WheelDelta::Pages(_) => WHEEL_PAGE_PX,
    };
    let moved = delta.strip_units();
    let along = if moved.x.abs() > moved.y.abs() {
        moved.x
    } else {
        moved.y
    };
    along * scale
}

/// The steps a wheel movement of `px` makes from the `carried` pixels
/// toward the next one, at most `limit`, and what it leaves carried: one
/// step for each [`WHEEL_STEP_PX`], forward when positive. A movement that
/// is not finite makes none and leaves the carry as it was.
fn wheel_steps(carried: f64, px: f64, limit: usize) -> (Option<Step>, usize, f64) {
    if !px.is_finite() {
        return (None, 0, carried);
    }
    let mut rest = carried + px;
    let way = if rest >= WHEEL_STEP_PX {
        Step::Forward
    } else if rest <= -WHEEL_STEP_PX {
        Step::Back
    } else {
        return (None, 0, rest);
    };
    let mut steps = 0_usize;
    while rest.abs() >= WHEEL_STEP_PX && steps < limit {
        steps = steps.saturating_add(1);
        rest -= WHEEL_STEP_PX.copysign(rest);
    }
    // Past the limit, what is left is less than a step.
    (Some(way), steps, rest % WHEEL_STEP_PX)
}

/// What one printing's card draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    /// A card-shaped ghost, its image loading hidden under it.
    Ghost,
    /// The image.
    Image,
    /// The set name on a plain card: no art, or the image failed.
    Missing,
}

/// The tile for a printing that `has_art`, its image `settled` as loaded (`true`), failed
/// (`false`) or still on its way (`None`).
fn tile(has_art: bool, settled: Option<bool>) -> Tile {
    match (has_art, settled) {
        (true, None) => Tile::Ghost,
        (true, Some(true)) => Tile::Image,
        (true, Some(false)) | (false, _) => Tile::Missing,
    }
}

/// The printings in a cover flow, the one at `index` in the middle. A tap
/// on the middle card calls `on_pick`, when given (a host that only shows
/// printings leaves it out).
#[component]
pub fn CoverFlow(
    printings: Vec<Card>,
    mut index: Signal<usize>,
    #[props(default)] on_pick: Option<EventHandler<()>>,
) -> Element {
    let mut wheel = use_signal(|| 0.0_f64);
    let mut drag_from: Signal<Option<f64>> = use_signal(|| None);
    let mut dragged = use_signal(|| false);
    // Each image URL once it loaded (`true`) or failed (`false`); every card
    // ghosts on its own until its image settles.
    let mut settled: Signal<HashMap<String, bool>> = use_signal(HashMap::new);
    let count = printings.len();
    let at = index().min(count.saturating_sub(1));

    let mut go = move |next: usize| {
        if count > 0 {
            index.set(next.min(count.saturating_sub(1)));
        }
    };
    let mut step = move |way: Step| {
        let current = *index.peek();
        go(match way {
            Step::Forward => current.saturating_add(1),
            Step::Back => current.saturating_sub(1),
        });
    };
    let cards = flow_window(at, count)
        .into_iter()
        .filter_map(|(i, offset)| printings.get(i).map(|card| (i, offset, card.clone())));

    rsx! {
        div { class: "cover-flow",
            if count > 1 {
                button {
                    r#type: "button",
                    class: "btn-action btn-icon cover-flow-step",
                    disabled: at == 0,
                    "aria-label": "Previous printing",
                    onclick: move |_| step(Step::Back),
                    // Enter presses the button and stays off the host's pick.
                    onkeydown: move |e: Event<KeyboardData>| {
                        if e.key() == Key::Enter {
                            e.stop_propagation();
                        }
                    },
                    DockIcon { path: ARROW_LEFT }
                }
            }
            div {
                class: "cover-flow-stage",
                onwheel: move |e: Event<WheelData>| {
                    let (way, steps, rest) = wheel_steps(*wheel.peek(), wheel_px(&e.delta()), count);
                    if let Some(way) = way {
                        for _ in 0..steps {
                            step(way);
                        }
                    }
                    wheel.set(rest);
                },
                onpointerdown: move |e: Event<PointerData>| {
                    drag_from.set(Some(e.client_coordinates().x));
                    dragged.set(false);
                },
                onpointermove: move |e: Event<PointerData>| {
                    let Some(from) = *drag_from.peek() else {
                        return;
                    };
                    let x = e.client_coordinates().x;
                    let run = x - from;
                    if run.abs() >= DRAG_STEP_PX {
                        // Dragging left brings the next printing in.
                        step(if run < 0.0 { Step::Forward } else { Step::Back });
                        drag_from.set(Some(x));
                        dragged.set(true);
                    }
                },
                onpointerup: move |_| drag_from.set(None),
                onpointercancel: move |_| drag_from.set(None),
                onpointerleave: move |_| drag_from.set(None),
                for (i, offset, card) in cards {
                    div {
                        key: "{card.scryfall_data.id}",
                        class: if offset == 0 { "cover-flow-card selected" } else { "cover-flow-card" },
                        style: flow_slot(offset).style(),
                        onclick: move |_| {
                            if *dragged.peek() {
                                return;
                            }
                            if offset == 0 {
                                if let Some(handler) = on_pick {
                                    handler.call(());
                                }
                            } else {
                                go(i);
                            }
                        },
                        {
                            let url = card.scryfall_data.primary_image_url(ImageSize::Large).map(str::to_owned);
                            let state = url.as_deref().and_then(|u| {
                                settled.read().get(u).copied().or_else(|| image_seen(u).then_some(true))
                            });
                            match (tile(url.is_some(), state), url) {
                                (Tile::Missing, _) | (_, None) => rsx! {
                                    div { class: "cover-flow-missing", "{card.scryfall_data.set_name}" }
                                },
                                (shown, Some(url)) => {
                                    let loaded = url.clone();
                                    let failed = url.clone();
                                    rsx! {
                                        if shown == Tile::Ghost {
                                            div { class: "cover-flow-ghost" }
                                        }
                                        img {
                                            class: if shown == Tile::Image { "shown" } else { "" },
                                            src: "{url}",
                                            alt: "{card.scryfall_data.name}, {card.scryfall_data.set_name}",
                                            loading: "lazy",
                                            draggable: false,
                                            onload: move |_| {
                                                mark_image_seen(&loaded);
                                                settled.write().insert(loaded.clone(), true);
                                            },
                                            onerror: move |_| {
                                                settled.write().insert(failed.clone(), false);
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if count > 1 {
                button {
                    r#type: "button",
                    class: "btn-action btn-icon cover-flow-step",
                    disabled: at.saturating_add(1) >= count,
                    "aria-label": "Next printing",
                    onclick: move |_| step(Step::Forward),
                    onkeydown: move |e: Event<KeyboardData>| {
                        if e.key() == Key::Enter {
                            e.stop_propagation();
                        }
                    },
                    DockIcon { path: ARROW_RIGHT }
                }
            }
        }
    }
}

/// How many printings show on each side of the selected one.
const FLOW_REACH: usize = 10;

/// The first side card's distance from the middle, in percent of a card's
/// width.
const FIRST_SHIFT_PCT: f64 = 52.0;
/// How much further each next side card sits, in percent of a card's width.
const STEP_SHIFT_PCT: f64 = 13.0;
/// The first side card's scale.
const FIRST_SCALE: f64 = 0.82;
/// How much smaller each next side card is.
const STEP_SCALE: f64 = 0.05;
/// The smallest a side card gets.
const MIN_SCALE: f64 = 0.36;
/// How far a side card turns toward the middle, in degrees.
const TURN_DEG: f64 = 38.0;
/// The first side card's brightness.
const FIRST_SHADE: f64 = 0.7;
/// How much darker each next side card is.
const STEP_SHADE: f64 = 0.05;
/// The darkest a side card gets.
const MIN_SHADE: f64 = 0.25;
/// The selected card's stacking layer; each step out is one lower.
const TOP_LAYER: i32 = 100;

/// A printing's place in the flow.
#[derive(Debug, Clone, Copy, PartialEq)]
struct FlowSlot {
    /// Sideways shift from the middle, in percent of a card's width;
    /// negative to the left.
    shift_pct: f64,
    /// Size against the selected card's.
    scale: f64,
    /// Turn about the vertical axis in degrees, toward the middle.
    turn_deg: f64,
    /// Stacking layer: the selected card on top, each step out lower.
    layer: i32,
    /// Brightness, 1 for the selected card.
    shade: f64,
}

impl FlowSlot {
    /// The slot as an inline style for a card whose left edge sits at the
    /// stage's middle: the transform (half a card back, then the shift),
    /// its stacking, and `--flow-dim`, how dark the card's veil is.
    #[must_use]
    fn style(&self) -> String {
        format!(
            "transform: translateX({:.2}%) rotateY({:.2}deg) scale({:.3}); z-index: {}; --flow-dim: {:.3};",
            self.shift_pct - 50.0,
            self.turn_deg,
            self.scale,
            self.layer,
            1.0 - self.shade
        )
    }
}

/// The slot of the printing `offset` places from the selected one: 0 for
/// the selected printing, negative to its left, positive to its right.
#[must_use]
fn flow_slot(offset: i32) -> FlowSlot {
    if offset == 0 {
        return FlowSlot {
            shift_pct: 0.0,
            scale: 1.0,
            turn_deg: 0.0,
            layer: TOP_LAYER,
            shade: 1.0,
        };
    }
    let side = if offset < 0 { -1.0 } else { 1.0 };
    let steps = offset.unsigned_abs();
    let further = f64::from(steps.saturating_sub(1));
    let layer = TOP_LAYER.saturating_sub(i32::try_from(steps).unwrap_or(i32::MAX));
    FlowSlot {
        shift_pct: side * (FIRST_SHIFT_PCT + STEP_SHIFT_PCT * further),
        scale: (FIRST_SCALE - STEP_SCALE * further).max(MIN_SCALE),
        turn_deg: -side * TURN_DEG,
        layer,
        shade: (FIRST_SHADE - STEP_SHADE * further).max(MIN_SHADE),
    }
}

/// The indexes the flow draws around `selected` in a list of `count`: the
/// selected one and up to [`FLOW_REACH`] on each side, with each one's
/// offset from the selected.
#[must_use]
fn flow_window(selected: usize, count: usize) -> Vec<(usize, i32)> {
    if count == 0 {
        return Vec::new();
    }
    let selected = selected.min(count.saturating_sub(1));
    let first = selected.saturating_sub(FLOW_REACH);
    let last = selected
        .saturating_add(FLOW_REACH)
        .min(count.saturating_sub(1));
    (first..=last)
        .map(|i| {
            let offset = if i >= selected {
                i32::try_from(i.saturating_sub(selected)).unwrap_or(i32::MAX)
            } else {
                i32::try_from(selected.saturating_sub(i))
                    .unwrap_or(i32::MAX)
                    .saturating_neg()
            };
            (i, offset)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use dioxus::html::geometry::LinesVector;

    use super::*;

    #[test]
    fn a_line_wheel_notch_steps_one_printing() {
        let notch = WheelDelta::Lines(LinesVector::new(0.0, 3.0, 0.0));
        let (way, steps, rest) = wheel_steps(0.0, wheel_px(&notch), 10);
        assert_eq!((way, steps), (Some(Step::Forward), 1));
        assert!(rest.abs() < f64::EPSILON, "{rest}");
    }

    #[test]
    fn a_small_scroll_carries_until_it_makes_a_step() {
        assert_eq!(wheel_steps(0.0, 40.0, 10), (None, 0, 40.0));
        let (way, steps, rest) = wheel_steps(40.0, 30.0, 10);
        assert_eq!((way, steps), (Some(Step::Forward), 1));
        assert!((rest - 10.0).abs() < f64::EPSILON, "{rest}");
        let (way, steps, _) = wheel_steps(0.0, -130.0, 10);
        assert_eq!((way, steps), (Some(Step::Back), 2));
    }

    #[test]
    fn a_huge_scroll_steps_at_most_the_limit_and_keeps_less_than_a_step() {
        let (way, steps, rest) = wheel_steps(0.0, 1.0e12, 5);
        assert_eq!((way, steps), (Some(Step::Forward), 5));
        assert!(rest.abs() < WHEEL_STEP_PX, "{rest}");
    }

    #[test]
    fn a_scroll_that_is_not_finite_changes_nothing() {
        assert_eq!(wheel_steps(12.0, f64::NAN, 10), (None, 0, 12.0));
        assert_eq!(wheel_steps(12.0, f64::INFINITY, 10), (None, 0, 12.0));
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn the_selected_printing_faces_the_viewer_on_top() {
        let slot = flow_slot(0);
        assert!(near(slot.shift_pct, 0.0));
        assert!(near(slot.scale, 1.0));
        assert!(near(slot.turn_deg, 0.0));
        assert!(near(slot.shade, 1.0));
        assert_eq!(slot.layer, TOP_LAYER);
    }

    #[test]
    fn sides_mirror_each_other() {
        for n in 1..=10 {
            let (left, right) = (flow_slot(-n), flow_slot(n));
            assert!(near(left.shift_pct, -right.shift_pct), "shift at {n}");
            assert!(near(left.turn_deg, -right.turn_deg), "turn at {n}");
            assert!(near(left.scale, right.scale), "scale at {n}");
            assert!(near(left.shade, right.shade), "shade at {n}");
            assert_eq!(left.layer, right.layer, "layer at {n}");
        }
    }

    #[test]
    fn side_cards_turn_toward_the_middle() {
        assert!(flow_slot(1).turn_deg < 0.0);
        assert!(flow_slot(-1).turn_deg > 0.0);
        assert!(flow_slot(1).shift_pct > 0.0);
        assert!(flow_slot(-1).shift_pct < 0.0);
    }

    #[test]
    fn each_step_out_is_further_smaller_darker_and_lower() {
        for n in 0..10 {
            let (near_slot, far_slot) = (flow_slot(n), flow_slot(n + 1));
            assert!(far_slot.shift_pct > near_slot.shift_pct, "shift at {n}");
            assert!(far_slot.scale <= near_slot.scale, "scale at {n}");
            assert!(far_slot.shade <= near_slot.shade, "shade at {n}");
            assert!(far_slot.layer < near_slot.layer, "layer at {n}");
        }
    }

    #[test]
    fn scale_and_shade_stop_at_their_floors() {
        let far = flow_slot(40);
        assert!(near(far.scale, MIN_SCALE));
        assert!(near(far.shade, MIN_SHADE));
        assert!(flow_slot(10).scale >= MIN_SCALE);
    }

    #[test]
    fn the_window_holds_the_selected_and_ten_each_side() {
        let window = flow_window(50, 300);
        assert_eq!(window.len(), 21);
        assert_eq!(window.first(), Some(&(40, -10)));
        assert_eq!(window.get(10), Some(&(50, 0)));
        assert_eq!(window.last(), Some(&(60, 10)));
    }

    #[test]
    fn a_printing_ghosts_until_its_own_image_arrives() {
        assert_eq!(tile(true, None), Tile::Ghost, "loading");
        assert_eq!(tile(true, Some(true)), Tile::Image, "loaded");
        assert_eq!(tile(true, Some(false)), Tile::Missing, "failed");
        assert_eq!(tile(false, None), Tile::Missing, "no art");
    }

    #[test]
    fn the_window_stops_at_the_list_ends() {
        assert_eq!(flow_window(0, 3), vec![(0, 0), (1, 1), (2, 2)]);
        assert_eq!(flow_window(2, 3), vec![(0, -2), (1, -1), (2, 0)]);
        assert_eq!(flow_window(0, 0), Vec::new());
        assert_eq!(flow_window(9, 3).last(), Some(&(2, 0)));
    }
}
