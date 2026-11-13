# 464. The filter's Rarity section grows from 3 to 6 chips while it opens

State: fixing | Owner: laptop | Type: issue | PR: - (pushed branch, hash in the composer's log) | Build seen: app.zwipe.net 4059d64f8, 635ec281a

## Report
surfer, during 458's before (Chrome macOS 1009x958, signed in, Surfer 399 Isshin, dock Filter, open Rarity): the section shows 3 chips, then 6, while it opens, so a quick click can land on the wrong chip. In before-458 a click aimed at Rare landed on Special. surfer's before-464 (635ec281a, 1440 and 390, deck page and Add, 3 of 3 each) logged it per frame:
- all 6 chips are in the DOM from the first frame;
- about 150 to 165 ms after the click the first row shows (C U R at 1440, C U R M at 390), and about 55 ms later the second.

## Cause
Nothing arrives late: Rarity's chips are `Rarity::all()`, static. Two things move them while the section opens:
1. `components/assets/filter.css`, `.accordion-content`: the section eases open (`grid-template-rows` 0fr to 1fr over 300 ms) with its content clipped. The chips wrap onto two lines, so the clip shows the first line before the second.
2. 50 ms after the click the opened section smooth-scrolled to the top of the sheet (`scroll_section_into_view` on the web, ten copies of the same script in the app's `card_filter_sheet.rs`), sliding the chips under the pointer as they appeared. The app's copies scrolled to counted positions, three of them the wrong item.

Every section of the filter opens this way, on the web and in the app, which share `filter.css`.

## Fix
laptop, per the composer's option (c):
- **CSS:** an opening section's rows stay hidden through the 300 ms height ease, then fade in whole over 150 ms (`accordion-content-in`, `filter.css`). Under reduced motion they show as before, with no fade.
- **Scroll:** one script, `components::OPEN_SECTION_SCROLL_JS`, scrolls the open item (`.accordion-item[data-open="true"]`) to the top once, instantly, at 300 ms, before the rows show. The web filter and every app filter section call it; the app no longer counts positions.
- **Tests:** `components/tests/filter_section_open.rs` pins the delayed fade and the instant scroll of the open item.

## Checked
Local web, shared Atraxa deck, 1009x958, headless Chrome screencast of dock Filter then Rarity:
- **Before:** the first chip line shows, then the second, while the sheet scrolls.
- **After:** the header holds still while the section opens empty, and the sheet moves once at about 300 ms. All 6 chips fade in together from about 330 ms, already in place.

## Follow-up
surfer's after (halo `~/surfer-shots/after-464/`) passed the main goal, with notes:
1. **Lag:** the empty beat read as lag: about 350 ms open but empty, then a 130 ms fade.
2. **Intermittent second scroll step:** 2 of 25.
3. **"Include x" line pushing the chips:** retracted by surfer, a sampler artifact; re-measured, no push.

Cause of 2: the section that closed for the new one eased shut over 300 ms, the same span as the one-shot scroll's delay. When the timer fired before the collapse ended, the rest of the collapse moved the content again, through the browser's scroll anchoring or a clamp.

Fix:
- **Open ease:** 180 ms. The rows fade in from 80 ms over 120 ms, overlapping its end.
- **Switching sections:** while a section is open, any other shuts at once (`.accordion:has(...)` with no transition), so a tap is one layout change. A lone section closing keeps its ease.
- **Scroll:** runs once, the frame after the tap draws (`requestAnimationFrame`), before the rows show.
- **Bottom pad:** so a section near the end reaches the top at once, the accordion takes a bottom pad of one sheet height, trimmed after the ease to the room the position still needs. A close resets it.

Checked: 24 opens (Rarity, Sort, Types, Config, Mana, Price, switching each way, three fresh loads, first opens included) at 1440 in headless Chrome, sampling every frame:
- the rows first show 74 to 94 ms after the section renders and are fully in by 175 to 195 ms;
- nothing moves once any row is visible (0 of 24);
- the sheet scrolls at most one step after the instant shut, both before the rows show.
