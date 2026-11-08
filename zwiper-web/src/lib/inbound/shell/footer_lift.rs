//! Keeps the toast stack just above the site footer while the footer is in
//! view, and at the window's foot otherwise. The browser reports the
//! footer's top against the viewport as the page scrolls or changes, and
//! the lift goes on the root as `--web-footer-lift`, which web.css raises
//! the stack by.

use dioxus::prelude::*;

/// How far, in CSS pixels, the footer reaches up into a viewport
/// `viewport_height` tall when its top edge sits at `footer_top` from the
/// viewport's top: none while it is below the fold or not drawn, never
/// more than the whole viewport.
fn footer_lift(viewport_height: f64, footer_top: Option<f64>) -> f64 {
    footer_top.map_or(0.0, |top| {
        (viewport_height - top).clamp(0.0, viewport_height.max(0.0))
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
/// is drawn) once a frame at most, whenever a scroll, a resize or a change
/// to the page could have moved the footer, and only when it changed; puts
/// each lift it receives back on the root.
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
        if (rect.height > 0 && (top === null || rect.top < top)) top = rect.top;
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

    #[test]
    fn a_footer_below_the_fold_or_not_drawn_lifts_nothing() {
        assert_eq!(footer_lift(700.0, Some(900.0)), 0.0);
        assert_eq!(footer_lift(700.0, Some(700.0)), 0.0);
        assert_eq!(footer_lift(700.0, None), 0.0);
    }

    #[test]
    fn a_footer_in_view_lifts_by_as_much_as_shows() {
        assert_eq!(footer_lift(700.0, Some(522.0)), 178.0);
        assert_eq!(footer_lift(600.0, Some(599.5)), 0.5);
    }

    #[test]
    fn the_lift_never_passes_the_viewport() {
        assert_eq!(footer_lift(700.0, Some(0.0)), 700.0);
        assert_eq!(footer_lift(700.0, Some(-40.0)), 700.0);
    }
}
