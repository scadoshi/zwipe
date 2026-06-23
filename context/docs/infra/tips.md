# Tips & Gotchas

Operational notes that don't fit neatly into other docs — things that will waste your
time if you forget them.

---

## Ghostty Terminal: `Error opening terminal: xterm-ghostty`

When SSHing into the server from [Ghostty](https://ghostty.org/), interactive terminal
programs (`nano`, `visudo`, `less`, `systemctl edit`, etc.) fail with:

```
Error opening terminal: xterm-ghostty.
```

The server doesn't have the `xterm-ghostty` terminfo entry. Fix: override `TERM` for
the specific command:

```bash
TERM=xterm-256color sudo visudo
TERM=xterm-256color sudo nano /etc/some/file
TERM=xterm-256color sudo systemctl edit some.service
```

**Do not use `xterm-256-color`** (with a hyphen before "color") — that also fails.
The correct value is `xterm-256color`.

You only need the prefix for interactive terminal programs. Regular commands (`cp`,
`systemctl start`, `cargo build`, etc.) are unaffected.
