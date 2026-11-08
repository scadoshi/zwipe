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

## Pass 4

Build: origin/main at 3e72d3344 (release 3e72d3344, live on prod), debug APK from ~/Developer/droid/zwipe, the same emulator and account. Shots are in ~/Developer/droid/shots/pass4-<n>.png.

The emulator ran out of room for both the zwipe and Cairn APKs, so I wiped its data (`-wipe-data`) and installed zwipe fresh. No logouts this pass: still signed in across several force-stops, a fresh install login, and airplane mode on and off.

### Rechecks

| # | Status | What I saw | Shot |
|---|---|---|---|
| 175 | mostly fixed | Remove now has an Undo in its dock (disabled until the first swipe). Dock Undo mid-stack brings the card back (chip "Undone"). A fast remove-then-swipe-down, sent back to back, ended with all 6 cards in the UI and on the server. At the end, an empty-state chip reads "Nothing left to remove", and the dock Undo brings the last card back. Swipe-down at the end still does nothing, though the dock button covers that. New nit: with Undo added, the dock is 193 px wide and overlaps the "All" board chip by 9 px (All 179..218, dock from 209). The chip still takes taps at its centre. | pass4-7, pass4-8 |
| 177 | fixed | The chip draws below the header at 16,53: "Skipped", "Added", "Removed", "Undone". | pass4-6 |
| 178 | fixed | Format picker, deselect Commander, system Back: "Discard format change?" (Keep/Discard). Discard, then Back leaves Edit deck, and the menu and header Back work. Deselect, ✓, Back: "Leave without saving?" (Stay/Leave). Leave works, and navigation stays alive. Odd: Back from Commander maybeboard after Leave went to Decks, not Deck. | pass4-10, pass4-11 |
| 179 | fixed | All three clear buttons have a 44x44 hit box now. Taps on empty space beside Mana value, by Produces and right of Color identity leave every filter in place. | pass4-12 |
| 171 | not reached | On GSM, the save landed within 0.3 s, before the force-stop. With airplane mode on, the toggle doesn't apply at all: the theme stays and a "Connection error" toast shows, so there's no pending change to resend. I couldn't make a change that's applied locally but not yet saved. | pass4-17 |
| 170 | fixed | With a pending change, Back or the header Back in the Format picker asks "Discard format change?". Reopening the picker shows the "Tap a format" hint, not the last format. | pass4-13, pass4-14 |
| 172 | partly | Remove's empty end shows "Nothing left to remove" twice: as the empty-state chip and as a toast. Elsewhere I saw one message per action. | pass4-8 |
| 169 | fixed | The toast close button is 32x32 with a 44x44 hit box. | pass4-8, pass4-17 |
| 165 | fixed | Nord light (theme class swapped locally): the warning row's Add cards and Import, and Data's Import and Export, all pass. The Nord light nav title and panel titles are still 3.17:1 (Cairn's sweep shows the same across many light themes). | pass4-15 |
| D5 | fixed | Deck tile art skeletons are 111x111. | pass4-2 |
| D6 | fixed | Playstyle's skeleton has the loaded page's chip rows, and Suggested commanders starts at the same height (about 2235 device px skeleton, 2241 loaded). | pass4-4, pass4-5 |
| D7 | fixed | Skeleton color rows are 27 px apart, like the loaded rows. | pass4-2 |
| D9 | fixed | Home's Colors is 8 px under the header and 13 px above the bottom. Home skeleton panels are within 2 px of loaded (Decks 246/244, Colors 227/226). The hero grows 5 px and Featured flavor 13 px on load. | pass4-2, pass4-3 |
| 176 | fixed | The welcome hint's chip now reads "Clear", matching the button. Every line matches the phone. | pass4-16 |

### New

1. **The connection error toast is cut off** (nit). "Connection error, check your network and try again" is clamped at two lines, so it ends at "network‥". Shot: pass4-17.
2. **The dock overlaps the "All" board chip on Remove** (nit). See 175.

### Cleanup

The Droid test deck is back to Edgar Markov, Lightning-Rig Crew and Vampire of the Dire Moon (2 entries on the server). I added 4 cards and removed them again, which leaves a few skips and removals. The theme is Gruvbox dark and swipe confirmations are on Chip, both confirmed on the server. The throttle and airplane mode are off. The clone's target is 11 GB.

## Pass 6

Build: main at f699acc37 (live on app.zwipe.net and zwipe.net), debug APK from ~/Developer/droid/zwipe on the Pixel_9a emulator against production, plus app.zwipe.net in the emulator's Chrome at 411px, both signed in as `test`. Shots are in ~/Developer/droid/shots/pass6-<n>.png (app) and web6-<n>.png (web). No logouts this pass.

### Rechecks

| # | App | Web | What I saw | Shots |
|---|---|---|---|---|
| 182 | n/a | not reached | The title tag only renders on a wide layout (`.web-screen-chip` is `display: none` below the breakpoint), so a phone width can't show it. | |
| 183 | fixed | fixed | Format picker: search to hint 22px, hint to chips 23px. | pass6-29 |
| 184 | fixed | fixed | The last skip leaves a plain ghost card, a "No cards left" toast and no chip. On web, ↓ on that ghost brings the last card back with no reload. Remaining: see New 3 and 4. | pass6-13, web6-15, web6-16 |
| 186 | n/a | fixed, now larger than the app | Web preview is 379px wide with 16px gutters. The app's is 333px with 39px gutters, so the two still differ, the other way round now. | web6-2, pass6-5 |
| 187 | fixed | fixed | The app's card preview dims the header too. The emulator's status bar is drawn by the system, so I couldn't judge it. | pass6-5 |
| 188 | fixed | fixed | Decks and Cards skeletons under a slow network: every chip ghost is an empty box. Row titles still ghost as bars, which reads fine. | pass6-33-2, pass6-36-1 |
| 189 | fixed | fixed | "Browsing your deck" scrolls clean to both rules, no mask on any element in the dialog. | pass6-30, pass6-31 |
| 190 | n/a | fixed | `touch-action: manipulation` on html and body, viewport without `maximum-scale`. Quick double taps on the card and the dock never zoom (scale stays 1). A quick double tap on the dock's ← skips one card, not two (New 5). | web6-4 |
| 191 | fixed | fixed (layout) | The letter rail is `overflow-y: hidden`. A vertical drag leaves it at scrollTop 0, a sideways drag scrolls 410px. Header to Search label is 77 device px, the same as the Format picker. On web the dictionary modal painted blank in the emulator (New 6), so I measured it through the DOM. | pass6-27, pass6-28 |
| 192 | broken with long names | broken with long names | The title and cost now sit in the header beside the ×, and there's no head band or fold. But the title starts under the ×: with "Edgar, Ancient Bloodlord" the × box ends at x 89 and the title starts at 81 in the app (71 on web), so the "E" is covered. The × is absolutely positioned at left 16px and the header's padding-left is only 24px. Short names like Phyrexian Arena are centred and clear it. On web I put the long name into the title by script to check, so the overlap is measured, not seen on a real card. | pass6-9, web6-10-cdp |
| 193 | n/a | fixed | Web swipe card is 347x484, the app's 348x483. | web6-3 |
| 194 | works | works | The empty maybeboard keeps its panel and Swipe with one "No commanders" row. Swipe select's hint lists left skips, up saves, right starts a deck and down undoes, on both. App: left skipped, down undid, up saved Olivia, Crimson Bride (server count went 0 to 1 to 2 over two ups), right opened Create deck with Commander and Olivia, Opulent Outlaw filled in. Remove takes off the right row (server kept the other one). I removed my test saves, so the server is back to 0. | pass6-15 to pass6-22, pass6-42, web6-19 |
| 195 | fixed | fixed | Home reads "as of Nov 7" on both. | pass6-1, web6-1 |
| Light contrast | fixed | fixed | All 50 light themes swapped in locally (no account change). App Cards screen and web Cards: the only text under 4.5:1 is the disabled Maybe and Side board chips, in 7 themes. App details drawer: 0 of 50. | pass6-37 |

### New

1. **Long card titles slide under the drawer's ×** (bug, app and web). See 192.
2. **The details drawer isn't centred** (question). The phone drawer sits at x 38 to 401 of 411, so the left gutter is 38px and the right 10px. Intended as a side drawer?
3. **Re-entering a swipe screen whose filter has nothing left shows only the ghost card** (nit, app and web). With a name filter whose cards I'd already skipped, Add opened on a bare ghost card with no toast or words, so nothing says why it's empty. Shots: pass6-38, web6-12.
4. **↓ on the ghost card does nothing when there's no history** (nit, web). After a filter change, the dock's ↓ toasts "Nothing to undo", but the same swipe on the ghost is silent. The dock's ↓ also looks enabled there. Shots: web6-14.
5. **A quick double tap on a dock arrow counts once** (question, web). Two taps about 100ms apart on ← skipped one card. Fine if the fly-out locks input; otherwise the second tap is lost.
6. **The web Oracle tags dictionary paints blank** (unclear, retest on a phone). Opened from Add's filter, Oracle tags, Dictionary, the modal is a flat grey fill in both the device screenshot and a CDP capture, even after scrolling. The DOM is complete (letter rail, rows, no opacity or visibility issue, nothing on top). The emulator's Chrome also cut off the details drawer's background, which a CDP capture showed was painted, so this may be the emulator. Shots: web6-22 to web6-24.
7. **The app's ghost card is a little smaller than the card** (nit). The ghost image is 333x465 CSS px against the live card's 348x483. On web the ghost matches the card.
8. **Back from Create deck, opened by a right swipe, leaves without asking** (question). The form arrives with Commander and a commander filled in. Back goes straight to the maybeboard, with no "Leave without saving?", which may be right since I hadn't typed anything.
9. **Decks fails to load on GSM** (watch). With `speed gsm` and `delay gprs`, Decks showed "Could not load decks" with Retry. Retry worked once the speed was back to full. Shots: pass6-34.

### Account notes

- The account's theme is Zenburn dark on the server (`theme: zenburn, dark_mode: true`), not the Gruvbox dark I left after pass 5. I didn't touch the theme this pass, so someone changed it on another device. I left it.
- The Commander maybeboard held Edgar Markov in the app at the start of the pass. Later the server and a fresh app start both showed it empty. My controlled test shows Remove deletes the right row, so the likely cause is a change from another device or a stale copy in the app. Unclear, retest. I didn't put Edgar back.
- Azula's skip is untouched: I filtered Swipe select by name so Azula never came up for a swipe.

### Cleanup

Droid test is as Scotty left it, apart from skips on Add (app and web). The maybeboard is empty on the server. Throttle and airplane mode are off. The clone's target is 11 GB.

## Pass 7

Build: main at f699acc37 (prod), debug APK on the Pixel_9a emulator against production, then app.zwipe.net at 411px in the emulator's Chrome, both as `test`. Shots are in ~/Developer/droid/shots/pass7-<n>.png (app) and web7-<n>.png (web). No logouts.

I made two throwaway decks ("Droid tmp" in the app, "Droid web tmp" on web) so swipes and imports didn't touch Droid test, and deleted both on web. Every setting I changed went back and I checked it on the server.

### What works

- **Deck create, edit, clone, delete** (app and web). Create with only a name, then Edit to set format, commander (Swipe select), power level, land target, price target, deck tag and other tag, and save. Clone opens "<name> (clone)" with a toast. Delete asks first and the list count drops.
- **Import** (app and web). Text import in Add mode handles "2 Swamp", "1x Name", a "Sideboard" header (Bolt went to the side board) and reports "Notacard Fakename not found". Replace mode and an export round trip work: importing the export gives the same list.
- **Export**. The Deck, Maybe and Side chips add their sections with "// Commander", "// Deck", "// Sideboard" headers, and copy toasts "Copied to clipboard".
- **Share**. Create link copies it ("Share link copied") and Stop sharing disables it ("Link disabled").
- **Filters on Add**: oracle text, flavor text, price max, mana value, artist (typeahead pick), Planeswalker plus Mythic, excluding Creature and Land plus the Removal role, legal in Pauper on its own, and sort by price. Every one returned cards that fit.
- **Swipe select in each mode** (app): commander (right picks Edgar Markov), oathbreaker, signature spell, background (Wilson, Refined Grizzly, then Scion of Halaster) and partner. Each mode's hint names what right does. Maybeboard mode was covered in pass 6.
- **Format changes** drop a command zone card that no longer fits (Edgar left when the deck became Oathbreaker), and the deck page warns about the new format: 60 cards, banned Sol Ring, no signature spell, no sideboards.
- **Themes**: picking Bluloco and toggling light mode both apply at once and save. Zenburn dark is restored.
- **Settings** (app and web): Pips (Themed and Classic), Swipe confirmations (Chip, Toasts, Off), Universes Beyond (Show and Hide, which enables Exceptions) all save at once with a toast. All restored.
- **Hints**: every "?" I opened had text, and Add's "See guide ↗" points at zwipe.net/guides/swipe-to-build/ (200).
- **About, Account, Help & feedback** open with sensible content (v1.18.0).
- **Offline Home** shows the cached counters and decks. Decks offline shows "Could not load decks" with Retry, and Retry works once online (app and web).
- **Rotation**: forcing landscape keeps the app in portrait, as the manifest locks it.

### New

1. **The app's header Back drops unsaved Edit deck changes without asking** (bug, app only). Rename the deck in Edit deck, then tap the header ‹: the Deck page opens and the rename is gone. System Back on the same edit shows "Leave without saving?", and so does the web's header Back. Pass 4 had the app's header Back asking (178), so this looks like a regression. Shots: pass7-25, pass7-26, web7-1.
2. **A colorless commander is treated as five-color** (bug, app and web). Droid tmp with Arzakon (a colorless planeswalker) shows W U B R G pips on Decks, and the signature-spell Swipe select offers Negate, Chaos Warp and Nature's Lore. Shot: pass7-29.
3. **The partner Swipe select offers the commander itself** (bug). With Tymna the Weaver as commander, the partner stack's second card is Tymna the Weaver.
4. **Import with set and number picks an Art Series card** (bug, app and web). "1 Bloodline Keeper (ISD) 90" imports "Bloodline Keeper // Bloodline Keeper", Common, Innistrad Remastered Art Series, grouped under Other. The export then carries that name. Shots: pass7-11, pass7-13, pass7-14.
5. **"Could not load deck" is a dead end** (bug, app and web). Opening a deck offline shows "Could not load deck" with no Retry, and it stays there after the network returns. Only leaving and coming back loads it. The same visit raises the connection toast twice. Shot: pass7-39.
6. **More than one "Is legal in" format means any of them** (question). With the deck's Commander preselected, adding Pauper still shows Elenda and other rares; Pauper alone shows commons. If "legal in both" is meant, it should narrow.
7. **Warnings lowercase card names** (nit): "Sol ring is banned in oathbreaker", "Bloodline keeper // bloodline keeper is not legal in oathbreaker".
8. **Replace import doesn't confirm** (question). It clears the mainboard straight away.
9. **The share dialog never shows the link** (nit). It offers Copy link and "Stop share" (the body says "Stop sharing"), but never the URL itself.
10. **Create and Edit treat the commander rows differently** (nit). The app's Create deck shows Commander, Partner, Background and Signature spell rows with no format set; Edit hides them until a format is chosen. On web, Create leaves an empty gap where they hide. Shots: pass7-2, web7-3.
11. **Switching to Oathbreaker keeps the 37 land target** (nit). It was set while the deck was Commander; a 60-card format may want its own default.
12. **A toast covers the Exceptions sheet's chips** (nit). "Universes Beyond hidden" sits over the last row of the picker (Warhammer 40,000). Shot: pass7-35.
13. **Lightning Bolt imports at $28.99** (question). With no set given, the printing chosen is a pricey one, which skews the deck total.
14. **Back from a deck goes to Decks, not where I came from** (nit, app). Open Droid test from Home's tile, and the header Back lands on Decks.
15. **Web's "See guide ↗" opens in the same tab** (question). The arrow suggests a new tab, but `target` is empty, so it leaves the app.

### Not covered

- The Universes Beyond exclusion's effect on Add results, the Archidekt import source and the Combat filter. I ran out of time on these.
- Slow network: GSM was tried in pass 6 (Decks timed out). This pass only did offline.

### Cleanup

Both throwaway decks are deleted (12 decks on the server, Droid test untouched). Server settings: zenburn, dark, theme pips, chip confirmations, Universes Beyond shown with the same four exceptions. The maybeboard is empty, as the composer asked. Airplane mode is off and rotation is back to portrait. The clone's target is 10 GB.

## Pass 8

Build: main at f699acc37 (prod), debug APK on the Pixel_9a emulator against production, plus app.zwipe.net in the emulator's Chrome, as `test`. Shots are in ~/Developer/droid/shots/pass8-<n>.png. I used one throwaway deck (deleted at the end) for imports, swipes and the long name.

### Nit 210 repro (app, header ‹ on Edit deck)

| Case | "Leave without saving?" | Result |
|---|---|---|
| (a) rename only | no | rename lost |
| (b) format only (None to Commander, shown in the form) | no | still None |
| (c) Commander format, then Edgar Markov via Swipe select | no | both lost |

System Back on the same edits shows the prompt (pass 7), and so does the web's header Back.

### Pass 7's Not covered list

- **Universes Beyond hidden on Add** works. With UB shown, Name "Dalek" gives The Dalek Emperor and friends. Hidden, it gives "No results for this filter". Spider-Man cards still show because Marvel is one of the four exceptions, and Sliver Queen (not UB) is unaffected. Restored to Show. Shot: pass8-1.
- **Combat filter** works. Power exact 4 returned Terry Pin 4/3, Inga and Esika 4/4, Treasury Thrull 4/4, Kheru Lich Lord 4/4, Monet 4/4, Everlasting Lich 4/0 (each checked in card details).
- **Archidekt import** works. https://archidekt.com/decks/59349 brought in all 100 cards ($587.51), Bloodline Keeper as "Bloodline Keeper // Lord of Lineage" (unlike the text import's Art Series card, pass 7 New 4). But the deck stayed format None with no commander: Edgar Markov arrived as an ordinary creature. Question: should an Archidekt import set the commander (and format) from Archidekt's Commander category? Shots: pass8-2, pass8-3.

### Slow network

1. **Big loads time out on EDGE in the app because responses come uncompressed** (bug, app only). With `adb emu network speed edge` and `delay edge`, Decks shows skeletons for about 31s, then "Could not load decks" with a connection toast. Playstyle, Activity, Commander maybeboard, the Droid test deck, its Cards and Add stayed on skeletons past 25s. Home, Settings, Account and About show at once from cache. The cause looks like the client: the workspace `reqwest` has `features = ["json", "rustls-tls", "http2"]` and no `gzip` or `brotli`, so the app never sends Accept-Encoding. The server does compress: the 100-card deck is 512,820 bytes raw and 51,395 bytes with `accept-encoding: br`. Add `REQUEST_TIMEOUT` = 30s in `zwipe-client/src/lib.rs`, and a few decks' worth of raw JSON can't arrive in time at EDGE speed. The web isn't affected (the browser asks for compression). With DevTools throttled to about 30 KB/s and 400ms latency, web Decks, Activity and the maybeboard showed in under a second and Playstyle in about 5s. Shot: pass8-10.
2. **Add stalls on a bare ghost card on a slow network** (bug, app). On EDGE, Droid test's Add sat on an empty ghost card for over a minute with no message, toast or error. It stayed empty after the network went back to full, and Add's refresh button didn't help; leaving and coming back loaded 10 cards. Shots: pass8-11, pass8-12.

### Free roam

- **100-card deck**: the Cards page drew 90 rows in under 0.6s. The Type, Mana value, Color and Card role groupings all add up (Type 27+14+6+12+9+32 = 100, Mana value 13+26+14+6+4+5+32 = 100). Shot: pass8-6.
- **Long deck names**: a 94-character name is refused on save with "Deck name maximum length is 64 characters". The field itself has no maxlength, so you can keep typing past 64 and only learn at save (nit). A 64-character name wraps cleanly on the Deck page, the Cards header and the Decks list, and Home truncates it with "…". Shots: pass8-4 to pass8-8.
- **Rapid swipes and undo** (temp deck, server counts after each burst). Ten right swipes sent back to back added 5 cards (100 to 105). The other five were ignored mid-animation. UI and server agreed, so nothing was lost or doubled. Bursts of five swipe-downs undid two cards each (105 to 103 to 101), then one more brought it to 100 exactly, and the next said "Nothing to undo". The "Undone" chip was still on screen next to that toast (nit, one message per action).
- **Help & feedback**: "Join the Discord ↗" is a live invite (discord.gg/s2UReqUUeg, server "Zwipers", no expiry). "Report a problem ↗" opens the mail app (a mailto), which is the Gmail welcome tour on the emulator.
- **About**: "View" opens the in-app Privacy policy. "zwipe.net ↗" opens a Chrome Custom Tab. The Scryfall, Fan Content Policy, zwipe.net and zwipe.net/privacy links all return 200.

### Cleanup

The temp deck is deleted (12 decks on the server, Droid test untouched). Server settings are unchanged (zenburn, dark, theme pips, chip, Universes Beyond shown with the same four exceptions), the maybeboard is empty, the network is back to full and airplane mode is off.

## Pass 9

Build: debug APK from origin/release, app only, against production (api.zwipe.net), on the Pixel_9a emulator, installed with `-r` so `test` stayed signed in. First build a0cb6c918; I rebuilt at 1cb240dd0 once #346 (210) landed. Shots are in ~/Developer/droid/shots/pass9-<n>.png. One throwaway Modern deck, deleted at the end.

Production still runs the main-branch server, so fixes that changed zerver or the shared warning code can't show yet: 200's import, 201, and 208's copy. 209 has no commit in release.

| # | Status | What I saw | Shots |
|---|---|---|---|
| 197 | fixed (app) | After +, Fix to 4, To sideboard, Quick add and To mainboard, the Deck page's card count and Warnings matched each change on return, with no reload. Fix to 4 toasts "Quantity set to 4", and count and warnings change at once. I didn't reach a "Fix to 1" case (Modern offers Fix to 4). | pass9-1, pass9-3 |
| 198 | fixed | `.tag-grid` has no mask in the Format or Oracle tags picker. Scrolled chips cut at the hint box's edge. | pass9-13, pass9-14 |
| 199 | fixed | "Edgar, Ancient Bloodlord": the × ends at x 89, the title starts at 97 and wraps to two lines with its pips centred under it. | pass9-12 |
| 200 | export fixed, import needs a server deploy | Export now writes set and number, e.g. "4 Lightning Bolt (SLC) 83" and "1 Snapcaster Mage (INR) 478". Import on production still maps "(ISD) 90" to "Bloodline Keeper // Bloodline Keeper" (Innistrad Remastered Art Series) and "(ISD) 78" to Snapcaster Mage from Innistrad Remastered. 8116af609 changed zerver. | |
| 201 | needs a server deploy | Set: Innistrad shows Silent Departure and the rest as the Innistrad Remastered printing (zerver change f69990dba). | pass9-8 |
| 202 | not fixed in the app | The Set typeahead pills float above the input, covering the Oracle tags and Set headers. After picking "Innistrad" they stay open over the picked chip until the text is cleared. The Cards page's Quick add pills do the same: after a pick they stay up over the header and cover its ‹, so Back can't be tapped until the field is cleared. 548e33d2d may be web only. | pass9-5 to pass9-7 |
| 203 | fixed (with a double message) | A filter with no results shows "No cards match" inside the ghost card, plus a "No results for this filter" toast. "Is promo: Yes" still matches nothing, even with Name "Sol Ring". | pass9-9 |
| 204 | not fixed in the app | Mana value typed with the real keyboard ("4") and left 2s in focus: the section's × doesn't appear, and Apply while still focused applies no filter (no dot on the funnel) and leaves a bare ghost with no message. Blurring the field commits it. 8121e7a68 may be web only. | pass9-10, pass9-11 |
| 208 | needs a server deploy | Warnings still read "Deck has 7 cards, modern requires at least 60", "Bloodline keeper // bloodline keeper is not legal in modern" and "Lightning bolt exceeds copy limit (5/4)". | pass9-2 |
| 209 | not in release | The share dialog still says "Stop share", and the Cards page shows a Command zone chip on a Modern deck. | pass9-5 |
| 210 | fixed (1cb240dd0) | Header ‹ on Edit deck asks "Leave without saving?" for (a) rename only, (b) format only and (c) Commander format plus Edgar Markov picked in Swipe select. Stay keeps the edits in the form; Leave discards them (the deck stays Modern with its name). | |

### New

1. **Quick add doesn't offer the exact match** (nit). Typing "Mountain" lists Call the Mountain Chocobo, Celebrate the Mountain-king, Cori Mountain Monastery, Cori Mountain Stalwart and Goblin Mountaineer, but not the basic Mountain. Shot: pass9-4.

### Cleanup

The temp deck is deleted (12 decks, Droid test untouched). Server settings are unchanged (zenburn, dark, theme pips, chip, Universes Beyond shown with the same four exceptions), and the maybeboard is empty. The emulator now runs the 1cb240dd0 debug build, still signed in.

## Pass 10

Build: debug APK from origin/release at 9665fd9e3 (includes #351, #350 and 209's #342), app only, against production, on the Pixel_9a emulator, installed with `-r` so `test` stayed signed in. Shots are in ~/Developer/droid/shots/pass10-<n>.png. One throwaway deck (Modern, then Commander with Kozilek), deleted at the end. I made all taps below with real `adb input tap`; the scripts only found where to tap.

| # | Status | What I saw | Shots |
|---|---|---|---|
| 202 | fixed | Set, Artist, Oracle tags, Oracle words, Keywords and Type line: the pills sit under their box, none above it or over a section header. A tap on a pill adds the chip, empties the box and closes the pills. Quick add on Cards: pills start under the box (box 99..135, first pill at 151) and the header ‹ is clear (hit test lands on the back button). Picking Goblin Mountaineer adds it, empties the box, and ‹ then works. | pass10-4 to pass10-6 |
| 204 | fixed | Mana value "4" and Price Max "0.10", typed with the real keyboard and left focused, show the section's × within 1.5s. Apply while still focused applies them (mana value 4 cards; Swift Maneuver and other cheap cards). | |
| 203 | fixed | A no-result filter shows "No cards match" in the ghost card with no toast. With Synergy on (⚡ pressed) it adds "Synergy is on, turn it off to search everything". With Synergy off, only the first line shows. | pass10-11, pass10-12 |
| 209 | fixed | The share dialog's button and body say "Stop sharing". A Modern deck's Show row has Show, Tokens and Art, no Command zone. The maybeboard hint covers its Swipe (left skips, up saves, right starts a deck) and the row actions (Quick add, Create deck, Remove). Its See guide link (zwipe.net/guides/commander-maybeboard/) returns 200. | pass10-1, pass10-2 |

### A picked filter chip's × ignores real taps (bug, app)

Laptop's report reproduces. After picking Innistrad in Set, a real tap on the chip's × (at 368,1509, the button's centre) does nothing, and neither does a tap on the chip body. The × is `button.chip-remove`, 10x16 CSS px, and `document.elementFromPoint` at its centre returns the parent `div.chip.selected`, not the button. So touches never reach the button, even though its click handler works (a scripted `.click()` removes the chip). That target is also far under 44px.

### Colorless commander (composer's add)

Droid tmp10 with Kozilek, the Great Distortion as Commander: after I removed an off-color card, the Decks row shows no color pips (pass 7's Arzakon deck showed all five). After Reset, Add's filter starts with Color identity set to nothing (colorless) plus synergy Oracle tags. The stack is all colorless: Expedition Map, Planar Bridge, Eldrazi Temple, Hedron Archive, Basalt Monolith, Void Winnower, Dreamstone Hedron, Ugin, the Spirit Dragon, Stonespeaker Crystal, Everflowing Chalice, Geode Golem, Sanctum of Ugin, Kozilek, Butcher of Truth, Ulamog, the Infinite Gyre, Gilded Lotus, Artisan of Kozilek. Before the Reset, a Price filter left over from my 204 check had no color limit and dealt white, red and green cards. That's expected for a hand-set filter, but worth knowing.

### Nit 217 correction

Pass 7 item 10 said the app's Create deck shows the Commander, Partner, Background and Signature rows with no format set. That came from page text, not a screenshot. On 9665fd9e3 the rows are in the DOM but collapsed to 0 height, so Create hides them like Edit does (pass10-7). The web half stands: app.zwipe.net's Create leaves an empty gap where the rows collapse (web7-3).

### Other

- The swipe chip "Skipped" stays on screen after a new filter returns no cards (nit). Shot: pass10-11.
- Quick add still doesn't list the basic Mountain for "Mountain" (pass 9).

### Cleanup

The temp deck is deleted (12 decks, Droid test untouched). Server settings are unchanged, the maybeboard is empty, and the emulator runs the 9665fd9e3 debug build, still signed in.

## Pass 11

Release 80d5b43ab is live. Web: app.zwipe.net at 411px in the emulator's Chrome. App: the 9665fd9e3 debug build against production (now on the new server). Signed in as `test`. Shots are in ~/Developer/droid/shots/web11-<n>.png and pass11-<n>.png. One throwaway web deck ("Droid web11"), deleted at the end. Every tap that mattered was a real `adb input tap`. Note that the web filter's accordion ignores scripted clicks; the app's does not.

### Server-side fixes, now live

| # | Status | What I saw | Shots |
|---|---|---|---|
| 200 | fixed | Web import of "1 Snapcaster Mage (ISD) 78" and "1 Bloodline Keeper (ISD) 90" lists "Bloodline Keeper // Lord of Lineage". Export writes "(ISD) 90" and "(ISD) 78" back, and Bloodline Keeper now shows its real art and groups as a Creature. | web11-13 |
| 201 | fixed | Set: Innistrad on web shows Silent Departure as the Innistrad printing, no longer Innistrad Remastered. | web11-12 |
| 203 promo | partly | Corrected after a retest (app, Add on a deck with no format, filter Reset, Config defaults). My first web check was on a Modern deck, where the default "Is legal in: Modern" excludes Sol Ring, so the format, not Is promo, emptied it (the same applies to pass 9's app note). With no format: Name "Sol Ring" + Is promo: Yes returns Sol Ring, but its details show the Foundations Commander printing, which isn't a promo, so it matches the card and then shows the default printing (like 201 before its fix). Is promo: Yes alone runs no search: bare ghost, no dot on the funnel, no message or toast. | |
| 208 | fixed | Web warnings on a Modern deck: "Deck has 8 cards, Modern requires at least 60", "Pyroblast is not legal in Modern", "Lightning Bolt has 5 copies, Modern allows 4". | web11-3 |
| 197 (web) | fixed | Fix to 4 toasts "Quantity set to 4", and the count and warnings drop to 7 cards and 3 warnings at once. | |

### Web rechecks (same nits as pass 10's app run)

| # | Status | What I saw | Shots |
|---|---|---|---|
| 202 filters | fixed | Set: the pills (DIV chips) sit under the box (box 449..485, first pill 501) and close on a real-tap pick, which also empties the box. The pills can sit behind the open keyboard, so a tap aimed at one can land on a keyboard key. Closing the keyboard first keeps them up and tappable. | web11-6, web11-11 |
| 202 Quick add | not fixed on web | On the web Cards page, "Mountain" pills float above the box (box 99..135, pills at -42..61 CSS px). They cover the deck title and the header ‹, whose hit test lands on a pill. The app's Quick add is fixed. | web11-13 |
| 204 | fixed | Mana value "7" typed with the keyboard and left focused shows the section's ×. Apply while focused returns 7-drops (Dream Harvest, Conspiracy Unraveler, Never // Return, Terror Ballista). | |
| 203 | fixed | No-result filter: "No cards match" with no toast. | |
| 209 | fixed | The share dialog says "Stop sharing". A Modern deck's Show row has no Command zone. The maybeboard hint covers its Swipe and row actions. | |
| 225 (chip ×) | not fixed on web | Same as the app: the picked chip's `button.chip-remove` is 10x16, `elementFromPoint` at its centre returns the chip DIV, and a real tap does nothing. | |

### Free app run-through

- **Activity totals don't add up to the swipe count** (question). The Activity panel reads 303 swipes, but its Totals bars are Added 25, Skipped 52, Maybeboard 5, Removed 64 = 146. My guess is that swipes on decks since deleted (lots of temp decks in passes 7 to 11) leave the bars but not the counter. Shot: pass11-2.
- **Draw odds start at "by turn 0"** (question). The stepper works (+ to turn 3), but turn 0 is an odd default label; "opening hand" may read better.
- Playstyle, Activity, Distributions, Mana and Draw odds all render with data. Printings opens the cover flow, → steps and updates the price chips (INR 328: $32.79 €27.47 1.72 TIX), and closing without picking toasts "Printing unchanged". Shots: pass11-1 to pass11-6.

### Cleanup

The web temp deck is deleted. The server has 13 decks: Droid test plus Surfer's "Surfer p19 renamed" and the rest. Server settings are unchanged (zenburn, dark, theme pips, chip, Universes Beyond shown with the same four exceptions), and the maybeboard is empty. The emulator runs the 9665fd9e3 debug build, signed in.
