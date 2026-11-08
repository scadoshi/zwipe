# Verify: the 1e03520ef deploy

Deployed 2026-11-07. Surfer (web, wide and 390) and Droid (Android app, plus phone-width web) check each line on live and mark it pass or fail with what they saw. Anything new they notice goes into this folder as a `reported` issue first, never straight to a fix.

Hard-refresh the web app before starting (see 282).

Not in this deploy (their PRs merged late): 221 on web, 258, 270, 273. They go in the next verify file.

| Nit | What to check | Web | App | Result |
|---|---|---|---|---|
| 214 | Add offline shows "Couldn't load cards" + Retry, one toast | Surfer | Droid | |
| 219 | Toast stays below the Exceptions chips with a sheet open | Droid (390) | | |
| 226 | App downloads compressed (deck loads on slow network) | | Droid | |
| 227 | Escape closes zwipe.net's phone menu, focus back on the toggle | Surfer | | web PASS |
| 228-229 | zwipe.net focus ring visible; skip link on zwipe.net and the portfolio | Surfer | | web PASS |
| 230 | Portfolio orangui source links resolve | Surfer | | web PASS |
| 231 | zwipe.net/reset/<bogus> shows "already used or expired" on load | Surfer | | web PASS |
| 232 | Token pages have their own titles | Surfer | | web PASS |
| 233-234 | Auth forms have autocomplete; Forgot password email field is type=email | Surfer | Droid | web BLOCKED (signed in; Account forms PASS) |
| 235 | Empty email says "Email is required" | Surfer | Droid | web BLOCKED (signed in) |
| 236 | Signed-out deep link returns to the deck after sign-in | Surfer | | web BLOCKED (needs sign-out) |
| 237 | Portfolio stats say showcased repos | Surfer | | web PASS |
| 238-240, 250 | Tab reaches deck rows, group headers, Create deck chips, Decks rows; collapsed groups skipped; no drag | Surfer | | web PASS |
| 243 | Deck name stops at 64 and shows its error live; no password checks in the UI | Surfer | Droid | web PASS |
| 244 | Totals say "N unpriced"; price sort puts unpriced last | Surfer | Droid | web PASS |
| 246 | Long deck names wrap beside the art on Decks | Surfer | Droid | web PASS wide |
| 247, 252 | Color group headers and color chips have screen-reader names | Surfer | | web PASS |
| 248 | Decks sort: name, size, format, price, last changed | Surfer | Droid | web PASS |
| 249 | Undo on Add works after a filter change | Surfer | Droid | |
| 251 | A 2,000-character password on login gets "password too long" | Surfer | | PASS (composer curl: 422 password too long) |
| 253, 256 | Card row art alt=""; Quick add chip is the input's label | Surfer | | web PASS |
| 254 | Save disabled while a field error shows; closing a sheet clears its draft | Surfer | Droid | web PASS (Save) |
| 257 | Decks: deck names are h2 | Surfer | | web PASS |
| 260 | Quick add "Mountain" offers the basic first | Surfer | Droid | web PASS |
| 263-264 | Playstyle Summary labels; Impact ignores one-deck tags | Surfer | Droid | |
| 265-266 | Tymna's partner list holds only plain Partner; Partner-with commanders autofill | Surfer | Droid | web 266 PASS; 265 FAIL (Swipe stack still offers Partner-with and Friends forever; issue 265) |
| 267 | Deck header says "1 card" | Surfer | | web PASS |
| 268 | Bad Archidekt link says "That isn't an Archidekt deck link" | Surfer | Droid | |
| 269 | Web swipes show up in Activity and the stats counters | Surfer | | |
| 271 | Floating search chips have the accent outline | Surfer | Droid | web PASS |
| 272 | Is legal in Commander + Pauper counts as a filter and widens the pool | Surfer | Droid | |
| 275 | Android Back with a sheet open on the deck form closes the sheet | Droid (web) | Droid | |
| 277 | Archidekt import sets format and commander on an empty deck | Surfer | Droid | |
| 336 | Share links: zwipe-site.pages.dev/deck/<token> has the deck's own og tags | composer (curl) | | |
