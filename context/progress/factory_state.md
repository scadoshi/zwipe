# Factory state

The handoff for the composer session: where the factory stands, so a fresh or compacted composer can pick it up. The rules are in [`../development/factory.md`](../development/factory.md). Open issues are in zwipe-context's GitHub issues and [`../issues/`](../issues/README.md). Older handoff logs are in this file's git history.

## As of 2026-11-16: stopped

Scotty wound the factory down on 2026-11-16 (weekly context limit). Every agent has stood down; no builds, dev servers or test sessions are running, apart from droider's emulator (release app 6c0b24ead). Start nothing until Scotty says.

- `main` = `release` at `2a329c957`, deployed green: web, zerver, zite. Web is 1.21.9; UPCOMING 1.21.10 holds the undo, shared changelog, stacks, password rules, Try a filter and About lines.
- Stores: Zwipe 1.21.9 (iOS 117, Android vc79) and Cairn 1.4.3 (build 17) submitted 2026-11-15/16. `MOBILE_STORE_VERSION` is 1.21.9.
- orangui main has PR #58 (ReleaseCard `tags`), merge `242c208`; zwipe pins its commit `db8a9d7`. Portfolio moves its one ReleaseCard call to `tags` on its next repin.
- Signups: 104 in the week of Nov 10 (9 the week before). Dashboard: https://claude.ai/artifact/TW4A9bxiDQfRcz3EAiLEsP (fixed copies of the zerver reports; rerun them to refresh).

## Done 2026-11-15 to 11-16

- Undo: rapid undos coalesce; per-deck undo history on web Add (the cross-deck leak dated from 655d462bb, Nov 7); Android down-swipe bursts land (flick speed, touch-action, stale-card touches); writes for one card go in order (`spawn_write_in_order`); a promote's undo waits for the promote.
- Web writes survive leaving a screen: `components::spawn_write` and `use_mounted` everywhere; the `zwiper-web/tests/screen_writes.rs` guard finds 0 screen-scoped writes.
- Stacks: regroup ~1.1s to ~0.3s, raise 1.0s to 0.5s, a raised card holds still on phones, drawer flip 0.39s to 0.23s, render-count guard tests, `zcripts/perf/stacks_probe.py`. Keep-cards-mounted is parked (zwipe-context `ideas/stacks_keep_cards_mounted.md`).
- Shared changelog: one `components::Changelog` on zwipe.net, app.zwipe.net and the app, compiled copy first, then the server's. About cleanup: one look, no Version panel, version chip, both Web and Mobile tags.
- Password rules checked as you type with zwipe-core's policy on Register, Change password and the reset form; the migration rule is in `context/development/api_evolution.md`.
- Square deck art, deck loading ghosts that match the loaded page, card count with the command zone, Try a filter on Add's ghost.
- heron audit (quinn bumps), zynergy repinned and its stale rsa audit ignore removed.

## Wednesday's plan (Scotty, 2026-11-16)

Keep it to these three, in order:

1. Warm-up: **Send to**. From any card, 'Send to' -> a deck -> Mainboard, Maybeboard or Sideboard, added without leaving the screen. Tap only, web and app.
2. Warm-up: **Unpriced warning**. A Warnings entry counts unpriced cards, collapsed by default; expanding shows them as chips; a chip opens the card's details drawer to pick a priced printing. '(N unpriced)' leaves the Budget rows.
3. The big one: **collection import** from ManaBox and Archidekt CSV, per `context/plans/collection.md`. Real exports are in zwipe-context `research/collection-samples/` (private; trim rows for tests here).

AI quick actions is parked on zwipe-context's idea list.

## Open for next week

- Android undo: one unreproduced maybeboard save that stayed saved after a burst (before 289331eae); watch for it.
- App write hand-off on leaving a screen: not reproduced on Android; the ready patch is zwipe-context `ideas/patches/app-undo-handoff.patch`.
- Budget's '(N unpriced)' wraps at phone width (25px loading jump): waits on a design idea; the composer suggested `$497.16+` plus a Warnings entry.
- Signup funnel: rerun `funnel.sql` to see whether the live password rules cut the retries (65 of 135 sessions retried before).
- zwiper-tui must drop zwipe-ui `about::Status` and `FAILED` when it merges.
- zynergy's RUSTSEC-2026-0459 ignore waits for a sqlx-sqlite release that allows libsqlite3-sys 0.39.
- Small About nits from surfer: zwipe.net's 390 gutter (347 vs 358 wide), panel top margin, the version link on zwipe.net only.
- Videos: Scotty disliked the No mouse and Vim previews; more shooting later, Vim re-recorded in its theme. Card stacks is final.
- Test account has 12 decks; big fat chud, satya, shorikai and Snake are probably video decks, kept.

## Agents

| Agent | Where | Role |
|---|---|---|
| composer | scotland-server | Triages, plans, merges `release` to `main`, keeps the Factory Board current. |
| halo | `herdr --machine halo`, pane `wX:p1` | Worker. Store builds run on halo. |
| laptop | `herdr --machine laptop`, pane `wV:p1` | Worker. Never pushes `release` or `main`: it pushes a branch and the composer merges. Video renders (ElevenLabs key on the laptop). |
| thinker | `herdr --machine thinker`, pane `w5:p1` | Worker. |
| scotland | local, `herdr agent prompt scotland` | Worker. |
| surfer | halo `wS:p1` | QA for app.zwipe.net, zwipe.net and the portfolio in its own Chrome tab. Keys only through the browser tool into its tab, never OS-level keystrokes. |
| droider | halo `wW:p1` | QA for Android in the emulator; video footage. |

Factory Board: https://claude.ai/artifact/HQVvAtuU45Ze4FZE3eN8iw (collections `items`, `meta/board`, `meta/team`). Update it with each status message, `updatedAt` from `date -u`.

## How the loop runs

- Workers gate locally (nightly fmt, clippy `-D warnings`, tests, rustdoc, wasm clippy) and push to `release`, rebasing if it moved. No PRs in zwipe. Laptop pushes a branch instead.
- The composer fast-forwards `main` from `release` without a second gate (the worker's check counts), checks the deploys in the background (zerver and zite only deploy on their own paths; trigger them by hand when a fix doesn't touch them), and sends QA to check the fix live. Never watch Actions in the foreground.
- orangui and portfolio: green worker PRs merge without asking; repin the hosts after an orangui merge. Cairn goes straight to its `main`, with every changelog entry mirrored in portfolio `src/changelog.rs`.
- A worker checking hosts against an orangui branch must confirm with `cargo tree -i orangui` that the patch applied. A CLI `--config` patch reaches `cargo check` but not `cargo clippy`.
- Workers stop and ask before any wire change; keep shipped apps' wire text and shapes as they are.
- Security fixes get no changelog line. User-visible fixes get one line in UPCOMING.
- A worker stuck on a permission prompt: Esc, move the cause into the repo's `target/`, re-prompt. Never approve a settings change.
- Scotty's standing limits: production DB reads are his; the keystore password is never written down; Droider never prints the test password; Surfer never types passwords on production; never sign into Scotty's profile; don't move or resize windows.
