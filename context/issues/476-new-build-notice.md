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

## Follow-up: a dismissed notice never showed for a later build
Surfer: after × on the notice for one build, a newer build deployed later showed no notice.

- Cause: the `Banner` holds its dismissed state, and `key: "{build}"` was meant to remount it per build, but Dioxus 0.7 compares keys only between items of a list. A lone keyed component is diffed in place, so the dismissed banner stayed dismissed for every later build. The once-a-minute throttle and the five-minute tick were not involved: the later build was fetched and offered.
- Before: a build stamped `aaa`, served locally with `version.json` switched by hand, asked by a focus event a minute apart. bbb showed, × hid it, ccc offered later showed nothing. The new test failed: offering ccc produced no DOM edits at all.
- Fix: the banner renders as a list of one (`for build in [build]`), so a new build's key mounts a fresh banner.
- Test: `a_later_build_mounts_a_fresh_banner_so_a_dismissed_one_shows_again` counts the click handlers a render attaches: a later build mounts the × and Reload again; the same build offered again (negative control) attaches none.
- After, same setup: bbb shows, × hides it, bbb asked again stays hidden, ccc shows.

## Verification
-
