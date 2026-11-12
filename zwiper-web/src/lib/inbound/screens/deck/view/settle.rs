//! The deck page's cards settling into place when a Boards chip or a Group by
//! chip changes what the bands show, in the List and the Stacks alike.
//!
//! A click on an element marked [`TRIGGER`] notes where every card (a list
//! row or a stack card, by its `data-row`) and every group (by its
//! `data-fold`) sits in the bands. As soon as the bands have re-rendered, in
//! the same task and before the browser paints, so the first painted frame
//! already shows the old layout:
//! - a card or group still shown slides from where it was to where it is;
//! - a new one fades in;
//! - one that left fades out where it was, as a copy laid over the page, and
//!   the cards after it close up as they slide.
//!
//! Each move runs on the kit's `--ease-enter` over [`SETTLE_MS`]. Under
//! `prefers-reduced-motion` nothing moves. Filtering, searching and folding
//! are not triggers, so they change the bands at once, as before.

use dioxus::prelude::*;

/// The attribute that marks a chip row whose clicks settle the bands.
pub(crate) const TRIGGER: &str = "data-settle";

/// How long a card takes to settle, in milliseconds, in line with the folds.
pub(crate) const SETTLE_MS: u32 = 250;

/// The settling, installed once per page: a capture-phase click listener that
/// notes the layout before the page's own handler changes it, and an observer
/// that plays the moves after the next render of `.deck-bands`.
const SETTLE_JS: &str = r"
(() => {
  if (window.__zwSettle) return;
  window.__zwSettle = true;
  const MS = __MS__;
  const reduce = () => window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const ease = () => getComputedStyle(document.documentElement).getPropertyValue('--ease-enter').trim() || 'ease-out';
  const bands = () => document.querySelector('.deck-bands');
  const items = (root) => {
    const found = new Map();
    for (const el of root.querySelectorAll('.deck-fold[data-fold], .deck-stack[data-fold]')) {
      const key = 'f:' + el.getAttribute('data-fold');
      if (!found.has(key)) found.set(key, el);
    }
    for (const el of root.querySelectorAll('.web-deck-row[data-row], .card-stack-slot[data-row]')) {
      const key = 'r:' + el.getAttribute('data-row');
      if (!found.has(key)) found.set(key, el);
    }
    return found;
  };
  const groupOf = (el) => el.parentElement && el.parentElement.closest('.deck-fold[data-fold], .deck-stack[data-fold]');
  let before = null;
  let observer = null;
  let expire = null;
  const capture = () => {
    const root = bands();
    if (!root || reduce()) return;
    before = new Map();
    for (const [key, el] of items(root)) {
      before.set(key, { rect: el.getBoundingClientRect(), el, group: groupOf(el) });
    }
    if (observer) observer.disconnect();
    clearTimeout(expire);
    // The observer's callback runs in the task that changed the bands, before
    // the browser paints them, so the moves start on the first painted frame.
    observer = new MutationObserver(() => {
      observer.disconnect();
      observer = null;
      clearTimeout(expire);
      play();
    });
    observer.observe(root, { childList: true, subtree: true, attributes: true, attributeFilter: ['class'] });
    expire = setTimeout(() => { if (observer) observer.disconnect(); observer = null; before = null; }, 600);
  };
  const play = () => {
    const root = bands();
    const was = before;
    before = null;
    if (!root || !was) return;
    const timing = { duration: MS, easing: ease() };
    const now = items(root);
    const moved = new Map();
    for (const [key, el] of now) {
      const old = was.get(key);
      if (!old) continue;
      const rect = el.getBoundingClientRect();
      moved.set(key, { dx: old.rect.left - rect.left, dy: old.rect.top - rect.top });
    }
    for (const [key, el] of now) {
      const old = was.get(key);
      if (!old) {
        if (!(el.parentElement && el.parentElement.closest('[data-settle-new]'))) {
          el.setAttribute('data-settle-new', '');
          el.animate([{ opacity: 0 }, { opacity: 1 }], timing).finished.then(() => el.removeAttribute('data-settle-new'), () => {});
        }
        continue;
      }
      const delta = moved.get(key);
      const group = groupOf(el);
      const parent = group && moved.get('f:' + group.getAttribute('data-fold'));
      const dx = delta.dx - (parent ? parent.dx : 0);
      const dy = delta.dy - (parent ? parent.dy : 0);
      if (Math.abs(dx) < 0.5 && Math.abs(dy) < 0.5) continue;
      el.animate([{ translate: dx + 'px ' + dy + 'px' }, { translate: '0px 0px' }], timing);
    }
    const leftGroups = new Set();
    for (const [key, old] of was) {
      if (now.has(key) || !key.startsWith('f:')) continue;
      leftGroups.add(old.el);
    }
    for (const [key, old] of was) {
      if (now.has(key)) continue;
      if (key.startsWith('r:') && old.group && leftGroups.has(old.group)) continue;
      const ghost = old.el.cloneNode(true);
      Object.assign(ghost.style, {
        position: 'fixed', left: old.rect.left + 'px', top: old.rect.top + 'px',
        width: old.rect.width + 'px', height: old.rect.height + 'px',
        margin: '0', pointerEvents: 'none', zIndex: '5', boxSizing: 'border-box',
      });
      ghost.removeAttribute('data-row');
      ghost.removeAttribute('data-fold');
      document.body.appendChild(ghost);
      ghost.animate([{ opacity: 1 }, { opacity: 0 }], timing).finished.then(() => ghost.remove(), () => ghost.remove());
    }
  };
  document.addEventListener('click', (e) => {
    if (e.target instanceof Element && e.target.closest('[__TRIGGER__] button')) capture();
  }, true);
})();
";

/// Installs the settling on the deck page, once.
pub(crate) fn use_settle() {
    use_hook(|| {
        document::eval(
            &SETTLE_JS
                .replace("__MS__", &SETTLE_MS.to_string())
                .replace("__TRIGGER__", TRIGGER),
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_script_listens_for_the_trigger_and_times_the_moves() {
        let script = SETTLE_JS
            .replace("__MS__", &SETTLE_MS.to_string())
            .replace("__TRIGGER__", TRIGGER);
        assert!(script.contains("closest('[data-settle] button')"));
        assert!(script.contains("const MS = 250;"));
        assert!(
            !script.contains("__MS__") && !script.contains("__TRIGGER__"),
            "every placeholder is filled"
        );
    }
}
