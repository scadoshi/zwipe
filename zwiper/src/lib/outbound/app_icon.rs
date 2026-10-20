//! The home-screen icon: which theme's colors it is drawn in, dark or light.
//!
//! iOS keeps the choice itself, so nothing is stored here or on the account:
//! every read asks the OS for its alternate icon name. Android has no
//! alternate icons, so the setting is hidden there. Desktop holds the pick for
//! the session so the sheet behaves the same in development.
//!
//! The icon can instead follow the app's theme. That flag belongs to the
//! device, not the account: iOS keeps it in the keychain beside the cached
//! theme, and desktop holds it for the session like the pick.
//!
//! `zcripts/ios/icon.py` renders an icon per theme and mode into
//! `zwiper/assets/app-icons/`, and `zcripts/ios/icons.sh` bundles each as an
//! alternate icon named `AppIcon-<theme>-<mode>`.

use ::components::ThemeConfig;

/// The theme the bundled primary icon is drawn in, in dark mode.
pub const PRIMARY: &str = "gruvbox";

/// Whether this platform can change its icon.
pub const SUPPORTED: bool = cfg!(not(target_os = "android"));

/// The prefix `zcripts/ios/icons.sh` gives each alternate icon.
const PREFIX: &str = "AppIcon-";

/// The bundled primary icon.
pub fn primary() -> ThemeConfig {
    ThemeConfig {
        name: PRIMARY.to_string(),
        is_dark: true,
    }
}

/// The theme and mode the icon is drawn in now.
pub fn current() -> ThemeConfig {
    platform::current()
        .and_then(|name| parse(&name))
        .unwrap_or_else(primary)
}

/// Switches the icon to `icon`'s colors. On iOS the OS shows its own alert
/// saying the icon changed.
pub fn set(icon: &ThemeConfig) -> anyhow::Result<()> {
    platform::set(alternate_name(icon).as_deref())
}

/// Whether the icon follows the app's theme on this device.
pub fn follows_theme() -> bool {
    SUPPORTED && platform::follows_theme()
}

/// Turns following the app's theme on or off for this device.
pub fn set_follows_theme(on: bool) -> anyhow::Result<()> {
    platform::set_follows_theme(on)
}

/// Switches the icon to `theme` when it follows the theme and shows another.
/// Called only with a theme the account holds, never a sheet's preview.
pub fn follow(theme: &ThemeConfig) {
    if !follows_theme() || current() == *theme {
        return;
    }
    if let Err(e) = set(theme) {
        tracing::error!("failed to match app icon to theme: {e}");
    }
}

/// "dark" or "light", as the icon files and bundle names spell it.
pub fn mode(is_dark: bool) -> &'static str {
    if is_dark { "dark" } else { "light" }
}

/// The alternate icon's bundle name, or `None` for the primary icon.
fn alternate_name(icon: &ThemeConfig) -> Option<String> {
    (*icon != primary()).then(|| format!("{PREFIX}{}-{}", icon.name, mode(icon.is_dark)))
}

/// The icon an alternate's bundle name stands for.
fn parse(name: &str) -> Option<ThemeConfig> {
    let (theme, mode) = name.strip_prefix(PREFIX)?.rsplit_once('-')?;
    let is_dark = match mode {
        "dark" => true,
        "light" => false,
        _ => return None,
    };
    Some(ThemeConfig {
        name: theme.to_string(),
        is_dark,
    })
}

// iOS: UIApplication's alternate icon API.
#[cfg(target_os = "ios")]
mod platform {
    use objc2::{
        Encode, Encoding, MainThreadMarker, class, msg_send, rc::Retained, runtime::NSObject,
    };
    use objc2_foundation::NSString;
    use std::ffi::c_void;

    /// Empty stand-in for the Objective-C completion block, the same one
    /// `open_url` passes.
    #[repr(transparent)]
    struct FakeBlock(*const c_void);

    // SAFETY: `#[repr(transparent)]` over a pointer, same layout as an optional
    // block reference, which is what `completionHandler:` expects.
    unsafe impl Encode for FakeBlock {
        const ENCODING: Encoding = Encoding::Block;
    }

    fn app() -> anyhow::Result<Retained<NSObject>> {
        let _mtm = MainThreadMarker::new()
            .ok_or_else(|| anyhow::anyhow!("UIApplication must be used on the main thread"))?;
        // SAFETY: correct selector/signature; we hold the main-thread marker.
        let app: Option<Retained<NSObject>> =
            unsafe { msg_send![class!(UIApplication), sharedApplication] };
        app.ok_or_else(|| anyhow::anyhow!("UIApplication is NULL"))
    }

