# Session 4 — Frontend (CSS Themes, Client, Preferences Screen)

Depends on: Session 3 (JWT must carry preferences before frontend can use them)

---

## 1. CSS Themes

**Modify:** `zwiper/assets/main.css`

Port all 9 theme definitions from `portfolio/assets/main.css`. Each theme needs a class
that overrides Zwipe's existing CSS variables.

### Variable mapping (portfolio → Zwipe)

| Portfolio variable | Zwipe variable | Usage |
|-------------------|---------------|-------|
| `--bg0-h` | `--bg-primary` | Main background |
| `--bg0` | (same or alias) | Alternate bg |
| `--bg1` | `--border-secondary` | Borders, cards |
| `--bg2` | `--border-muted` | Subtler borders |
| `--bg3` | `--border-primary` | Prominent borders |
| `--fg0` | `--text-primary` | Main text |
| `--fg2` | `--text-subtle` | Subtle text |
| `--fg3` | `--text-muted` | Muted text |
| `--red` | `--color-error` | Error states |
| `--green` | `--color-success` | Success states |
| `--yellow` | `--color-warning` | Warning states |
| `--red` (dimmed) | `--border-error` | Error borders |
| `--green` (dimmed) | `--border-success` | Success borders |
| `--yellow` (dimmed) | `--border-warning` | Warning borders |

### CSS class pattern

Use `.theme-{name}` classes that override the `:root` variables:

```css
/* Default (zwipe) is already in :root — no override needed */

.theme-rustbox-dark {
    --bg-primary: #211a16;
    --bg-overlay: rgba(0, 0, 0, 0.6);
    --text-primary: #f0dcc8;
    --text-muted: #a8896a;
    --text-subtle: #c9ad8f;
    --border-primary: #4d3d35;
    --border-secondary: #2e2420;
    --border-muted: #3d302a;
    --color-error: #c4432b;
    --color-success: #8a9a5b;
    --color-warning: #d4943a;
    --border-error: #8b3030;
    --border-success: #5a6a3a;
    --border-warning: #8a7020;
    --shadow-sm: 0 4px 12px rgba(0, 0, 0, 0.3);
    --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.5);
}

.theme-rustbox-light {
    --bg-primary: #f0e6da;
    --text-primary: #2a1f18;
    --text-muted: #6b5545;
    /* ... etc for all variables ... */
}

/* Repeat for all 9 themes (dark + light where applicable) */
/* zwipe and vantablack: dark only */
```

Generate all theme classes by mapping the portfolio's color values to Zwipe's variable
names. There are 9 themes, 7 with light variants = 16 total CSS blocks.

### Where to put them

Add a new section in `main.css` at the top, after the existing `:root` block:

```css
/* ==================
        themes
 ==================== */

/* Default theme (zwipe) is in :root above */

.theme-rustbox-dark { ... }
.theme-rustbox-light { ... }
/* ... etc ... */
```

---

## 2. Theme Config Type

**Create or modify:** a theme config in the frontend domain.

```rust
#[derive(Clone, PartialEq)]
pub struct ThemeConfig {
    pub name: String,
    pub is_dark: bool,
}

impl ThemeConfig {
    pub fn css_class(&self) -> String {
        if self.name == "zwipe" || self.name == "vantablack" {
            format!("theme-{}", self.name)
        } else {
            let mode = if self.is_dark { "dark" } else { "light" };
            format!("theme-{}-{}", self.name, mode)
        }
    }

    pub fn has_light_mode(&self) -> bool {
        self.name != "zwipe" && self.name != "vantablack"
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self { name: "zwipe".to_string(), is_dark: true }
    }
}
```

This can live in `zwiper/src/lib/domain/` or a new `theme.rs` module.

---

## 3. App Root — Theme Context Provider

**Modify:** `zwiper/src/bin/zwipe.rs`

Provide a `ThemeConfig` signal via context, initialized from session preferences:

```rust
let theme: Signal<ThemeConfig> = use_signal(ThemeConfig::default);
use_context_provider(|| theme);
```

**Modify:** the screen layout wrapper (likely in router or main app) to apply the theme
class to the outermost div:

```rust
let theme: Signal<ThemeConfig> = use_context();
let theme_class = theme.read().css_class();

div { class: "{theme_class}",
    // ... app content ...
}
```

### Initialize from session on login

In the login success handler (`screens/auth/login.rs`), after setting the session:

```rust
Ok(new_session) => {
    // Update theme from preferences
    let mut theme: Signal<ThemeConfig> = use_context();
    theme.set(ThemeConfig {
        name: new_session.preferences.theme.clone(),
        is_dark: new_session.preferences.dark_mode,
    });
    // ... existing login logic ...
}
```

Same pattern in the session upkeep/refresh flow.

---

## 4. Client Traits

**Create:** `zwiper/src/lib/outbound/client/user/preferences.rs`

```rust
use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::{
        auth::models::session::Session,
        user::models::preferences::UserPreferences,
    },
    inbound::http::{
        handlers::user::update_preferences::HttpUpdatePreferences,
        routes::preferences_route,
        ApiError,
    },
};

pub trait ClientGetPreferences {
    fn get_preferences(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<UserPreferences, ApiError>> + Send;
}

pub trait ClientUpdatePreferences {
    fn update_preferences(
        &self,
        request: HttpUpdatePreferences,
        session: &Session,
    ) -> impl Future<Output = Result<UserPreferences, ApiError>> + Send;
}

impl ClientGetPreferences for ZwipeClient {
    async fn get_preferences(&self, session: &Session) -> Result<UserPreferences, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&preferences_route());
        let response = self
            .client
            .get(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}

impl ClientUpdatePreferences for ZwipeClient {
    async fn update_preferences(
        &self,
        request: HttpUpdatePreferences,
        session: &Session,
    ) -> Result<UserPreferences, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&preferences_route());
        let response = self
            .client
            .put(url)
            .json(&request)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
```

