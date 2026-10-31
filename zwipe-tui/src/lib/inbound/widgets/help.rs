//! The `!` Help & feedback dialog: close it, email a problem report, or join the Discord.
//!
//! [`open`] raises it from any screen; `!` opens it where no text field has the cursor, and F1 opens it from anywhere. The loop hands it every key while it is up.

use crossterm::event::KeyCode;
use zwipe_core::domain::site::{DISCORD_URL, SUPPORT_EMAIL};

use super::alert::{AlertChoice, AlertDialog};
use super::browser;
use crate::inbound::app::Context;
use crate::inbound::screens::VERSION;

/// The dialog's title.
pub(crate) const TITLE: &str = "Help & feedback";

/// The dialog's body.
const BODY: &str = "Reach the team or join the community.";

/// What the report email names as the platform.
const PLATFORM: &str = "Terminal";

/// The Help & feedback dialog: `Close`, `Report a problem ↗` and `Join the Discord ↗`.
pub(crate) fn dialog() -> AlertDialog {
    AlertDialog::new(TITLE, BODY)
        .with_button("Close", Some('c'))
        .with_button("Report a problem \u{2197}", Some('r'))
        .with_button("Join the Discord \u{2197}", Some('d'))
}

/// Opens the dialog over the screen.
pub(crate) fn open(cx: &mut Context<'_>) {
    cx.shared.help = Some(dialog());
}

/// Hands `code` to the open dialog: a press closes it, and the two links open in the browser first.
pub(crate) fn handle_key(cx: &mut Context<'_>, code: KeyCode) {
    let Some(mut dialog) = cx.shared.help.take() else {
        return;
    };
    match dialog.handle_key(code) {
        Some(AlertChoice::Pressed(1)) => browser::open(cx, &report_mailto()),
        Some(AlertChoice::Pressed(2)) => browser::open(cx, DISCORD_URL),
        Some(AlertChoice::Pressed(_) | AlertChoice::Dismissed) => {}
        None => cx.shared.help = Some(dialog),
    }
}

/// The problem report's `mailto:` link, its body naming this version and the platform.
pub(crate) fn report_mailto() -> String {
    let subject = encode("Zwipe problem report");
    let body = encode(&format!(
        "Describe the problem:\n\n\n---\nZwipe {VERSION} \u{00b7} {PLATFORM}"
    ));
    format!("mailto:{SUPPORT_EMAIL}?subject={subject}&body={body}")
}

/// Percent-encodes `input` for a `mailto:` query: RFC 3986's unreserved characters pass through, every other byte becomes `%XX`.
fn encode(input: &str) -> String {
    use std::fmt::Write;

    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(byte));
        } else {
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::theme::Palette;
    use crate::inbound::app::App;
    use crate::test_support::{render, screen_text};

    #[test]
    fn the_report_link_encodes_everything_but_the_unreserved_characters() {
        assert_eq!(encode("a b·~"), "a%20b%C2%B7~");
        let link = report_mailto();
        assert!(link.starts_with(
            "mailto:support@zwipe.net?subject=Zwipe%20problem%20report&body=Describe%20the%20problem%3A%0A%0A%0A---%0AZwipe%20"
        ));
        assert!(link.ends_with("%20%C2%B7%20Terminal"));
    }

    #[test]
    fn the_dialog_lists_the_apps_three_actions_and_closes_on_any_press() {
        let mut app = App::new(Palette::terminal());
        let mut cx = Context::new(&mut app.shared);
        open(&mut cx);
        handle_key(&mut cx, KeyCode::Right);
        assert!(cx.shared.help.is_some(), "moving keeps it open");
        let screen = screen_text(&app);
        assert!(screen.contains("Help & feedback"));
        assert!(screen.contains("Reach the team or join the community."));
        assert!(screen.contains("Close"));
        assert!(screen.contains("Report a problem ↗"));
        assert!(screen.contains("Join the Discord ↗"));
        render(&app, 12, 4);
        let mut cx = Context::new(&mut app.shared);
        handle_key(&mut cx, KeyCode::Char('c'));
        assert!(cx.shared.help.is_none());
        open(&mut cx);
        handle_key(&mut cx, KeyCode::Esc);
        assert!(cx.shared.help.is_none());
    }
}
