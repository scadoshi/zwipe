// The floating action docks rise above the keyboard. The visual viewport
// shrinks when the keyboard comes up, accessory bar included, and the strip
// it hides, in pixels, lands in --keyboard-inset on the root, which
// .app-float adds to its bottom. It guards itself so a second load does
// nothing, and does nothing where the visual viewport API is missing.
(() => {
    if (window.__zwipeKeyboardInset) return;
    window.__zwipeKeyboardInset = true;
    const viewport = window.visualViewport;
    if (!viewport) return;

    const update = () => {
        const hidden = window.innerHeight - viewport.height - viewport.offsetTop;
        const inset = Math.max(0, Math.round(hidden));
        document.documentElement.style.setProperty("--keyboard-inset", `${inset}px`);
    };

    viewport.addEventListener("resize", update);
    viewport.addEventListener("scroll", update);
    update();
})();
