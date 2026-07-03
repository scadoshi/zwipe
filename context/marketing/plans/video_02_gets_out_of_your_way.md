# Video 2 — It gets out of your way

**Theme:** Zwipe respects your choices. Two "genius"-tier wins under one promise:
swipe memory (durable skips) + per-deck stack memory. Land auto-stop moved to
video #3, which already owns the targeting story — deck memory is the stronger
pairing here.
**Priority:** P1. Freshest differentiator, answers the loudest launch complaint
("closing the app resets the skip pile").
**Hook line:** "Swipe left once. It never shows you that card again."

## Beats (~20s)

| Time | Beat | On-screen caption |
|------|------|-------------------|
| 0:00–0:02 | Fast left-swipe on a card, it flies off | Swipe left once. |
| 0:02–0:05 | Close the app (swipe to app switcher, kill it), reopen to the same deck | It remembers. |
| 0:05–0:08 | Back on the add screen: the skipped card never reappears while swiping | That card is gone for good. |
| 0:08–0:11 | Back out, open a second deck, swipe a couple of cards there | Switch decks anytime. |
| 0:11–0:14 | Return to the first deck's add screen: same card waiting on top | Every deck holds your place. |
| 0:14–0:16 | More sheet → "Clear skips" → confirm dialog → "Skips cleared" toast | Changed your mind? Clear skips. |
| 0:16–0:20 | End card | (video_end_card.png) |

## Shots to capture

1. **Left-swipe** on the add-cards screen (Search source), card animates off.
   Skips post to the server per swipe in 1.3.0, so the kill in shot 2 can be
   immediate — no need to linger before killing.
2. **App kill + relaunch** to the same deck (screen-record the full app-switcher
   kill so the "it survived a restart" beat is unambiguous).
3. **Continued swiping** proving the skipped card does not return.
4. **Deck switch**: back out to the deck list, open a second deck's add screen,
   swipe 2–3 cards. Pick a second deck with a visually distinct commander so
   the cut reads instantly.
5. **Return to deck one**: its add screen resumes on the exact card it was
   left on. Film shots 4–5 in one take WITHOUT killing the app between them —
   per-deck resume is in-session memory; a restart starts a fresh (correctly
   skip-filtered) stack instead of resuming mid-stack.
6. **More sheet → Clear skips → confirmation dialog → confirm** + the
   "Skips cleared" toast. Three taps in ~2s — rehearse it, or trim the dialog
   in the edit and cut straight to the toast.

## Notes
- Film against a 1.3.0 build (iOS build 57 / Android versionCode 18, or the dev
  build) — per-swipe skips and per-deck resume are both 1.3.0 features.
- Keep beats 1–3 tight; "kill and reopen, still gone" is the money shot, and
  beat 5's instant resume is the close-second thumb-stopper.
- Dialog copy for reference: title "Clear skips", body "Cards you've skipped or
  removed will start showing up again when adding cards. This can't be undone."
