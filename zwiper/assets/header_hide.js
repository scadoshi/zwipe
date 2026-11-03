// The header follows the scroll, as in X: scrolling a screen's content down
// slides its header up behind the status bar strip, point for point, until it
// is gone, and any scroll up slides it back the same way. At the top of the
// content it is always fully shown. One listener, in the capture phase since
// scroll events do not bubble, serves every screen. It guards itself so a
// second load does nothing.
(() => {
    if (window.__zwipeHeaderHide) return;
    window.__zwipeHeaderHide = true;

    // The last scroll position seen per scroll area, and how far each header is
    // pushed up, in pixels.
    const lastTop = new WeakMap();
    const offset = new WeakMap();

    document.addEventListener(
        "scroll",
        (event) => {
            const area = event.target;
            if (!(area instanceof Element) || !area.classList.contains("screen-content")) return;
            const nav = area.parentElement && area.parentElement.querySelector(":scope > .app-nav");
            if (!nav) return;
            // The menu is open: leave the header where it is.
            if (nav.querySelector(".nav-panel-open")) return;

            const top = Math.max(0, area.scrollTop);
            const bottom = area.scrollHeight - area.clientHeight;
            const previous = lastTop.has(area) ? lastTop.get(area) : top;
            lastTop.set(area, top);
            // The bounce past the bottom edge, and its spring back, are not
            // scrolling: at the bottom the header may only go, never come back.
            if (top > bottom || previous > bottom) return;
            if (top < previous && top >= bottom - 2) return;

            const height = nav.offsetHeight;
            let pushed = (offset.get(nav) || 0) + (top - previous);
            pushed = top <= 0 ? 0 : Math.min(height, Math.max(0, pushed));
            offset.set(nav, pushed);
            nav.style.transform = pushed ? `translateY(${-pushed}px)` : "";
        },
        { capture: true, passive: true },
    );
})();