    pub fn current() -> Option<String> {
        let app = app().ok()?;
        // SAFETY: `alternateIconName` is a nullable NSString getter.
        let name: Option<Retained<NSString>> = unsafe { msg_send![&*app, alternateIconName] };
        name.map(|n| n.to_string())
    }

    pub fn set(name: Option<&str>) -> anyhow::Result<()> {
        let app = app()?;
        // False only when Info.plist lists no alternates, which is a build
        // that skipped zcripts/ios/icons.sh.
        // SAFETY: `supportsAlternateIcons` is a BOOL getter.
        let supported: bool = unsafe { msg_send![&*app, supportsAlternateIcons] };
        if !supported {
            anyhow::bail!("No alternate icons in this build");
        }
        let name = name.map(NSString::from_str);
        let name: Option<&NSString> = name.as_deref();
        let handler = FakeBlock(std::ptr::null());
        // [app setAlternateIconName:name completionHandler:nil]
        // SAFETY: correct selector/signature; nil restores the primary icon and
        // a null completion block is allowed.
        let _: () =
            unsafe { msg_send![&*app, setAlternateIconName: name, completionHandler: handler] };
        Ok(())
    }

    fn follows_entry() -> Result<crate::outbound::keyring_entry::Entry, keyring::Error> {
        crate::outbound::keyring_entry::entry(
            &(env!("CARGO_PKG_NAME").to_string() + "-service"),
            &(env!("CARGO_PKG_NAME").to_string() + "-icon-follows-theme"),
        )
    }

    pub fn follows_theme() -> bool {
        follows_entry().is_ok_and(|entry| entry.get_secret().is_ok())
    }

    pub fn set_follows_theme(on: bool) -> anyhow::Result<()> {
        let entry = follows_entry()?;
        if on {
            entry.set_secret(b"1")?;
        } else {
            match entry.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => {}
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }
}

// Everywhere else: the pick and the flag, held for the session.
#[cfg(not(target_os = "ios"))]
mod platform {
    use std::sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    };

    static PICKED: Mutex<Option<String>> = Mutex::new(None);
    static FOLLOWS: AtomicBool = AtomicBool::new(false);

    pub fn follows_theme() -> bool {
        FOLLOWS.load(Ordering::Relaxed)
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn set_follows_theme(on: bool) -> anyhow::Result<()> {
        FOLLOWS.store(on, Ordering::Relaxed);
        Ok(())
    }

    pub fn current() -> Option<String> {
        PICKED.lock().ok()?.clone()
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn set(name: Option<&str>) -> anyhow::Result<()> {
        if let Ok(mut picked) = PICKED.lock() {
            *picked = name.map(str::to_string);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icon(name: &str, is_dark: bool) -> ThemeConfig {
        ThemeConfig {
            name: name.to_string(),
            is_dark,
        }
    }

    #[test]
    fn only_the_primary_icon_has_no_alternate_name() {
        assert_eq!(alternate_name(&primary()), None);
        assert_eq!(
            alternate_name(&icon("vantablack", true)).as_deref(),
            Some("AppIcon-vantablack-dark")
        );
        assert_eq!(
            alternate_name(&icon(PRIMARY, false)).as_deref(),
            Some("AppIcon-gruvbox-light")
        );
    }

    #[test]
    fn a_bundle_name_parses_back_keeping_the_themes_hyphens() {
        assert!(parse("AppIcon-tokyo-night-light") == Some(icon("tokyo-night", false)));
        assert!(parse("AppIcon-nord-dark") == Some(icon("nord", true)));
        assert!(parse("AppIcon-nord").is_none());
    }

    #[test]
    fn every_other_theme_and_mode_has_a_rendered_icon() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/app-icons");
        for theme in ::components::ALLOWED_THEMES {
            for is_dark in [true, false] {
                let png = dir.join(format!("{theme}-{}.png", mode(is_dark)));
                if icon(theme, is_dark) == primary() {
                    assert!(
                        !png.exists(),
                        "the primary icon is assets/favicon, not an alternate"
                    );
                } else {
                    assert!(
                        png.exists(),
                        "{theme} {} has no icon; run zcripts/ios/icon.py",
                        mode(is_dark)
                    );
                }
            }
        }
    }

    // One test, since the pick and the flag are process-wide.
    #[test]
    fn a_pick_reads_back_and_the_icon_follows_only_when_asked() {
        assert!(set(&icon("nord", false)).is_ok());
        assert!(current() == icon("nord", false));

        follow(&icon("dracula", true));
        assert!(current() == icon("nord", false));

        assert!(set_follows_theme(true).is_ok());
        assert!(follows_theme());
        follow(&icon("dracula", true));
        assert!(current() == icon("dracula", true));

        assert!(set_follows_theme(false).is_ok());
        assert!(set(&primary()).is_ok());
        assert!(current() == primary());
    }
}
