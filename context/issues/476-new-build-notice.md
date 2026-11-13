# 476. A web page that stays open never hears of a newer build

State: fixing | Owner: laptop | Type: feature | PR: - (pushed branch, hash in the composer's log) | Build seen: release bf1bba725

## Report
Scotty, via the composer: when the web app notices a newer build, it shows a small "New version available" notice with a Reload action, and never interrupts typing. Decision: keep the existing reload at a page change, which keeps people on a current build; the notice is for people who stay on one page.

## Before
Local web with a baked build (`ZWIPER_BUILD`), `/version.json` answered with another build, headless Chrome at 1280. Staying on one page and firing focus and visibility every 4s for 20s: 0 asks of `version.json`, no notice, 3 of 3 runs.

## Cause
`zwiper-web/src/lib/inbound/chrome/fresh_build.rs` asks only from `after_move`, after a page change the leave guard lets through. Nothing asks while a page stays put, and there was no notice to show.

## Fix
laptop, in `fresh_build.rs`:
- **Watch:** `use_new_build_watch` (mounted at the app's root) asks as the window takes focus, as the tab comes back into view, and every five minutes while it shows. It uses the existing fetch and rules, at most once a minute (`rules::poll_due`).
- **Notice:** once a newer build is known (from that watch or from a page change's ask), `NewBuildNotice` shows orangui's Banner in the announcement corner: "New version available" with a Reload button and its ×. It has no countdown (`web.css` hides the bar) and is keyed by the build, so × dismisses that build and a later one shows again. It never takes focus. Reload reloads the page; a screen with unsaved changes gets the browser's own leave prompt, as the leave guard already arranges.
- **Page change:** the existing reload at a page change stays.
- **Tests:** `poll_due` and `notice_for`, red before.

## Checked
Same setup as the before, 3 of 3 runs:
- 1 ask while staying (the rest wait out the minute) and the notice showing with Reload;
- with "typing-in-progress" typed into Log in's username first, focus and the text stay in the field while the notice shows;
- × hides it and a later focus doesn't bring it back;
- Reload reloads the page.

## Verification
-
