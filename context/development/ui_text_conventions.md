# UI Text & Typography Conventions

How user-facing text is cased, which fonts to use, and the CSS reset patterns that keep them applied.

Established during the 2026-05-24 zwiper casing revamp (commits `b1cacaaf`, `4cbe57ea`).

---

## Casing

| Use case | Rule | Examples |
|---|---|---|
| Nav links, screen titles, section headings, card titles | **Title Case** | "Account", "Preferences", "Edit Deck", "Card Filters" |
| **Button labels & action CTAs (zwiper mobile)** | **Sentence case** (Apple HIG / Material convention) | "Log in", "Send reset link", "Save changes", "Back to login" |
| **Button labels & action CTAs (zite web)** | **Title Case** (marketing-page convention) | "Download on the App Store", "Set New Password" |
| Body text, descriptions, helper text, placeholders, captions | **Sentence case** | "Enter your email address.", "Choose a new password." |
| Form input labels | **Sentence case** ("Email address", "New password"); single proper nouns get Title Case ("Email", "Password") |
| **Backend-sourced text** (errors from `zwipe-core`, toasts derived from API responses, `Display` impls on newtypes) | **Display as-is — do NOT transform** | Whatever case the source produces is what's shown. Remove `.to_lowercase()` wrappers but do NOT re-case the source. |
| Frontend-authored toast/status text | **Sentence case** | "Card added to deck", "Saved" |
| Crate names | **lowercase preserved** | zerver, zwiper, zite, zwipe-core, zervice |
| Brand handles, URLs, code identifiers, CLI flags | **lowercase preserved** | scadoshi, scottyfermo.com, `UserId`, `sqlx` |
| Proper nouns | **Always capitalize** | Magic: The Gathering, Commander, Oathbreaker, Scryfall, Moxfield, Archidekt, iOS, Android, GitHub, PostgreSQL, Discord, Stripe, Resend |
| Hashtags / tag chips | **lowercase** (Twitter convention) | `#rust`, `#full-stack` |
| MTG card names | **preserve Scryfall casing** | "Sol Ring", "Lightning Bolt" — already cased correctly from the API |
| MTG format names | **Title Case** | "Standard", "Commander", "Oathbreaker" |
| Mana value | Use the words **"Mana value"** (or **"MV"** as a column abbreviation), never "CMC" — Wizards renamed it. The struct field is still `avg_cmc`/`cmc` (don't churn identifiers), only display strings change. |

### Why mobile and web split on button casing

zwiper is a mobile app — sentence case follows Apple HIG and Material Design conventions and reads more naturally on small screens. zite is a marketing site — Title Case fits the web/marketing genre and matches what Stripe/GitHub/Linear use for CTAs. The split is intentional.

### Backend-sourced text policy

`zwipe-core` produces text in whatever case the underlying `Display` impl, validation error, or API response chose (often lowercase by historical convention). The frontend must **not** transform this text — no `.to_lowercase()`, no `.to_uppercase()`, no `format!("{}", text.chars().next()...)` capitalization. Display it as-is. If a piece of `zwipe-core` text needs to be cased differently, fix it at the source (zwipe-core), not in the frontend.

This rule exists because the same backend message may surface in multiple frontends (zerver responses, zwiper toasts, zite forms, future CLI tools). One source of truth.

---

## Font: JetBrains Mono

The whole product (zwiper + zite) uses [JetBrains Mono](https://www.jetbrains.com/lp/mono/) at weight 400. Was previously Cascadia Code at weight 300. The two apps load it differently.

### zwiper: self-hosted

The woff2 files live in `zwiper/assets/fonts/` (400, 500, 700), are registered with `asset!()` so they ship inside the app bundle, and the `@font-face` rules are injected from `zwiper/src/bin/zwipe.rs`. No CDN: an app that has to work offline can't wait on jsdelivr, and the full font carries the U+2580-U+259F block elements the ASCII logo needs at the monospace advance width, which fontsource's subsets drop. `zwiper/assets/main.css:3` documents this at the top of the file.

### zite: CDN

`zite/assets/style.css` still imports from the `@fontsource/jetbrains-mono` CDN (top of file):

```css
@import url("https://cdn.jsdelivr.net/npm/@fontsource/jetbrains-mono@latest/400.css");
@import url("https://cdn.jsdelivr.net/npm/@fontsource/jetbrains-mono@latest/700.css");
```

### Body declaration

```css
body {
    font-family: 'JetBrains Mono', monospace;
    font-weight: 400;
}
```

Always name the font first with `monospace` as fallback. Never use bare `font-family: monospace` — see below.

### Form-element reset (zwiper)

Form elements (`<input>`, `<textarea>`, `<select>`, `<button>`) do NOT inherit `font-family` from body by default — the user-agent stylesheet hard-codes a system sans-serif. To opt them into body inheritance, `main.css` includes:

```css
input, textarea, select, button {
    font-family: inherit;
}
```

This is the standard CSS reset pattern (Tailwind preflight does the same). Without it, every form control needs its own `font-family` declaration, which inevitably drifts.

### `font-family: monospace` is a code smell

Bare `font-family: monospace` resolves to the OS generic — Menlo on macOS/iOS, Droid Sans Mono on Android, Consolas on Windows. **Never JetBrains Mono.** This was a real bug source: chart labels in `deck_charts.rs` and the import/export textareas were rendering in the OS default for months because they had inline `style="font-family:monospace"`.

**Always** either:
- `font-family: 'JetBrains Mono', monospace` — named font first, generic as fallback, OR
- omit `font-family` entirely and rely on body inheritance (preferred — single source of truth)

If you grep `font-family.*monospace` and find a result without `'JetBrains Mono'` in it, it's a bug.

---

## ASCII Logo Line Height

ASCII block-character art (the Z logo in zwiper home + zite nav) needs `line-height: 0.9` to remove horizontal gaps between rows. JetBrains Mono has taller vertical metrics than Cascadia Code at the same `font-size`, so the default `line-height: 1` leaves visible gaps in the block characters.

Applies to `.logo` (hero ASCII) and `.nav-logo` (tiny version in nav bar). Both in `zwiper/assets/main.css` and `zite/assets/style.css`.

---

## Theme Display Names

Theme slugs (`gruvbox`, `tokyo-night`, `rose-pine`, etc.) are stored as-is in `ALLOWED_THEMES` (zwipe-core) and drive CSS class names. For display in theme pickers, `display_theme_name(slug)` capitalizes each hyphen-split word.

**Special case:** `rose-pine` displays as **"Rosé Pine"** (accent on the first e). Hardcoded in the helper rather than renaming the slug because the slug drives CSS class names + DB stored values for every user with that theme selected. A display-only special-case is cheaper than a SQL migration + CSS rename. Same treatment for `vscode` → "VS Code", `github` → "GitHub", `synthwave-84` → "Synthwave '84", `powershell` → "PowerShell", `docs-rs` → "docs.rs".

The canonical copy is `zwipe-components/src/theme_picker.rs:21`, a `match` at the top of the function — add new overrides there. zite no longer has its own; it renders the shared `ThemePicker`. `zwiper/src/lib/inbound/screens/profile/preferences.rs:27` still holds a duplicate for its preferences sheet, so an override added to only one of the two will disagree across surfaces.

---

## When You Add a New Screen / Component

1. **Headings** (`h1`/`h2`/`h3`, page titles, section labels) → Title Case
2. **Buttons** → sentence case in zwiper, Title Case in zite
3. **Form labels & placeholders** → sentence case ("Email address"), Title Case for single proper nouns ("Email")
4. **Toasts you author** → sentence case, no trailing period for short status ("Card added"), period for full sentences ("Verify your email to enable password recovery.")
5. **Toasts derived from `e.to_user_message()` / `e.to_string()`** → pass through, no `.to_lowercase()` wrappers
6. **Don't add inline `font-family`** — body inheritance handles it (form elements covered by the reset)
7. **Avoid `style="font-family:monospace"`** — see above
