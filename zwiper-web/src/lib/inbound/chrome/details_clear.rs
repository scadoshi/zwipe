//! The card on a swipe screen slides clear of its docked details panel.
//!
//! On a wide window the card's details dock at the right as a panel
//! (`details_eye`). When the shown panel would lie over the card and the room
//! left of the card fits it clear, the card slides left just far enough to
//! clear the panel, on the panel's own easing; with too little room it stays
//! where it is and the panel lies over it, and a panel that misses the card
//! moves nothing. Hidden, the card slides back. Add, Remove, Commander swipe
//! and the Oracle tag examples share it.

use dioxus::prelude::*;

/// Each surface: its body, its docked panel and its card stack, as selectors. The stack slides;
/// its top card's image is what clears the panel.
pub(crate) const SURFACES: [(&str, &str, &str); 3] = [
    (".web-cards-swipe", ".web-swipe-details", ".web-swipe-stack"),
    (
        ".web-swipe-select-body",
        ".web-swipe-details",
        ".web-swipe-stack",
    ),
    (
        ".web-examples-body",
        ".web-examples-details",
        ".web-swipe-stack",
    ),
];

/// The gap the card keeps from the panel, and from its body's left edge, in
/// CSS pixels.
const CLEAR_GAP_PX: u8 = 16;

/// Installs the slide once per tab: it measures each surface on a resize and
/// whenever a class changes (the panel shown or hidden, a card coming in).
/// The card's own `translate` carries the shift, so its other transforms are
/// untouched; the panel's and the card's positions are read at rest, less any
/// slide in flight.
const DETAILS_CLEAR_JS: &str = r"
if (!window.__zwipeDetailsClear) {
    window.__zwipeDetailsClear = true;
    const surfaces = __SURFACES__;
    const gap = __GAP__;
    const offsetX = (el, prop) => {
        const v = getComputedStyle(el)[prop];
        if (!v || v === 'none') return 0;
        if (prop === 'transform') return new DOMMatrix(v).m41;
        return parseFloat(v) || 0;
    };
    const run = () => {
        for (const [bodySel, panelSel, cardSel] of surfaces) {
            for (const body of document.querySelectorAll(bodySel)) {
                const card = body.querySelector(cardSel);
                if (!card) continue;
                const top = card.querySelector('.swipe-stack-card.top img')
                    || card.querySelector('.swipe-stack-card.top')
                    || card;
                const panel = body.querySelector(panelSel);
                let shift = 0;
                const shown = panel && !body.classList.contains('web-details-hidden')
                    && getComputedStyle(panel).position === 'fixed';
                if (shown) {
                    const c = top.getBoundingClientRect();
                    const at = offsetX(card, 'translate');
                    const left = c.left - at;
                    const right = c.right - at;
                    const panelLeft = panel.getBoundingClientRect().left - offsetX(panel, 'transform');
                    const need = right - (panelLeft - gap);
                    const room = left - (body.getBoundingClientRect().left + gap);
                    if (need > 0 && room >= need) shift = -need;
                }
                const next = shift ? `${shift}px 0` : '';
                if (card.style.translate !== next) card.style.translate = next;
            }
        }
    };
    let queued = false;
    const schedule = () => {
        if (queued) return;
        queued = true;
        requestAnimationFrame(() => { queued = false; run(); });
    };
    window.addEventListener('resize', schedule);
    new MutationObserver(schedule).observe(document.body, {
        subtree: true,
        childList: true,
        attributes: true,
        attributeFilter: ['class'],
    });
    schedule();
}
";

/// The slide's script with its surfaces and gap filled in.
fn script() -> String {
    let surfaces = SURFACES
        .iter()
        .map(|(body, panel, card)| format!("['{body}', '{panel}', '{card}']"))
        .collect::<Vec<_>>()
        .join(", ");
    DETAILS_CLEAR_JS
        .replace("__SURFACES__", &format!("[{surfaces}]"))
        .replace("__GAP__", &CLEAR_GAP_PX.to_string())
}

/// Installs the slide. Call once, in the app's root.
pub fn use_details_clear() {
    use_hook(|| {
        document::eval(&script());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_swipe_surface_and_the_examples_slide() {
        let js = script();
        for (body, panel, card) in SURFACES {
            assert!(
                js.contains(&format!("['{body}', '{panel}', '{card}']")),
                "{js}"
            );
        }
        assert!(js.contains(".web-examples-body"), "the examples slide too");
    }

    #[test]
    fn the_card_moves_only_when_it_overlaps_and_the_room_fits_it() {
        assert!(
            DETAILS_CLEAR_JS.contains("if (need > 0 && room >= need) shift = -need;"),
            "{DETAILS_CLEAR_JS}"
        );
    }
}
