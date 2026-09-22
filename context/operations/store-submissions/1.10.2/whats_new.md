# 1.10.2 release notes

One text, both stores. 485 characters, 15 to spare.

```
- Clearing a filter on the add screen works. Apply used to refuse an empty one, so there was no way off once set.
- Applying an unchanged filter leaves the card stack alone. Refresh still deals a new one.
- Shaking the phone no longer opens the system Undo prompt. Undo is still swipe down.
- When the filter lists can't load, the app says so and retries.
- A deck's command zone strip holds its height as images load.
- The welcome greets you once at startup, not on every visit home.
```

Drafted from the 1.10.2 block in
`zwipe-core/src/content/changelog/mod.rs`, compressed to fit Play's cap. All
six changelog entries are represented; the in-app copy carries the fuller
wording. Uses the in-app term "command zone" rather than working around it.

Kept a margin rather than writing to exactly 500: the two consoles need not
agree on how they count a newline, and a rejection at paste time is a bad
place to find out.
