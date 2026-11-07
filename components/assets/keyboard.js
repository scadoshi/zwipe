// The on-screen keyboard. While a text field has focus and the viewport has
// shrunk under it (Android shrinks the page, iOS the visual viewport), the
// root carries `keyboard-up`, which hides the floating docks, and the field
// scrolls into view if it sits under the keyboard or the header. The
// viewport's height with nothing focused is the reference, taken afresh
// whenever no field has focus, so a rotation resets it. It guards itself so
// a second load does nothing.
(() => {
    if (window.__zwipeKeyboard) return;
    window.__zwipeKeyboard = true;
    const viewport = window.visualViewport;
    // How much the viewport has to shrink to count as a keyboard.
    const KEYBOARD_PX = 120;
    // Room kept clear above a revealed field, for the header.
    const HEADER_PX = 64;
    const FIELD =
        'input:not([type="checkbox"]):not([type="radio"]):not([type="button"]):not([type="submit"]):not([type="range"]), textarea, select, [contenteditable="true"]';
    const height = () => (viewport ? viewport.height : window.innerHeight);
    const focusedField = () => {
        const el = document.activeElement;
        return el && el.matches && el.matches(FIELD) ? el : null;
    };
    let resting = height();

    const reveal = () => {
        const field = focusedField();
        if (!field) return;
        const top = viewport ? viewport.offsetTop : 0;
        const rect = field.getBoundingClientRect();
        if (rect.top < top + HEADER_PX || rect.bottom > top + height() - 8) {
            field.scrollIntoView({ block: "center", behavior: "smooth" });
        }
    };

    const update = () => {
        const field = focusedField();
        if (!field) resting = height();
        const up = Boolean(field) && height() < resting - KEYBOARD_PX;
        document.documentElement.classList.toggle("keyboard-up", up);
        if (up) reveal();
    };

    (viewport || window).addEventListener("resize", update);
    document.addEventListener("focusin", () => {
        update();
        // After the keyboard has come up and the page has settled.
        setTimeout(() => {
            update();
            reveal();
        }, 300);
    });
    document.addEventListener("focusout", () => setTimeout(update, 100));
})();
