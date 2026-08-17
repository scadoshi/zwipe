# Moat — the non-EDH cross-format dataset

## The format unlock

Serving today is effectively Commander-only because it pivots on the commander (color
identity + the synergy worker's per-commander payload). With no commander, we have
nothing to pivot from. otags + color identity generalize that:

- A deck with a **color identity** + a set of **selected otags** behaves "commander-like"
  for serving — we have a pivot even without a commander.
- Then collect swipe correlation keyed by **(format, color identity, otag set)**: every
  left/right swipe in that context builds correlation data *for that format*.
- Over time this serves synergistic cards for **Standard, Modern, Pioneer, etc.** — a
  capability we do not have today.

## Why it is a moat, not just a feature

Be honest about the timeline: day-1 serving quality for non-EDH will be weak, because the
`(format, CI, otag)` signal starts empty. That is fine. **The moat is the dataset, not the
feature.**

- Every other deck builder does essentially **nothing** for non-EDH format synergy.
- We would be the **only** builder quietly accruing a cross-format functional-synergy
  dataset — from the very first swipe, at effectively zero marginal cost (the collection
  path is the same metrics write we already run).
- Once we have volume it is **uncopyable** without the same user base and the same time.

So it is cheap to start, it compounds, and nobody can shortcut it. That is a better
framing than "marketable feature": it is a background accrual that becomes a differentiator.

## The honest caveat

It compounds **only** with non-EDH swipe volume, which depends on our existing (mostly
Commander) users branching into other formats, or on non-EDH acquisition. So:

- **Do not headline it at launch.** Land otags on Commander first, where the data and the
  usage already are.
- **Do start collecting immediately.** Turn on `(format, CI, otag)` signal keying as soon
  as deck otag selection ships, even before non-EDH serving is good, so the flywheel has
  been spinning for months by the time we surface it.

Priority note carried from the original doc: **Commander dominates usage by a wide
margin.** Non-EDH serving is a long-game accrual layered on top, never a reason to delay
the Commander payoff.
