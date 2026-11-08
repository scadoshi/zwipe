# Share preview assets

What zerver draws a shared deck's preview image with (`inbound/http/share_preview`). Everything here is embedded in the binary with `include_bytes!`/`include_str!`.

| File | Source | License |
|---|---|---|
| `JetBrainsMono-Regular.ttf`, `JetBrainsMono-Bold.ttf` | [JetBrains Mono](https://github.com/JetBrains/JetBrainsMono), `fonts/ttf` on `master` | SIL OFL 1.1, `OFL-JetBrainsMono.txt` |
| `mana.ttf` | [Mana](https://github.com/andrewgioia/mana) 1.18.0, the `mana-font` npm package's `fonts/mana.ttf`, the same font the app's pips use | SIL OFL 1.1, `OFL-Mana.txt` |
| `themes.css` | orangui's `assets/themes.css` at the commit `Cargo.lock` pins | orangui's own; a byte-for-byte copy |

The mana symbols themselves are copyright Wizards of the Coast, shown under the Fan Content Policy like everywhere else in the app.

`themes.css` must stay identical to `orangui::THEMES_CSS`: `components`' test `the_share_preview_themes_match_orangui` fails when `cargo update -p orangui` moves the pin and this copy falls behind. Copy the new file over when it does.
