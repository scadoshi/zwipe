//! The home-screen icon: which theme's colors it is drawn in.
//!
//! iOS keeps the choice itself, so nothing is stored here or on the account:
//! every read asks the OS for its alternate icon name. Android has no
//! alternate icons, so the setting is hidden there. Desktop holds the pick for
//! the session so the sheet behaves the same in development.
//!
//! `zcripts/ios/icon.py` renders an icon per theme into
//! `zwiper/assets/app-icons/`, and `zcripts/ios/icons.sh` bundles each as an
//! alternate icon named `AppIcon-<theme>`.

/// The theme the bundled primary icon is drawn in.
pub const PRIMARY: &str = "gruvbox";

/// Whether this platform can change its icon.
pub const SUPPORTED: bool = cfg!(not(target_os = "android"));

/// The prefix `zcripts/ios/icons.sh` gives each alternate icon.
const PREFIX: &str = "AppIcon-";

/// The theme the icon is drawn in now.
pub fn current() -> String {
    platform::current()
        .and_then(|name| name.strip_prefix(PREFIX).map(str::to_string))
        .unwrap_or_else(|| PRIMARY.to_string())
}

/// Switches the icon to `theme`'s. On iOS the OS shows its own alert saying
/// the icon changed.
pub fn set(theme: &str) -> anyhow::Result<()> {
    platform::set(alternate_name(theme).as_deref())
}

/// The alternate icon's bundle name, or `None` for the primary icon.
fn alternate_name(theme: &str) -> Option<String> {
    (theme != PRIMARY).then(|| format!("{PREFIX}{theme}"))
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
        // SAFETY: `supportsAlternateIcons` is a BOOL getter.
        let supported: bool = unsafe { msg_send![&*app, supportsAlternateIcons] };
        if !supported {
            anyhow::bail!("this device can't change its app icon");
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
}

// Everywhere else: the pick, held for the session.
#[cfg(not(target_os = "ios"))]
mod platform {
    use std::sync::Mutex;

    static PICKED: Mutex<Option<String>> = Mutex::new(None);

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

    #[test]
    fn the_primary_theme_restores_the_bundled_icon() {
        assert_eq!(alternate_name(PRIMARY), None);
        assert_eq!(
            alternate_name("vantablack").as_deref(),
            Some("AppIcon-vantablack")
        );
    }

    #[test]
    fn every_other_theme_has_a_rendered_icon() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/app-icons");
        for theme in ::components::ALLOWED_THEMES {
            let png = dir.join(format!("{theme}.png"));
            if *theme == PRIMARY {
                assert!(
                    !png.exists(),
                    "the primary icon is assets/favicon, not an alternate"
                );
            } else {
                assert!(png.exists(), "{theme} has no icon; run zcripts/ios/icon.py");
            }
        }
    }

    #[test]
    fn a_pick_reads_back_as_its_theme() {
        assert!(set("nord").is_ok());
        assert_eq!(current(), "nord");
        assert!(set(PRIMARY).is_ok());
        assert_eq!(current(), PRIMARY);
    }
}
