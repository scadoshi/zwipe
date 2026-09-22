# 1.10.2 release notes

One text, both stores. 496 characters, 4 to spare.

```
- Clearing a filter on the add screen works. Apply used to refuse an empty filter and put the old one back, so once one was on there was no way off.
- Applying a filter you haven't changed leaves the card stack where it is. Refresh still deals a new one.
- Shaking the phone no longer opens the system Undo prompt. Undo is still a downward swipe.
- When the filter lists can't load, the app says so and tries again.
- The leader strip on a deck holds its height instead of jumping as images load.
```

Drafted from the `UPCOMING` block in
`zwipe-core/src/content/changelog/mod.rs`. Says "leader" rather than the
in-app "command zone" so one text clears Play's copycat rule.
