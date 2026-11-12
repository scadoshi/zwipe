//! Cards, rows and groups settling into place instead of snapping when a
//! control reorders, adds or removes them: a board, Group by, Show, Sort or
//! filter chip, a row's action, an undo.
//!
//! The page marks what moves and what moves it:
//! - a container marked [`SCOPE`] holds the things that settle, each named by
//!   `data-row`, `data-fold` (a group) or `data-settle-key`. A keyed thing
//!   inside a group settles relative to it, so it rides along with its
//!   group's move;
//! - a click or typing inside an element marked [`TRIGGER`], a click on a
//!   row's action button inside a scope or on a toast's button, or one of
//!   [`KEYS`] outside a text field starts a watch. Opening a row, folding a
//!   group or folding them all ([`NOT_TRIGGERS`]) never does.
//!
//! A trigger notes where every keyed thing sits, once, before the page's own
//! handler changes anything. For [`WATCH_MS`] after it, each change to a
//! scope is caught by a mutation observer, in the task that made it and so
//! before the browser paints it, and measured at most once a frame. Only
//! what can have moved is measured again: the groups, the things outside
//! any group, and the things inside the groups the change touched. Then:
//! - one that moved more than [`JUMP_PX`] slides from where it showed to
//!   where it is, each move added on top of any still running;
//! - a new one fades in;
//! - one that left fades out where it was, as a copy laid over the page.
//!
//! Every move runs on the kit's `--ease-enter` over [`SETTLE_MS`]. Under
//! `prefers-reduced-motion` nothing is watched.

use dioxus::prelude::*;

/// The attribute that marks a control row whose clicks settle the page.
pub const TRIGGER: &str = "data-settle";

/// The attribute that marks a container whose keyed contents settle.
pub const SCOPE: &str = "data-settle-scope";

/// How long one thing takes to settle, in milliseconds, in line with the folds.
pub const SETTLE_MS: u32 = 250;

/// How long a trigger keeps the page watched, in milliseconds: long enough to
/// cover the render after a server answer.
pub const WATCH_MS: u32 = 900;

/// The smallest move that counts as a jump, in pixels.
pub const JUMP_PX: u32 = 40;

