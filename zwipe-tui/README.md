# zwipe-tui

ZWIPE in the terminal. Sign in with your ZWIPE account, pick a deck, and swipe cards into it with the keyboard. It serves the same cards as the app, in the same order: your commander's synergy picks first, then cards like your deck.

## Install

```bash
cargo install --git https://github.com/scadoshi/zwipe zwipe-tui
zwipe-tui
```

Needs Rust 1.88 or newer. Decks are made in the app; zwipe-tui builds them.

## Keys

| Screen | Key | Action |
| --- | --- | --- |
| Sign in | `Tab` | Switch between username and password |
| | `Enter` | Sign in |
| Decks | `j` / `k` or arrows | Move |
| | `Enter` | Swipe cards into the deck |
| | `r` | Reload |
| | `O` | Sign out |
| Swipe | `l` or right | Add to the deck |
| | `h` or left | Pass; the deck won't be served it again |
| | `m` or up | Add to the maybeboard |
| | `u` or down | Undo the last swipe |
| | `Esc` | Back to decks |
| Anywhere | `Ctrl+C` | Quit |

## Theme

zwipe-tui draws in the theme set on your account, dark or light, with each theme's own mana colors. Before you sign in it uses your terminal's colors. The themes are 24-bit color, which iTerm2, Ghostty, kitty, WezTerm and current Terminal.app show correctly; inside tmux, turn on its truecolor support.

## Session

Signing in saves the session to `zwipe-tui/session.json` in your config folder (`~/Library/Application Support` on macOS, `~/.config` on Linux), readable by your user only. It refreshes itself like the app's and lasts until you sign out with `O` or the server ends it.

## Development

`ZWIPE_BACKEND_URL` points it at another server, such as `http://127.0.0.1:3000` for a local zerver. Anything but this machine must use https.

The theme palettes in `src/lib/domain/theme/palettes.rs` are generated from orangui's `assets/themes.css` by `scripts/gen_themes.py`; a test fails when they drift from zwipe-core's `ALLOWED_THEMES`.
