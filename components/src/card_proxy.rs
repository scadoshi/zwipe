//! The card a missing or failed image leaves: the card's own text laid out
//! like its expanded row, in the exact box the image would fill.
//!
//! [`CardProxy`] is always the card's 63:88 shape, so nothing around it moves
//! when an image is missing. It follows an expanded card row from the row's
//! own pieces: the name and mana cost, a rule, the kind chips, the rules
//! text, the Keywords and Card roles rows, then the facts chips. The text is
//! sized against the box, and [`use_card_proxy_fit`] shrinks a card whose text
//! would still overflow until it fits, dropping rows when the type can shrink
//! no further. At a thumbnail's size only the name shows.

use dioxus::prelude::*;
use zwipe_core::domain::card::card_profile::CardProfile;
use zwipe_core::domain::card::card_role;
use zwipe_core::domain::card::scryfall_data::ScryfallData;

use crate::card_details::{FactsChips, KindChips};
use crate::oracle_text::OracleText;

/// What a [`CardProxy`] shows for one face of a card.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProxyFace {
    /// The face's name.
    pub name: String,
    /// The face's mana cost in Scryfall's `{R}{1}` form, when it has one.
    pub mana_cost: Option<String>,
    /// The face's type line.
    pub type_line: Option<String>,
    /// The card's rarity, as its row names it.
    pub rarity: Option<String>,
    /// The card's set name.
    pub set_name: Option<String>,
    /// The face's rules text.
    pub oracle_text: Option<String>,
    /// Power and toughness as `2/2`, or loyalty, when the face has either.
    pub stats: Option<String>,
}

impl ProxyFace {
    /// Face `face` of `sd` (the front when it has no such face): the face's own
    /// name, cost, type, text and stats where it carries them, else the
    /// card's. Rarity and set are the card's.
    #[must_use]
    pub fn of(sd: &ScryfallData, face: usize) -> Self {
        let side = sd.card_faces.as_ref().and_then(|faces| faces.get(face));
        let pick = |own: Option<&String>, card: Option<&String>| own.or(card).cloned();
        let power = pick(side.and_then(|f| f.power.as_ref()), sd.power.as_ref());
        let toughness = pick(
            side.and_then(|f| f.toughness.as_ref()),
            sd.toughness.as_ref(),
        );
        let loyalty = pick(side.and_then(|f| f.loyalty.as_ref()), sd.loyalty.as_ref());
        let stats = match (power, toughness) {
            (Some(p), Some(t)) => Some(format!("{p}/{t}")),
            _ => loyalty,
        };
        Self {
            name: side.map_or_else(|| sd.name.clone(), |f| f.name.clone()),
            mana_cost: pick(side.map(|f| &f.mana_cost), sd.mana_cost.as_ref())
                .filter(|cost| !cost.is_empty()),
            type_line: pick(
                side.and_then(|f| f.type_line.as_ref()),
                sd.type_line.as_ref(),
            ),
            rarity: Some(sd.rarity.to_long_name()),
            set_name: Some(sd.set_name.clone()),
            oracle_text: pick(
                side.and_then(|f| f.oracle_text.as_ref()),
                sd.oracle_text.as_ref(),
            ),
            stats,
        }
    }
}

/// The no-image card for face `face` of `sd`, laid out as an expanded card
/// row lays the card out, from the row's own pieces: the name with the mana
/// cost, a rule, the kind chips (type line, rarity), the rules text, the
/// Keywords row, the Card roles row when `profile` gives roles, then the facts
/// chips (set, prices, stats). The rows' chips are plain here, since the
/// proxy sits inside cards a click already acts on. `trailing` sits at the
/// head's right end, before the cost, for a host's control such as Flip.
#[component]
pub fn CardProxy(
    sd: ScryfallData,
    #[props(default)] face: usize,
    #[props(default)] profile: Option<CardProfile>,
    #[props(default)] trailing: Option<Element>,
    #[props(default = String::new())] class: String,
) -> Element {
    let ProxyFace {
        name,
        mana_cost,
        type_line,
        rarity,
        set_name,
        oracle_text,
        stats,
    } = ProxyFace::of(&sd, face);
    let keywords = sd.keywords.clone().unwrap_or_default();
    let prices = zwipe_ui::card::info::prices(&sd);
    let roles: Vec<(String, bool)> = profile
        .as_ref()
        .map(|p| {
            p.card_roles
                .iter()
                .map(|slug| {
                    let tagged = p
                        .oracle_tags_by_role
                        .get(slug)
                        .is_some_and(|t| !t.is_empty());
                    (card_role::role_label(slug), tagged)
                })
                .chain((!p.other_oracle_tags.is_empty()).then(|| ("Other tags".to_owned(), true)))
                .collect()
        })
        .unwrap_or_default();
    rsx! {
        div { class: "card-proxy {class}", role: "img", "aria-label": "{name}",
            div { class: "card-proxy-fit",
                div { class: "card-proxy-head",
                    span { class: "card-proxy-name", "{name}" }
                    span { class: "card-proxy-head-end",
                        {trailing}
                        if let Some(cost) = mana_cost {
                            OracleText { text: cost, class: "card-proxy-cost".to_string() }
                        }
                    }
                }
                hr { class: "card-proxy-rule" }
                KindChips { type_line: type_line.unwrap_or_default(), rarity: rarity.unwrap_or_default() }
                if let Some(text) = oracle_text {
                    OracleText { text, class: "card-proxy-text".to_string() }
                }
                if !keywords.is_empty() {
                    div { class: "keyword-chips card-proxy-row",
                        span { class: "stat-chip stat-chip-p1 chip-row-label", "Keywords" }
                        for keyword in keywords {
                            span { key: "{keyword}", class: "chip", "{keyword}" }
                        }
                    }
                }
                if !roles.is_empty() {
                    div { class: "keyword-chips card-proxy-row",
                        span { class: "stat-chip stat-chip-p1 chip-row-label", "Card roles" }
                        for (label, tagged) in roles {
                            span { key: "{label}", class: if tagged { "chip" } else { "stat-chip" }, "{label}" }
                        }
                    }
                }
                FactsChips { set_name: set_name.unwrap_or_default(), prices, stats }
            }
        }
    }
}