/// Controls inside a [`TRIGGER`] row that never start a watch, since they
/// fold groups rather than reorder them: the deck page's fold-all button.
pub const NOT_TRIGGERS: &str = ".deck-fold-all";

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
  const scopes = () => [...document.querySelectorAll('[data-settle-scope]')];
  // The group a thing sits in, or null outside any group.
  const groupOf = (el, scope) => {
    const up = el.parentElement && el.parentElement.closest('[data-fold]');
    return up && scope.contains(up) ? up : null;
  };
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
  // A keyed wrapper drawn as `display: contents` has no box of its own, so
  // its first child is measured and moved in its place.
  const boxOf = (el) => (getComputedStyle(el).display === 'contents' && el.firstElementChild) || el;
  // Where each keyed thing sits. `touched` names the groups whose insides to
  // measure (scope index and group key); null measures everything.
  const snapshot = (touched) => {
    const found = new Map();
    scopes().forEach((scope, s) => {
      for (const el of scope.querySelectorAll(KEYED)) {
        if (el.closest('[data-settle-ghost]')) continue;
        const group = groupOf(el, scope);
        const groupKey = group ? s + '|' + keyOf(group) : null;
        if (touched && groupKey && !el.hasAttribute('data-fold') && !touched.has(groupKey)) continue;
        const key = s + '|' + keyOf(el);
        if (found.has(key)) continue;
        const box = boxOf(el);
        const r = box.getBoundingClientRect();
        if (r.width === 0 && r.height === 0) continue;
        const up = parentItem(el, scope);
        const [tx, ty] = own(box);
        const ax = r.left - tx, ay = r.top - ty;
        let x = ax, y = ay;
        let parent = null;
        if (up) {
          // Against the group's box as it shows, its own moves included, so
          // a thing rides along with its group's move.
          const pr = boxOf(up).getBoundingClientRect();
          x -= pr.left; y -= pr.top;
          parent = s + '|' + keyOf(up);
        }
        found.set(key, { el: box, x, y, ax, ay, rect: r, parent, group: groupKey });
      }
    });
    return found;
  };
  // The groups a batch of changes touched; undefined when none reached a
  // scope. Groups and things outside any group are measured either way.
  const touchedBy = (records) => {
    const all = scopes();
    const touched = new Set();
    let inScope = false;
    for (const r of records) {
      const target = r.target instanceof Element ? r.target : r.target.parentElement;
      if (!target || target.closest('[data-settle-ghost]')) continue;
      const s = all.findIndex((scope) => scope.contains(target));
      if (s < 0) continue;
      inScope = true;
      const group = target.closest('[data-fold]');
      if (group && all[s].contains(group)) touched.add(s + '|' + keyOf(group));
      for (const n of [...r.addedNodes, ...r.removedNodes]) {
        if (n instanceof Element && n.hasAttribute('data-fold')) touched.add(s + '|' + keyOf(n));
      }
    }
    return inScope ? touched : undefined;
  };
  let prev = null, until = 0, observer = null, measured = false, queued = null;
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
  // Measures what the change can have moved and moves what jumped.
  const check = (touched) => {
    if (!prev) return;
    const now = snapshot(touched);
    const timing = { duration: MS, easing: ease() };
    const fresh = new Set();
    for (const [key, cur] of now) {
      const old = prev.get(key);
      if (!old) {
        fresh.add(key);
        if (cur.parent && fresh.has(cur.parent)) continue;
        cur.el.animate([{ opacity: 0 }, { opacity: 1 }], timing);
        continue;
      }
      const sameGroup = old.parent === cur.parent;
      const dx = sameGroup ? old.x - cur.x : old.ax - cur.ax;
      const dy = sameGroup ? old.y - cur.y : old.ay - cur.ay;
      if (Math.hypot(dx, dy) < JUMP) continue;
      cur.el.animate([{ translate: dx + 'px ' + dy + 'px' }, { translate: '0px 0px' }], { ...timing, composite: 'add' });
    }
    // Only what was measured can have left: a thing in an untouched group
    // is still where it was.
    const gone = [];
    for (const [key, old] of prev) {
      if (now.has(key)) continue;
      if (touched && old.group && !touched.has(old.group) && !key.includes('|f:')) continue;
      gone.push(key);
    }
    const goneSet = new Set(gone);
    for (const key of gone) {
      const old = prev.get(key);
      if (!(old.parent && goneSet.has(old.parent))) ghost(old);
      prev.delete(key);
    }
    for (const [key, cur] of now) prev.set(key, cur);
  };
  const stop = () => {
    if (observer) observer.disconnect();
    observer = null;
    prev = null;
    queued = null;
  };
  const changed = (records) => {
    if (performance.now() > until) { stop(); return; }
    const touched = touchedBy(records);
    if (touched === undefined) return;
    if (!measured) {
      measured = true;
      requestAnimationFrame(() => {
        measured = false;
        if (queued) { const t = queued; queued = null; check(t); }
      });
      check(touched);
      return;
    }
    // A second batch in the same frame waits for the next one.
    if (!queued) queued = new Set();
    for (const g of touched) queued.add(g);
  };
  const watch = () => {
    if (reduce()) return;
    until = performance.now() + WATCH;
    if (observer) return;
    prev = snapshot(null);
    observer = new MutationObserver(changed);
    observer.observe(document.body, { childList: true, subtree: true, attributes: true, attributeFilter: ['class', 'hidden', 'open'] });
    setTimeout(function end() { if (performance.now() < until) setTimeout(end, until - performance.now()); else stop(); }, WATCH);
  };
  const TRIGGERS = '[data-settle], [data-settle-scope] .card-action-btn, .toast-container button';
  // Controls in a trigger row that fold or show rather than reorder.
  const NOT_TRIGGERS = '__SKIP__';
  document.addEventListener('click', (e) => {
    const t = e.target;
    if (t instanceof Element && t.closest(TRIGGERS) && !t.closest(NOT_TRIGGERS)) watch();
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
        .replace("__SKIP__", NOT_TRIGGERS)
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
        for placeholder in ["__MS__", "__WATCH__", "__JUMP__", "__KEYS__", "__SKIP__"] {
            assert!(!script.contains(placeholder), "{placeholder} is filled");
        }
        assert!(script.contains("const MS = 250, WATCH = 900, JUMP = 40"));
        assert!(script.contains(r#"KEYS = ["u", "U", "m", "s", "S", "-", "+"]"#));
        assert!(script.contains(&format!("[{TRIGGER}]")));
        assert!(script.contains(&format!("[{SCOPE}]")));
    }
}
