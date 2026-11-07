# Droid pass 1: zwiper on Android

Build: origin/release at cef3be34f (v1.18.0), debug APK from ~/Developer/droid/zwipe, Pixel_9a emulator (Android 16, 411x845 CSS px, dpr 2.625), production API, account `test`. Measurements are CSS px from CDP unless marked device px. Screenshots are in ~/Developer/droid/shots, and the `.json` next to a screenshot is its CDP audit.

Severity counts: 1 broken, 16 ugly, 21 nit.

## Issues

1. **Login, keyboard covers the password field** (broken). I tapped Username, then pressed Tab to move to Password. With the keyboard up the viewport is 499 px tall and the focused password input sits at y 486..533, so only 13 px of it shows and the Log in button is fully hidden. The page doesn't scroll the focused field into view. Expected: the focused input scrolls above the keyboard. Shot: pass1-91.

2. **Toasts cover the Back button** (ugly). After every swipe or action ("Added to deck", "Skipped", "Commander selected", "Undid maybeboard", "Dark mode off") a toast sits at 16,8 182x61, on top of Back (12,8 34x27) and part of the title for about 3.5 s. `elementFromPoint` at Back's centre returns the toast, so a tap meant for Back lands on the toast. Swiping steadily keeps Back covered the whole time. Expected: toasts clear the nav bar, or don't take taps there. Shots: pass1-20, pass1-25, pass1-30, pass1-80.

3. **Remove, empty stack is a dead end** (ugly). I kept Lightning-Rig Crew (left) and removed Vampire of the Dire Moon (right), which emptied the stack. The screen goes blank: no empty-state chip, and swipe-down undo does nothing because there's no card to swipe. The removal could only be undone from the Cards screen's Undo button ("Re-added Vampire of the Dire Moon"). The 1.18.1 changelog says "Empty lists and empty Activity panels say so in a centered gray chip". Expected: an empty-state chip, plus an Undo the user can reach on this screen. Shots: pass1-57, pass1-58.

4. **Load errors have no retry** (ugly). Throttled to GSM (`adb emu network speed gsm`, delay gprs), Home showed "Could not load your decks" and "Could not load your playstyle" after about 60 s, Decks showed "Could not load decks" after about 14 s, and Featured flavor stayed on its skeleton for more than 60 s with no error. Logcat says `load_decks failed: network error: error decoding response body` (a slow body read, under the 30 s client timeout). When the panels turned into short error rows, Home re-centered its content (`.home-hero` margin-top 23.7 px) and the hero jumped. Expected: a Retry on each error, and an error state for Featured flavor. Shots: pass1-6, pass1-67.

5. **Home, Decks skeleton is the wrong shape** (ugly). While loading, each deck tile's art skeleton is 111x10, a thin bar, where the loaded art is a 111x111 square. The skeleton panel is 113 px tall and the loaded one about 238 px, so everything below jumps. Expected: square art skeletons. Shot: pass1-3 (skeleton) vs pass1-1.

6. **Playstyle, skeleton doesn't match the loaded layout** (ugly). The skeleton has one row of 6 filter chips, the loaded page has 6 rows (Show, colors, 12 guild chips, Art). The Colors panel is 213 px as a skeleton and 255 px loaded. Together that moves Suggested commanders from y≈520 to y≈816, a 296 px jump. Expected: a skeleton with the loaded page's rows. Shots: pass1-69 vs pass1-70.

7. **Color bar skeleton rows are too short** (nit). On Home and Playstyle the skeleton color rows are 24 px apart and the loaded rows 27.4 px, so six rows come to 144 px against 165. Shots: pass1-3, pass1-69.

8. **Decks, Show-row chip skeletons are short** (nit). The five color-chip skeletons are 42x27, the loaded chips 43.6x34, so the row grows 7 px on load. The skeleton chips are correctly empty outlines. Shot: pass1-64.

