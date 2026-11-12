//! Cards, rows and groups settling into place instead of snapping when a
//! control reorders, adds or removes them: a board or Group by chip, a fold,
//! a Show chip, a row's action, an undo, a sort or a filter.
//!
//! The page marks what moves and what moves it:
//! - a container marked `data-settle-scope` holds the things that settle,
//!   each named by `data-row`, `data-fold` or `data-settle-key`. A keyed thing
//!   inside another (a row in its group) settles relative to it;
//! - a click or typing inside an element marked [`TRIGGER`], a click on a
//!   button inside a scope or a toast, a click on a fold's header, or one of
//!   [`KEYS`] outside a text field starts a watch.
//!
//! For [`WATCH_MS`] after that, each change to the page, in the task that
//! made it and so before the browser paints it, and every frame compare each
//! thing's place with where it last was:
//! - one that jumped more than [`JUMP_PX`] in that frame slides from where it
//!   was to where it is;
//! - a new one fades in;
//! - one that left fades out where it was, as a copy laid over the page.
//!
//! Gradual movement, as a fold's body easing shut, is left alone, while a
//! group that jumps to the other column partway through is caught on the
//! frame it jumps. Every move runs on the kit's `--ease-enter` over
//! [`SETTLE_MS`]. Under `prefers-reduced-motion` nothing is watched.

use dioxus::prelude::*;

/// The attribute that marks a control row whose clicks settle the page.
pub const TRIGGER: &str = "data-settle";

/// The attribute that marks a container whose keyed contents settle.
pub const SCOPE: &str = "data-settle-scope";

/// How long one thing takes to settle, in milliseconds, in line with the folds.
pub const SETTLE_MS: u32 = 250;

/// How long a trigger keeps the page watched, in milliseconds: long enough to
/// cover a fold's own ease and the render after a server answer.
pub const WATCH_MS: u32 = 900;

/// The smallest move in one frame that counts as a jump, in pixels. A fold's
/// gradual slide stays under it; a re-render's jump or a column change does
/// not.
pub const JUMP_PX: u32 = 40;

/// The keys that start a watch outside a text field, as `KeyboardEvent.key`:
/// the deck page's undo, board moves, star and quantity steps.
pub const KEYS: [&str; 7] = ["u", "U", "m", "s", "S", "-", "+"];

