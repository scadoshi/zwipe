//! The card details crossfading when they show another card or another face.
//!
//! A click on a stack card or on a Flip or Meld button, or a key, copies
//! what the card details show as they show it: an open card details
//! drawer's title bar and content, each open card row's art and detail, a
//! details pane's head and body beside the card, and each printing an open
//! Printings sheet shows. If one then shows another card (a drawer's title changes) or
//! another face (a face marker, `data-face`, changes: a flip, or a meld
//! half melding), the copies are laid over it in the same task, before the
//! browser paints the change, and fade out over `SWAP_MS` while the new
//! face fades in. Nothing runs under `prefers-reduced-motion`, and details
//! that kept their card and face drop the copies unseen. A held key's
//! repeats swap at once, dropping any fade still running. Stack card images
//! have no copies, so they swap at once.

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
  // The face shown: every face marker inside, in order.
  const faceOf = (el) => [...el.querySelectorAll('[data-face]')].map((m) => m.getAttribute('data-face')).join(' ');
  const DRAWER_PARTS = ':scope > .sheet-header, :scope > .sheet-content';
  const ROW_PARTS = ':scope > .card-row-compact > .card-row-art, :scope > .card-row-collapse > .card-row-collapse-inner';
  const openRows = () => [...document.querySelectorAll('.card-row.expanded')].filter((row) => row.getClientRects().length > 0);
  // Every printing an open Printings sheet shows, each turned in its cover flow, so its copy is
  // laid in the card's own layout box rather than its turned outline.
  const PRINTING_PARTS = ':scope > img';
  const openPrintings = () => [...document.querySelectorAll('.bottom-sheet.show .cover-flow-stage > .cover-flow-card')].filter((card) => card.getClientRects().length > 0);
  // A card's details as a pane of their own beside the card, as on a wide swipe screen; a
  // drawer's panel crossfades with its drawer instead.
  const PANE_PARTS = ':scope > .panel-head, :scope > .panel-body';
  const openPanes = () => [...document.querySelectorAll('.card-details-panel')].filter((panel) => !panel.closest('.bottom-sheet') && panel.getClientRects().length > 0);
  // A row's own color is often see-through, so its copies take the first solid one behind it.
  const solidBehind = (el) => {
    for (let at = el; at; at = at.parentElement) {
      const fill = getComputedStyle(at).backgroundColor;
      if (fill && fill !== 'transparent' && !/^rgba\(.*,\s*0\)$/.test(fill)) return fill;
    }
    return getComputedStyle(document.body).backgroundColor;
  };
  let pending = null, observer = null, expire = null;
  const shotsOf = (box, parts, local) => [...box.querySelectorAll(parts)].map((live) => ({
    live, copy: live.cloneNode(true), rect: live.getBoundingClientRect(), scroll: live.scrollTop,
    at: local ? { left: live.offsetLeft, top: live.offsetTop, width: live.offsetWidth, height: live.offsetHeight } : null,
  }));
  const play = () => {
    const was = pending;
    pending = null;
    if (!was) return;
    const timing = { duration: MS, easing: 'ease' };
    for (const surface of was.surfaces) {
      if (!surface.changed()) continue;
      const box = surface.box.getBoundingClientRect();
      const fill = surface.fill();
      // A row or a pane lays its copies over itself, so it holds them while they fade.
      const hold = getComputedStyle(surface.box).position === 'static';
      if (hold) surface.box.style.position = 'relative';
      let left = surface.shots.length;
      const done = () => { left -= 1; if (left === 0 && hold) surface.box.style.position = ''; };
      for (const shot of surface.shots) {
        const ghost = shot.copy;
        ghost.setAttribute('data-drawer-ghost', '');
        ghost.setAttribute('aria-hidden', 'true');
        const at = shot.at || { left: shot.rect.left - box.left, top: shot.rect.top - box.top, width: shot.rect.width, height: shot.rect.height };
        Object.assign(ghost.style, {
          position: 'absolute', left: at.left + 'px', top: at.top + 'px',
          width: at.width + 'px', height: at.height + 'px', margin: '0', boxSizing: 'border-box',
          overflow: 'hidden', pointerEvents: 'none', zIndex: '3', backgroundColor: fill,
        });
        surface.box.appendChild(ghost);
        ghost.scrollTop = shot.scroll;
        ghost.animate([{ opacity: 1 }, { opacity: 0 }], timing).finished.then(() => { ghost.remove(); done(); }, () => { ghost.remove(); done(); });
        shot.live.animate([{ opacity: 0 }, { opacity: 1 }], timing);
      }
    }
  };
  const capture = () => {
    if (reduce()) return;
    const surfaces = [];
    const sheet = openDrawer();
    if (sheet) {
      const surface = {
        box: sheet,
        title: titleOf(sheet),
        face: faceOf(sheet),
        shots: shotsOf(sheet, DRAWER_PARTS),
        fill: () => getComputedStyle(sheet).backgroundColor,
      };
      surface.changed = () => sheet.classList.contains('show') && (titleOf(surface.box) !== surface.title || faceOf(surface.box) !== surface.face);
      surfaces.push(surface);
    }
    for (const row of openRows()) {
      const surface = {
        box: row,
        face: faceOf(row),
        shots: shotsOf(row, ROW_PARTS).filter((shot) => shot.rect.width > 0 && shot.rect.height > 0),
        fill: () => solidBehind(row),
      };
      surface.changed = () => row.isConnected && row.classList.contains('expanded') && faceOf(row) !== surface.face;
      surfaces.push(surface);
    }
    for (const panel of openPanes()) {
      const surface = {
        box: panel,
        face: faceOf(panel),
        shots: shotsOf(panel, PANE_PARTS).filter((shot) => shot.rect.width > 0 && shot.rect.height > 0),
        fill: () => solidBehind(panel),
      };
      surface.changed = () => panel.isConnected && faceOf(panel) !== surface.face;
      surfaces.push(surface);
    }
    for (const card of openPrintings()) {
      const surface = {
        box: card,
        face: faceOf(card),
        shots: shotsOf(card, PRINTING_PARTS, true).filter((shot) => shot.at.width > 0 && shot.at.height > 0),
        fill: () => 'transparent',
      };
      surface.changed = () => card.isConnected && faceOf(card) !== surface.face;
      surfaces.push(surface);
    }
    if (!surfaces.length) return;
    pending = { surfaces };
    if (observer) observer.disconnect();
    clearTimeout(expire);
    // The callback runs in the task that changed the details, before it paints.
    observer = new MutationObserver(() => {
      observer.disconnect();
      observer = null;
      clearTimeout(expire);
      play();
    });
    for (const surface of surfaces) observer.observe(surface.box, { childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ['data-face', 'src'] });
    expire = setTimeout(() => { if (observer) observer.disconnect(); observer = null; pending = null; }, 600);
  };
  document.addEventListener('click', (e) => {
    if (e.target instanceof Element && e.target.closest('.card-stack-card, [data-action=flip], [data-action=meld]')) capture();
  }, true);
  // A held key swaps the card at once: its repeats capture nothing and drop what still fades.
  const drop = () => {
    if (observer) observer.disconnect();
    observer = null;
    pending = null;
    clearTimeout(expire);
    for (const ghost of document.querySelectorAll('[data-drawer-ghost]')) ghost.remove();
    const sheet = openDrawer();
    if (sheet) for (const part of sheet.querySelectorAll(DRAWER_PARTS)) for (const run of part.getAnimations()) run.cancel();
    for (const row of openRows()) {
      row.style.position = '';
      for (const part of row.querySelectorAll(ROW_PARTS)) for (const run of part.getAnimations()) run.cancel();
    }
    for (const panel of openPanes()) {
      panel.style.position = '';
      for (const part of panel.querySelectorAll(PANE_PARTS)) for (const run of part.getAnimations()) run.cancel();
    }
    for (const card of openPrintings()) for (const part of card.querySelectorAll(PRINTING_PARTS)) for (const run of part.getAnimations()) run.cancel();
  };
  document.addEventListener('keydown', (e) => {
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    if (e.repeat) { drop(); return; }
    capture();
  }, true);
})();
";