9. **Colors panels have extra bottom space** (nit). On Home the Colors panel ends 34 px below its last bar, while the Decks and Featured flavor panels end 17 px below theirs. `.color-bars` has 14.4 px bottom padding on top of the panel's 16 px. Playstyle's Colors panel shows the same thing (33 px below, 25 px above). Shots: pass1-1, pass1-2, pass1-70.

10. **Format picker, tapped chip jumps away** (ugly). I tapped Commander. The hint card grows from 89 px (the "Tap a format" hint) to about 236 px of format details and pushes the whole chip grid down, so the chip under my finger moved 148 px. Expected: the details don't shift the chips, for example by sitting below them or at a fixed height. Shots: pass1-15, pass1-16.

11. **Format picker, uneven gaps** (nit). Search to hint card is 24 px, hint card to the first chip row 8 px. Shot: pass1-15.

12. **Side gutters differ between screens** (ugly). The content's left edge is at 16 px on Home, Decks, Edit/Create deck, Account, Settings and Playstyle, at 32 px on the deck page and About, 30 px on the Oracle tags Dictionary and 46 px on the Format picker. Shots: pass1-7, pass1-10, pass1-15, pass1-48, pass1-84.

13. **Deck page, currency chips sit off the right column** (nit). TIX ends at x 366.7, while the Profile icon buttons and every row value end at 362.7. Shot: pass1-10.

14. **Deck page, Buy row heights differ** (nit). The "Buy" label chip is 40x27 and the TCGplayer and Card Kingdom buttons beside it are 32 px tall. Shot: pass1-10.

15. **Deck page, collapsible sections start low and space rows wider** (ugly). Mana and Budget open with an `<hr>` (margin 8 px above, 9.6 px below) under the header's own rule, so the first row starts about 18 px below the header, where Profile and Data have none. Mana's rows also have a 12 px gap, giving a 70 px row pitch against Profile's 58 px. The text looks like it sits low in its row. Shot: pass1-13.

16. **Draw odds, minus not disabled at turn 0** (nit). "−" stays enabled at "by turn 0". Tapping it does nothing, so the floor isn't shown. Shot: pass1-12.

17. **Distributions, "CREAT" when there's room** (nit). The Types chart's single bar spans 322 px but is still labeled with the short form (`deck_metrics.rs:91`). Could the full name show when it fits? Shot: pass1-11.

