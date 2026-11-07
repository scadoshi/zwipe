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