/// Fits every [`CardProxy`] on the page into its box, and again whenever one
/// is added or resized. Its text scale steps down from 1, to no less than
/// 0.45 of the box-sized type. A browser with a minimum font size can stop
/// the type shrinking, so a card that still overflows drops its rows one at a
/// time, rules text first, then the Keywords and Card roles rows, then the
/// kind chips, and fits again: the name and the facts row always show. A
/// thumbnail shows only its name, which wraps between words and shrinks until
/// every word and line fits, clipping to an ellipsis only at the smallest
/// size.
pub const FIT_JS: &str = r"
(() => {
  if (window.__zwProxyFit) return;
  window.__zwProxyFit = true;
  const DROPS = 3;
  const full = new WeakMap();
  const nameText = (name) => (name && name.firstChild && name.firstChild.nodeType === 3 ? name.firstChild : null);
  // The name with its widest words cut to an ellipsis until each fits its line.
  const cut = (name, text, whole) => {
    const words = whole.split(' ');
    while (name.scrollWidth > name.clientWidth + 1) {
      let at = 0;
      words.forEach((w, i) => { if (w.length > words[at].length) at = i; });
      const word = words[at].replace(/\u2026$/, '');
      if (word.length <= 1) return;
      words[at] = word.slice(0, -1) + '\u2026';
      text.data = words.join(' ');
    }
  };
  const fit = (card) => {
    const inner = card.querySelector('.card-proxy-fit');
    if (!inner || card.clientWidth === 0) return;
    const thumb = card.clientWidth <= 110;
    const name = card.querySelector('.card-proxy-name');
    const text = nameText(name);
    const was = text && full.get(text);
    if (was && text.data === was.shown) text.data = was.whole;
    const over = () =>
      inner.scrollHeight > card.clientHeight + 1 ||
      (thumb && !!name && (name.scrollHeight > name.clientHeight + 1 || name.scrollWidth > name.clientWidth + 1));
    for (let drop = 0; drop <= (thumb ? 0 : DROPS); drop++) {
      if (drop) card.setAttribute('data-proxy-drop', String(drop));
      else card.removeAttribute('data-proxy-drop');
      let scale = 1;
      card.style.setProperty('--proxy-scale', '1');
      while (over() && scale > 0.45) {
        scale = Math.round((scale - 0.05) * 100) / 100;
        card.style.setProperty('--proxy-scale', String(scale));
      }
      if (!over()) return;
    }
    if (thumb && text) {
      const whole = text.data;
      cut(name, text, whole);
      full.set(text, { whole, shown: text.data });
    }
  };
  const sizes = new ResizeObserver((entries) => { for (const e of entries) fit(e.target); });
  const watch = (root) => {
    const cards = root.matches && root.matches('.card-proxy') ? [root] : [];
    if (root.querySelectorAll) cards.push(...root.querySelectorAll('.card-proxy'));
    for (const card of cards) { sizes.observe(card); fit(card); }
  };
  watch(document.body);
  new MutationObserver((records) => {
    for (const r of records) for (const n of r.addedNodes) if (n instanceof Element) watch(n);
  }).observe(document.body, { childList: true, subtree: true });
})();
";

/// Installs the fitting for every [`CardProxy`] on the page, once per document.
pub fn use_card_proxy_fit() {
    use_hook(|| {
        document::eval(FIT_JS);
    });
}
