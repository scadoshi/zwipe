# Surfer report: app.zwipe.net + zwipe.net (2026-11-06)

Browser: Chrome "scotland-halo", signed into the test account. Window 1450px wide. I couldn't test narrow: Chrome won't resize below about 1420px in this setup, so every phone-width check is still untested. Themes: Gruvbox dark, plus Gruvbox light with Dark Reader locked out (see #1). The theme is back to Gruvbox dark.

Writes, all on the test account (Scotty OK'd this): I created the deck "Surfer test" (Commander, Edgar Markov, tag Tribal) and added 5 cards by swiping. I also changed Edgar's printing to Judge Gift Cards 2021, ran a remove + undo, and created a share link (https://zwipe.net/deck/f1c34d48-e388-477c-9b1a-74f1e56f6216). I left the deck in place, since Scotty said to play freely.

Not an issue: fast key bursts drop swipes (5 instant Rights added only 1 card). Scotty says that's the intended rate limit.

## Issues

1. **Dark Reader repaints the app.** app.zwipe.net (every page), wide, light themes. Dark Reader is installed in this Chrome and injects about 18 style tags on app.zwipe.net. That turns the light theme into dark brown on olive. zwipe.net doesn't get them. Adding `<meta name="darkreader-lock">` to the page fixed it. Expected: the app opts out the same way zwipe.net does. Severity: ugly (affects any Dark Reader user).
2. **Deck price differs between the app and the shared page.** Deck "Surfer test", wide, dark. I switched Edgar Markov's printing to Judge Gift Cards 2021. That printing has no USD price: the cover flow and the expanded row show only the set and P/T, with no USD chip. The collapsed row still shows **$112.50**, and the app's deck total is **$187.19** (it includes the $112.50). The shared page on zwipe.net shows **$74.69** for the same deck (it excludes the $112.50). Before the swap, the INR printing showed "$2825.00" in the collapsed row, but its details listed only "6.81 TIX". Expected: the collapsed price, the details chips and both totals all agree on the chosen printing's price. Severity: broken (wrong numbers).
3. **Enter doesn't close a picker drawer's `?` hint.** Deck form, Format drawer, `?` at the top right, wide, dark. I opened the hint and pressed Enter. It closes and reopens right away, because focus stays on the `?` button, so Enter clicks it again. "Got it" with the mouse works. Enter does close page hints elsewhere (Password, Version, Commander maybeboard, Oracle tags). Expected: Enter closes it, and focus moves into the hint or back somewhere harmless. Earlier in the same flow, a Commander pick I had made in that drawer was gone after Enter + ✓, and I got the toast "Format unchanged". I couldn't reproduce that, but it may be the same focus problem. Severity: broken (for keyboard users).
4. **Picker ✓ covers chips, with a ghost button behind it.** Deck form, Deck tags drawer, wide, dark. The floating ✓ sits on top of the tag list and hides "Dice Rolling" in the list's last visible row. The page's own ✓/funnel button also shows through just behind and to the right of it, as a second rounded square. The same ghost shows on zwipe.net's shared-deck filter drawer. Expected: padding at the bottom of the list so it clears the ✓, and the page dock hidden while the drawer is open. Severity: ugly.
5. **White scrollbar in zwipe.net's filter drawer.** Shared deck page, funnel, wide, Gruvbox dark. The drawer's scrollbar is a wide, light gray/white bar, not the thin themed one the app uses. Severity: ugly.
6. **Swipe screens have a page head and no nav.** Commander → Swipe on the deck form ("Swipe select") and Playstyle → Swipe ("Suggested commanders"), wide, dark. Both show a title bar with `‹` and `?`, and no app nav. Add and Remove keep the normal nav, with `?` in the icon row. The brief says there are no page heads except Home's hero. If full-screen swipe modes are exempt, ignore this. Otherwise they're inconsistent with Add and Remove. Severity: nit / question.
7. **Card details panel footer has only Printings.** Add, Remove and both swipe selects, wide. The footer shows only "Printings", with no Image (and no Flip, though every card I saw was single-faced). The brief lists Flip/Image/Printings. Leaving out Image makes sense when the card is already on screen. Flag it only if that wasn't intended. Severity: nit.
8. **Hover preview covers the expanded row's text.** Deck page, Edgar Markov row expanded, wide, dark. The hover preview opens over the right end of the expanded row and hides the end of the oracle text ("…create a 1/1 black Vampire [creature token]"). Expected: no preview while the row is expanded, or a position clear of it. Severity: nit.
9. **Lowercase warnings.** Deck page Warnings: "deck has 6 cards, commander requires at least 100" and "deck has 0 lands, below the land target of 37" both start lowercase. The field errors now start with a capital, so these stand out. Severity: nit.
10. **Awkward "Deck name" error.** Create deck, Deck name, after typing and then clearing it: "Deck name minimum length is 1 character". It's capitalized, but it reads like a validator message. "Deck name is required" would read better. Severity: nit.
11. **Errors stay on cleared fields.** app.zwipe.net/register (logged out), wide, light. Type "a" in Username and "x" in Email, blur, then clear both. "Must be at least 3 characters long" and "Missing @ symbol" stay under the empty fields. Severity: nit.
12. **"Theme" pip toggle on zwipe.net deck pages.** zwipe.net/deck/…, wide. The nav pill "Theme" switches the mana pip style between Theme and Classic. Sitting next to Gruvbox and Dark, it reads like a second theme control. On a not-shared page (/deck/abc) it shows even though there are no cards. Expected: a label like "Pips", and hidden when there's nothing to show. Severity: nit.
13. **Version hint has no period.** app.zwipe.net/about, Version `?`: "…What changed in each release is listed below". It has no closing period. Severity: nit.
14. **P/T chip is faint in light.** Deck page card rows, wide, Gruvbox light. The P/T chip (for example "5/5") is light orange on peach and hard to read next to the price chip. Severity: nit.

## Looked fine

- **Nav:** `?` sits in the icon row just before Log out on Decks, Add, Remove, Playstyle, Commander maybeboard and the deck page. It's bold and evenly spaced. Home has no `?`. No page heads on Home, Decks, the deck form, the deck page, Add, Remove, Playstyle, Commander maybeboard or About.
- **Decks page:** two columns at 1450px. Hovering a deck lifts it (translateY −3px plus a shadow). "No commanders" on the Commander maybeboard is a centered gray chip.
- **Deck form:** Profile and Budget on the left, Tags on the right. Pickers open as right-side drawers with the "Search" label in accent aqua, a clear × once you type, and a list that scrolls to the drawer's bottom edge.
- **Hints:** the Format, Deck tags and Commander maybeboard hints show "See guide". It's a same-tab link to zwipe.net/guides. The filter's Oracle tags hint covers include, exclude, All and the Dictionary.
- **Card details panel:** title with mana pips, type and rarity chips, oracle text, set/price/P/T chips, and Keywords and Card roles as label-chip rows. "Other tags" expands the oracle tags in place. The panel scrolls on its own, and the eye slides it out and back. Same on Oracle tag Examples, inside the modal.
- **Card rows:** collapsed shows name, pips, price and P/T. Expanded shows the same chip layout, with Image and Printings side by side.
- **Printings:** the cover flow works. Arrow keys move it and Enter picks, with a "Printing saved" toast.
- **Swipe docks:** labeled arrow dock (Skip / Maybeboard / Undo / Add, and Keep / Move / Undo / Remove on Remove) next to the tool dock. Spaced-out swipes all land. Remove + Undo works ("Undid remove"). The Synergy bolt toasts "Synergy off" / "Synergy on". The card shadow isn't clipped.
- **Footers and scrollbars:** the footer sits below the fold on Add and the deck page, compact with pills. The app's scrollbars are thin and themed.
- **zwipe.net:** the shared deck page renders. The funnel opens the full card filter, hovering a card row shows a large preview, the footer is compact, and no link has `target`, so they all open in the same tab. Home, guides and the not-shared page work in dark and light.
- **Logged-out app pages:** login, register and forgot password look right in both themes. Enter closes the Password hint.

---

# Part 2: zwipe.net full pass + scottyfermo.com (same day)

Wide (1450px), Gruvbox dark and light. Both sites are back on Gruvbox dark. I made no writes.

## zwipe.net

Crawl: all 31 sitemap URLs return 200. There are no broken internal links or images, every link opens in the same tab, every page has one h1 and its own title, robots.txt is sane, and unknown paths return 404.

15. **Sitemap and canonical URLs all redirect.** All 31 sitemap URLs and their canonical tags have no trailing slash (`/guides`, `/about`, …), but the server redirects each one to the slashed form (`/guides/`). Google sees every canonical pointing at a redirect. Expected: pick one form and use it everywhere (sitemap, canonical, internal hrefs). Severity: nit (SEO).
16. **The home h1 is ASCII art.** zwipe.net/ and scottyfermo.com/. The page's only h1 is the block-glyph logo, so crawlers and screen readers read "▎▎▉▉▉…". Expected: an `aria-label` / visually hidden "Zwipe" (and "Scotty Fermo"), or a real h1 elsewhere with the art moved to a non-heading. Severity: ugly (SEO and accessibility).
17. **Layout jumps on short pages.** /community and /contribute (pages too short to scroll). With no scrollbar, the nav and content sit about 7px further right than on the other pages, so moving between pages jumps. Expected: `scrollbar-gutter: stable` on html. Severity: nit.
18. **Community grid looks unfinished.** /community. Two cards (Discord, Email) sit in a 3-column grid with an empty third column, left-aligned under a full-width header. Expected: two columns, or centered. Severity: nit.
19. **Empty space under testimonials.** Home, "Deck builder testimonials": about 100px of empty card below the quotes. Severity: nit.
20. **"Free" and "No ads" chips are faint in light.** Home hero, Gruvbox light. Peach on cream, hard to read. Severity: nit.
21. **/download/ios is in the sitemap but redirects.** It has a title and description but sends you straight to the App Store. Keeping it out of the sitemap is cleaner. Severity: nit.

Looked fine on zwipe.net: home (hero stats, demo carousel, feature cards, featured flavor, testimonials marquee), guides index and filter, a guide page (synergy), changelog, about (architecture diagram carousel, crate cards), community, contribute, privacy, the shared deck page (#2 and #5 above aside), and the compact footer everywhere, in both themes.

## scottyfermo.com

Crawl: all 22 sitemap URLs return 200. Internal links and images are fine, no link sets `target`, titles are unique, every page has one h1, and unknown paths return 404. Live stats load (the baked "as of" fallback shows first, then "Live").

22. **Same trailing-slash redirect as #15.** 21 of 22 sitemap URLs redirect to the slashed form. Severity: nit (SEO).
23. **Email link only works with JS.** Every page's raw HTML has Cloudflare's `/cdn-cgi/l/email-protection` link, which returns 404 on a plain fetch. With JS it becomes the correct mailto, but crawlers and no-JS readers get a dead link. Expected: either accept that, or turn off Cloudflare Email Obfuscation for the site. Severity: nit.
24. **Escape doesn't close nav dropdowns.** Home, Products ▾ (and the other dropdowns). Escape leaves the menu open, though clicking outside closes it. Severity: nit (keyboard accessibility).
25. **Projects intro miscounts.** /projects: "in four sections: what I've shipped, how I learn, and what I've built for work" names three things. The page has four sections (Products, Systems, Tooling, Experiments). Severity: nit.
26. **"Finished" next to "Doing".** /projects Products card: "Finished products" introduces Zwipe and Cairn, which are both badged "Doing" on their cards. Severity: nit.
27. **Tooling description is too long.** /projects/tooling's meta description is 166 characters, so Google will cut it off around 160. Severity: nit.

Looked fine on scottyfermo.com: home (hero, stats, commit chart, heatmap, project cards, How I Use AI, footer), the nav dropdowns, the projects index, the zwipe and cairn project pages (architecture carousel, demo), and light theme.

---

# Screenshots (~/Developer/surfer-shots on halo)

Each file name starts with the number of the item it shows.

- 02: `02-price-row-vs-details-and-08-stale-preview.jpg`: Edgar's collapsed row shows $112.50, but the expanded details have no USD chip. The app's header total is $187.19 (top of the deck page). `05-white-drawer-scrollbar-and-02-shared-total-74.jpg` shows the shared page's $74.69 for the same deck.
- 04: `04-picker-check-covers-chips.jpg` and `04-picker-check-covers-chips-zoom.png`: the ✓ sits on top of "Dice Rolling". I couldn't catch the ghost button behind the ✓ this time.
- 05: `05-white-drawer-scrollbar-…jpg`: the white scrollbar on the right edge of the filter drawer.
- 06: `06-swipe-screen-page-head-no-nav.jpg`: "Suggested commanders" title bar, with no nav.
- 07: `07-panel-footer-only-printings.jpg`: the details panel footer on Add has only Printings.
- 08: `02-…-08-stale-preview.jpg` and `08b-preview-stuck-after-scroll.jpg`: the preview covers the end of the expanded row's oracle text.
- 09: `09-lowercase-warnings.png`
- 10: `10-deck-name-error-wording.png`
- 12: `12-theme-pip-toggle-label.png`
- 13: `13-version-hint-no-period.png`
- 14: `14-pt-chip-light.png`: the P/T chips in light. Up close it's borderline; drop it if you think it's fine.
- 17: `17a-nav-on-guides.png` vs `17b-nav-shifted-on-community.png`: the same crop on both pages. The nav moves right when the scrollbar disappears.
- 18: `18-community-two-cards-in-three-columns.jpg`
- 20: `20-free-no-ads-faint-light.png`: also shows "Free" and "No ads" text sitting near the top of their chips, not centered (both themes). Logged as #30.
- 24: `24-dropdown-stays-open-after-escape.png`
- 25/26: `25-26-projects-copy.jpg`: "four sections" with three named, and "Finished products".

No screenshot:
- #1 (Dark Reader): on this pass Dark Reader injected nothing on app.zwipe.net, light or dark, so I couldn't reproduce it. It may be a per-site toggle in the extension. Treat #1 as unconfirmed.
- #3: a keyboard behavior, nothing to see.
- #11: needs a logged-out session, and I'm not logging out.
- #19: the testimonials marquee changes height as it scrolls; this time a long quote filled the card, so the empty space didn't show. Treat as unconfirmed.
- #15, #16, #21–23, #27: SEO and markup.

New while shooting:

28. **Hover preview gets stuck and stacks.** Deck page, wide, dark. I scrolled with the mouse resting over the card list. The large preview stayed on Charismatic Conqueror while the pointer was over other rows and over Edgar's expanded details (08b, 02 shots). Mid-scroll I also saw several previews stacked and cross-fading on top of each other (Clavileño over Drana over Charismatic) for a moment, but my screenshot was too late to catch it. Expected: the preview follows the row under the pointer, or hides on scroll. Severity: ugly.
29. **zwipe.net**: see #30.
30. **"Free" and "No ads" text isn't centered in its chips.** zwipe.net home hero, both themes. The text sits near the top of the chip, while "iOS ↗" and "Android ↗" next to them are centered. Severity: nit.

---

# 31. Theme drawer: clicking off it flashes the page dark (all three sites)

**Repro** (app.zwipe.net, zwipe.net, scottyfermo.com; wide, Gruvbox dark): open the theme drawer with the palette or Gruvbox button, wait for it to finish opening, then click the dimmed page off the panel. The page goes noticeably darker for an instant, then fades back over about 300ms. Severity: ugly.

**Measured** with a requestAnimationFrame sampler on `.sheet-backdrop` (identical on all three sites):
- **Open, settled:** class `sheet-backdrop show`, opacity 0.4, background rgba(0,0,0,0.6). The effective dim is 0.24.
- **First frame after the click:** class `sheet-backdrop`, opacity **1**, background still rgba(0,0,0,0.6). The effective dim jumps to **0.6**, 2.5× darker than when it was open.
- **The next ~300ms:** opacity stays 1 while the background alpha fades 0.6 → 0 (0.576, 0.533, 0.467 … 0).

**Cause** (orangui origin/main, `assets/components.css`; my local orangui checkout is behind, so the rule isn't in it):
- L1489: `.sheet-backdrop { background: transparent; transition: background 0.3s ease; }`. Only background is transitioned.
- L1498: `.sheet-backdrop.show { background: var(--bg-overlay); }`
- L1687, wide media query: `.sheet-form-drawer > .sheet-backdrop.show { opacity: 0.4; }`. Added in ad47337, "SheetFrame: the modal and drawer forms…".

When `show` drops, opacity snaps from 0.4 to 1 instantly, because opacity has no transition. The background is still at full overlay alpha, so the page flashes dark, and only then does the background fade.

**Fix options** (any one):
- **Move the 0.4 off `.show`:** put it on `.sheet-form-drawer > .sheet-backdrop` itself, so opacity never changes on close.
- **Put the drawer's dim in the color:** drop the opacity rule and give the drawer its own lighter overlay color (for example `color-mix(in srgb, var(--bg-overlay) 40%, transparent)`), so only the transitioned background changes.
- **Transition opacity too:** add `opacity 0.3s ease` to the backdrop's transition.

The first is the smallest change. It affects every `.sheet-form-drawer` (the theme picker, the deck form pickers, the filter drawer), so check those after the fix.

---

# 32. Oracle tags dictionary: entries hard-cut above the modal's bottom edge

Found 2026-11-06 at Scotty's request. app.zwipe.net, wide (1450px), Gruvbox dark, zwipe main as deployed that evening.

**Steps:**
1. Decks (now `/decks`), then open any deck (I used "Surfer test").
2. Click + in the deck dock to open Add.
3. Click the funnel to open the filter drawer.
4. Expand "Oracle tags".
5. Click "Dictionary" under "Oracle tags include". The Oracle tags modal opens.
6. Scroll the list a little.

**What you see:** the entry at the bottom is cut off along a hard horizontal line about 24px above the modal's rounded bottom edge, mid-line (for example, "Has an activated ability: an effect you turn on by paying a cost." loses the bottom of its letters), with an empty band below it. The top of the list fades out softly. See `32-dict-list-hard-cut-zoom.png` and `32-dict-list-hard-cut.jpg`.

**Measured:**
- The scroller is `.dict-list`: top 250, bottom 704, `overflow-y: auto`, `mask-image: linear-gradient(transparent 0, black 16px)`, so it fades only at the top.
- Its parent `.sheet-content` ends at 728 with `padding-bottom: 24px`, and `.bottom-sheet` ends at 729.
- Every entry stops at y=704, and the 24px of padding below is empty sheet.
- Scrolled all the way down, the last entry ends at 688, so nothing is unreachable. The problem is the clip line, not lost content.

**Likely fix:** let the list run to the sheet's bottom edge (drop or move the 24px bottom padding on `.sheet-content` for this modal, or give the list the padding instead), and/or add a matching bottom fade to the mask, e.g. `linear-gradient(transparent 0, black 16px, black calc(100% - 16px), transparent 100%)`.

---

# Part 3: recheck after deploy 25ff79ae0 (2026-11-06)

Hard-refreshed all three sites first. Wide only (1450px; narrow is still blocked by the window), Gruvbox dark, plus Gruvbox light where an item is about light. All three sites are back on Gruvbox dark. Writes stayed on the test account and nothing was saved. I cleared and retyped "Surfer test" in Edit deck without saving, and closed Printings without picking. Screenshots are `p3-*` in ~/Developer/surfer-shots.

## Recheck of 1–31

| # | Item | Now |
|---|---|---|
| 1 | Dark Reader repaints the app | **Fixed**. `meta darkreader-lock` is in the app; 0 Dark Reader styles |
| 2 | Price mismatch app vs shared | **Fixed**. Edgar's unpriced printing shows no price in the row; app total $74.69 = shared $74.69 (p3-2) |
| 3 | Enter on a picker drawer's `?` hint | **Fixed**, Enter closes it. **Changed**: focus then lands on the nav logo link, so a second Enter navigates Home off the form (see new #36) |
| 4 | Picker ✓ over chips, ghost dock | **Still there**: the ✓ covers chips mid-scroll ("Fly…" in Deck tags, p3-4/p3-4b). Ghost: **gone in the app**, **still there on zwipe.net**'s shared-deck filter (page funnel at 1384–1422 × 705–744 shows behind the ✓ dock, visible, opacity 1, p3-16) |
| 5 | White drawer scrollbar on zwipe.net | **Fixed**. Thin and themed (p3-15) |
| 6 | Swipe select page head, no nav | **Still there**. "Suggested commanders" title bar, no nav (p3-7) |
| 7 | Panel footer only Printings | Intended, unchanged |
| 8 | Preview covers expanded row | **Fixed**. No preview while a row is expanded (p3-2) |
| 9 | Lowercase warnings | **Fixed**. "Deck has 6 cards…", "Deck has 0 lands…" |
| 10 | Deck name error wording | **Fixed**. "Deck name is required" (p3-3) |
| 11 | Register errors stay on cleared fields | Not rechecked (needs a logged-out session) |
| 12 | "Theme" pip pill | **Fixed**. Reads "Pips" (p3-14) |
| 13 | Version hint period | **Fixed** |
| 14 | P/T faint in light | **Fixed**. P/T and price chips now match, both 4.29:1 on the page background (a hair under 4.5:1 AA for small text, same as the price chip) (p3-11) |
| 15 | zwipe.net sitemap/canonical redirects | **Fixed**. 29 sitemap URLs, 0 redirects, canonicals have the slash |
| 16 | ASCII-art h1 | **Fixed**. zwipe.net h1 reads "Zwipe, the Magic: The Gathering…"; scottyfermo.com h1 reads "Scotty Fermo" with the art aria-hidden |
| 17 | Layout jump on short pages | **Fixed**. `scrollbar-gutter: stable` |
| 18 | Community grid | **Fixed**. Two equal columns (p3-13) |
| 19 | Testimonials gap | Not reproduced before, not rechecked |
| 20 | Free/No ads faint in light | **Fixed**. Dark brown on cream (p3-17) |
| 21 | /download/ios in sitemap | **Fixed**. Gone |
| 22 | Portfolio trailing-slash redirects | **Fixed**. 22 URLs, 0 redirects |
| 23 | Cloudflare email-protection | Intended, unchanged |
| 24 | Escape doesn't close dropdowns | **Fixed**. Escape closes and keeps focus on the trigger |
| 25 | "four sections" with three named | **Fixed**. Names all four |
| 26 | "Finished" vs "Doing" | **Fixed**. Now "Shipped products" |
| 27 | Tooling description too long | **Fixed**. 155 chars |
| 28 | Stuck or stacked hover preview | **Fixed**. Frame sampler shows at most one `.hover-stack-card`, it follows the row, and it fades out (opacity 1 → 0 over ~280ms) when the pointer leaves (p3-1) |
| 30 | Free/No ads text not centered | **Fixed**. Text and chip midlines match (p3-12) |
| 31 | Drawer dark flash on close | **Fixed on all three sites**. Effective dim falls smoothly 0.24 → 0 over ~370ms, with no jump |

(29 was only a pointer to 30.)

## New items

33. **Swipe select details panel: chips hard-cut under the host section.** Playstyle → Swipe ("Suggested commanders"), wide, dark. The panel's middle `.panel-body` scrolls in a 259px box (y 191–450, scrollHeight 427). "Why this commander?" takes the rest. The set/price/P/T chip row is cut through the middle at the boundary with no fade ("Fallout | 1.20 TIX | 8/8" half visible, p3-8). On a 759px-tall window you see only the type chips and oracle text before having to scroll a small inner box. Expected: a fade at the cut (like the dictionary's top fade) and/or a taller body, e.g. letting the host section scroll with the rest. Severity: ugly.
34. **Docked details panel always starts open.** Add, Remove, swipe select and Examples all start with the panel open (p3-5, p3-7, p3-9). On Add I closed it with the eye and reloaded, and it came back open, so the choice isn't remembered. Fine if that's intended. Flagging so it's a decision. The card stays centered (x=719 of 1450 with the panel open or closed), and there's no lift on hover (transform none, no shadow change). Severity: question.
35. **Old `/deck` URL is a "Page not found".** `app.zwipe.net/deck` (the old Decks route) now shows the 404 page. The new route is `/decks`, and deck pages are `/decks/<id>`, `/decks/<id>/add`, `/remove`, `/edit`. Bookmarks and old links in posts or the changelog break. Expected: redirect the old `/deck…` paths to `/decks…`. Severity: nit (ugly if old links are out in the wild).
36. **After Enter closes a hint, focus jumps to the nav logo.** Edit deck → Deck tags drawer → `?` → Enter. The hint closes, but `document.activeElement` is the logo link, so a second Enter leaves the form for Home and unsaved edits are lost. Expected: focus returns to the `?` button (or stays in the drawer). Severity: ugly (keyboard).
37. **App overscroll color is one shade off.** app.zwipe.net: `html` is rgb(40,40,40) while `body` and the page are rgb(34,34,34), and `overscroll-behavior` is auto, so a trackpad rubber-band at the document edge can show a lighter strip. zwipe.net has the same colors but `overscroll-behavior: none`, and scottyfermo.com is `none` with transparent html/body and `color-scheme: dark`. Measured only; I couldn't trigger a rubber-band from the browser tools. Severity: nit, unconfirmed.

## Looked fine (new this round)

- **Docks:** stop above the footer on Add (the footer scrolls into view below both docks, p3-6). Labeled arrow dock and tool dock use SVG icons (11 SVGs in the docks).
- **Cover flow:** ← → arrow buttons move it, Previous disables at "1 of 8", the edge fade shows on the outer cards, and × closes it without picking (p3-10, p3-10b).
- **Examples:** panel docked inside the modal, card centered, Next/Back one dock (p3-9).
- **Expanded rows:** chip rows evenly spaced, Image and Printings side by side at the bottom (p3-2).
- **OG tags:** og:image on all three sites returns 200 image/png. app.zwipe.net `/assets/og-app.png` is 1200×630 and matches the declared width and height. zwipe.net and scottyfermo.com use `/assets/og-default.png`. The app also has full twitter:* tags.
- **Build:** About still says v1.18.0. The 1.18.1 changes are live but the label hasn't moved. Fine if the label waits for the store release.

---

# Part 4: half-width sweep (2026-11-06, deploy 25ff79ae0)

Window: **720 × 815 CSS px** (innerWidth 720, DPR 2; height later read 759). Hard-refreshed all three sites. Gruvbox dark throughout, plus Gruvbox light on the app's Cards view and on scottyfermo.com. All three sites are back on Gruvbox dark. Writes: none (nothing saved and nothing swiped). Screenshots are `p4-*` in ~/Developer/surfer-shots.

720px gets the **phone layout** on app.zwipe.net (title bar with back, page title and `?`, hamburger menu, Profile/Cards split, bottom sheets) and the **collapsed nav** on zwipe.net and scottyfermo.com. I left out the in-flight items (picker ✓ room, focus after a hint, swipe-screen nav and title chip, details panel scroll, the eye being remembered, the unsaved-changes guard, skeletons, "phone and web" copy, the phone details drawer).

## Parts 1–3 rechecked at 720px

- **Nav:** collapses cleanly on all three. The app's hamburger opens an overlay: four full-width nav buttons, then the icon row with Log out (p4-2). zwipe.net and the portfolio push the page down with full-width buttons, then theme and dark controls (p4-19, p4-25). The portfolio's dropdowns open inline as chips. Nothing overflows horizontally: scrollWidth ≤ innerWidth everywhere, and the testimonial marquee is clipped as intended.
- **Page heads:** at this width every app screen has a title bar (title + `?`), which is the phone design. The wide-only "no page heads" rule doesn't apply.
- **Drawers vs sheets:** every drawer becomes a bottom sheet (filter, pickers, theme, Printings, zwipe.net filter) (p4-11, p4-14, p4-16, p4-21). Closing fades smoothly with no dark flash (#31 holds: 0.6 → 0 over ~370ms, no jump).
- **Printings:** the phone picker with dots, prices and ✓, as specified (p4-9).
- **Swipe docks:** labeled arrow dock stacked over the tool dock, same width (p4-10). The card is centered, and the details panel starts closed (the phone drawer is in flight).
- **Deck form:** a single column, Profile → Budget → Tags, with nothing clipped (p4-13).
- **Card rows:** collapsed (name, pips, price, P/T) and expanded (chip rows, then Image / Printings / − 1 + / Star / To maybeboard / To sideboard) wrap cleanly onto two lines (p4-8). No hover preview at this width.
- **Charts:** Colors, Mana curve, Card roles and Oracle tags bars all fit (p4-6).
- **Theme and light:** the light theme reads fine on the app Cards view and the portfolio (p4-27). The light/dark choice survives a reload.
- **Footer:** present on zwipe.net and the portfolio. The app has no footer at this width, which I take to be the phone design.
- **#32 dictionary:** at this width the dictionary is a full-screen page, so the hard cut doesn't apply. See #39 for the column instead.

## New items at 720px

38. **Toast covers the header's back button.** app.zwipe.net Cards view, light theme. Toggling dark/light shows the "Dark mode off" toast at the top left, right over the title bar's ‹ back button, for several seconds (p4-17). At wide width toasts sit bottom-left. Expected: keep toasts clear of the title bar (bottom, or below the header). Severity: ugly.
39. **Dictionary and picker sheets use a phone-width column inside a 720px window.** The Oracle tags dictionary (`.dict-list` has `max-width: 384px`) and the Deck tags picker sheet (search, hint and chips about 320px wide) sit in a narrow centered column with about 170–200px empty on each side. The dictionary's A–Z row is then cut at "I" and has to scroll sideways (p4-12, p4-14). Other sheets (filter, theme) use the full width. Expected: let these fill the sheet up to a sensible max, e.g. the filter sheet's width. Severity: ugly.
40. **zwipe.net shared deck: the funnel sits on the footer.** At the bottom of `/deck/<token>`, the floating funnel covers the end of the footer's fan-content line ("…Fan Content Policy… property of", p4-20). The app's docks now stop above the footer; zwipe.net's funnel doesn't. Severity: ugly.
41. **Commander group has no collapse caret on the narrow Cards view.** "Commander" has no ▾ while "Creatures (5)" does (p4-7). On wide both have it. Severity: nit.
42. **Types chart with one type is a big solid block.** Distributions → Types with only creatures: one bar fills the whole chart width, about 650×85px, and reads like an empty placeholder (p4-5). Expected: cap the bar width, or show a single-row bar like Card roles. Severity: nit.
43. **The portfolio commit chart loses its axis labels.** At 720px the line chart and heatmap sit in a horizontal scroller (`.chart-scroll`, 707 wide in 611) that starts scrolled to the end. The y-axis values (2,500 / 1,250 / 0) and the Mon/Wed/Fri row labels scroll out of view, and "Dec" is cut to "ec" (p4-24). Expected: keep the axis and row labels pinned (sticky) while the plot scrolls. Severity: ugly.
44. **zwipe.net About: the architecture diagram scrolls sideways inside its card.** At 720px the diagram is cut through the right-hand boxes ("zite", "components", "orangui") and needs a horizontal scroll, with a visible scrollbar under it and the ← → carousel arrows sitting on top of the diagram's edges (p4-23). Expected: scale the diagram down to fit, or wrap it. Severity: nit.
45. **Escape doesn't close the portfolio's mobile menu.** The wide dropdowns now close on Escape (#24), but the hamburger panel stays open after Escape. Severity: nit.
46. **Swipe select docks are a different width from Add's.** At 720px, swipe select's stacked docks are 335px wide with a 10px right margin. Add's are 374px with a 16px margin (p4-15 vs p4-10). Severity: nit.

## Looked fine at 720px

- **App:** Home (count-up stats, deck tiles), Decks (single column, filters wrap), deck Profile (rows, Budget, collapsible sections, floating "Cards" button), Cards view, Playstyle, the Suggested commanders swipe, the theme sheet.
- **zwipe.net:** home (hero, chips centered, demo), guides (two-column cards, filter chips wrap), the shared deck (header chips, card rows, footer).
- **scottyfermo.com:** home (hero, stats, live chip swaps in), projects (single column), footer, light theme.

---

# Part 5: narrowest pass (2026-11-06, deploy 25ff79ae0)

Window: **504 × 815 CSS px, DPR 2** (innerWidth = outerWidth = 504). That's the narrowest this Chrome goes on halo, so it's wider than a real phone (375–430). Hard-refreshed all three sites. Gruvbox dark throughout, plus Gruvbox light on app Settings and zwipe.net home. All three sites are back on Gruvbox dark. Writes: none (no swipes, nothing saved). Screenshots are `p5-*` in ~/Developer/surfer-shots.

On every page I visited, a script checked sideways overflow (any element past the viewport edge, ignoring intentional scrollers), tap targets under 44px, and clipped text. I skipped everything in the known/in-flight list (board 116–138 and Part 4's #38–46).

## Parts 1–4 at 504px

- **No sideways overflow anywhere.** scrollWidth ≤ innerWidth on every page of all three sites (app: Home, menu, Decks, deck Profile, Cards, expanded row, Add, filter sheet, dictionary, Edit deck, Format sheet, Activity, Account, Settings; zwipe.net: home, menu, guides, a guide, changelog, community, contribute, privacy, about, shared deck; scottyfermo.com: home, projects, a project page). The only clipped text is intentional ellipsis (deck tile names on Home).
- **Nav menus fit.** The app's hamburger icon row (7 icons + Log out) fits on one line, and zwipe.net and the portfolio stack full-width buttons (p5-2, p5-16).
- **Docks fit.** On Add the arrow dock drops its labels to icons only and the two docks stack at 193px, right-aligned, with the pile chips bottom-left (p5-6).
- **Chip rows wrap cleanly.** Group by / Boards / Show on Cards; card-row price and P/T wrap under long names; deck tags wrap under titles on Decks (p5-3, p5-5).
- **Deck form:** one column, nothing clipped (p5-8).
- **Sheets:** filter, Format and theme cover the page as bottom sheets with × and ✓ reachable. Their narrow inner column is known (#39).
- **Charts:** Activity Totals bars and the Playstyle/Home Colors bars fit.
- **Light theme:** reads fine on app Settings and zwipe.net home (p5-13, p5-20).

## New items at 504px

47. **Most tap targets are under 44px.** Measured on visible controls. App title bar: back 34×27, hamburger 34×27, `?` 24×24. Menu icon row: 29×27 each. Every chip button is 27px tall (Group by, Boards, Show, filter pills, Budget currency). Swipe docks are 38×38. In the expanded card row: Image 62×22, Printings 95×22, and **− and + at about 30×22 sitting side by side**, where a mis-tap on − removes a copy. In the filter: Dictionary 85×22 and the × clears 25×22. zwipe.net and the portfolio: store, social and theme icon buttons 27×27, hamburger 34×27, and the "1.18.0 is out" toast close **✕ is 10×13** (p5-21). Expected: at least 44×44 hit areas on touch, by padding or a pseudo-element hit box if the visual size should stay. Severity: ugly (the −/+ pair and the toast ✕ are the worst).
48. **App Activity heatmap: white horizontal scrollbar.** The heatmap scroller (`.chart-scroll.scroll-fade-x`) has `scrollbar-width: thin` but `scrollbar-color: auto`, so in dark it draws a bright white track and thumb across the card, and the left scroll-fade gradient shows against the white (p5-9, p5-10). Expected: themed `scrollbar-color` like the other scrollers. The Mon/Wed/Fri labels are also scrolled out of view, the same as the portfolio chart in flight. Severity: ugly.
49. **zwipe.net two-column grids stay two columns at 504px.** Community (Discord / Email) and About's crate cards stay two-up, so each column is about 210px. The text wraps every two or three words, "Join the Discord ↗" breaks onto two lines with the ↗ floating right, and the About cards are long and narrow (p5-17, p5-19). Expected: one column below about 560px. Severity: ugly.
50. **Settings "Pips" button still says "Theme".** App Settings → Appearance → Pips. The button reads "Theme", and this is right under the real Theme row with Gruvbox / Dark / Change (p5-12). zwipe.net's pill now says "Pips" (#12). Expected: the same label here (e.g. the current pip style, "Theme"/"Classic", with clearer wording, or "Pips: Theme"). Severity: nit.
51. **Light/dark can flip back a few seconds after a page load (intermittent).** Twice tonight the app ended up in the other theme. In Part 4 I toggled light, went to other sites, and came back to dark. This pass, I toggled light from Settings, navigated to Cards, and it rendered dark (p5-14). Instrumented runs: one Settings toggle didn't update `zwiper-web.theme` in localStorage, and after a reload the page rendered dark, then switched to light about 3s later when the server copy synced down. Other runs wrote localStorage at once (and sent `PATCH /api/user/settings`) and held across reloads. So when local and server disagree, the page shows the local theme first and flips when the account settings arrive, and occasionally the local write seems to be missed. Expected: apply the server value before first paint, or keep local and server in step on every toggle. Severity: ugly, intermittent.

## Looked fine at 504px

- **App:** Home (stats fit three-up, deck tiles with ellipsis), Decks (one column, tag chips wrap), deck Profile (rows, Budget, Buy links), Cards (rows wrap price and P/T under long names; the expanded host row wraps to two lines and clears the dock at the end of the list), Add, Activity Totals, Account (Email row with Verified + Change fits), Settings.
- **zwipe.net:** home (stats three-up, Free/No ads/iOS/Android on one line), guides (two-column cards hold), changelog, shared deck (rows wrap, collapsible sections), menu, light theme.
- **scottyfermo.com:** home (stats three-up, "as of" chip wraps to its own line, project cards and stat chips wrap), zwipe project page.

---

# Part 6: fix check, deploy 811c3bbbe (2026-11-06)

Hard-refreshed all three sites. Gruvbox dark; the portfolio briefly switched to Darcula for #125 and back to Gruvbox. Writes: none. Screenshots are `p6-*`.

The deploy logged the app out (`zwiper-web.session` was gone after the hard refresh, p6-1). Scotty signed back in before the app checks.

## Width 1: 1450 × 815 (wide)

| # | Check | Result |
|---|---|---|
| 115 (S32) | Dictionary list to the sheet edge, bottom fade | **Fixed**. `.dict-list` bottom 781 = `.sheet-content` 781 (sheet 782), padding-bottom 0, mask fades 16px at both ends (p6-7) |
| 116 (S6) | Both swipe selects: app nav, `?` in the icon row, no title bar | **Fixed** on Playstyle → Swipe and Edit deck → Commander → Swipe (p6-8, p6-9) |
| 117 | Quiet title chip top-left on wide swipe screens | **Fixed**: "Add", "Remove", "Suggested commanders", "Swipe select" (p6-6, p6-8, p6-9) |
| 118 (S33) | Swipe select panel scrolls as one, soft fades, no 259px box | **Fixed**. One `.panel-body` scroller (y 202–651, 449px), and "Why this commander?" is inside it under a soft fade (p6-8). On Commander swipe select the panel fits without scrolling |
| 119 (S34) | Eye state survives a reload | **Fixed**. Closed → reload → closed; open → reload → open (Add) |
| 120 (S4) | Picker ✓ never covers a chip; no ghost funnel on zwipe.net | **Fixed**. The app's Deck tags list stops above a reserved band, so the ✓ (1360–1398 × 738–776) sits clear of every visible chip (p6-11). On zwipe.net the page funnel is `visibility: hidden` while the filter drawer is open (p6-3). **Note:** the chip list ends in a hard cut at the band, with no fade like the dictionary's (see #52) |
| 121 (S36) | After Enter closes a hint, focus stays put and a second Enter does nothing | **Fixed**. In the Deck tags drawer: Enter closes the `?` hint, focus stays in the drawer, and a second Enter doesn't reopen it or navigate (p6-10) |
| 124 | Portfolio © in the footer button row | **Fixed**. `.footer-row` = "© 2026 Scotty Fermo · GitHub · X · LinkedIn · Email · Contribute" (p6-4) |
| 125 | Portfolio theme change is one left-to-right sweep, drawer included | **Not landed**. The page and the drawer still wipe as two separate sweeps that start together: `::view-transition-new(root)` and `(site-nav)` run `theme-wipe` (400ms) while `::view-transition-new(theme-sheet)` runs its own `drawer-wipe` (400ms, no delay). Mid-transition the page wipe is at about x=150 while the drawer's has already started at its own left edge, x≈1065 (p6-5). For one sweep, the drawer has to wait until the page line reaches its left edge (~72% through) or share the root's clip geometry |
| 126 | Enter on a deck form field's `?` doesn't submit or leave | **Fixed**. Deck name `?`: Enter closes the hint, focus stays in the Profile panel, a second Enter does nothing, no submit and no pushState |
| 127 | zwipe.net "for phone and web" | **Fixed**. Tab title "Magic: The Gathering Deck Builder for Phone and Web \| Zwipe", meta/OG/Twitter descriptions and the hero all say "for phone and web" (p6-2) |
| 2 | Price consistency | **Holds**. Edgar's unpriced printing shows no price; app and shared totals both $74.69 |
| 3 | Enter on a picker hint | **Holds** (see 121) |
| 31 | No dark flash on drawer close | **Holds** on all three sites. Dim falls 0.24 → 0 over ~370ms |
| 32 | Dictionary hard cut | **Holds fixed** (see 115) |

## New at 1450

52. **Picker chip list hard-cuts at the ✓ band.** Deck tags (and the other pickers). With the ✓ now in its own band, the chip list stops at about y=715 with the next row sliced through mid-chip ("Group Hug…", p6-11), with no fade. The dictionary got a 16px bottom fade in 115, so this reads as the one place left with a hard edge. Severity: nit.
53. **zwipe.net stats are stale.** zwipe.net's hero shows 2,268 decks / 31.8k searches / 264.6k swiped, the same numbers it has shown all evening, while app.zwipe.net's Home shows 3,125 / 43.8k / 364.5k. Both say "Live". Expected: the same live counts on both (or label zwipe.net's as cached). Severity: ugly (public-facing).
54. **The deploy logged users out.** After this deploy, `zwiper-web.session` was gone and app.zwipe.net opened on Login (p6-1). If sessions are meant to survive deploys, this is a bug (could also be a refresh token that expired overnight). Severity: question.

## Width 2: 720 × 759 (half)

(My first two width reads after Scotty narrowed the window still said 1450. They caught it before the resize settled. The page then reported 720 × 759, DPR 2.)

| # | Check | Result |
|---|---|---|
| 115 / 32 | Dictionary to the edge, bottom fade | **Fixed**. At this width the dictionary is full-screen, and `.dict-list` runs to the viewport bottom (759) with the 16px top and bottom fade (p6-13). The ~384px column is the known #39 |
| 116 / 117 | Nav and title chip on swipe screens | Wide-only by spec. At 720 the swipe screens use the phone title bar ("Add ?", "Suggested commanders ?"), as before |
| 118 (S33) | Swipe select details scroll as one | **Fixed on wide, still there in the phone drawer.** At 720 the details open as a right-hand drawer (new this deploy). In swipe select its `.panel-body` is again a **261px inner box** (y 180–441, scrollHeight 427, `mask-image: none`), with "Why this commander?" pinned below. The set/price/P/T row is cut through the middle with no fade ("Fallout · 1.20 TIX · 8/8", p6-15, p6-16). The wide fix needs to apply to the drawer form too |
| 119 (S34) | Eye remembered | At 720 the eye opens the details drawer (p6-12). After a reload the drawer starts closed, which suits a modal drawer. The wide panel's remembered state isn't affected |
| 120 (S4) | Picker ✓ clear of chips | **Fixed**. In the Deck tags sheet the chips stop above the ✓ band (p6-14). Same hard cut as #52 |
| 121 (S36) | Enter after a hint | **Fixed**. In the Deck tags sheet, Enter closes the hint, focus stays in the sheet, and a second Enter does nothing |
| 126 | Enter on a form field's `?` | **Fixed**. Focus stays in the Profile panel, no submit, no navigation |
| 31 | No dark flash | **Holds**. Sheet dim 0.6 → 0 over ~360ms, no jump |
| 2 | Prices | **Holds** (same deck data) |

New at 720: none beyond the #118 drawer regression above. The phone details drawer (in flight earlier) now works on Add: × at top-left, the card details in a panel, and Printings as a floating button at bottom-right.

## Width 3: 504 × 759 (narrowest on halo, DPR 2)

| # | Check | Result |
|---|---|---|
| 115 / 32 | Dictionary to the edge, bottom fade | **Fixed**. `.dict-list` runs to 759 (viewport bottom) with the top and bottom fade (p6-19) |
| 118 (S33) | Swipe select details scroll as one | **Still there in the phone drawer**, same as at 720: `.panel-body` is a 261px box (180–441, scrollHeight 427, no mask), "Why this commander?" is pinned below, and the chip row is cut (p6-17) |
| 119 | Eye | The eye opens the details drawer (full-height, from x=88). It starts closed after a reload, as at 720 |
| 120 (S4) | Picker ✓ clear of chips | **Fixed**. The Deck tags sheet's chips stop above the ✓ band (p6-18). The #52 hard cut is still there |
| 121 (S36) | Enter after a hint | **Fixed**. Focus stays in the sheet, and a second Enter does nothing |
| 124 | Portfolio © in the footer row | **Fixed**. "© 2026 Scotty Fermo" + four icons + Contribute fit on one row at 504 (p6-20) |
| 126 | Enter on a form field's `?` | **Fixed**. No submit, no navigation |
| 31 | No dark flash | **Holds**. Sheet dim 0.6 → 0, smooth |

New at 504: none.

## Part 6 summary

- **Fixed across widths:** 115/32, 116, 117, 119, 120, 121, 124, 126, 127. **Holds:** 2, 3, 31.
- **Not landed:** 125 (the portfolio theme change is still two sweeps).
- **Regression:** 118 is fixed in the wide docked panel but **still there in the new phone details drawer** at 720 and 504 (swipe select's 261px inner box with no fade).
- **New:** #52 (picker chip list hard-cuts at the ✓ band, nit), #53 (zwipe.net stats stale, ugly), #54 (deploy logged the app out, question).

---

# Part 7: fix check, deploy 60d705459 (2026-11-06/07)

**Width: 1450 × 759 only** (DPR 2, `pointer: fine`). Scotty is in bed, and `resize_window` is refused ("Bounds must be at least 50% within visible screen space"), so every narrow-only item is untested: 128, 129, 130, 133's full-screen branch, 137 (stacked docks are narrow-only), 142 (504px). 140 needs a coarse pointer, and this desktop Chrome reports `pointer: fine`, so it doesn't apply here. Hard-refreshed the app and zwipe.net. Gruvbox dark, with a Darcula pick and back, and a dark → light → (left light) toggle (see the end). Writes: none saved. The "Surfer test x" rename was discarded with Leave both times, and the deck is still "Surfer test". Screenshots are `p7-*`.

**Blocker partway through: the app logged out.** I opened a second app tab (for 144's cross-tab check), and that tab landed on /login. The first tab was then logged out too: `zwiper-web.session` was gone from localStorage in both. See #55. Everything after that point that needs a session is untested: 141, 147 (panel titles, warnings, contrast), 132, 118 wide recheck, and 144's cross-tab half.

## Results at 1450

| # | Check | Result |
|---|---|---|
| 122 | Leave-without-saving guard | **Fixed.** No changes → nav leaves with no dialog. After renaming to "Surfer test x": Decks nav, logo, and the in-app ‹ back each show "Leave without saving? Your changes to this deck will be lost. Stay / Leave" (p7-1). Browser back (in-app history, `history.back()`) with the Deck tags drawer open shows the dialog **above the drawer** (p7-2). **Stay** keeps the drawer open, the edited name, the URL (/edit) and the title (p7-3). **Leave** discards (the name stayed "Surfer test"). **Changed:** with a drawer open, clicking a nav link only closes the drawer (backdrop click) and doesn't navigate or show the dialog. A second click then shows it. Fine if intended, but the dialog-above-drawer case only happens through browser back. Also, during the browser-back guard the tab title briefly switches to the destination's ("Surfer test \| Zwipe") before Stay puts back "Edit deck \| Zwipe" |
| 133 | Format, Deck tags, Oracle tags are side drawers on wide | **Fixed at 1450.** Oracle tags opens as `bottom-sheet show drawer-every-width web-picker-sheet` docked right (x 1050–1434) (p7-4), and Deck tags is a side drawer too. The dictionary's A–Z row is one row (27 letters at y=207) with `overflow-x: auto` (p7-5). The narrow full-screen branch is untested |
| 150 | Theme sheet over Home is opaque | **Fixed.** `.bottom-sheet` background rgb(40,40,40), opacity 1, nothing showing through (p7-6) |
| 152 | No toast on a theme pick or the light/dark toggle | **Fixed.** A MutationObserver saw no toast nodes for Darcula → Gruvbox or the dark → light toggle |
| 144 | Toggle, reload, no flip; cross-tab | **Half checked.** Toggled dark → light, then reloaded: sampled every 250ms for 5s, light the whole time, no flip. localStorage now stores `{name, is_dark, server:{…}}`. **Cross-tab untested** because opening the second tab logged the account out (#55) |
| 147 | Hero chip order | **Fixed on zwipe.net**: "Live, Everybody", the same order as the app (p7-7). Panel title sizes, warnings rows and contrast are untested (logged out) |
| 127 | (spot check) | Still "for phone and web" in the zwipe.net tab title and hero |
| 132, 141, 118 | Toasts bottom-left, Activity heatmap, wide panel | **Untested** (logged out) |
| 128, 129, 130, 137, 142 | Narrow-only | **Untested** (window stuck at 1450) |
| 140 | Coarse-pointer tap areas | Doesn't apply (`pointer: fine` in desktop Chrome) |

## New

55. **Opening a second app tab logs the account out (both tabs).** Steps: signed in at app.zwipe.net in tab 1 (Home, working), open a new tab and load app.zwipe.net/settings. Tab 2 lands on /login, and tab 1 is then logged out on its next navigation (/decks → /login). `zwiper-web.session` is gone from localStorage in both tabs. My guess: both tabs refresh the stored refresh token at about the same time, the server treats the second use as a replay, and the "any replay kills the family" rule (from the session refresh fix) revokes the whole family. That would also explain #54 (logged out after a deploy reload). Expected: tabs share one refresh (a cross-tab lock, or BroadcastChannel/storage-event coordination, or a short reuse window for the same client), so a second tab never logs you out. Severity: **broken** (anyone who opens the app in two tabs gets signed out).
56. **#53 update: the counters may not be stale, there may be two.** Signed in, the app's Home shows 2,269 / 31.8k / 264.6k (same as zwipe.net). The 3,125 / 43.8k / 364.5k I saw in Part 6 was on the **logged-out Login page**. So the login page's counters come from a different source (or include something the others don't) than signed-in Home and zwipe.net. Severity: question.

Theme state left: the app was left on **Gruvbox light** (I toggled to light for the 144 reload check, then got logged out before switching back). It's stored server-side, so Scotty's next sign-in will show light. Needs flipping back to dark.

## Part 7, after Scotty signed back in (1450 × 759, one tab only)

| # | Check | Result |
|---|---|---|
| 141 | Activity heatmap themed scrollbar, Mon/Wed/Fri pinned | **Fixed at wide.** `.chart-scroll.activity-scroll` has `scrollbar-width: thin; scrollbar-color: rgb(60,56,54) transparent`. At 1450 the full year fits, so there's no scroll, and Mon/Wed/Fri show at the left (p7-8). Pinning while it scrolls is narrow-only, untested |
| 147 | Panel titles, pink on card-row panels, warnings rows | **Fixed at wide.** Panel titles are all 16px (Activity, Card roles, Oracle tags, Warnings, Data, Danger zone). "Most skipped", a card-row panel, keeps the pink (rgb 211,134,155) while the others are aqua (p7-9). The deck page's Warnings rows read cleanly: capitalized, Add cards / Import buttons aligned (p7-10) |
| 144 | Theme state | Found already back on **dark** at sign-in (server and local agree), so the light left from earlier was reverted |
| 132, 118 wide | Toasts bottom-left; wide panel | **Untested**: logged out again before I got to them |

**#55 revised: the session drops without a second tab.** After Scotty signed in, I used one tab only: Settings → Activity → deck page → Add, each a full page load (`navigate`), over about 1.5 minutes. On the load of /add the app landed on /login, and `zwiper-web.session` was gone again (07:44:56Z). So the second tab wasn't the cause, or at least not the only one. A theory that fits all three logouts (deploy reload, second tab, plain navigations): when a page load triggers a token refresh and the page is unloaded (a new full navigation) before the response is stored, the server has rotated the refresh token, but the client still holds the old one. The next load sends the old token, it's seen as a replay, and the family is revoked. That would hit real users on reloads, back/forward to the site, or link clicks during a refresh. Worth checking zerver's logs for replay-revocations from the test account between 07:40 and 07:45Z. Severity: **broken**.

## Part 7, last checks before standing by (1450, signed in)

- **132 (toasts bottom-left, clear of docks): holds at wide.** "Synergy off" toast at x 16–178, y 518–579, above the From/Search pile chips (y 606–633) and well left of the docks (x 848–1423) (p7-11).
- **Unconfirmed: page-wide highlight after the Synergy toggle.** After clicking the Synergy bolt on Add, all page text (details panel, docks, footer, nav Log out) was painted as if selected, yet `getSelection()` was an empty Range. Clicking elsewhere didn't clear it; a reload did. I didn't get to retry before standing by, so it may be a one-off paint artifact. Worth a quick repro: on Add, toggle the bolt and look for selection highlight across the page (p7-11).
- 118 wide recheck and 144 cross-tab: not done (stood down until the session fix, 156, ships).

## Part 8, narrow pass (720 × 725, live main 60d705459, zwiper-web v1.18.0)

Width: 720 × 725 viewport. The old tab still read 1450 (a different window), so I closed it and worked in a fresh tab at 720.

Logouts: **none this pass.** I was signed in from 00:00 to 00:08 across five full page loads (Add, reload after crash ×2, Add again) plus one zwipe.net round trip, and `zwiper-web.session` never dropped. I didn't need the credentials file and didn't open it.

| # | Check | Result at 720 |
|---|---|---|
| 122 | Unsaved-changes guard | **Holds.** Picked a deck tag, confirmed the picker, pressed back: "Leave without saving?" with Stay / Leave (p8-10). Leave discarded it; the deck still has only Tribal. See the note on picker × below |
| 128, 129 | Card details float in from the right, inset, rounded, Printings pill | **Fixed.** Drawer at x 294–710, y 54–598 (10px right inset), 16px radius, Printings as a text pill at the bottom (p8-4) |
| 130 | Printings is the cover flow | **Fixed** (p8-5). New issue below: at 725 tall the sheet doesn't fit |
| 132 | Toasts bottom-left, clear of docks | **Holds.** "Synergy off" at x 16–178, y 540–601; docks start at x 328, From/Search chips at y 691 (p8-20). No page-wide highlight this time (`getSelection` empty, nothing painted), so the Part 7 highlight looks like a one-off |
| 133 | Format / Deck tags / Oracle tags: side drawer 600–900 | **Holds for Format and Deck tags.** Both open as the right side drawer (x 294–710) (p8-7, p8-8). The Oracle tags *dictionary* (from the filter) opens full-screen at 720, list below, and its A–Z row scrolls (1009px in 688, `overflow-x: auto`) (p8-11). If the dictionary is meant to be a side drawer at 720 too, it isn't |
| 137 | Stacked docks one width | **Fixed.** Both docks x 328–704, 376 wide (p8-3). Same on Swipe select |
| 141 | Activity heatmap themed scrollbar, days pinned | **Fixed.** Scrolls 737 in 670, `scrollbar-width: thin`, themed color; Mon/Wed/Fri stay pinned while months fade under them (p8-14, p8-15) |
| 142 | zwipe.net Community and About two columns at 720 | **Fixed.** Community: Discord and Email side by side. About: both crate card grids are two columns (x 24 and 363) (p8-1, p8-2). Couldn't check the 560 breakpoint (no resize) |
| 147 narrow | Panel titles, warnings rows, hint dialog width | **Holds.** Deck page titles all 16px. Warnings rows read cleanly with Add cards / Import aligned (p8-12). Hint dialog 448px wide, centered, long body scrolls under a fade (p8-13) |
| 150 | Theme sheet opaque | **Holds.** rgb(40,40,40), opacity 1 over Home (p8-16) |
| 152 | No toast on theme pick | **Holds.** Darcula → Gruvbox, no toast; theme restored to Gruvbox |
| 118 | Swipe select details scroll as one | **Still there (known, 159).** Commander swipe select, Fire Lord Azula: `.panel-body` is a 320px inner box (y 190–510) holding 388px, no mask, bottom cut with no fade (p8-17) |

### New

1. **Crash: browser back from Swipe select kills the app.** On Edit deck, tap Commander › Swipe, then press browser back (or `history.back()`). The URL goes to the deck page but the view stays on loading placeholders with "Edit deck" still in the header and tab title. The app is dead from then on: further back/forward changes the URL but nothing renders until a reload. Console shows two `RuntimeError: unreachable` traps from the wasm (func 4019, via a wasm_bindgen closure), no panic text. Reproduced twice (00:07:26 and 00:07:54). Plain back from Edit deck *without* Swipe select open is fine. Untested at wide, so it may not be narrow-only (p8-18, p8-19).
2. **Printings sheet doesn't fit at 725 tall.** The sheet's content is 657px in a 542px box: the centered printing's bottom runs past the viewport on open, and the price chips and the ✓ select button sit 115px below the fold, so you have to scroll inside the sheet to pick a printing (p8-6).
3. **Picker × discards without asking.** In the Deck tags picker the form behind updates live as you tap tags, but × throws the picks away with only a "Deck tags unchanged" toast. Probably by design (× = cancel, ✓ = keep), but the live preview makes it look already applied (p8-9). Also, the hint box kept the last tapped tag's text (Aggro) after reopening.
4. **Toast covers the field it describes.** After confirming a deck tag, "Oracle tags derived from your deck tags" sits bottom-left directly over the Oracle tags chip row it's talking about (720 × 725, Tags section scrolled into view). Small, since it times out.

Not done: the 133 full-screen branch below 600 and 142 below 560 (window can't be resized), 144 cross-tab.

## Part 9, release 48e5f81b9 (2026-11-07, 08:13–08:31)

The window is 1450 × 815 and still can't be resized by the tool. For phone widths: the app refuses to load in a frame (correct, its framing protection), so I opened it in a popup sized from script (390 × 777, and 720 × 725 for Printings). I can't screenshot that popup, so the app results at 390 are **DOM measurements only, no shots**. zwipe.net and the portfolio allow same-origin frames, so they have screenshots at 375, 390 and 430.

### 1. Rechecks

| Item | Result |
|---|---|
| S125 portfolio single sweep (wide) | **Fixed.** The theme drawer's layer now uses a clip edge that runs from viewport x 0 to 1439 over the same 400ms curve as the page (0 to 1450), so the page and drawer wipe as one edge. They differ by at most about 8px where they overlap |
| Printings at 720 × 725 | **Fixed.** The sheet content is 518/518, nothing to scroll. Save at y 657–695, prices end at y 690, inside the viewport |
| Browser-back crash (161) | **Fixed.** Back from Swipe select at 1450 and at 390: no freeze, no console errors. At 1450 back goes from Swipe select straight to the deck page (it skips Edit), at 390 it went to the previous page. Fine, just noting it |
| Escape closes dropdowns | **App: fixed** (theme sheet, filter, card details, Printings, Format/Deck tags pickers, nav menu at 390). **Portfolio: fixed** (Products dropdown at wide, theme panel, phone menu at 375). **zwipe.net: not fixed.** Escape leaves the theme panel open (`aria-expanded` stays true). Its bundle was `zite-dxh762c74b1eefc6afb.js` all morning, so the zwipe.net deploy may not have landed yet |
| Live counter labels | zwipe.net's chip now reads "as of 08:20" with "Everybody" (it said "Live" at 08:15). On the portfolio, "Counted by Heron · Cached in Steller · Live / as of 2026-11-07" is consistent. The numbers scramble for about 1.1s on load, then settle. **Odd:** the portfolio's live Commits value was 7,711 on two loads and 7,632 on two others, while "Live" both times |
| Scrollbars | App and zwipe.net: page and panels use `scrollbar-color` from the theme (thin). Heatmap scrollbar themed at 720 and 390. **Portfolio's root page scrollbar isn't themed**: `html` is thin but `scrollbar-color: auto`, and only `.theme-wrapper *` gets the theme color (p9-2). Minor |
| Chip text contrast across themes | **Dark: mostly fine.** I audited every bordered chip or button on the deck page across all 50 themes. Worst: muted chips (Maybe, Side, the "?" buttons) at 2.6 on Material and 2.7 on Bluloco, red danger buttons at 3.05 on Nord, 3.25 on Solarized, 3.61 on Rustbox. **Light: still weak.** Most light themes have chips at 2–3:1. Worst: "Add cards" / "Import" in Warnings at **1.35 on Nord light** (pale yellow on light gray, p9-1), 1.87 Ayu, 1.91 Everforest, 2.04 Kanagawa. Power/toughness chips (4/4) at 2.2–2.8 on about 20 light themes, set and price chips at 2.5–2.9, rarity chips at 2.7. Theme restored to Gruvbox dark |

### 2. Session (156)

**No logouts.** Signed in 08:13–08:31 with two app tabs open the whole time, plus a third app window for about 10 minutes. About 16 full page loads, including both tabs reloading together. `zwiper-web.session` never dropped in any of them. Before, a second tab logged both out at once, so this looks fixed.

### 3. Phone widths

**App at 390 × 777 (DOM only).** No page overflows sideways: Home, Decks, Playstyle, Activity, Maybeboard, deck page, Edit, Add all have scrollWidth 390. Docks are icon-only at x 180–380. Card details drawer at x 38–380, with a fade on its panel body. Theme sheet full width and opaque. Heatmap days stay pinned. Filter sheet full width, scrolls (586/963).
- **New: Format and Deck tags pickers at 390 are cut off on the left.** The picker drawer is `width: 390px` (the whole viewport) but keeps its 9.6px right inset, so it sits at x −9.6 to 380. The left border and corner are off-screen, and the content has a 6px left gutter against about 26 on the right. That's also not the full-screen selector 133 describes below 600.
- **118 changed:** in Swipe select the details panel body is still an inner scroller (372px holding 441) but now has a fade mask, so it no longer looks cut off.
- Tap targets: header icons 34 × 27, the "?" buttons 24 × 24, filter chips 27 tall. Small but consistent everywhere, so I'm flagging it once, not per page.

**zwipe.net at 390 and 430.** Home, Guides (8 guide pages), About, Changelog, Community, Privacy, Contribute: no sideways overflow, and Community and About are one column (p9-3, p9-4). The "1.18.0 is out" toast's ✕ is 13px tall, a small target.

**Portfolio at 375 and 430.** Home and 13 of 16 project pages are fine (p9-5).
- **New: long URLs overflow at 375.** `/projects/upsee/` (page width 452), `/projects/sharpmas/` (404) and `/projects/rustmas/` (394) scroll sideways because bare URLs in the text (`https://huggingface.co/qualcomm/Movenet`, `https://github.com/fornwall/advent-of-code`, `https://github.com/sonos/tract`) don't wrap: `overflow-wrap: normal` on the `a` inside `p` (p9-6). `overflow-wrap: anywhere` on prose links would fix it. At 430 rustmas happens to wrap at a hyphen.

### Part 9 addendum: 156 logout after all (08:44:15)

Both tabs were signed in at 08:31:05: tab A reloaded /decks at 08:31:05, tab B reloaded Home at 08:31:02. Both then sat idle. At 08:46 I opened the deck page for the 175 repro and both tabs were on /login, with `zwiper-web.session` gone. Tab B's console:
- 08:31:02 `POST /api/auth/cookie/refresh` → "refreshed session"
- 08:44:15 `POST /api/auth/cookie/refresh` → WARN `ensure_fresh.rs:76 refresh rejected; clearing session: invalid refresh token`

Tab A's console was cleared by my navigation, so I can't see whether it refreshed just before 08:44:15. The likely cause is two tabs' idle refresh timers (started 3s apart) firing close together and racing on the rotating refresh token, so the loser is told its token is invalid and clears the shared session. That fits the "two tabs" half of 156: reloads are now fine, but idle multi-tab refresh is not. The 175 repro is blocked until someone signs in.

## Part 10, nit 175 repro: Remove to the end (08:48–08:51, live 48e5f81b9)

**Width: 720 × 759, not wide.** The window was at 720 when Scotty signed me back in, and the tool can't resize it. At 720 the details panel is a modal drawer, not the docked panel Scotty had, so this didn't run in his exact setup. One app tab open, signed in throughout.

Deck: Surfer test (commander Edgar Markov plus 5 Main creatures: Drana, Welcoming Vampire, Bloodthirsty Conqueror, Charismatic Conqueror, Clavileño). Cards › − opens `/decks/…/remove` on Main (p10-1).

What happened:
- **Swipes with details open:** at 720 the drawer dims the docks, and → / ↓ do nothing while it's open (no request). Closed it to swipe. That's expected for the modal drawer.
- **Remove (→):** each swipe sent `DELETE /api/deck/{id}/card/{card}` → 204 and showed a "Removed" chip top-left.
- **Undo mid-stack (after card 2):** Welcoming Vampire came back on top, "Undone" chip, `POST /api/deck/{id}/card` → 201 (p10-2).
- **Stack runs out:** after the last remove the stack shows an **empty placeholder card labeled "No cards"**. Keep/Move/Remove and the details button go dimmed, Undo stays active. **No toast** ("No more cards" or anything else). The details drawer can't open (its button is disabled), so it can't show a stale card (p10-3).
- **Undo at the end:** works. Clavileño came back, `POST …/card` → 201 (p10-4).
- **Keep (←) at the end:** on the last card Keep shows a "Kept" chip, but **the same card stays on the stack**. The stack never reaches the end via Keep; Keep seems to put the card back at the bottom, which with one card is the same card. Undo after it shows "Undone" and the same card.
- **Deck page afterwards:** Cards = 2 (Edgar Markov plus Clavileño), Creatures (1), budget $0.83. The removes really saved, and the two undone cards came back (p10-5).
- **Console and network:** every request logged and succeeded (5 DELETEs, 2 POSTs, all 2xx), no errors or warnings, no failed calls.

So at 720 I couldn't reproduce Scotty's report: the empty state exists ("No cards", if faint), and Undo works both mid-stack and at the end. What's left to check is the wide layout, with the docked details panel visible while swiping. That's the one difference from his setup. The missing toast at the end is real.

Deck state left behind: Main is down to Clavileño (Drana, Welcoming Vampire, Bloodthirsty Conqueror and Charismatic Conqueror removed). I can re-add them with Quick add if the deck should be restored.

### Part 10b, 175 rerun at wide (1450 × 759, details panel docked, 09:00–09:01)

Same deck, Remove on Main, details panel docked on the right and showing the top card. **Still not reproduced.**
- **Mid-stack:** removed Drana and Welcoming Vampire (DELETE each), then Undo: Welcoming Vampire came back, "Undone" chip, details panel switched to it, `POST /card` (p10-6).
- **To the end:** after the last remove the stack shows the **"No cards" placeholder**. Keep, Move, Remove and Card details are `disabled` at opacity 0.35, while Undo, Filter and Refresh stay live. **The details panel unmounts** rather than showing the last card. **No toast** (p10-7).
- **Undo at the end:** Charismatic Conqueror came back, the details panel came back showing it, docks re-enabled, `POST /card` (p10-8). The same thing happened again using mouse clicks on the dock (Remove, then Undo) instead of keys.
- **Console:** 6 DELETEs, 2 POSTs logged, no errors or warnings.

Differences from Scotty's shot (blank stack, docks live, stale details, dead Undo): none of them showed up on web for me, with keys or clicks. It may be timing (swiping faster than the throttle), a deck where some cards were kept and cycled, or Android-only. droid's repro is on Android.

Deck restored after 10b: Edgar Markov plus 5 creatures (4 re-added with Quick add).

**New, small: Cards page summary chips go stale after Quick add.** At wide, after adding 4 cards with Quick add on `/decks/…/cards`, the list showed Creatures (5) but the header chips still said "2 cards · $32.87". They caught up only after a reload ("6 cards · $81.25"). The Quick add success toast fires and the list updates; just the summary row doesn't.

## Part 11, release 3e72d3344 (2026-11-07, 09:51–10:00)

Wide is 1450 × 759. Phone width (390 × 777) was checked in a script-sized popup, DOM measurements only. Signed in throughout, **no logouts**. I used one app tab, plus the popup for about 5 minutes at the end. Deck and theme restored afterwards (Surfer test: Edgar Markov plus 5 creatures; Gruvbox dark).

| # | Check | Result |
|---|---|---|
| 163 | Details and dialogs scroll edge to edge between dividers | **Fixed.** Wide Add page: details `.panel-body` spans y 158–439, exactly between its two rules. At 390: details drawer body 174–578 between rules; the deck hint dialog's description spans 202–565 between rules and scrolls there (363/532) |
| 167 | Pickers full-screen and in the viewport at phone width | **Fixed.** At 390, Format, Deck tags and Oracle tags open at x 0–390, y 0–777, with the chip grid below and nothing off the left edge |
| 169 | Bigger toast ×, hover fill | **Fixed.** 32 × 32, fills on hover (p11-8). zwipe.net's banner × not rechecked (its deploy isn't live) |
| 173 | Page dock slides down and fades with the filter drawer | **Fixed.** On open the dock moves from y 699 to 781 and fades from 1 to 0 over about 300ms; on close it reverses |
| 174 | Swipe screen title tag tinted, not button-like | **Fixed.** Remove is red-tinted (p11-3); Add (p11-9) and Swipe select (p11-10) are green-tinted. No border or pointer cursor |
| 176 | Deck welcome hint shows + and − on wide | **Fixed.** "+ adds cards and − removes them" is the first line (p11-2) |
| 165 | Light-theme controls at 4.5:1 | **Partly.** "Add cards" and "Import" (warning fix buttons) now pass on all 50 light themes (lowest 4.82). Other chips still fail on 42 of 50 light themes, mostly 2.4–2.8: the "?" buttons (2.45 Bluloco, 2.48 Material), power/toughness chips (2.62 Ayu), type chips (2.65 Tokyo Night, Monokai), set and price chips, rarity chips (2.68 Miasma) |
| 170 | Picker × with changes asks Keep/Discard; hint cleared on open | **Fixed.** × after picking Aggro asks "Discard tag changes?" with Keep / Discard (p11-6). Discard reverts the form, and reopening shows the default hint |
| 181 | Deck page count/price follow Quick add and removes | **Fixed.** Quick add Vampire Nighthawk: "6 cards · $81.25" → "7 cards · $81.45" live (p11-1). Removing it with the row's −: back to 6 · $81.25 and Warnings updated, no reload. It kept up across 4 more adds too |
| 172 | One toast per action | **Holds.** Add → one "Added …", remove → one "Card removed" |
| 175 | Remove to the end, then Undo, including a fast undo | **Works on web.** Fast: last → plus ↓ together sent DELETE and POST in the same second, and the card came back with the details panel (p11-4). Slow: the end shows "No cards", Keep/Move/Remove disabled, details unmounted (p11-5); Undo restores everything. Still no toast at the end |
| 164 | Escape closes zwipe.net's theme panel | **Not yet.** zwipe.net still serves `zite-dxh762c74b1eefc6afb.js`, and Escape leaves the panel open |
| 166 | Portfolio root scrollbar themed | **Fixed.** `html` has `scrollbar-color: rgb(60,56,54) transparent`, thin |
| 168 | Portfolio no sideways scroll at 375 | **Fixed.** upsee, sharpmas, rustmas: scrollWidth 364 in a 375 frame (p11-7) |

Small: after a Quick add, the suggestion pill under the field flips to "No results" while the input still holds the added card's name, which reads like the add failed (p11-1).

## Part 12, release f2c6e80a0 (10:49–10:53, wide 1450 × 759, hard-reloaded)

Fresh app wasm `zwiper-web_bg-dxh9751c7d2ad9bda6e.wasm` (2.36 MB transferred). Signed in throughout, **no logouts**. Deck and theme restored (Surfer test: Edgar Markov plus 5 creatures; Gruvbox dark).

| # | Check | Result |
|---|---|---|
| 182 | Swipe screen title tag outlined in its action color | **Fixed.** Remove has a red outline on its red tint (p12-1) |
| 183 | Picker search-to-hint gap equals hint-to-chips | **Fixed.** Format, Deck tags, Oracle tags: 8px and 8px in all three (p12-4) |
| 184 | Remove to the end: plain ghost card, no chip, "No cards left" toast | **Fixed.** Empty ghost card with no label, one toast "No cards left" (p12-2). No per-remove toasts |
| 175 | Undo at the end | **Works.** Charismatic Conqueror back, details panel back, docks re-enabled, `POST /card` |
| Toasts | Whole message, no 2-line clamp | **Fixed.** "Added Clavileño, First of the Blessed" wraps to 3 lines in full (p12-3) |
| Board chips | Wrap clear of the dock | Wide only: Main/Maybe/Side/All sit at x 16–220 and the dock starts at about x 930, so they're clear. Narrow not checked (Droid has mobile web) |
| zwipe.net counters | Still honest | **Not on first paint.** After a hard reload (new `zite_bg-dxhd13f5913dad17271.wasm`), the hero showed **4,076 decks / 57.2k searches / 475.5k swiped, "as of 09:16"** for at least 3s. The next load showed the live values from `api/marketing/stats`: **2,270 / 31.8k / 264.7k, "Live"**, which matches the app's Home (2,270 / 31.8k / 264.7k, "as of 09:57"). So the built-in snapshot numbers are about 1.8× too high, possibly from a different database at build time. Anyone with a slow API call, or a failed one, sees the inflated figures (p12-5 is the live state) |

## Part 13, release f699acc37 (2026-11-07, done 14:09)

Fresh app wasm `zwiper-web_bg-dxhf5246e19639ede98.wasm` (2.35 MB transferred) after a hard reload. Signed in throughout, **no logouts**. Wide is 1450 × 759. Phone width (390 × ~780) was checked in a script-sized popup, DOM measurements only. Deck, maybeboard and theme restored (Surfer test: Edgar Markov plus 5 creatures, $81.25; commander maybeboard empty; Zenburn dark, which is what the account had when I started).

| # | Check | Result |
|---|---|---|
| 194 | Commander maybeboard Swipe | **Works.** "Swipe" sits in the Commanders header, in the empty state and with commanders listed. It opens Swipe select with the dock Skip / Save / Undo / Pick. ← skips (no request), ↑ saves (`POST /api/user/commander-maybeboard/{id}` 204, "Maybe" tag), ↓ undoes the save (`DELETE` same path 204, "Undone"), → opens Create deck with that commander and Format Commander filled in. A saved commander isn't dealt again: with Edgar on the board he no longer came up second. The help dialog spells out all four directions (p13-1) |
| 192 | Phone card-details drawer: title in its header, no fold | **Fixed.** At 390 the drawer header (76px, 1px bottom border) holds ×, the title and the mana cost on one row. There is no separate head band. The body (`.panel-body`) starts exactly at the header's bottom edge and scrolls (481/447) under it |
| 193 | Swipe card fills a phone screen | **Fixed.** 326 × 454 at 390 wide (32px gutters) on maybeboard Swipe, Add and Remove. Previously about half the width |
| 186 | Card preview as wide as the app's on a phone | **Fixed.** Home's Featured flavor "Image" opens at 358px in a 390 window (16px gutters) |
| 188 | Chip skeletons hold no inner bar | **Fixed visually.** The ghost chips on the deck page (group-by, profile icons, stat chip, buttons) still contain a `.skeleton-bar` child, but it's `visibility: hidden` and only sizes the chip. What shows is an empty chip |
| 189 | No fade at bordered edges | **Mostly.** "Welcome to your deck" hint at 390: the description scrolls (609/363) between its two rules with no mask. Card-details drawer: no mask. **Still faded:** the picker's chip grid (`.web-picker-body > .tag-grid`, 16px top and bottom). Its top edge is right under the hint box's bordered bottom, so the chips fade out at a border. The Oracle tags dictionary list (`.dict-list`, top) and Import/Export (`.import-scroll`, top) also keep a top fade, under their controls rather than a rule. Worth a look at whether the picker grid should drop its top fade |
| 191 | A–Z rail scrolls on x only; tighter top gap | **Fixed.** `.dict-letter-rail` is `overflow: auto / hidden`, scrollHeight = clientHeight (35), scrollWidth 1017 in 366. Oracle tags picker: header bottom 75 → Search label 93 |
| 190 | No double-tap zoom | **Fixed (CSS).** `touch-action: manipulation` on html and body on app.zwipe.net and zwipe.net. Viewport is `width=device-width, initial-scale=1`, so pinch zoom stays. Not tested with a real touch device |
| 195 | "as of <date>" | **Fixed.** zwipe.net's served HTML says **"as of Nov 7"** with 2,270 / 31.8k / 264.7k. After load the numbers scramble and settle on the live values, labeled "Live". The portfolio's two cached labels also read "as of Nov 7". The app's Home showed "Live" |
| zwipe.net counters | Snapshot honest | **Fixed.** The built-in snapshot now matches the live API (2,270 / 31.8k / 264.7k). Part 12's 1.8× inflation is gone. The figures I saw at first glance (e.g. 2,257) were mid-scramble frames |
| 165 | Light-theme contrast | **Fixed.** On the deck page (109 enabled chips, tags and buttons), all 50 light themes clear 4.5:1, the lowest at 4.60 (Flexoki "Tribal", Kanagawa Dragon "Keywords", Rustbox "$81.25"). Dark: the same, apart from the off-state Maybe/Side board chips (2.6 on Material, 2.7 on Bluloco), which are disabled and exempt |
| 182–184 | Rechecked in Part 12 | Held (outline on the title tag, picker gaps, ghost card without chip) |

**New: deck page Warnings go stale after board moves and adds.** While restoring the deck I moved Drana from Maybeboard and Welcoming Vampire from Sideboard to Main, then Quick added Clavileño. The header chips and Creatures (5) updated live to "6 cards · $81.25", but Warnings still said **"Deck has 3 cards, commander requires at least 100"** and **"Commander does not use sideboards"** with the sideboard empty (p13-2). A reload fixed it ("Deck has 6 cards", sideboard warning gone). 181 made Warnings follow the row's −. It doesn't follow "To mainboard", or (maybe) Quick add.

Small: the maybeboard Swipe's dock labels the right swipe "Pick" while the help says "starts a new deck with that commander". It's clear enough, but "Create deck" (the row action's label) would match.

Tester's note: my first light-theme contrast run clicked buttons outside the theme sheet (a bad selector). It toggled Surfer test's boards and moved, starred and removed some of its cards, and opened Log out's confirm, which it then closed. Nothing outside the test account was touched. The session stayed signed in, and everything was put back as described above.

## Part 14, deep flows on live f699acc37 (2026-11-07, done 14:41)

Same wasm as Part 13 (`zwiper-web_bg-dxhf5246e19639ede98.wasm`), hard-reloaded. Wide is 1450 × 759. Phone width (390) was checked in a script-sized popup, DOM measurements only. Signed in throughout, **no logouts**. Test-account state restored: I created and deleted two decks of my own ("Surfer temp", its clone). Settings went back to Themed / Chip / Show, the theme to Zenburn dark, zwipe.net's local theme to Gruvbox dark. Surfer test (6 cards) and the commander maybeboard (empty) weren't touched. Other testers' decks weren't touched. "Droid tmp" appeared mid-pass, which is why the list reads 13.

On the composer's question: **the misclick script in Part 13 did not touch the Commander maybeboard.** It only ran on the Surfer test deck page. Edgar Markov was on the maybeboard because I saved him while testing 194's Swipe, and I removed him myself afterwards with the row's Remove, since the board was empty when I started.

### Works

| Area | What I did | Result |
|---|---|---|
| Decks list | Group by None/Format/Color/Tag, color and tag chips | Groups add up to all decks (1+6+1+1+2+1 = 12 at the time). "All" also clears the tag chips |
| Create deck | Name, format (searched "mod" → Modern), power, Other tags, land target, price target | Save stays hidden and disabled until the form has a name. The price field drops letters ("abc12.5" → 12.5). The first + on Land target jumps from "Not set" to 17. Saved and opened the deck page with the right chips and Budget ($0.00 / $12.50) |
| Import (text, Add) | `4 Lightning Bolt`, `2x Counterspell`, `1 Snapcaster Mage (ISD) 78`, a bare `Ragavan, Nimble Pilferer`, a fake card, `20 Mountain`, a `Sideboard` heading, `2 Blood Moon` | All formats parsed. The fake card is listed under Failed as "not found". Blood Moon went to the sideboard, and Main counts 28 |
| Export | Deck, then + Side | Plain list, plus a `// Sideboard` section when Side is on |
| Round trip | Pasted the export back with Replace | `// Deck` and `// Sideboard` are understood, nothing failed, quantities and boards held |
| Edit | Rename, format Modern → Commander, Back with unsaved changes | "Leave without saving?" (Stay/Leave) guard works. Saving as Commander with no commander is allowed and raises the right warnings |
| Fix to 1 | Warning button on Counterspell (2/1) | Saves (see the stale bug below) |
| Clone | Empty name, then Enter | Save is disabled while the name is empty. Enter saves, and the clone carries the cards, sideboard and tags |
| Delete | Danger zone → Delete → confirm | Deck gone, back on Decks |
| Share | Create link → open it signed out (zwipe.net, where this browser is signed out) | "Share link copied". The page renders the deck (title "Surfer temp", chips, groups, filter) at wide and at 390 with no sideways scroll (p14-1). It picked up the rename. Stop share → the link shows "Deck not shared" (p14-6) |
| Filter drawer | Every section on Add: Name (contains / doesn't contain), Oracle text, Types, Mana value, color identity, Produces, Combat (power 5), Flavor text, Artist, Rarity, Card roles (include, two at once = OR), Oracle tags, Set, Format ("Is legal in" Pauper), Price min/max, Sort (Price USD), Config (Is digital) | Each returns matching cards (checked against the API response: e.g. Pauper = 25/25 legal, repeatable-treasures = 17/17 tagged, power 5 = all P5). Section headers have role=button and tabindex 0 |
| Oracle tags dictionary | Wide, plus 390 | **Not reproduced.** Wide paints normally; letter jump (M) and Examples work. At 390: 95 rows, 4 in view, overlay and list at opacity 1 with no transform, and a hit test lands inside the first row at 300ms, 1s and 2.5s. One lead for Droid's blank paint: the list is ~16,700px tall and carries a `mask-image` (top/bottom fade). At DPR 2 that's ~33,000 device px, past the 16,384 texture limit many emulator GPUs have, and a mask forces the whole layer onto the GPU. Dropping that mask (189 already argues against it) or virtualizing the list would be the thing to try |
| Settings | Pips Themed → Classic → Themed; Swipe confirmations Chip → Toasts → Off → Chip; Universes Beyond Show → Hide → Show | Each change toasts. With UB hidden, Exceptions unlocks, and the Add stack's first 25 had no UB sets. All restored |
| Theme | zwipe.net signed out: Light, theme panel, pick, Escape | Applies at once, Escape closes. App theme switching was covered in Part 13 (50 × 2) |
| Hints | Every `?` on Settings, Home, Surfer test, Playstyle, Activity, Create deck (37 dialogs) | All open with a title and body, none clip or scroll. At 390 the Draw odds hint is 358 wide, body 210/210 |
| Guides | All 22 links on zwipe.net/guides | All 200 |
| 390 overflow | Decks, Create deck, deck page, Import, Export, Settings, Account | scrollWidth 390 on all, nothing past the edges |

### Findings

1. **Deck page doesn't refresh after "Fix to 1".** Toast "Quantity set to 1", but the header still read 28 cards / $182.12, and Warnings still said "Counterspell exceeds copy limit (2/1)". After a reload: 27 cards / $179.55, warning gone (p14-2). Same family as Part 13's stale Warnings after To mainboard. Warnings (and here the header) don't follow every write.
2. **Import ignores set codes.** `1 Snapcaster Mage (ISD) 78` imported the Innistrad Remastered printing, not Innistrad #78. Export doesn't write set codes either, so printings don't survive a round trip.
3. **Set filter shows a different set's printing.** Set = Innistrad returns cards printed in Innistrad, but every card shows its Innistrad Remastered printing. It reads like the filter is wrong, though it's the display printing. Probably the same one-printing-per-card model as 2.
4. **Typeahead suggestions cover the drawer.** Set, Artist and Oracle tags suggestions open upward as loose pills over the section headers above, with no backdrop. They hide the section's own header (p14-3). They stay open after a pick and cover the new chip (p14-4). The last pill can also sit under the Reset/Apply dock.
5. **Zero filter results are silent.** Config "Is promo: Yes" matched nothing. The stack showed a blank ghost card with the dock disabled and no message (none visible ~3s after Apply). Also: whether "Is promo: Yes" can ever match, given the one-printing pool, is a question.
6. **Mana value only commits on blur.** After typing "1" in Mana value, Apply stays disabled until focus leaves the field. A tap on ✓ blurs first, so phones are probably fine. On wide, typing then pressing Enter doesn't apply.
7. **Hidden form rows still take keyboard focus.** On Create deck, the collapsed Commander, Partner, Background and Signature spell rows (height 0, opacity 0) keep their `?` and input at tabindex 0, not inert. That's 8 invisible tab stops before Power level.
8. **Hint dialog widths swing with text length.** At wide they run from 320px (Types) to 1,184px (Activity), so long hints are single lines over 1,000px wide (p14-5, Draw odds 1,113px). A max width around 40rem would keep line lengths readable.
9. **Share link is a 404 to anything that doesn't run JS.** `curl https://zwipe.net/deck/<id>` returns **HTTP 404** with the home page's `og:title`/`og:description`/`og:url`. The deck renders client-side. Link previews (iMessage, Discord, X) will show the generic home card, and crawlers see a dead page. The signed-out share page also shows no sideboard (no Boards control), so Blood Moon was missing.
10. **Warning copy.** "Deck has 28 cards, modern requires at least 60" (lowercase format). "The format, commander, requires a commander" (awkward, lowercase). "Lightning bolt exceeds copy limit (4/1)" (the card name lowercased).
11. **Small copy and consistency.**
    - The share dialog's button says "Stop share"; its own text and the guide say "Stop sharing".
    - A Modern deck still shows the "Command zone" Show chip.
    - The Import placeholder is a Commander list (Krenko, Urza, Sol Ring) on any format.
    - The commander maybeboard guide doesn't mention 194's Swipe on the maybeboard page yet, and says a row has three actions where the app shows four (Image, Printings, Create deck, Remove).

Not verified: the phone filter drawer at 390. In the background popup its slide-in never ran (`show` class set, still `translateY(662px)`), which matches Part 13's picker caught mid-slide. That's the popup's throttling, not the app, and Droid covers it on mobile.

## Part 15, zwipe.net, the portfolio and the app's signed-out screens (2026-11-07, done 14:58)

zwipe.net serves `zite_bg-dxh9074ac72e5c198ca.wasm`; the app is still on live f699acc37. Wide is 1450. 390 was checked with same-origin iframes for zwipe.net and the portfolio (screenshots work), and with the script-sized popup for the app (DOM only). The test session stayed signed in throughout, **no logouts**. zwipe.net's and the portfolio's local themes went back to Gruvbox dark and Poimandres dark.

**How I saw the app signed out without signing out:** from app.zwipe.net/robots.txt I opened a popup and wrote the app's own index.html into it with a small shim first:
- reads of `zwiper-web.session` return nothing;
- writes and removals of that key are dropped, so the real session can't be touched;
- calls to `/auth/(cookie|refresh|logout|login|register|password|reset)` get a canned 401.

The popup boots the real wasm signed out. None of the guards fired (0 blocked writes, 0 blocked requests), and the main tab's session was present and working afterwards. I didn't submit any sign-in, sign-up or reset, and typed no passwords. Production is no place for test passwords, so password strength, mismatch and the reset submit are untested.

### Works

| Area | Check | Result |
|---|---|---|
| Crawl | Every internal link and asset from each home page | zwipe.net: 58 URLs over 29 pages, all 200. Portfolio: 43 URLs over 22 pages, all 200, apart from Cloudflare's `/cdn-cgi/l/email-protection` (S23, skipped before) |
| Meta | title, description, og:title/url/image, canonical, h1 on all 51 pages | Unique titles and descriptions, og:url and canonical match each page, og:image present, exactly one h1 everywhere |
| Sitemaps/robots | zwipe.net | sitemap has 29 locs (= the crawl). robots allows / and disallows `/deck/`, `/verify/`, `/confirm-email/`, `/reset/` |
| External links | 89, skipping retailers | All fine apart from the two below. LinkedIn's 999 is its bot wall |
| Changelog | Latest, year → months → minor chips, Show more, `#v1-12-1` | Latest shows 1.18.0 and the upcoming 1.18.1. 2026 shows 5 at a time, and 11 Show mores reach all 59 releases. Sep narrows to 5. The fragment lands on 1.12.1 just under the nav with 2026 › Oct › 1.12 lit. Cairn's changelog on the portfolio lists 1.3.0 and 1.3.1 |
| Theme toggles | zwipe.net and portfolio, signed out: Light, pick a theme, Escape | Both apply at once and keep the choice. Escape closes the theme panel on both. On zwipe.net, opening it moves focus to the current theme's row and Escape gives focus back to the trigger |
| Metrics | zwipe.net counters, portfolio stats | zwipe.net matches live (2,270 / 31.8k / 264.7k, "as of Nov 7" then Live). Portfolio "7,897 commits · 14 repos · 88 stars" lines up with Heron's allowlist (the 14 showcased repos sum to about 88 stars). GitHub's overall totals are 40 public repos and 128 stars |
| Keyboard | Tab through zwipe.net and portfolio homes | Logical order (nav, banner, content, footer). The closed theme panels' 50 buttons are out of the tab order. Portfolio cards waiting on scroll-reveal come in at full opacity when focused |
| 390 | zwipe.net: home, guides, about, changelog, community, privacy, contribute, two guides. Portfolio: home, projects, zwipe, systems, contribute, cairn changelog | No sideways scroll on any (scrollWidth = clientWidth 379 in a 390 frame). The phone menu lists every link with the theme and dark buttons at the bottom (p15-1) |
| App signed out: Login | 390 and 1400 | Renders with counters, form, Create profile and Forgot password. Empty submit, or a name with no password, shows "Enter your username or email and password" and sends nothing |
| Create profile | Empty, `ab`, `a b!`, `not-an-email`, valid name and email with no password | "Must be at least 3 characters long", "Cannot contain whitespace", "Missing @ symbol", "Must be at least 8 characters long". No request sent |
| Forgot password | Empty, `bad` | "Please enter a valid email address", no request |
| Deep links | `/`, `/decks`, a deck, its Add, `/settings`, `/account`, `/maybeboard`, `/playstyle` | All land on `/login` (history replaced, length 1) with no deck or user API calls. `/nonexistent-page` gives the app's "Page not found" |
| Token pages | zwipe.net `/verify/…`, `/confirm-email/…` with a bogus token | "Link already used or expired" with what to do next |

### Findings

1. **Escape doesn't close zwipe.net's phone menu.** At 390, with the menu open and focus on Guides, a real Escape key leaves it open (`aria-expanded` stays true) (p15-1). The portfolio's phone menu closes on Escape, and zwipe.net's theme panel does too (164).
2. **zwipe.net doesn't declare `color-scheme`.** It's `normal` on dark themes (the portfolio sets `dark`). So Chrome draws its default dark-blue focus ring, rgb(0,95,204), which is **2.66:1 on Gruvbox's background**, under the 3:1 non-text minimum. Nearly every control uses it (only the testimonial link has a themed 3px ring). The same default also affects scrollbars and native controls. The portfolio's ring is light blue and easy to see. The app wasn't checked for this.
3. **zwipe.net has no `<main>` landmark** (only nav and footer), and neither site has a skip link.
4. **Two dead links on the portfolio's orangui page:** `github.com/scadoshi/orangui/blob/main/src/chart_math.rs` and `.../src/theme.rs` are 404. orangui's `src/` no longer has those files. The other 20 file links on that page resolve.
5. **Reset link doesn't check the token up front.** `zwipe.net/reset/<bogus>` shows the full "Set new password" form, so someone with a dead link only finds out after choosing a password. Verify and confirm-email check on load and say "Link already used or expired". (Submit not tested; see above.)
6. **Generic titles on the token pages.** `/reset/…`, `/verify/…` and `/confirm-email/…` all have the home page's title ("Magic: The Gathering Deck Builder for Phone and Web | Zwipe") instead of "Reset password" etc. They're disallowed in robots, so this only shows in tabs and history.
7. **No autocomplete hints on auth forms.**
    - Login's fields have no `autocomplete` and no `name`; they should be `username` and `current-password`.
    - Create profile's and zwipe.net's reset form's password fields have none either; they should be `new-password`.
    - Without these, password managers guess or skip the forms.
8. **Forgot password's email field is `type=text`.** Phones won't show the email keyboard. Create profile's is `type=email`.
9. **Email errors are worded differently.** Create profile says "Missing @ symbol", even when the field is empty. Forgot password says "Please enter a valid email address".
10. **A deep link signed out probably loses its target.** A signed-out deck link redirects to plain `/login`, with no `?next=` and nothing in localStorage or sessionStorage. So after signing in you'd likely land on Home, not the deck. I can't confirm the after-sign-in step without signing in.
11. **Login's counters keep "as of Nov 7" after fetching live stats.** They settle on the live 2,270 / 31.8k / 264.7k, but the tag still says "as of Nov 7". Home (signed in) says "Live" once fresh (195).
12. **Portfolio stats don't say what they count.** "14 repos · 88 stars" reads as the whole GitHub account (40 repos, 128 stars). "Counted by Heron" is there, but not that it covers showcased repos only. Wording question for Scotty.

## Part 16, deck page in depth, plus Home and Decks order (2026-11-07, live f699acc37, wide 1450)

Signed in throughout, **no logouts**. I cloned isshin (101 cards, not mine) into a temp deck and did everything on the clone, then deleted it. isshin itself was only read. Surfer test, settings and theme untouched. Decks reads 13 because Droid added a long-named deck during the pass.

### Works

| Area | Check | Result |
|---|---|---|
| Price totals | Sum of qty × row price vs header and Budget | **Exact:** $568.94 in all three over 83 rows / 101 cards. Changing a printing (Boros Charm → OTJ Commander $4.99) updated the row, header and Budget live to $573.93. The dock's Undo restored the old printing ("Boros Charm printing restored") and $568.94. Sideboard cards are left out of the total (moving Chaos Warp to Side dropped it $0.76) |
| Row actions | Image, Printings (arrow keys, Select), −, +, Star, To maybeboard / To sideboard | All work. Star adds ★ to the row and puts the card in the deck head as an "MVP" card, and the button becomes Unstar. +/− change the count and header live. To sideboard moves the card and the header live |
| Group by | Type, Mana value, Color, Card role | Type and Mana value add up to 100 (+1 commander = 101). Color adds up to 100 (W 10, B 4, R 13, WB 5, WR 13, BR 3, WBR 5, UBR 1, colorless 8, lands 38). Card role adds up to 221, since cards carry several roles |
| Stats | Types chart, mana curve | Types match the Type groups. The curve (5/18/19/10/7/3) matches the Mana value groups. The Card roles chart reads 1–2 higher than the Card role groups (AGGRO 35 vs 33, RAMP 17 vs 16, SACRF 9 vs 8), probably because the chart counts the commander. Worth labeling |
| Sort | Filter → Sort → Price (USD), Descending | Rows within each group come out by price, high to low (46.85, 28.74, 12.25…). See 7 for unpriced cards |
| Large deck | 101 cards, 83 rows | Rows render ~0.8s after navigation. ~6,900 DOM nodes, 44 MB heap, 82 of 84 images lazy-loaded |
| Long names | 64-character deck name | Accepted at 64 exactly. Home's deck tile cuts it off with an ellipsis. Decks wraps it (see 9) |
| Decks order | List | Alphabetical, ignoring case (big fat chud … Surfer test, test, WWWW…). No sort control |
| Home order | Decks panel | The three most recently changed decks (the new clone first) |

### Findings

1. **Card rows can't be used from the keyboard.** A row (`div.web-card-row`) and its head have no tabindex or role, so Tab never lands on a row and nothing expands it from the keyboard. Group headers (`div.deck-fold-head`) are the same. On a phone or with a mouse that's fine. For keyboard users it's a dead end.
2. **…and collapsed rows still own over a thousand tab stops.** Every collapsed row keeps its chips and its Image / Printings / − / + / Star / To maybeboard / To sideboard buttons at tabindex 0 inside a 0-height, opacity-0 wrapper (`card-row-collapse`). On this deck that's **1,017 of the page's 1,056 tab stops**, all invisible. Real Tab presses from Quick add go: Group by chips, Boards, Show, "Shut every group", then hidden "Aggression", "Image", "Printings", "−", "+"… An Enter there changes the deck unseen. The page dock (+ − filter undo edit clone share) comes after all of them. The fix is `inert` (or tabindex -1) on collapsed content, plus a focusable row head with `aria-expanded`.
3. **Warnings don't follow +, − or board moves** (adds to Part 13's To mainboard and Part 14's Fix to 1).
    - **+** took Chaos Warp to 2 in a Commander deck. The header went 101 → 102, but the Warnings count stayed at 2 with no copy-limit line.
    - **To sideboard** dropped the header to 100, but Warnings still said "Deck has 101 cards" and had no "Commander does not use sideboards".
    - After a reload both are right: "Crackling doom exceeds copy limit (2/1)" and the sideboard warning show up.
4. **Hover preview covers the row actions.** The floating card preview opens over the list itself, on whichever side has more room, so it sits on the expanded row above. In p16-1 the pointer is on Chaos Warp, and its preview hides Boros Charm's details and its Image and Printings buttons. Moving to those buttons means passing other rows, which swaps the preview.
5. **Drag between boards does nothing.** All 82 rows are `draggable="true"`, but a real mouse drag onto the Sideboard group and a synthetic dragstart/dragover/drop both did nothing (no data set, no drop highlight). If drag isn't a feature, `draggable` should go. Moving with the row buttons works.
6. **Deck name length isn't limited in the field.** The clone (and probably the create/edit) name input has no `maxlength`. A 98-character name only fails on Save with the raw server message "Invalid deck name: deck name maximum length is 64 characters" in a toast. A maxlength of 64, or a counter, would stop it at the field.
7. **Unpriced cards: the total silently excludes them, and they sort first.** 11 of the 101 cards have no USD price on their chosen printing (Phyrexian Arena, Boros Charm FDN, Sword of the Animist, Mountain…). The total and "Average card price" ($7.39 = $568.94 / 77 priced cards) leave them out with no note. Sorting by Price (USD) Descending puts these unpriced cards **first** in each group, ahead of $46.85.
8. **Card names lowercased in warnings, badly.** "Nicol bolas, the ravager // nicol bolas, the arisen is outside commander's color identity", "Crackling doom exceeds copy limit". Same root as Part 14's "Lightning bolt".
9. **A long name drops below the art on Decks.** Short names sit beside the thumbnail. The 64-character one wraps onto its own lines under it, so that card's layout differs from its neighbors (p16-2).
10. **Color groups are icon-only.** Group by Color headers show just mana symbols and "(10)", with no text or aria-label, so a screen reader hears "(10)".
11. **No sort on Decks or Home.** Decks is alphabetical only. Home shows recent. There's no way to sort by size, price, format or last changed. That's a feature question, not a bug.
