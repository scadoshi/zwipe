// A card's badges ease. In the stacks a badge that comes scales and fades in with a pop, one that
// goes scales and fades out where it was, the badges beside it slide over, and a count that
// changes pops. The details drawer's count pops as it changes and its Flag's icon as it turns on.
// Only a badge or its holder coming or going on its own moves, never one inside a whole card the
// page just added, so a deck loading or a regroup stays still. Transform and opacity only.
(() => {
    if (window.__zwipeBadgeEase) return;
    window.__zwipeBadgeEase = true;
    const HOLDER = '.card-stack-badges';
    const BADGE = '.card-stack-star, .card-stack-flag, .card-stack-qty';
    const COUNTS = '.card-stack-qty, .card-action-count';
    const FLAG = '[data-action="flag"]';
    const POP_MS = 200;
    const MOVE_MS = 250;
    const still = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
    const ease = () => getComputedStyle(document.documentElement).getPropertyValue('--ease-enter').trim() || 'ease-out';
    // A copy left behind to fade out is not a badge coming.
    const isBadge = (node) => node.nodeType === 1 && (node.matches(BADGE) || node.matches(HOLDER)) && !node.hasAttribute('data-badge-copy');
    // Each badge's box in its card as last laid out, for the slide from it and the copy left there.
    const spots = new WeakMap();
    const note = (holder) => {
        for (const badge of holder.children) {
            spots.set(badge, {
                x: holder.offsetLeft + badge.offsetLeft,
                y: holder.offsetTop + badge.offsetTop,
                w: badge.offsetWidth,
            });
        }
    };
    const pop = (el) => {
        el.getAnimations().forEach((a) => { if (a.id === 'badge-pop') a.cancel(); });
        const motion = el.animate(
            [{ transform: 'none' }, { transform: 'translateY(-2px) scale(1.25)', offset: 0.4 }, { transform: 'none' }],
            { duration: POP_MS, easing: ease(), composite: 'add' },
        );
        motion.id = 'badge-pop';
    };
    const enter = (badge) => {
        badge.animate(
            [
                { transform: 'scale(0.4)', opacity: 0 },
                { transform: 'scale(1.2)', opacity: 1, offset: 0.6 },
                { transform: 'scale(1)', opacity: 1 },
            ],
            { duration: MOVE_MS, easing: ease() },
        );
    };
    const leave = (badge, card) => {
        const spot = spots.get(badge);
        if (!spot || !card || !card.isConnected) return;
        const copy = badge.cloneNode(true);
        copy.setAttribute('aria-hidden', 'true');
        copy.setAttribute('data-badge-copy', '');
        copy.style.cssText = `position: absolute; left: ${spot.x}px; top: ${spot.y}px; width: ${spot.w}px; box-sizing: border-box; margin: 0; z-index: 1; pointer-events: none;`;
        card.appendChild(copy);
        const motion = copy.animate(
            [{ transform: 'scale(1)', opacity: 1 }, { transform: 'scale(0.4)', opacity: 0 }],
            { duration: MOVE_MS, easing: ease() },
        );
        motion.onfinish = () => copy.remove();
        motion.oncancel = () => copy.remove();
    };
    const slide = (holder, entered) => {
        for (const badge of holder.children) {
            const was = spots.get(badge);
            if (!was || entered.has(badge)) continue;
            const dx = was.x - (holder.offsetLeft + badge.offsetLeft);
            if (Math.abs(dx) < 0.5) continue;
            badge.animate(
                [{ transform: `translateX(${dx}px)` }, { transform: 'none' }],
                { duration: MOVE_MS, easing: ease() },
            );
        }
    };
    new MutationObserver((records) => {
        const holders = new Set();
        const entered = new Set();
        const popped = new Set();
        for (const record of records) {
            const at = record.target.nodeType === 1 ? record.target : record.target.parentElement;
            if (!at) continue;
            if (record.type === 'attributes') {
                if (at.matches(FLAG) && at.getAttribute('aria-pressed') === 'true') {
                    const icon = at.querySelector('.action-dock-icon');
                    if (icon) popped.add(icon);
                }
                continue;
            }
            const count = at.closest(COUNTS);
            if (count) popped.add(count);
            if (record.type !== 'childList') continue;
            const holder = at.closest(HOLDER);
            if (holder) holders.add(holder);
            record.addedNodes.forEach((node) => {
                if (isBadge(node)) {
                    (node.matches(HOLDER) ? [...node.children] : [node]).forEach((badge) => entered.add(badge));
                    holders.add(node.matches(HOLDER) ? node : node.parentElement);
                } else if (node.nodeType === 1) {
                    node.querySelectorAll(HOLDER).forEach(note);
                }
            });
            record.removedNodes.forEach((node) => {
                if (!isBadge(node) || still()) return;
                const card = at.closest('.card-stack-card');
                (node.matches(HOLDER) ? [...node.children] : [node]).forEach((badge) => leave(badge, card));
            });
        }
        if (!still()) {
            entered.forEach(enter);
            holders.forEach((holder) => { if (holder && holder.isConnected) slide(holder, entered); });
            popped.forEach((el) => { if (!entered.has(el)) pop(el); });
        }
        holders.forEach((holder) => { if (holder && holder.isConnected) note(holder); });
    }).observe(document.body, { childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ['aria-pressed'] });
    document.querySelectorAll(HOLDER).forEach(note);
})();