**Modify:** `zwiper/src/lib/outbound/client/user/mod.rs` — add `pub mod preferences;`

---

## 5. Preferences Screen

**Create:** `zwiper/src/lib/inbound/screens/profile/preferences.rs`

### Layout

```
┌──────────────────────────┐
│      preferences         │  ← page header
├──────────────────────────┤
│  theme                   │
│  ┌────────────────────┐  │
│  │ rustbox        [x] │  │  ← selected theme has checkmark
│  │ gruvbox            │  │
│  │ dracula            │  │
│  │ everforest         │  │
│  │ catppuccin         │  │
│  │ tokyo night        │  │
│  │ nord               │  │
│  │ zwipe          [x] │  │
│  │ vantablack         │  │
│  └────────────────────┘  │
│                          │
│  dark mode     [toggle]  │  ← disabled for zwipe/vantablack
│                          │
├──────────────────────────┤
│  [back]        [save]    │  ← util-bar
└──────────────────────────┘
```

### Component structure

```rust
#[component]
pub fn Preferences() -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let mut theme_config: Signal<ThemeConfig> = use_context();
    let toast = use_toast();

    // Local state — initialized from current theme
    let mut selected_theme = use_signal(|| theme_config.read().name.clone());
    let mut selected_dark = use_signal(|| theme_config.read().is_dark);
    let mut is_loading = use_signal(|| false);

    let themes = vec![
        "rustbox", "gruvbox", "dracula", "everforest", "catppuccin",
        "tokyo-night", "nord", "zwipe", "vantablack",
    ];

    let is_dark_only = move || {
        let t = selected_theme();
        t == "zwipe" || t == "vantablack"
    };

    let mut save = move || {
        is_loading.set(true);
        let request = HttpUpdatePreferences {
            theme: selected_theme(),
            dark_mode: if is_dark_only() { true } else { selected_dark() },
        };
        spawn(async move {
            session.upkeep(client);
            let Some(session_val) = session() else {
                toast.error("session expired", ...);
                is_loading.set(false);
                return;
            };
            match client().update_preferences(request, &session_val).await {
                Ok(prefs) => {
                    theme_config.set(ThemeConfig {
                        name: prefs.theme,
                        is_dark: prefs.dark_mode,
                    });
                    toast.success("preferences saved", ...);
                    is_loading.set(false);
                }
                Err(e) => {
                    toast.error(e.to_user_message(), ...);
                    is_loading.set(false);
                }
            }
        });
    };

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "preferences" }
                }
                div { class: "screen-content centered content-enter",
                    div { class: "container-sm",
                        // Theme list
                        for theme in themes {
                            button {
                                class: if selected_theme() == theme { "pref-row selected" } else { "pref-row" },
                                onclick: move |_| {
                                    selected_theme.set(theme.to_string());
                                    // Force dark mode for dark-only themes
                                    if theme == "zwipe" || theme == "vantablack" {
                                        selected_dark.set(true);
                                    }
                                },
                                "{theme}"
                            }
                        }
                        // Dark mode toggle
                        // ... toggle button, disabled when is_dark_only()
                    }
                }
                div { class: "util-bar",
                    button { class: "util-btn", onclick: move |_| navigator.go_back(), "back" }
                    button {
                        class: "util-btn",
                        disabled: is_loading(),
                        onclick: move |_| save(),
                        if is_loading() { "saving..." } else { "save" }
                    }
                }
            }
        }
    }
}
```

### CSS for preference rows

Add minimal CSS to `main.css`:

```css
.pref-row {
    display: block;
    width: 100%;
    padding: 0.6rem 1rem;
    background: transparent;
    border: 1px solid var(--border-secondary);
    border-radius: 0.4rem;
    color: var(--text-muted);
    font-family: "Cascadia Code", monospace;
    font-weight: 300;
    font-size: 0.85rem;
    text-align: left;
    margin-bottom: 0.4rem;
    cursor: pointer;
}

.pref-row.selected {
    border-color: var(--border-primary);
    color: var(--text-primary);
}
```

---

## 6. Router & Navigation

**Modify:** `zwiper/src/lib/inbound/router.rs` — add:

```rust
#[route("/preferences")]
Preferences {},
```

**Modify:** `zwiper/src/lib/inbound/screens/profile/mod.rs` — add a button in the
profile screen to navigate to preferences:

```rust
button {
    class: "profile-action-btn",
    onclick: move |_| navigator.push(Router::Preferences {}),
    "preferences"
}
```

---

## After this session

1. `dx serve` — test locally
2. Select each theme — verify colors change
3. Toggle light/dark — verify it works (and is disabled for zwipe/vantablack)
4. Save — verify toast appears and theme persists
5. Log out, log back in — theme should apply from JWT immediately
6. Test on mobile viewport — preferences screen should be usable
7. `cargo sqlx prepare --workspace` — commit `.sqlx/`
