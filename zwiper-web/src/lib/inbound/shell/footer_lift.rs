//! How far the site footer reaches up into the window, for the side drawers:
//! while the footer is in view a drawer and its dock end just above it, and at
//! the window's foot otherwise. The browser reports the footer's top against
//! the viewport as the page scrolls or changes, and the reach goes on the root
//! as `--web-footer-lift`, which web.css turns into the drawers' floor.

use dioxus::prelude::*;

/// How far, in whole CSS pixels, the footer reaches up into a viewport
/// `viewport_height` tall when its top edge sits at `footer_top` from the
/// viewport's top: none while it is below the fold or not drawn, never
/// more than the whole viewport. A fractional reach rounds up, so what
/// rests on the lift clears the footer and keeps its edges on whole pixels.
fn footer_lift(viewport_height: f64, footer_top: Option<f64>) -> f64 {
    footer_top.map_or(0.0, |top| {
        (viewport_height - top)
            .ceil()
            .clamp(0.0, viewport_height.max(0.0))
    })
}

/// Watches the footer for the life of the app and keeps
/// `--web-footer-lift` current. Off the browser it does nothing.
pub fn use_footer_lift() {
    use_hook(|| {
        spawn(async move {
            let mut watch = document::eval(FOOTER_LIFT_JS);
            let mut last: Option<f64> = None;
            while let Ok((viewport_height, footer_top)) = watch.recv::<(f64, Option<f64>)>().await {
                let lift = footer_lift(viewport_height, footer_top);
                if last != Some(lift) {
                    last = Some(lift);
                    if watch.send(lift).is_err() {
                        break;
                    }
                }
            }
        });
    });
}

/// Sends `[viewport height, footer top]` (the top `null` while no footer
/// is drawn and painted; a screen a layer covers leaves its own unpainted)
/// once a frame at most, whenever a scroll, a resize or a change to the page
/// could have moved the footer, and only when it changed; puts each lift it
/// receives back on the root.
const FOOTER_LIFT_JS: &str = r"
if (window.__zwipeFooterLift) window.__zwipeFooterLift();
let frame = 0;
let last = '';
const sized = new ResizeObserver(() => queue());
const measure = () => {
    frame = 0;
    let top = null;
    for (const footer of document.querySelectorAll('footer.web-site-footer')) {
        const rect = footer.getBoundingClientRect();
        const painted = getComputedStyle(footer).visibility === 'visible';
        if (painted && rect.height > 0 && (top === null || rect.top < top)) top = rect.top;
        if (footer.previousElementSibling) sized.observe(footer.previousElementSibling);
    }
    const key = window.innerHeight + ':' + top;
    if (key !== last) {
        last = key;
        dioxus.send([window.innerHeight, top]);
    }
};
const queue = () => {
    if (!frame) frame = requestAnimationFrame(measure);
};
const changes = new MutationObserver(queue);
changes.observe(document.body, { childList: true, subtree: true });
document.addEventListener('scroll', queue, { capture: true, passive: true });
window.addEventListener('resize', queue);
window.__zwipeFooterLift = () => {
    changes.disconnect();
    sized.disconnect();
    document.removeEventListener('scroll', queue, { capture: true });
    window.removeEventListener('resize', queue);
};
queue();
while (true) {
    const lift = await dioxus.recv();
    document.documentElement.style.setProperty('--web-footer-lift', lift + 'px');
}
";

#[cfg(test)]
mod tests {
    use super::*;

    /// The web's own sheet and the drawer sheet it loads from components.
    const WEB_CSS: &str = include_str!("../../../../assets/web.css");
    const FILTER_CSS: &str = include_str!("../../../../../components/assets/filter.css");

    #[test]
    fn every_drawer_stops_at_the_footer_s_top() {
        assert!(
            WEB_CSS.contains("--sheet-drawer-floor: var(--web-footer-lift, 0px);"),
            "the kit's drawer floor is the footer in view"
        );
        let narrow = FILTER_CSS
            .split(".sheet-form-drawer > .bottom-sheet.drawer-every-width {")
            .nth(1)
            .and_then(|rest| rest.split('}').next())
            .expect("the narrow drawer rule");
        let bottom = narrow
            .lines()
            .map(str::trim)
            .find(|line| line.starts_with("bottom:"))
            .expect("the narrow drawer sets its bottom");
        assert!(
            bottom.contains("var(--sheet-drawer-floor, 0px)"),
            "the narrow drawer rises with the floor too: {bottom}"
        );
    }

    #[test]
    fn a_footer_a_layer_covers_lifts_nothing() {
        let hidden = WEB_CSS
            .split("\n.web-scroll-screen:has(.under-layer) > .web-site-footer {")
            .nth(1)
            .and_then(|rest| rest.split('}').next())
            .expect("the covered screen's footer rule");
        assert!(hidden.contains("visibility: hidden;"), "{hidden}");
        assert!(
            FOOTER_LIFT_JS.contains("getComputedStyle(footer).visibility === 'visible'")
                && FOOTER_LIFT_JS.contains("if (painted && "),
            "only a painted footer lifts the drawers"
        );
    }

    #[test]
    fn a_footer_below_the_fold_or_not_drawn_lifts_nothing() {
        assert_eq!(footer_lift(700.0, Some(900.0)), 0.0);
        assert_eq!(footer_lift(700.0, Some(700.0)), 0.0);
        assert_eq!(footer_lift(700.0, None), 0.0);
    }

    #[test]
    fn a_footer_in_view_lifts_by_as_much_as_shows() {
        assert_eq!(footer_lift(700.0, Some(522.0)), 178.0);
    }

    #[test]
    fn a_fractional_footer_top_lifts_by_whole_pixels() {
        assert_eq!(footer_lift(700.0, Some(522.4)), 178.0);
        assert_eq!(footer_lift(700.0, Some(522.9)), 178.0);
        assert_eq!(footer_lift(600.0, Some(599.5)), 1.0);
    }

    #[test]
    fn the_lift_never_passes_the_viewport() {
        assert_eq!(footer_lift(700.0, Some(0.0)), 700.0);
        assert_eq!(footer_lift(700.0, Some(-40.0)), 700.0);
    }
}
