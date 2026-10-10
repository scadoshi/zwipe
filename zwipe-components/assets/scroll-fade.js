// Scroll fade for browsers without scroll timelines (Firefox, older Safari):
// sets the --fade-left/--fade-right widths that components.css otherwise
// animates on every .scroll-fade-x and .diagram-scroll, and --fade-top/
// --fade-bottom on every .scroll-fade-y, following the same keyframes. No
// fade at the start edge at the start, none at the end edge at the end, the
// full 1.5rem once the box is 8% in from either end, and none on a box that
// does not overflow. Browsers with scroll timelines return at once.
//
// The MutationObserver picks up boxes the SPA router mounts after load.
//
// Inlined in the head, it waits for the document to finish parsing, the
// moment a deferred script would run.
(() => {
    if (CSS.supports("animation-timeline: scroll()")) return;

    const BOXES = ".scroll-fade-x, .diagram-scroll, .scroll-fade-y";
    const RAMP = 0.08;

    const update = (el) => {
        const vertical = el.classList.contains("scroll-fade-y");
        const range = vertical
            ? el.scrollHeight - el.clientHeight
            : el.scrollWidth - el.clientWidth;
        let start = 0;
        let end = 0;
        if (range > 0) {
            const progress = (vertical ? el.scrollTop : el.scrollLeft) / range;
            start = Math.min(progress / RAMP, 1);
            end = Math.min((1 - progress) / RAMP, 1);
        }
        el.style.setProperty(vertical ? "--fade-top" : "--fade-left", `${start * 1.5}rem`);
        el.style.setProperty(vertical ? "--fade-bottom" : "--fade-right", `${end * 1.5}rem`);
    };

    const start = () => {
        // Watched element to its box: a box, or the content inside one.
        const boxOf = new WeakMap();
        const resize = new ResizeObserver((entries) => {
            for (const entry of entries) update(boxOf.get(entry.target));
        });

        const seen = new WeakSet();
        const scan = () => {
            for (const el of document.querySelectorAll(BOXES)) {
                if (seen.has(el)) continue;
                seen.add(el);
                el.addEventListener("scroll", () => update(el), { passive: true });
                boxOf.set(el, el);
                resize.observe(el);
                // The box's content can widen it without resizing the box.
                const content = el.firstElementChild;
                if (content) {
                    boxOf.set(content, el);
                    resize.observe(content);
                }
                update(el);
            }
        };

        scan();
        new MutationObserver(scan).observe(document.body, {
            childList: true,
            subtree: true,
        });
    };

    if (document.readyState === "loading") {
        addEventListener("DOMContentLoaded", start, { once: true });
    } else {
        start();
    }
})();