18. **Edit/Create deck, uneven field spacing** (ugly). Deck name's input to the Format label is 16 px. After the Commander field it's 32 px (input margin-bottom 16 px inside the collapsible plus Power level's margin-top 16 px), and the Tags section's fields are also 32 px apart. Label to input is 11 px for Deck name and 6 px for Commander. Shots: pass1-17, pass1-88.

19. **Cards, Commander group header out of line** (nit). The "Commander" group title starts at x 33 with no chevron and can't collapse. "Creatures (1)" has ▸ at 33 and its title at about 59. Shot: pass1-23.

20. **Cards, keyboard pushes the dock over content** (ugly). Typing in Quick add, the + − filter undo dock floats up above the keyboard and covers the Group by and Boards rows. The "Counterspell" result pill also floats over the commander's "Edgar Markov" and "Commander" chips. Shot: pass1-59.

21. **Filter sheet, tiny clear button inside the section toggle** (ugly). Mana's and Format's "×" clear is 24x21.5 and sits inside a 380 px-wide header that toggles the section, so a near miss clears the filter or opens the section by accident. Shot: pass1-31.

22. **Filter sheet, first label touches the open header** (nit). In every open section the first label starts 0 px below the header (Config: header bottom 402, "Is playable" top 402). Below a section there are about 16 to 20 px before the next header. Shots: pass1-32 to pass1-45.

23. **Filter sheet, Mana value placeholder clipped** (ugly). The input is 80 px wide with 62 px for text, and "Mana value" needs 84 px, so it reads "Mana val". Shots: pass1-35, pass1-46.

24. **Examples, card sits lower than on Add** (nit). Examples' card top is at 213 and Add's at 173, the same component 40 px apart. Shot: pass1-49 vs pass1-24.

25. **Remove help, copy points the wrong way** (nit). It says "The board chips up top pick which board you are trimming", but the Main, Maybe, Side and All chips are at the bottom left. Shot: pass1-54.

26. **Warnings, first row squeezed** (ugly). With two fix buttons the text gets 164 px and wraps to 4 lines, and "Add cards" wraps inside its button (63x46, white-space normal). The other warnings get 236 to 314 px. Expected: the buttons stack under the text, or the label stays on one line. Shot: pass1-62.

27. **Commander maybeboard, empty chip touches the panel** (nit). "No commanders" ends at y 219 and the Danger zone panel starts at 219, a 0 px gap. Clear maybeboard is also enabled with nothing to clear. Shot: pass1-77.

28. **Two panel-title styles on one screen** (nit). On Playstyle, "Suggested commanders" and "Staples" are 13.6 px pink with normal spacing at x 33, while "Colors", "Summary", "Card roles" and "Oracle tags" are 16 px teal with 1.6 px letter spacing at x 56. Activity's "Most skipped" uses the pink style too. Shots: pass1-71, pass1-72, pass1-76.

29. **Playstyle, Art toggle sits among the guild chips** (nit). "Art" is a show-art toggle but sits at the end of the last guild-chip row, styled like the filters. Shot: pass1-70.

30. **Account and Settings, rows wrap their buttons** (nit). The Email row puts Change under the Verified chip, and Settings' Theme row puts Change under Dark. Those rows are taller than the rest and their buttons fall out of the button column. In that same pair, Dark is 33.9 px tall and Change 31.1. Shots: pass1-78, pass1-79.

31. **Contrast below 4.5:1** (ugly). Settings and deck-page row labels (`.settings-row-label`) are 3.83:1 in Gruvbox dark and 2.80:1 in Gruvbox light. In light, the hero stats are 3.48:1 (20 px yellow) and the selected "USD" chip is 3.33:1 at 12 px. Measured with ~/Developer/droid/contrast.js. Shots: pass1-80, pass1-81, pass1-82.

32. **Status and navigation bars ignore the theme** (ugly). The Android status bar is the same grey band, and the gesture area the same black band, in both dark and light themes, so light mode has a grey bar across the top. Expected: system bar colors that follow the theme's background. Shots: pass1-1, pass1-80.

33. **Touch targets under 44 px** (ugly). Nav Back and menu are 33.6x27.2, every "?" is 24x24, chips are 26.7 px tall (Group by, Boards, Show, filters, Power level), dock buttons 38.4x38.4, steppers 28.9x29, the menu's icon row 35.2x27.2 and Log out 77.8x27.2. About's inline Scryfall link is 65x18. Each `.json` file lists the targets on its screen. Shots: pass1-1, pass1-7, pass1-68, pass1-84.

34. **Dock insets differ** (nit). The Decks + button is 9 px from the right and 9 px from the bottom. The Add dock is 16 px and 6 px, so its centre sits 3 px lower. Shots: pass1-7, pass1-24.

35. **Hint dialogs, narrow text column** (nit). The Colors hint's text is 235 px wide (57% of the screen), about 19 monospace characters a line, so three short sentences take nine lines. Shot: pass1-87.

36. **Light theme, white bar blends into its track** (nit). In Gruvbox light the white color bar's cream fill is almost the same as the track, and only its outline shows. Shot: pass1-81.

37. **Hero chips change order when logged out** (nit). Logged out it reads "Everybody · Live", logged in "Live · Everybody · Activity". Shots: pass1-91 vs pass1-1.

38. **List rows wrap their chips unevenly** (nit). On Decks, Suggested commanders and Staples, the pip, count, price and P/T chips sit on the name line or wrap to a second line depending on the name's length, so rows of the same kind have different shapes. Shots: pass1-7, pass1-71, pass1-72.

## Looked fine

- Logging in and out, creating a deck, and Edit deck saving format Commander and commander Edgar Markov.
- Add swipes: right adds, left skips, up sends to the maybeboard and down undoes, each with the right toast. Short drags of 61 and 38 px snap back, and diagonals go by the stronger axis.
- Remove: left keeps and right removes. The Cards screen's Undo restores a removed card.
- Commander Swipe select: skip, select, and the selection fills the field.
- The deck page accordion keeps one section open, and the color-identity warning's Remove button fixes it (4 cards and 3 warnings became 3 and 2).
- Filter state survives a trip to the Dictionary and Examples and back. Every section opens, and content clears the dock at the end (80 px bottom padding).
- Card details' Other tags expands, and Printings opens with 8 printings and their prices.
- The header hides on scroll down and comes back on a small scroll up.
- Help "See guide ↗" opens a Custom Tab. On this fresh emulator that shows Chrome's first-run screen, which is not an app issue.
- Changelog: Latest, 2026 and Show more (5 to 10 releases).
- Themes: switching Gruvbox to light and back, and the theme sheet's groups and selection. The account is back on gruvbox-dark.
- Activity's heatmap scrolls sideways. The figures that roll before landing are by design (`home_hero.rs:255`).
- No horizontal page overflow and no clipped text on any screen, apart from intended ellipses and sideways scrollers.
- The home logo matches the web and marketing art. My spike note calling it garbled was wrong.

## Test data and cleanup

The "Droid test" deck now has format Commander, commander Edgar Markov, and Lightning-Rig Crew and Vampire of the Dire Moon in the main board. Counterspell went in to trigger a warning and was removed. Maybeboard swipes were undone. It also has a few week-long skips from Add.

One write outside the deck: in commander Swipe select I skipped Fire Lord Azula, and that skip is per account. Undoing it means Playstyle's "Clear skips", which clears every commander skip on the account, so I left it for you to decide.

The theme is back on Gruvbox dark, no account settings were changed, the network throttle is off, and the clone's target is 7.4 GB.

## Pass 2

Build: origin/main at 60d705459, debug APK from ~/Developer/droid/zwipe, the same emulator and account. I recorded the status of all 38 items, ran the new phone pieces, and noted new issues. Shots are in ~/Developer/droid/shots/pass2-<n>.png.

One housekeeping note: a scripting slip overwrote pass1-2 to pass1-5 with pass-2 frames, which I moved to pass2-2 to pass2-5. Items 5 and 9 above cite pass1-3 and pass1-2, and pass2-33 and pass2-1 show the same things now.

Install note: `adb install -r` failed with INSUFFICIENT_STORAGE because /data was 73% full. Uninstalling first fixed it (log in again afterwards). I've added that to the runbook.

### Scheduled for this build

| # | Status | What I saw | Shot |
|---|---|---|---|
| 2 | fixed | Toasts sit bottom left at 16,713 (122x61), above From/Search and the dock. A tap on Back reaches Back. | pass2-14 |
| 10 | changed | The chips hold still now. But the details card is a fixed 176 px, and Commander's details are 236 px, so 62 px ("OK)" and "Commander damage: 21") sit behind an inner scroll. The text is cut mid-line at the card's bottom edge with no fade, so nothing shows there's more. Brawl overflows by 15 px. | pass2-8, pass2-9, pass2-10 |
| 16 | fixed | "−" is disabled at turn 0 (opacity 0.5). | pass2-4 |
| 25 | fixed | The help reads "The board chips at the bottom left pick which board you are trimming". | pass2-22 |
| 26 | fixed | Warning text gets the full 314 px, with "Add cards" (92x30, one line) and "Import" below it. | pass2-5 |
| 28 | fixed | "Suggested commanders", "Staples" and Activity's "Most skipped" are 16 px with 1.6 px spacing, still pink. | pass2-25, pass2-26 |
| 30 | fixed | Email and Theme put the label on its own line, then the value and buttons on one row. Every button ends 32 px from the right, and Dark and Change are both 34 px tall. | pass2-27, pass2-28 |
| 31 | partly | Row labels pass in both themes (no hits on Settings or the deck page). Still under 4.5:1 in Gruvbox light: the hero stats at 3.48 (20 px, weight under 700) and the selected "USD" chip at 3.33 (12 px). | pass2-30, pass2-31 |
| 33 | fixed | The `::after` hit box is live (pointer: coarse). On Home, Decks, the deck page and Cards, every small control takes a tap 21 px out from its centre in all four directions. The only exceptions are where a closer neighbor's box wins (chip rows 12 px apart), a FAB sits on top, or the control is inside a collapsed section. | pass2-1, pass2-2, pass2-3, pass2-12 |
| 35 | fixed | The hint text column is 295 px (was 235). | pass2-32 |
| 37 | fixed | Logged out, the hero reads "Live · Everybody". | pass2-37 |

### Not yet built (all still there)

| # | Rechecked | Shot |
|---|---|---|
| 1 | The focused password field sits at 486 to 533 in a 499 px viewport. | pass2-38 |
| 3 | Partly changed. The empty stack now shows a ghost card and a "No more cards" toast, where it was blank. Swipe-down undo still does nothing, and the Cards Undo restores both cards. Also new: kept cards cycle back into the stack, so keeping alone never empties it. Is that intended? | pass2-23 |
| 4 | Errors arrive after about 10 s on GSM, still with no retry. | pass2-34 |
| 5 | Deck tile art skeleton is 111x10. | pass2-33 |
| 6 | Playstyle's skeleton has one chip row. | pass2-36 |
| 7 | Skeleton color rows are 24 px apart. | pass2-33 |
| 8 | Decks Show-row skeleton chips are 42x27. | pass2-35 |
| 9 | Colors ends 34 px below its bars, the other panels 17. | pass2-1 |
| 11 | Hint card to chips is still tight. | pass2-8 |
| 12 | Gutters: deck page 32, Dictionary 30. | pass2-3, pass2-20 |
| 13 | TIX ends at 366.7 against the 362.7 column. | pass2-3 |
| 14 | The Buy label is 27 px next to 32 px links. | pass2-3 |
| 15 | Mana's rows are still spaced as before. | pass2-6 |
| 17 | The single bar is still labeled "CREAT". | pass2-7 |
| 18 | The Commander field is still 32 px from Power level. | pass2-11 |
| 19 | The Commander group title is at x 33 with no chevron. It now uses the panel title type. | pass2-12 |
| 20 | The dock rides the keyboard over Group by and Boards, and the result pill covers the commander chips. | pass2-13 |
| 21 | Changed. The "×" still draws at 24x22, but its 44 px hit box now covers 44 of the header's 47 px height, so a tap meant for the header near the × now clears the filter across a wider area. | pass2-15 |
| 22 | The first label still touches the open header. | pass2-15 |
| 23 | Still "Mana val" (80 px input, needs 84). | pass2-15 |
| 24 | The Examples card top is at 213, Add's at 173. | pass2-21 |
| 27 | "No commanders" to Danger zone is 0 px, and Clear is enabled. | pass2-24 |
| 34 | Decks FAB is 9/9 from the edges, the Add dock 16/6. | pass2-2, pass2-14 |
| 38 | Row chips still wrap by name length. | pass2-25 |

Left as is by Scotty: 29, 32, 36.

### New phone pieces

- **Card details:** this works. It floats in from the right, inset 38 px left, 9 px right, 10 px under the header (top 54) and 69 px above the bottom, clear of the docks. It has 16 px corners and a shadow, and no card inside a card (the only bordered children are two panel rules). Nit: the × sits alone in a 50 px header band, and the Printings pill is pinned to the bottom under a tall empty gap. Shot: pass2-16.
- **Printings pill:** this works. It's a 94x22 text pill whose hit test passes 4/4. Shot: pass2-16.
- **Cover flow:** this works. Neighbors show at the sides, the arrows step, a swipe steps, and Next is disabled at the last printing (opacity 0.5). Nit: the arrows sit flush on the screen edges (left 0 and right 0) and overlap the neighbor cards. A printing with no prices drops its price row, and the sheet's height shifts. Shots: pass2-17, pass2-18, pass2-19.
- **Deck page gap under the header:** fixed. The Profile panel starts 16 px under the header, like Account and Decks. Shot: pass2-3.
- **Theme toast:** gone. No toast on light or dark, from Settings or the menu.

### New issues

1. **Stat chips sit just under 4.5:1 in dark** (nit). On Home, "Live", the deck count and the deck tile names are 4.49, and the muted "Commander" chip is 4.36. On Decks, the "Group by" and "Show" labels are 4.49. Settings' Exceptions chips are 4.36. Shot: pass2-1.
2. **Format details are clipped with no cue** (ugly). This is the leftover from item 10 above: the fixed 176 px card cuts Commander's details mid-line. A fade, or a card that fits its tallest format, would fix it. Shot: pass2-10.
3. **Cover flow arrows touch the screen edges** (nit). The 36x36 arrows sit at left 0 and right 0 over the neighbor cards. Shot: pass2-17.
4. **Theme lost once, not reproduced** (watch). I set Gruvbox light from Settings, then dark from the menu toggle, then within about a minute throttled the network to GSM and force-stopped the app. After logging out and back in, the account came back light. Two retries without the throttle (menu toggle, then a force-stop under 1 s, then logging out and in) both kept the choice. So it's probably a save that didn't finish on the slow link.
5. **Two toasts at once when the stack runs out** (nit). Removing the last card shows "No more cards" and "Removed from deck" together. Shot: pass2-23.

### Cleanup

The Droid test deck is back to Edgar Markov, Lightning-Rig Crew and Vampire of the Dire Moon. I removed both main-board cards to recheck item 3 and restored them with Undo. Pass 2 added a few skips to the deck. The theme is Gruvbox dark, confirmed after a relaunch. The network throttle is off. The clone's target is 8.4 GB.

## Pass 3

Build: origin/main at 48e5f81b9 (release 48e5f81b9, live on prod), debug APK from ~/Developer/droid/zwipe, the same emulator and account. A release build turns WebView DevTools off, and CDP needs them, so I built debug; it talks to production the same way. Shots are in ~/Developer/droid/shots/pass3-<n>.png.

Surfer was cycling themes on the same account during this pass, so I ran the theme-on-resume checks last and read the server's settings before each one.

My slip: at 08:34, one tap on Settings' "Chip" cycled swipe confirmations to Toasts and saved it, and my next scripted tap hit the Dark toggle. I put both back within a minute (Off, then Chip; Dark), and the server confirmed gruvbox, dark, chip. The composer has been told.

### Open items rechecked

| # | Status | What I saw | Shot |
|---|---|---|---|
| 1 | fixed | With the keyboard up (499 px viewport), the focused password field sits at 226 to 273 and Log in at 289 to 336. | pass3-48 |
| 3 | partly | Emptying the pile shows only "Nothing left to remove" (P2-5 fixed). Undo at the end still does nothing; see new issue 4. | pass3-13, pass3-51 |
| 4 | fixed | On GSM, each panel shows its error with its own Retry, Featured flavor included. Retry with the network back loads each panel. | pass3-36, pass3-37 |
| 5 | still | Deck tile art skeleton is still 111x10. | pass3-35 |
| 6 | still | Playstyle's skeleton has one chip row where the loaded page has six, so Suggested commanders jumps about 300 px. | pass3-38, pass3-39 |
| 7 | still | Skeleton color rows are 24 px apart, loaded rows 27.4. | pass3-35, pass3-38 |
| 8 | not rechecked | | |
| 9 | partly | Home's Colors rows now end 20 to 23 px above the bottom (was 34), but start only about 6 px under the header (Decks: 12). Playstyle's Colors is even at 18/17. | pass3-37, pass3-39 |
| 10 | fixed | Format details scroll inside the fixed card and fade at the bottom edge (P2-2). | pass3-25 |
| 11 | fixed | Search to card 9 px, card to chips 9 px. | pass3-25 |
| 12 | fixed | Gutter is 16 on the deck page, Format picker, Dictionary and About. | pass3-2, pass3-25, pass3-46 |
| 13 | fixed | TIX and the Profile icons both end at 378.7. | pass3-2 |
| 14 | fixed | Buy, TCGplayer and Card Kingdom are all 32.2 px tall. | pass3-2 |
| 15 | fixed | Mana's rows space at 58 px like Profile's, with no extra rule. | pass3-3 |
| 17 | fixed | A single type reads "CREATURES". | pass3-4 |
| 18 | fixed | Deck form fields are about 20 px apart, labels about 9 px above their inputs. | pass3-24 |
| 19 | fixed | Commander has a chevron and lines up with Creatures. | pass3-5 |
| 20 | fixed | The dock no longer rides the keyboard. Quick add results now float above the field, over the commander art (new issue 7). | pass3-6 |
| 21 | changed | Mana's header × is now a separate red button beside the toggle. Its 44 px hit box still covers 44 of the header's 46 px height. The label-row ×s are a new problem (new issue 3). | pass3-22 |
| 22 | fixed | The first label sits 16 px under an open header. | pass3-22 |
| 23 | fixed | The Mana value input is 102 px with 86 px for text, and the placeholder fits (needs 84). | pass3-22 |
| 24 | fixed | The Examples card top is at 173, the same as Add's. | pass3-47 |
| 27 | fixed | "No commanders" sits 16 px above Danger zone, and Clear is disabled. | pass3-34 |
| 31 | fixed | Light hero stats and the selected USD chip pass (USD about 5.2:1). No contrast hits on Home, Decks, Settings or the deck page in Gruvbox light or dark. | pass3-41, pass3-42, pass3-44 |
| 34 | fixed | The Decks FAB and the Add dock are both 9/9 from the edges. | pass3-8 |
| 38 | fixed | Suggested commanders rows all put their chips on one line under the name. | pass3-40 |
| P2-1 | fixed | Stat chips, Group by, Show and Exceptions pass in dark. | pass3-1, pass3-43 |
| P2-3 | fixed | Cover flow arrows sit 16 px in from the screen edges. | pass3-19 |
| P2-4 | not reproduced | Light from the menu, then GSM and a force-stop within 3 s, kept the choice. So did throttling first, then toggling, then a force-stop 3 s later. Both times the server had the save before the stop. | |
| P2-5 | fixed | Only "Nothing left to remove" shows when the pile empties. | pass3-13 |

Left as is by Scotty: 29, 32, 36.

### New in this release

- **Stay signed in (156):** works. Three force-stop and relaunch cycles, a cold restart after the access token expired, and a warm resume after 17 minutes in the background all stayed signed in. The resume did one refresh, then loaded data and took taps (151).
- **Theme check on resume:** works. With the app in the background, I changed the theme through the API (gruvbox to nord, then back), and the app showed it within a second of resuming, on Home and on the deck page. The server hadn't moved between my checks.
- **Lifecycle (151):** no issues seen. Taps land and data reloads after a long background.
- **Chip contrast (157):** passes. See items 31 and P2-1.
- **Printings sheet fit (162):** holds. The sheet fits with Save visible at 845, 752 and 647 px tall (`wm size`), and the arrows keep the gutter.
- **Themed pips:** Settings shows Pips "Themed".
- **Swipe confirmations:** Settings shows Chip by default, and the chip is set after each swipe (Skipped, Added, Removed, Kept, Undone), but it never shows on screen. See new issue 1.

### Composer's extras

- **175, Remove to the end** (deck at 7 main cards): mid-stack Undo (swipe down) brings the card back, with the chip at "Undone". At the end there's a ghost card and a "Nothing left to remove" toast, but no empty-state text. Swipe-down Undo at the end does nothing (tried twice). The cards really leave: the server showed the deck's entries list empty. The Cards screen's Undo restored them one tap each ("Re-added …"). Afterwards I trimmed the deck back to its two original cards.
- **Chip mode messages:** the end-of-stack toast still shows on Chip. The chip itself, "Undone" included, is invisible (new issue 1). After the pile empties, the chip keeps the previous word ("Kept"), which won't matter until the chip shows.
- **Deck welcome hint:** every line matches the phone. The Cards pill is bottom right; pencil, clone and share sit by Profile; Import, Export and Delete are at the bottom; Warnings is there. One nit: the hint's "Clear skips" chip names a row whose button reads "Clear". It doesn't mention + and −, which are on the Cards screen.

### New issues

1. **The swipe confirmation chip is hidden behind the header** (broken). `.swipe-confirm` is `position: absolute; top: 0.5rem` inside `.screen-content`, and that starts at y 0 under the fixed nav (screens-b.css:1294). The chip draws at 16,8 and `elementFromPoint` at its centre returns NAV, so the default setting shows nothing on Add or Remove. Shots: pass3-9, pass3-10.
2. **The leave guard sticks after the Format picker** (broken). On Edit deck, open Format, tap the selected Commander chip (it deselects), then press the system Back twice. The second Back leaves for the deck page with no dialog, and the guard stays armed. From then on, menu links, the header's Back and the Cards pill do nothing, with no dialog in the DOM; only the system Back works, until a restart. Reproduced twice. A plain Edit then Back is fine. Shot: pass3-29.
3. **Filter label-row × buttons take taps across the whole section** (broken). `.clear-btn` uses `all: unset` (filter.css:171), which drops the coarse-pointer `position: relative`. So its 44 px `::after` sizes against the section and became 363x421. A tap on empty space left of the Mana value input cleared the Color identity filter. Shots: pass3-22, pass3-23.
4. **Undo at the end of the Remove stack does nothing** (ugly). See 175 above. Mid-stack Undo works.
5. **Card roles counts run past the value column** (nit). On the deck page's Distributions, the counts end at 382.7 where the value column ends at 378.7. The role chips start at 28.8 where the rows' labels start at 32.8. Shot: pass3-4.
6. **"Suggested commanders" wraps to two lines** (nit), now that it uses the larger panel-title type beside Swipe and "?". Shot: pass3-40.
7. **Quick add results float over the commander art** (nit). The result pills sit 4 px apart over the commander card and its name chips, which show through the gaps. Shot: pass3-7.
8. **Featured flavor's error chip sits close under its header** (nit). It's about 3 px below the rule, where the Decks and Colors errors sit about 16 px below. Shot: pass3-36.
9. **One tap on a settings value changes and saves it** (question). Swipe confirmations and Pips cycle their value on a tap with no picker, so a stray tap writes an account setting at once. Is that intended?
10. **Hero chip after a throttled start** (watch). After a slow start and Retry, the hero showed neither "Live" nor "as of" for a while. Later it read "as of 08:15", a count fetched at the first launch and not refreshed by Retry. Shots: pass3-37, pass3-41.

### Cleanup

The Droid test deck is back to Edgar Markov, Lightning-Rig Crew and Vampire of the Dire Moon (I added 5 cards for 175, then removed them). Pass 3 added a few skips and removals on Add and Remove. The theme is Gruvbox dark and swipe confirmations are on Chip, both confirmed on the server. The network throttle is off and the screen size is reset. The clone's target is 10 GB.
