# Card oracle-text / stats fallback for unreadable printings

**Source:** User feedback, 2026-06-30.

> "Is there a way to change the printing of a card that pops up? Sometimes I get
> a card that doesn't have any of the card text and I'd like to be able to change
> that instead of going and looking the card up to read what it does. For example
> the SLD [Secret Lair] of Rankle, Master of Pranks." Follow-up: also surface
> other values (power/toughness, mana cost, type line, other stats).

## The problem

Some printings render little or no rules text on the card image — Secret Lair
art cards, full-art/textless printings, borderless, foreign-language. When one of
these lands in the swipe stack (add / remove / Zwipe-select), the user can't tell
what the card does without leaving the app to look it up.

The user first framed it as "let me change the printing," but the simpler solve
they landed on is: **show the oracle text (and key stats) in-app** as a fallback,
regardless of printing.

## Assessment

**Small.** All the data is already on the client — no server or data work.
`card.scryfall_data` carries `oracle_text`, `power`, `toughness`, `loyalty`,
`mana_cost`, `type_line` (see `zwipe-core/.../scryfall_data/mod.rs`). The existing
`CardInfoDisplay` (`zwiper/.../deck/card/components/card_info.rs`) currently shows
only name + set, so this is a display addition to that one component, applied
across the add / remove / Zwipe-select flows that already use it.

## Design options (undecided)

- **Always-on compact block** under the image: type line, mana cost, P/T or
  loyalty, oracle text. Simplest; adds vertical content to every card.
- **Tap / toggle to reveal** an oracle detail panel — keeps the swipe view clean,
  costs one interaction.
- **Auto-reveal only for text-light printings** — Scryfall exposes hints
  (`full_art`, textless/art-series printings); show the fallback only when the
  image likely lacks rules text. Nicest UX, slightly more logic.

Lean: tap-to-reveal, or auto-reveal for detected text-light printings, to avoid
cluttering the core swipe loop.

## Related

- `feature_requests.md` **#8** ("Always show card name + a detail view, esp.
  foreign/alt-art printings", P1) — this is the oracle/stats half of that item.
- **#9 / #18** (printing-selection preferences) — the fuller "change the printing"
  solve the user originally asked for; larger. This fallback is the cheap win that
  removes the pain without the printing-selection subsystem.

## Verdict

**Recommend build — small.** High value (kills the "unidentifiable card" pain),
data already present, one component. Fold into / promote alongside #8. Only open
question is the reveal UX above.
