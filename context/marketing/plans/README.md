# Marketing video plans

Short-form vertical videos (TikTok / Reels / Shorts). One file per video, each a
20-ish second, fast-paced beat sheet matching video #1 (basic functionality:
add cards, Zwipe commander, import deck). Structure every video the same way:

- **9:16, 1080×1920.**
- **~20s**, hard cuts on the beat, one feature per beat (~2–3s each).
- **0:00–0:02 hook**, **0:02–~0:16 feature beats**, **~0:16–0:20 end card**.
- **End card**: reuse `../video_end_card.png` (Zwipe wordmark, "Free on iOS &
  Android · No ads", zwipe.net).
- **Caption rules** (on-screen text is user-facing copy): no em dashes, sentence
  case, "Zwipe" capitalized.
- **Screen-record source**: the real app (`dx serve --platform ios` or a device).
- **Snappy first.** Fast cuts, no dead air — the whole job is grab-and-hold.
  Because it's this fast, features that group naturally can share one video (a
  beat each) instead of getting their own — combine freely as long as the cut
  stays quick and each beat still reads. The per-video files are the raw
  material; a shoot can merge them.

## Slate

(Realigned 2026-07-27 against 1.7.3 — the slate predated oracle tags; #11 is the
otag flagship, #5/#6/#7 were updated to fold in the tag-era surface.)

| # | File | Theme | Lead features | Status / priority |
|---|------|-------|---------------|-------------------|
| 1 | (done, external) | Basic functionality | Add cards, Zwipe commander, import deck | **shipped — did well** (first stint; the general overview) |
| 2 | [video_02_gets_out_of_your_way.md](video_02_gets_out_of_your_way.md) | It gets out of your way | Swipe memory + per-deck stack resume | **posted — underperforming** (fine, not everything blows up) |
| 3 | [video_03_stops_when_youre_done.md](video_03_stops_when_youre_done.md) | Smart targeting | Land target, budget target, price filter | P2 |
| 4 | [video_04_draw_odds.md](video_04_draw_odds.md) | Consistency math | Turn-by-turn draw odds | **P1** |
| 5 | [video_05_synergy.md](video_05_synergy.md) | Synergy-aware swipes | Synergy ON/OFF toggle (now community-signal blended) | P2 |
| 6 | [video_06_deck_profile.md](video_06_deck_profile.md) | Make it yours | Tags, brackets, archetype→otag seed, 31 themes | P3 |
| 7 | [video_07_know_every_card.md](video_07_know_every_card.md) | Know every card | Rules dialog, DFC flip, printings, tap-a-tag definitions | P3 |
| 8 | [video_08_share_your_deck.md](video_08_share_your_deck.md) | Share your deck as a link | Public `/deck/:token` page, web render, MVP headline | **P1** |
| 9 | [video_09_deck_mvps.md](video_09_deck_mvps.md) | Star your MVPs | 3 MVP slots, ★ in list, share-page headline | P2 |
| 10 | [video_10_pick_your_commander.md](video_10_pick_your_commander.md) | Swipe to pick your commander | Popularity-ranked select + partner autofill | P2 |
| 11 | [video_11_oracle_tags.md](video_11_oracle_tags.md) | 4,500 tags know what every card does | Role→tag drill-down, definitions, Examples, otag filter, dictionary | **P1 (new — the otag flagship)** |

**Ship order:** #8 first — the share page is the growth surface (a link markets
itself, works with one user). Then **#11 (oracle tags)** — the biggest
differentiator since launch and the freshest thing to show. Then #4 (draw odds,
still a strong P1). #9 (MVPs) is cheap to shoot right after #8 and pairs with it.

**Combine candidates (per the snappy rule):**
- **#8 + #9 (share + MVPs)** group the tightest — build a deck, star its MVPs,
  share the link, MVPs headline the page. One 20s cut can carry both.
- **#10** already groups two beats (popularity-ranked select + partner autofill).
- **#11's dictionary beat** can carry its own future video if the flagship runs
  hot; its archetype-seed beat is the same take as #6 beat 5 — film once.

**Timing note:** every feature on this slate is **live in the 1.7.x store
builds** — film everything against the current store build (or a dev build for
clean data). The old 1.3.0/1.4.0 gating is obsolete. One nuance: #11's
in-dialog tag definitions during swiping are **1.7.3** (in review as of
2026-07-27) — if filming before it clears, capture that beat on a dev build or
use the deck-row definition reveal (1.7.1) instead.