/// Installs the crossfade on the page, once.
pub(crate) fn use_drawer_swap() {
    use_hook(install);
}

/// Installs the crossfade on the page; a page that has it already keeps it.
pub(crate) fn install() {
    document::eval(&SWAP_JS.replace("__MS__", &SWAP_MS.to_string()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_held_key_swaps_at_once_and_clears_the_ghosts() {
        assert!(
            SWAP_JS.contains("if (e.repeat) { drop(); return; }"),
            "a key repeat captures nothing and drops what still fades"
        );
        assert!(
            SWAP_JS.contains("const drop = () => {"),
            "the drop clears the pending capture and every ghost"
        );
        assert!(
            SWAP_JS.contains("for (const ghost of document.querySelectorAll('[data-drawer-ghost]')) ghost.remove();"),
            "{SWAP_JS}"
        );
    }

    #[test]
    fn a_face_change_plays_as_a_card_swap_does() {
        assert!(
            SWAP_JS.contains("const faceOf = (el) =>"),
            "a surface's face is read from its face markers: {SWAP_JS}"
        );
        assert!(
            SWAP_JS.contains(
                "titleOf(surface.box) !== surface.title || faceOf(surface.box) !== surface.face"
            ),
            "the drawer plays on a new card or a new face: {SWAP_JS}"
        );
    }

    #[test]
    fn an_open_card_row_crossfades_its_art_and_detail() {
        assert!(
            SWAP_JS.contains("document.querySelectorAll('.card-row.expanded')"),
            "{SWAP_JS}"
        );
        assert!(
            SWAP_JS.contains(":scope > .card-row-compact > .card-row-art, :scope > .card-row-collapse > .card-row-collapse-inner"),
            "{SWAP_JS}"
        );
    }

    #[test]
    fn a_flip_or_meld_tap_captures_as_a_key_does() {
        for action in crate::data_action::FLIP_OR_MELD {
            assert!(
                SWAP_JS.contains(&format!("[data-action={action}]")),
                "{action}: {SWAP_JS}"
            );
        }
    }

    #[test]
    fn every_visible_printing_crossfades_together() {
        assert!(
            SWAP_JS.contains("const openPrintings = () =>"),
            "the open Printings sheet's printings are surfaces: {SWAP_JS}"
        );
        assert!(
            SWAP_JS.contains(".cover-flow-stage > .cover-flow-card"),
            "each printing in the cover flow: {SWAP_JS}"
        );
        assert!(
            SWAP_JS.contains("offsetLeft"),
            "a turned card's copy is laid in its own layout box, not its turned outline: {SWAP_JS}"
        );
    }

    #[test]
    fn a_details_pane_beside_the_card_crossfades_its_head_and_body() {
        assert!(
            SWAP_JS.contains("const openPanes = () =>"),
            "a details panel outside any drawer is a surface: {SWAP_JS}"
        );
        assert!(
            SWAP_JS.contains("!panel.closest('.bottom-sheet')"),
            "a drawer's own panel crossfades with its drawer, once: {SWAP_JS}"
        );
        assert!(
            SWAP_JS.contains(":scope > .panel-head, :scope > .panel-body"),
            "{SWAP_JS}"
        );
    }

    /// The z-index the script gives a ghost copy.
    fn ghost_layer() -> u32 {
        let (_, rest) = SWAP_JS.split_once("zIndex: '").unwrap();
        let (layer, _) = rest.split_once('\'').unwrap();
        layer.parse().unwrap()
    }

    #[test]
    fn a_row_s_keyboard_ring_stays_over_its_ghost_copies() {
        let (_, rule) = crate::MAGIC_CSS
            .split_once(".card-row-compact:focus-visible::after {")
            .unwrap_or_else(|| panic!("the row's ring is a layer of its own"));
        let (rule, _) = rule.split_once('}').unwrap();
        let (_, layer) = rule
            .split_once("z-index:")
            .unwrap_or_else(|| panic!("{rule}"));
        let (layer, _) = layer.split_once(';').unwrap();
        let layer: u32 = layer.trim().parse().unwrap();
        assert!(
            layer > ghost_layer(),
            "the ring ({layer}) over the ghosts ({}): {rule}",
            ghost_layer()
        );
        assert!(rule.contains("pointer-events: none"), "{rule}");
    }

    #[test]
    fn the_script_carries_its_duration() {
        let script = SWAP_JS.replace("__MS__", &SWAP_MS.to_string());
        assert!(script.contains("const MS = 200;"));
        assert!(!script.contains("__MS__"));
    }
}
