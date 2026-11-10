# 324. Released apps can't change pip style: the server refuses theme and natural

State: reported | Owner: thinker | PR: - | Build seen: bd233324a

## Report
Scotty, 2026-11-09, on a 1.18.16 app build: changing the pip style fails with "Failed to deserialize the JSON body ... pip_style: unknown variant". Droid confirmed it the same day on an older app build, in both directions.

## Repro
1. On an app build from 1.18.16 or earlier, signed in against api.zwipe.net (Droid used an emulator debug build at 1fd04e479, versionName 1.18.14, as test), open Settings.
2. Tap Pips to switch Themed to Classic. A toast says "Failed to deserialize the JSON body into the target type: pip_style: unknown pip color at line 1 column 119", and the setting stays Themed. Logcat has `update_preferences failed:` with the same text.
3. With Classic set from the web, switch Classic to Themed in the app: the same toast appears, at column 117, and the setting stays Classic.

Reading still works: the app shows whatever the server holds. Live web (bd233324a, v1.18.16) switches both ways with no error, and zwipe.net's own pip toggle is unaffected.

## Cause
Up to and including the 1.18.16 cut (e1bd855d2), `PipStyle` writes the old wire names, `theme` and `natural`, so every released app sends them. Pip style Phase 4 (#491, #492), deployed in bd233324a, makes requests take only `themed` and `classic`, and the old names hit "unknown pip color" (zwipe-core/src/domain/user/models/settings.rs). The server stopped accepting the old names before any released client had started writing the new ones.

## Fix
In progress (thinker): the server accepts the old names in requests again, until store builds that write `themed` and `classic` have replaced the old ones.
