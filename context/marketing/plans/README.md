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

| # | File | Theme | Lead features | Status / priority |
|---|------|-------|---------------|-------------------|
| 1 | (done, external) | Basic functionality | Add cards, Zwipe commander, import deck | **shipped — did well** (first stint; the general overview) |
| 2 | [video_02_gets_out_of_your_way.md](video_02_gets_out_of_your_way.md) | It gets out of your way | Swipe memory + per-deck stack resume | **posted — underperforming** (fine, not everything blows up) |
| 3 | [video_03_stops_when_youre_done.md](video_03_stops_when_youre_done.md) | Smart targeting | Land target, budget target, price filter | P2 |
| 4 | [video_04_draw_odds.md](video_04_draw_odds.md) | Consistency math | Turn-by-turn draw odds | **P1** |
| 5 | [video_05_synergy.md](video_05_synergy.md) | Synergy-aware swipes | Synergy ON/OFF toggle | P2 |
| 6 | [video_06_deck_profile.md](video_06_deck_profile.md) | Make it yours | Tags, power brackets, other-tags | P3 |
| 7 | [video_07_know_every_card.md](video_07_know_every_card.md) | Know every card | Name-while-swiping, rules dialog, DFC flip, printings | P3 |
| 8 | [video_08_share_your_deck.md](video_08_share_your_deck.md) | Share your deck as a link | Public `/deck/:token` page, web render, MVP headline | **P1 (new)** |
| 9 | [video_09_deck_mvps.md](video_09_deck_mvps.md) | Star your MVPs | 3 MVP slots, ★ in list, share-page headline | P2 (new) |
| 10 | [video_10_pick_your_commander.md](video_10_pick_your_commander.md) | Swipe to pick your commander | Popularity-ranked select + partner autofill | P2 (new) |

**Ship order:** #8 first — the share page is the growth surface (a link markets
itself, works with one user), and it's the freshest thing to show. Then #4 (draw
odds, still a strong P1). #9 (MVPs) is cheap to shoot right after #8 and pairs
with it.

**Combine candidates (per the snappy rule):**
- **#8 + #9 (share + MVPs)** group the tightest — build a deck, star its MVPs,
  share the link, MVPs headline the page. One 20s cut can carry both.
- **#10** already groups two beats (popularity-ranked select + partner autofill).

**Timing note:** #2–#7 need **1.3.0** (iOS build 57 / Android vc18, submitted
2026-07-02) or earlier and work on any current build. The **new videos #8–#10
are 1.4.0 features** — the in-app Share button, Deck MVPs, and the popularity
select + partner autofill all ride the next store build (client legs are on
main). Film them against a 1.4.0 or dev build. Exception: the **shared deck web
page is already live** at `zwipe.net/deck/:token`, so its half of #8/#9 can be
filmed now against a real shared link.
