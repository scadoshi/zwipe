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

## Verification
-
