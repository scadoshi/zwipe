//! A full-screen layer that slides in and out, such as a deck's Cards over its Profile: it stays
//! mounted while it slides out and drops once its slide ends, so closing eases as opening does.
//! The slide is `magic.css`'s `deck-cards-layer-in` and `deck-cards-layer-out`; it times on the
//! animation's own end, so it needs no clock and runs the same in the browser and the apps.

use dioxus::prelude::*;

/// The keyframes a closing layer runs, whose end drops it.
pub const LAYER_OUT: &str = "deck-cards-layer-out";

/// Whether a layer shows, and whether it is sliding out. `Copy`: two signals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerPresence {
    shown: Signal<bool>,
    leaving: Signal<bool>,
}

impl LayerPresence {
    /// Whether the layer is mounted: open, or sliding out.
    #[must_use]
    pub fn shown(self) -> bool {
        (self.shown)()
    }

    /// Whether the layer is sliding out, which its `leaving` class runs.
    #[must_use]
    pub fn leaving(self) -> bool {
        (self.leaving)()
    }

    /// A layer's animation `name` ended: the slide out drops the layer, and any other animation
    /// (its slide in, an animation inside it) leaves it.
    pub fn ended(self, name: &str) {
        let Self {
            mut shown,
            mut leaving,
        } = self;
        if name == LAYER_OUT && *leaving.peek() {
            leaving.set(false);
            shown.set(false);
        }
    }
}

/// The presence of a layer that is `open`: it mounts as it opens and slides out as it closes.
pub fn use_layer_presence(open: bool) -> LayerPresence {
    let mut shown = use_signal(|| open);
    let mut leaving = use_signal(|| false);
    use_effect(use_reactive!(|open| {
        if open {
            leaving.set(false);
            shown.set(true);
        } else if *shown.peek() {
            leaving.set(true);
        }
    }));
    LayerPresence { shown, leaving }
}
