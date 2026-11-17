// A move key (outside a text field, with no Ctrl, Alt or Meta) puts the page in keyboard mode:
// `keyboard-nav` on the root hides the pointer and lets it through the cards. A mouse move that
// covers distance, a press or a wheel ends it. Chrome sends a still mouse a move with no distance
// whenever the page scrolls under it, so only a move with distance counts.
(() => {
    if (window.__zwipeKeyboardPointer) return;
    window.__zwipeKeyboardPointer = true;
    const root = document.documentElement;
    const keys = __KEYS__;
    const typingTags = __TYPING__;
    document.addEventListener('keydown', (e) => {
        if (e.ctrlKey || e.altKey || e.metaKey || e.isComposing || !keys.includes(e.key)) return;
        const f = document.activeElement;
        if (f && (typingTags.includes(f.tagName) || f.isContentEditable)) return;
        root.classList.add('keyboard-nav');
    }, true);
    const mouse = () => root.classList.remove('keyboard-nav');
    document.addEventListener('mousemove', (e) => { if (e.movementX || e.movementY) mouse(); }, true);
    document.addEventListener('pointerdown', mouse, true);
    document.addEventListener('wheel', mouse, { capture: true, passive: true });
})();
