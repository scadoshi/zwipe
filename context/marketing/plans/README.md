# Marketing video plans

Short vertical videos (TikTok, Reels, Shorts), one file per video. Each is built by the video pipeline in `~/Developer/video-factory` (not in this repo): a spec per video, Droid's screen recordings, Callum's narration from ElevenLabs, a music bed, captions, and the end card.

## Format

- **9:16, 1080x1920, 30 fps.**
- **10-20s of beats, then the end card** with the closing line read over it (about 7s). Four or five beats, one feature each, hard cuts, fast and a little erratic. Clips play at 1-2x to fit their line.
- **End card:** `../video_end_card.png` (Zwipe wordmark, "The Magic: The Gathering deck builder built for mobile and web", Free / No ads / iOS / Android / Web, zwipe.net).
- **Captions:** the narration, sentence by sentence as it's spoken, in the bottom third; the hook sits at the top over the first beat. No em dashes, sentence case, "Zwipe" capitalized.
- **Two videos at a time,** so each pair gets room for back-and-forth before the next.

## Rules for every video

- **Say it's Magic in the first beat.** The first line says "Magic" (or "Magic" with a clear MTG cue such as commander), and the hook caption says "Magic: The Gathering", so a viewer knows within the first 2-3 seconds. The closing line says it again.
- **Narrator:** Callum (ElevenLabs), eleven_v3, stability 0.0, speed 1.15, read as one take. The mix delivery: `[dramatically]` on the opening line, then `[shouts]` and `[serious]` where the last beat turns (for example "[shouts] Gather your cards! Build your army! [serious] One swipe at a time."). A mood tag such as `[curious]` may stand in for `[dramatically]` where it fits the video. No other tags: they're audible. Energy otherwise comes from speed and punctuation.
- **"Zwipe" is spoken "Za-wipe"** (two beats, zuh-WIPE): the narration is sent that respelling; captions keep "Zwipe".
- **Closing line, every video:** "Zwipe. The Magic: The Gathering deck builder. Free, no ads, on iOS, Android and web."
- **Music:** about 25s from ElevenLabs music, instrumental, a mood per video (each plan names it), leading silence trimmed, ducked under Callum and faded over the end card.
- **Real app, real numbers.** Film the current build; never mock a figure the narration reacts to.

## Slate

| # | File | Video | Soundtrack | Status |
|---|---|---|---|---|
| 1 | [01_swipe_to_build.md](01_swipe_to_build.md) | Swipe to build | Epic boss battle | Done (v3); v4 to open with Magic |
| 2 | [02_deck_stats.md](02_deck_stats.md) | Deck stats | Epic, darker | Current pair, recording |
| 3 | [03_playstyle.md](03_playstyle.md) | Playstyle | Heroic | Current pair, recording |
| 4 | [04_oracle_tags.md](04_oracle_tags.md) | Oracle tags | Curious | Next pair; spec written |
| 5 | [05_activity.md](05_activity.md) | Activity | Driving, rhythmic | Planned |
| 6 | [06_know_every_card.md](06_know_every_card.md) | Know every card | Mysterious, arcane | Planned |
| 7 | [07_share_your_deck.md](07_share_your_deck.md) | Share your deck | Upbeat, bright | Planned |
| 8 | [08_pick_your_commander.md](08_pick_your_commander.md) | Pick your commander | Epic, regal | Planned |
| 9 | [09_import.md](09_import.md) | Import | Quick, mechanical | Planned |
| 10 | [10_web_app.md](10_web_app.md) | The web app | Modern, confident | Planned |
| 11 | (spec only) goblin-build | A Krenko goblin deck from nothing | Creepy (Goblin) | Spec written, paused |
| 12 | (spec only) goblin-invasion | Import, the goblin way | Creepy (Goblin) | Spec written, paused |
| 13 | (spec only) commander-maybeboard | Save commanders for later | Whimsical (Mouse Knight) | Spec written, paused |
| 14 | (spec only) rules-keeper | The rules, none mandatory | Ceremonial (Rules Keeper) | Spec written, paused |
| 15 | (spec only) card-deep-dive | Mini-series 1: one card dug into | Curious (Wizard) | Spec written, paused |
| 16 | (spec only) teach-the-algorithm | Mini-series 2: the stack learns | Curious (Wizard) | Spec written, paused |
| 17 | (spec only) synergy-explained | Mini-series 3: what Synergy weighs | Curious (Wizard) | Spec written, paused |
| 18 | (spec only) remove-cards | Guide: Remove cards from a deck | Epic (Ogre) | Spec written |
| 19 | (spec only) swipe-memory | Guide: Swipe memory | Creepy (Dark Sorcerer) | Spec written |
| 20 | (spec only) filtering | Guide: Filter the card pool | Curious (Wizard) | Spec written |
| 21 | (spec only) organize-and-browse | Guide: Group, filter, and browse your lists | Whimsical (Gnome) | Spec written |
| 22 | (spec only) commander-swipe | Guide: Choose a commander and format | Epic (Ogre) | Spec written |
| 23 | (spec only) budgeting | Guide: Set a budget with price targets | Ceremonial (Rules Keeper) | Spec written |
| 24 | (spec only) land-targets | Guide: Set a land target | Ceremonial (Rules Keeper) | Spec written |
| 25 | (spec only) deck-tags | Guide: Tag decks by archetype | Curious (Wizard) | Spec written |
| 26 | (spec only) deck-mvps | Guide: Star your deck's MVPs | Heroic (Heroic Knight) | Spec written |
| 27 | (spec only) share-your-deck | Guide: Share a deck with a link | Heroic (Heroic Knight) | Spec written |
| 28 | (spec only) import-export | Guide: Import and export decklists | Creepy (Goblin) | Spec written |

Rows 11-28 have no plan file yet: their spec and shot list live in `~/Developer/video-factory/videos/<slug>/` (spec.yaml, shots.md). Rows 18-28 come from zite's guides (the How-to pages), one video per guide that had none, each script built on its guide's text. Guides already covered: Getting started and Build a deck by swiping (01), Deck stats (02), Playstyle (03), the Oracle tags guides (04), Activity (05), Read a card at a glance (15), Synergy-ranked cards (17), Save commanders for later (13); Deck tags, card roles, and Oracle tags is covered by Oracle tags explained. Some overlap the plans above: 07 Share your deck with 26-27, 08 Pick your commander with 22, 09 Import with 12 and 28. Voices are Scotty's custom ElevenLabs voices (Mouse Knight, Gnome, Rules Keeper, Wizard, Goblin, Ogre, Dark Sorcerer, Heroic Knight and the rest); the mapping lives in video-factory's voices.yaml.

## Where the old plans went

The earlier slate (videos 02-11, in git history) was folded into this one:

- 02 (it gets out of your way: swipe memory, stack resume): background in Swipe to build, not a beat.
- 03 (land and budget targets) and 04 (draw odds): Deck stats.
- 05 (Synergy): Pick your commander.
- 06 (deck profile: tags, brackets, archetype seeding, themes): retired; archetype-to-tag seeding is an optional cut-in for Oracle tags.
- 07 (know every card): Know every card, without tag definitions.
- 08 (share page) and 09 (MVPs): Share your deck.
- 10 (pick your commander): Pick your commander.
- 11 (oracle tags): Oracle tags.