/// The script, installed once per page by [`use_settle`].
const SETTLE_JS: &str = r"
(() => {
  if (window.__zwSettle) return;
  window.__zwSettle = true;
  const MS = __MS__, WATCH = __WATCH__, JUMP = __JUMP__, KEYS = __KEYS__;
  const reduce = () => window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const ease = () => getComputedStyle(document.documentElement).getPropertyValue('--ease-enter').trim() || 'ease-out';
  const KEYED = '[data-row], [data-fold], [data-settle-key]';
  const keyOf = (el) => el.hasAttribute('data-settle-key') ? 'k:' + el.getAttribute('data-settle-key')
    : el.hasAttribute('data-fold') ? 'f:' + el.getAttribute('data-fold') : 'r:' + el.getAttribute('data-row');
  const parentItem = (el, scope) => {
    const up = el.parentElement && el.parentElement.closest(KEYED);
    return up && scope.contains(up) ? up : null;
  };
  const own = (el) => {
    const t = getComputedStyle(el).translate;
    if (!t || t === 'none') return [0, 0];
    const [x, y] = t.split(' ').map(parseFloat);
    return [x || 0, y || 0];
  };
  const snapshot = () => {
    const found = new Map();
    document.querySelectorAll('[data-settle-scope]').forEach((scope, s) => {
      for (const el of scope.querySelectorAll(KEYED)) {
        if (el.closest('[data-settle-ghost]')) continue;
        const key = s + '|' + keyOf(el);
        if (found.has(key)) continue;
        const r = el.getBoundingClientRect();
        if (r.width === 0 && r.height === 0) continue;
        const up = parentItem(el, scope);
        const [tx, ty] = own(el);
        const ax = r.left - tx, ay = r.top - ty;
        let x = ax, y = ay;
        let parent = null;
        if (up) {
          // Against the group's box as it shows, its own moves included, so
          // a thing rides along with its group's move.
          const pr = up.getBoundingClientRect();
          x -= pr.left; y -= pr.top;
          parent = s + '|' + keyOf(up);
        }
        found.set(key, { el, x, y, ax, ay, tx, ty, rect: r, parent });
      }
    });
    return found;
  };
  let prev = null, until = 0, watching = false, observer = null;
  const ghost = (old) => {
    const copy = old.el.cloneNode(true);
    copy.setAttribute('data-settle-ghost', '');
    for (const keyed of [copy, ...copy.querySelectorAll(KEYED)]) {
      keyed.removeAttribute('data-row'); keyed.removeAttribute('data-fold'); keyed.removeAttribute('data-settle-key');
    }
    Object.assign(copy.style, {
      position: 'fixed', left: old.rect.left + 'px', top: old.rect.top + 'px',
      width: old.rect.width + 'px', height: old.rect.height + 'px', margin: '0',
      pointerEvents: 'none', zIndex: '5', boxSizing: 'border-box', translate: 'none',
    });
    document.body.appendChild(copy);
    copy.animate([{ opacity: 1 }, { opacity: 0 }], { duration: MS, easing: ease() })
      .finished.then(() => copy.remove(), () => copy.remove());
  };
  // Compares every keyed thing with where it last was and moves the ones that
  // jumped. Runs on each frame and, through the observer, in the task that
  // changed the page, before the browser paints the change.
  const check = () => {
    const now = snapshot();
    const timing = { duration: MS, easing: ease() };
    if (prev) {
      const fresh = new Set();
      for (const [key, cur] of now) {
        const old = prev.get(key);
        if (!old) {
          fresh.add(key);
          if (cur.parent && fresh.has(cur.parent)) continue;
          cur.el.animate([{ opacity: 0 }, { opacity: 1 }], timing);
          continue;
        }
        // Where the layout put it, against where it was: within its group
        // when it stays in the same one, on the page otherwise.
        const sameGroup = old.parent === cur.parent;
        const dx = sameGroup ? old.x - cur.x : old.ax - cur.ax;
        const dy = sameGroup ? old.y - cur.y : old.ay - cur.ay;
        if (Math.hypot(dx, dy) < JUMP) continue;
        // Each jump adds its own move on top of any still running, so a
        // thing that jumps again mid-move carries on from where it shows.
        const a = cur.el.animate([{ translate: dx + 'px ' + dy + 'px' }, { translate: '0px 0px' }], { ...timing, composite: 'add' });
        a.id = 'settle';
      }
      const gone = new Set();
      for (const [key, old] of prev) if (!now.has(key)) gone.add(key);
      for (const key of gone) {
        const old = prev.get(key);
        if (old.parent && gone.has(old.parent)) continue;
        ghost(old);
      }
    }
    prev = now;
  };
  const frame = () => {
    check();
    if (performance.now() < until) requestAnimationFrame(frame);
    else {
      watching = false;
      prev = null;
      if (observer) observer.disconnect();
      observer = null;
    }
  };
  const changed = (records) => records.some((r) => !(r.target instanceof Element && r.target.closest('[data-settle-ghost]'))
    && !(r.type === 'childList' && [...r.addedNodes, ...r.removedNodes].every((n) => n instanceof Element && n.hasAttribute('data-settle-ghost'))));
  const watch = () => {
    if (reduce()) return;
    until = performance.now() + WATCH;
    if (watching) return;
    watching = true;
    prev = snapshot();
    observer = new MutationObserver((records) => { if (prev && changed(records)) check(); });
    observer.observe(document.body, { childList: true, subtree: true, attributes: true, attributeFilter: ['class', 'hidden', 'open'] });
    requestAnimationFrame(frame);
  };
  const TRIGGERS = '[data-settle], [data-settle-scope] button, .toast-container button, .deck-fold-head';
  document.addEventListener('click', (e) => {
    if (e.target instanceof Element && e.target.closest(TRIGGERS)) watch();
  }, true);
  document.addEventListener('input', (e) => {
    if (e.target instanceof Element && e.target.closest('[data-settle]')) watch();
  }, true);
  document.addEventListener('keydown', (e) => {
    const t = e.target;
    if (t instanceof Element && t.closest('input, textarea, select, [contenteditable]')) return;
    if ((e.ctrlKey || e.metaKey) && (e.key === 'z' || e.key === 'Z')) { watch(); return; }
    if (!e.ctrlKey && !e.metaKey && !e.altKey && KEYS.includes(e.key)) watch();
  }, true);
})();
";

/// The script with its numbers filled in.
fn script() -> String {
    SETTLE_JS
        .replace("__MS__", &SETTLE_MS.to_string())
        .replace("__WATCH__", &WATCH_MS.to_string())
        .replace("__JUMP__", &JUMP_PX.to_string())
        .replace("__KEYS__", &format!("{KEYS:?}"))
}

/// Installs the settling on the page, once per document: every scope on the
/// page settles from then on.
pub fn use_settle() {
    use_hook(|| {
        document::eval(&script());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_script_carries_its_numbers_and_the_marks() {
        let script = script();
        for placeholder in ["__MS__", "__WATCH__", "__JUMP__", "__KEYS__"] {
            assert!(!script.contains(placeholder), "{placeholder} is filled");
        }
        assert!(script.contains("const MS = 250, WATCH = 900, JUMP = 40"));
        assert!(script.contains(r#"KEYS = ["u", "U", "m", "s", "S", "-", "+"]"#));
        assert!(script.contains(&format!("[{TRIGGER}]")));
        assert!(script.contains(&format!("[{SCOPE}]")));
    }
}
