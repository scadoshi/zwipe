// The header follows the scroll at phone width, as the app's does: scrolling
// a screen down slides the header up, point for point, until it is gone, and
// any scroll up slides it back the same way. At the top of the screen it is
// always fully shown. The web's copy of zwiper's header_hide.js, keyed to the
// shell: the header is `.web-shell > .web-app-nav` and the scroller is the
// shell's `.web-scroll-screen`. On a wide window the header sits in the flow
// and holds still. One listener, in the capture phase since scroll events do
// not bubble, serves every screen. It guards itself so a second load does
// nothing.
(() => {
    if (window.__zwipeWebHeaderHide) return;
    window.__zwipeWebHeaderHide = true;

    const phone = window.matchMedia("(max-width: 899px)");

    // The last scroll position seen per scroll area, and how far each header is
    // pushed up, in pixels.
    const lastTop = new WeakMap();
    const offset = new WeakMap();

    const headerOf = (area) => {
        const shell = area.closest(".web-shell");
        return shell && shell.querySelector(":scope > .web-app-nav");
    };

    document.addEventListener(
        "scroll",
        (event) => {
            if (!phone.matches) return;
            const area = event.target;
            if (!(area instanceof Element) || !area.classList.contains("web-scroll-screen")) return;
            const nav = headerOf(area);
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

    // A header whose screen sits at the top is fully shown. The shell keeps
    // its header across screens, so one that slid away on a scrolled screen
    // would otherwise arrive on the next screen already gone.
    const reset = () => {
        for (const nav of document.querySelectorAll(".web-shell > .web-app-nav")) {
            const area = nav.parentElement.querySelector(".web-scroll-screen");
            if (phone.matches && area && area.scrollTop > 0) continue;
            if (area) lastTop.delete(area);
            offset.delete(nav);
            nav.style.transform = "";
        }
    };
    new MutationObserver(reset).observe(document.documentElement, { childList: true, subtree: true });
    // A window widened past the fold shows the header in the flow again.
    phone.addEventListener("change", reset);
})();
