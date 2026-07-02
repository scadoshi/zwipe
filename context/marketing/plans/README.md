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

## Slate

| # | File | Theme | Lead features | Priority |
|---|------|-------|---------------|----------|
| 1 | (done, external) | Basic functionality | Add cards, Zwipe commander, import deck | shipped |
| 2 | [video_02_gets_out_of_your_way.md](video_02_gets_out_of_your_way.md) | It gets out of your way | Swipe memory + land auto-stop | **P1** |
| 3 | [video_03_stops_when_youre_done.md](video_03_stops_when_youre_done.md) | Smart targeting | Land target, budget target, price filter | P2 |
| 4 | [video_04_draw_odds.md](video_04_draw_odds.md) | Consistency math | Turn-by-turn draw odds | **P1** |
| 5 | [video_05_synergy.md](video_05_synergy.md) | Synergy-aware swipes | Synergy ON/OFF toggle | P2 |
| 6 | [video_06_deck_profile.md](video_06_deck_profile.md) | Make it yours | Tags, power brackets, other-tags | P3 |
| 7 | [video_07_know_every_card.md](video_07_know_every_card.md) | Know every card | Name-while-swiping, rules dialog, DFC flip, printings | P3 |

**Ship order:** #2 and #4 first (freshest differentiators, thumb-stopping hooks).

**Timing note:** video #2's swipe-memory *client UI* (durable skips, "Clear
skips" button) ships with the release after 1.2.3. Film it against that build,
not the current store version.
