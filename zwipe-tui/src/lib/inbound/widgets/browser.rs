//! Opens a link in the user's browser: `open` on macOS, `xdg-open` elsewhere. When that cannot start, the link shows in a toast to copy by hand.

use std::process::{Command, Stdio};
use std::{io, thread};

use super::toast::ToastKind;
use crate::inbound::app::Context;

/// The program that opens a link in the default browser on this system.
pub(crate) fn opener() -> &'static str {
    if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    }
}

/// Opens `url` in the browser, or shows it in a toast when the browser cannot be started.
pub(crate) fn open(cx: &mut Context<'_>, url: &str) {
    open_with(cx, opener(), url);
}

/// Opens `url` with `program`, or shows it in a toast when `program` cannot be started.
fn open_with(cx: &mut Context<'_>, program: &str, url: &str) {
    if spawn(program, url).is_err() {
        cx.shared
            .toasts
            .push(ToastKind::Info, format!("Open {url} in your browser"));
    }
}

/// Starts `program url` with no terminal I/O, so it cannot draw over the screen, and reaps it off the main thread.
fn spawn(program: &str, url: &str) -> io::Result<()> {
    let mut child = Command::new(program)
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    thread::spawn(move || child.wait());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::theme::Palette;
    use crate::inbound::app::App;

    #[test]
    fn the_opener_fits_the_system() {
        let want = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        assert_eq!(opener(), want);
    }

    #[test]
    fn a_browser_that_cannot_start_shows_the_link_instead() {
        let mut app = App::new(Palette::terminal());
        let mut cx = Context::new(&mut app.shared);
        open_with(&mut cx, "zwipe-tui-no-such-program", "https://zwipe.net");
        let toasts = app.shared.toasts.items();
        assert_eq!(toasts.len(), 1);
        assert_eq!(toasts[0].message, "Open https://zwipe.net in your browser");
    }

    #[test]
    fn a_browser_that_starts_shows_nothing() {
        let mut app = App::new(Palette::terminal());
        let mut cx = Context::new(&mut app.shared);
        open_with(&mut cx, "true", "https://zwipe.net");
        assert!(app.shared.toasts.is_empty());
    }
}
