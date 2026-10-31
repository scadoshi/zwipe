# zwiper-tui

ZWIPE in the terminal. Sign in with your ZWIPE account to see the home screen's counters, color map and featured flavor card, look through your decks and their charts, and swipe cards into them with the keyboard. It serves the same cards as the app, in the same order: your commander's synergy picks first, then cards like your deck.

## Install

```bash
cargo install --git https://github.com/scadoshi/zwipe zwiper-tui
zwiper-tui
```

Needs Rust 1.88 or newer. Decks are made in the app; zwiper-tui builds them.

## Keys

| Screen | Key | Action |
| --- | --- | --- |
| Sign in | `Tab` | Switch between username and password |
| | `Enter` | Sign in |
| Home | `d` | Your decks |
| | `a` | Account |
| | `c` | Config |
| | `s` | Switch the counters between everybody and only you |
| | `r` | Refresh |
| | `O` | Sign out |
| Decks | `j` / `k` or arrows | Move |
| | `Enter` | Open the deck |
| | `r` | Reload |
| | `Esc` | Home |
| Deck | `Tab` / `Shift+Tab` | Next or previous section: budget, tags, distributions, mana, draw odds, warnings |
| | `c` | Switch the budget between USD, EUR and TIX |
| | `+` / `-` | Draw odds one turn later or earlier |
| | `s` | Swipe cards into the deck |
| | `Esc` | Back to decks |
| Swipe | `l` or right | Add to the deck |
| | `h` or left | Pass; the deck won't be served it again |
| | `m` or up | Add to the maybeboard |
| | `u` or down | Undo the last swipe |
| | `i` | Switch between the card's image and its text, when the terminal is too narrow for both |
| | `Esc` | Back to the deck |
| Account | `j` / `k`, `Enter` | Pick a row: change your username, email or password, resend or check verification, log out, or delete the account |
| | `Tab`, `Enter`, `Esc` | In a form: next field, save, back |
| Config | `j` / `k`, `Enter` | Pick a row: theme, dark or light, pip colors, Universes Beyond and its exceptions |
| | `j` / `k`, `Enter`, `Esc` | In the theme list: preview, save, put the old theme back |
| | `Space`, `x`, `Enter` | In the exceptions list: pick, clear, save |
| Anywhere | `q` | Quit, except while typing |
| | `Ctrl+C` | Quit |

## Theme

zwiper-tui draws in the theme set on your account, dark or light, with each theme's own mana colors or Magic's natural ones. Config changes it the same way the app does, so a theme picked here shows on your phone too, and the other way round. It remembers that theme for the sign-in screen next time; before any sign-in it uses gruvbox dark, like the app. The themes are 24-bit color, which iTerm2, Ghostty, kitty, WezTerm and current Terminal.app show correctly; inside tmux, turn on its truecolor support.

## Card images

The swipe screen shows each card's Scryfall image beside its text. Ghostty, kitty and WezTerm draw them with the kitty graphics protocol and iTerm2 with its own, at full quality; other terminals get a blockier version made of half-height cells. Images come only from Scryfall's image CDN, a few cards ahead of the one on screen. Set `ZWIPER_TUI_IMAGES=off` to turn them off, which is worth doing inside tmux unless its passthrough is set up.

## Session

Signing in saves the session to `zwiper-tui/session.json` in your config folder (`~/Library/Application Support` on macOS, `~/.config` on Linux), readable by your user only. It refreshes itself like the app's and lasts until you sign out with `O` or the server ends it.

## Development

`ZWIPE_BACKEND_URL` points it at another server, such as `http://127.0.0.1:3000` for a local zerver. Anything but this machine must use https.

The theme palettes in `src/lib/domain/theme/palettes.rs` are generated from orangui's `assets/themes.css` by `scripts/gen_themes.py`; a test fails when they drift from zwipe-core's `ALLOWED_THEMES`.
