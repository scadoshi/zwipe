// Scroll reveal: panels below the fold fade up as they enter the viewport,
// staggered when several arrive in the same frame. Panels already on screen
// at setup are left alone, so this never fights the load-time entrance
// animations. Classes are removed once the transition finishes, returning
// each panel to plain styling (and its normal hover transition).
//
// The MutationObserver re-scans after route changes, since the SPA router
// swaps page content without a page load. Revealed elements are dropped from
// the WeakSet by GC when the router discards them.
(() => {
    if (matchMedia("(prefers-reduced-motion: reduce)").matches) return;

    const io = new IntersectionObserver(
        (entries) => {
            let i = 0;
            for (const entry of entries) {
                if (!entry.isIntersecting) continue;
                const el = entry.target;
                io.unobserve(el);
                el.style.transitionDelay = `${i++ * 70}ms`;
                el.addEventListener(
                    "transitionend",
                    () => {
                        el.classList.remove("reveal-pending", "reveal-in");
                        el.style.transitionDelay = "";
                    },
                    { once: true },
                );
                el.classList.add("reveal-in");
            }
        },
        // Fire once ~8% of the viewport height remains below the element's
        // top edge, so the motion is visible rather than pre-completed.
        { rootMargin: "0px 0px -8% 0px" },
    );

    const seen = new WeakSet();
    const scan = () => {
        for (const el of document.querySelectorAll(".panel-card")) {
            if (seen.has(el)) continue;
            seen.add(el);
            if (el.getBoundingClientRect().top > innerHeight) {
                el.classList.add("reveal-pending");
                io.observe(el);
            }
        }
    };

    scan();
    new MutationObserver(scan).observe(document.body, {
        childList: true,
        subtree: true,
    });
})();
