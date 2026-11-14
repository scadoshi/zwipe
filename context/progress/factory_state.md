# Factory state

The handoff for the composer session: where the factory stands, so a fresh or compacted composer can pick it up. The rules are in [`../development/factory.md`](../development/factory.md). Open issues are in zwipe-context's GitHub issues and [`../issues/`](../issues/README.md). Older handoff logs are in this file's git history.

## As of 2026-11-14: stopped

Scotty stopped the factory on 2026-11-14 after a wind-down. Nothing is in flight, every agent is standing down, and no builds, dev servers or test sessions are running. Start nothing until Scotty says.

- `main` = `release` at `f2c707615` (orangui repinned to `0a589a3`), deployed green: web, zerver, zite. Web is 1.21.4. UPCOMING 1.21.5 holds the 517 Shift+G, 518 Printings and #47 cheapest-printing lines.
- Stores: Zwipe 1.21.4 and Cairn 1.4.2 submitted 2026-11-13. Cairn's Unreleased 1.4.3 has the centered menu icons (also in portfolio's changelog).
- orangui main `0a589a3`, pinned in zwipe, portfolio (`fe77a09`) and Cairn (`357c8ec`).
- Next phase (Scotty): marketing and signups. The web app is settled; AI features come later. No nit work.

## Done in the last stretch (2026-11-12 to 11-14)

- 1.21.4 cut and submitted to both stores, with Cairn 1.4.2.
- Security fixes #43 (text criteria capped, search statement timeout), #44 (cookie logout), #46 (ACAO on /api/changelog). No changelog lines, by rule.
- scadoshi-rust audit closed: zwipe-context #29 to #42, orangui #40 to #43. `clippy::wildcard_enum_match_arm` is on in zwipe and orangui and in the scadoshi-rust lint set (one-variant picks use `if let` or `matches!`; foreign enums take an `#[expect]`).
- Verified live by Surfer: #47 (Quick add and import skip a printing whose USD is under a tenth of its EUR), #48 (no half-faded cursor image), 517 reopen (Shift+G with a lowercase key), portfolio #49 and the fleet chart no longer replaying on metric switches.
- Laptop's leftover branches pruned (13 deleted); the list is in zwipe-context `ideas/laptop_branches_2026-11-14.md`.
- Five QoL videos approved in zwipe-context `video-factory/finals/not-scheduled/`: Share your deck, Undo, Themes, Pick your commander, Quick add.

## Parked

- Themes: Scotty records the theme sweeps on his iPhone, then the video is re-cut with them.
- The three web videos (vim bindings, card stacks, no mouse): parked by Scotty. Shot lists in zwipe-context `marketing/plans/31_scotty_shot_lists.md`.
- `feat/tui` (zwipe-context #50): extra large, kept.
- Branches kept on purpose: settings phase 3, pip-color-classic, rustmas `scadoshi`.
- Scotty to re-upload `scadoshi-rust/dist/scadoshi-rust.zip` to claude.ai so agents load the two new match rules.
- zwiper's iOS and Android cfg code wasn't checked by the new lint (it doesn't compile on a Mac desktop); the next mobile build checks it.

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
- The composer fast-forwards `main` from `release` (`git push origin origin/release:main`) only after a full gate passed on the exact `release` head, and only once no `main` deploy is running. Then it waits for deploy-web, deploy-zerver and deploy-zite and sends QA to check the fix live.
- orangui and portfolio: green worker PRs merge without asking; repin the hosts after an orangui merge. Cairn goes straight to its `main`, with every changelog entry mirrored in portfolio `src/changelog.rs`.
- A worker checking hosts against an orangui branch must confirm with `cargo tree -i orangui` that the patch applied. A CLI `--config` patch reaches `cargo check` but not `cargo clippy`.
- Workers stop and ask before any wire change; keep shipped apps' wire text and shapes as they are.
- Security fixes get no changelog line. User-visible fixes get one line in UPCOMING.
- A worker stuck on a permission prompt: Esc, move the cause into the repo's `target/`, re-prompt. Never approve a settings change.
- Scotty's standing limits: production DB reads are his; the keystore password is never written down; Droider never prints the test password; Surfer never types passwords on production; never sign into Scotty's profile; don't move or resize windows.
