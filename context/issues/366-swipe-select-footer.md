# 366. Suggested commanders has no footer

State: fixing | Owner: thinker | PR: - | Build seen: production, web (screenshot of app.zwipe.net/playstyle, Suggested commanders swipe)

## Report
Scotty, 2026-11-11, web: Playstyle's Suggested commanders swipe has no footer, while other web screens do. Asked for: check which screens show the shared footer and make Suggested commanders match the swipe screens' pattern, with the shared footer and its spacing, no per-screen layout.

## Cause
Every screen under the nav (Add, Remove, the deck page, Activity) ends with the footer through `ScreenFrame`, which makes it a `.web-scroll-screen`: on a wide window the swipe fills the window and the footer starts below the fold, the docks rising above it as it scrolls in; on a phone a swipe screen has none. Suggested commanders is the swipe-select overlay (`SwipeSelect`), a full-window `.screen` of its own outside any `ScreenFrame`, so it had no footer. The deck form's Commander swipe and Commander maybeboard are the same overlay, so they had none either.

## Fix
thinker: the overlay is a `.web-scroll-screen` that ends with the frame's `WebFooter`, as a screen does, so all three hosts match Add and Remove. On a wide window it scrolls down to the footer (its content fills the window, so the footer starts at the fold), clips sideways at the window's edge as before, and scopes the footer's timeline to itself so its docks rise above its own footer rather than the screen's under it. On a phone the footer stays out, as on Add. Headless with mocked suggestions: at 1280 and 1000 the footer starts at the fold, scrolls in by 108px (Remove: 109px), and the docks end just above it; at 390, no footer and no scroll, as on Remove.
