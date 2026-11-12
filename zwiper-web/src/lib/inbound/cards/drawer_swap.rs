//! The card details drawer crossfading when it swaps to another card.
//!
//! A click on a stack card, or a key, while a card details drawer is open
//! copies the drawer's title bar and content as they show. If the drawer
//! then changes to another card, the copies are laid over the drawer in the
//! same task, before the browser paints the change, and fade out over
//! `SWAP_MS` while the new card's title bar and content fade in. Nothing
//! runs under `prefers-reduced-motion`, and a drawer that kept its card
//! drops the copies unseen.

use dioxus::prelude::*;

/// How long the old card takes to fade out and the new one in, in
/// milliseconds.
pub(crate) const SWAP_MS: u32 = 200;

/// The crossfade, installed once per page.
const SWAP_JS: &str = r"
(() => {
  if (window.__zwDrawerSwap) return;
  window.__zwDrawerSwap = true;
  const MS = __MS__;
  const reduce = () => window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const openDrawer = () => document.querySelector('.bottom-sheet.show.card-details-drawer');
  const titleOf = (sheet) => (sheet.querySelector('.sheet-title') || { textContent: '' }).textContent;
  let pending = null, observer = null, expire = null;
  const play = () => {
    const was = pending;
    pending = null;
    if (!was || !was.sheet.classList.contains('show') || titleOf(was.sheet) === was.title) return;
    const timing = { duration: MS, easing: 'ease' };
    const box = was.sheet.getBoundingClientRect();
    const fill = getComputedStyle(was.sheet).backgroundColor;
    for (const shot of was.shots) {
      const ghost = shot.copy;
      ghost.setAttribute('data-drawer-ghost', '');
      ghost.setAttribute('aria-hidden', 'true');
      Object.assign(ghost.style, {
        position: 'absolute', left: (shot.rect.left - box.left) + 'px', top: (shot.rect.top - box.top) + 'px',
        width: shot.rect.width + 'px', height: shot.rect.height + 'px', margin: '0', boxSizing: 'border-box',
        overflow: 'hidden', pointerEvents: 'none', zIndex: '3', backgroundColor: fill,
      });
      was.sheet.appendChild(ghost);
      ghost.scrollTop = shot.scroll;
      ghost.animate([{ opacity: 1 }, { opacity: 0 }], timing).finished.then(() => ghost.remove(), () => ghost.remove());
      shot.live.animate([{ opacity: 0 }, { opacity: 1 }], timing);
    }
  };
  const capture = () => {
    const sheet = openDrawer();
    if (!sheet || reduce()) return;
    const parts = sheet.querySelectorAll(':scope > .sheet-header, :scope > .sheet-content');
    pending = {
      sheet,
      title: titleOf(sheet),
      shots: [...parts].map((live) => ({ live, copy: live.cloneNode(true), rect: live.getBoundingClientRect(), scroll: live.scrollTop })),
    };
    if (observer) observer.disconnect();
    clearTimeout(expire);
    // The callback runs in the task that changed the drawer, before it paints.
    observer = new MutationObserver(() => {
      observer.disconnect();
      observer = null;
      clearTimeout(expire);
      play();
    });
    observer.observe(sheet, { childList: true, subtree: true, characterData: true });
    expire = setTimeout(() => { if (observer) observer.disconnect(); observer = null; pending = null; }, 600);
  };
  document.addEventListener('click', (e) => {
    if (e.target instanceof Element && e.target.closest('.card-stack-card')) capture();
  }, true);
  document.addEventListener('keydown', (e) => {
    if (!e.ctrlKey && !e.metaKey && !e.altKey) capture();
  }, true);
})();
";

/// Installs the crossfade on the page, once.
pub(crate) fn use_drawer_swap() {
    use_hook(|| {
        document::eval(&SWAP_JS.replace("__MS__", &SWAP_MS.to_string()));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_script_carries_its_duration() {
        let script = SWAP_JS.replace("__MS__", &SWAP_MS.to_string());
        assert!(script.contains("const MS = 200;"));
        assert!(!script.contains("__MS__"));
    }
}
