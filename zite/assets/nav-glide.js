// Nav glide: when a nav item changes width (the theme picker's label, the
// light/dark toggle), the items it pushes slide to their new places instead
// of jumping. Each item's spot is remembered; after a change in the nav, an
// item that moved starts at its old spot (a transform) and eases to the new.
//
// Spots are offsetLeft, which ignores transforms, so a change that lands
// mid-glide starts from where the layout was rather than where the item is
// drawn. Resizes and font loads update the spots without animating.
(() => {
    if (matchMedia("(prefers-reduced-motion: reduce)").matches) return;

    const ITEMS = ".nav-links > li, .nav-stores-persistent > *, .theme-switcher > *";
    let spots = new Map();

    const measure = () => {
        const now = new Map();
        document.querySelectorAll(ITEMS).forEach((el) => now.set(el, el.offsetLeft));
        return now;
    };

    const glide = () => {
        const now = measure();
        // During a theme wipe the snapshots carry the change; just catch up.
        if ("wiping" in document.documentElement.dataset) {
            spots = now;
            return;
        }
        now.forEach((x, el) => {
            const before = spots.get(el);
            if (before === undefined || Math.abs(before - x) < 0.5) return;
            el.style.transition = "none";
            el.style.transform = `translateX(${before - x}px)`;
            el.getBoundingClientRect();
            el.style.transition = "transform 0.27s cubic-bezier(0.65, 0, 0.35, 1)";
            el.style.transform = "";
            el.addEventListener("transitionend", () => (el.style.transition = ""), { once: true });
        });
        spots = now;
    };

    const inNav = (node) => {
        const el = node.nodeType === 1 ? node : node.parentElement;
        return el && el.closest(".nav-wrapper");
    };

    new MutationObserver((records) => {
        if (records.some((r) => inNav(r.target))) glide();
    }).observe(document.body, { childList: true, subtree: true, characterData: true });

    const remember = () => (spots = measure());
    remember();
    addEventListener("resize", remember);
    document.fonts.ready.then(remember);
})();
