//! Where each printing sits in the printing picker's cover flow on a wide
//! window: the selected one large and facing the viewer, the others stacked
//! behind it on either side, each further one smaller, more overlapped,
//! darker and turned toward the middle.

/// How many printings show on each side of the selected one.
pub const FLOW_REACH: usize = 10;

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
pub struct FlowSlot {
    /// Sideways shift from the middle, in percent of a card's width;
    /// negative to the left.
    pub shift_pct: f64,
    /// Size against the selected card's.
    pub scale: f64,
    /// Turn about the vertical axis in degrees, toward the middle.
    pub turn_deg: f64,
    /// Stacking layer: the selected card on top, each step out lower.
    pub layer: i32,
    /// Brightness, 1 for the selected card.
    pub shade: f64,
}

impl FlowSlot {
    /// The slot as an inline style for a card whose left edge sits at the
    /// stage's middle: the transform (half a card back, then the shift),
    /// its stacking, and `--flow-dim`, how dark the card's veil is.
    #[must_use]
    pub fn style(&self) -> String {
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
pub fn flow_slot(offset: i32) -> FlowSlot {
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
pub fn flow_window(selected: usize, count: usize) -> Vec<(usize, i32)> {
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
    use super::*;

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
    fn the_window_stops_at_the_list_ends() {
        assert_eq!(flow_window(0, 3), vec![(0, 0), (1, 1), (2, 2)]);
        assert_eq!(flow_window(2, 3), vec![(0, -2), (1, -1), (2, 0)]);
        assert_eq!(flow_window(0, 0), Vec::new());
        assert_eq!(flow_window(9, 3).last(), Some(&(2, 0)));
    }
}
