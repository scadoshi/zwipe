//! ZWIPE in the terminal: sign in, pick a deck, and swipe cards into it with the keyboard.
//!
//! - `config`: where the API is
//! - `domain`: themes, the text form of a card, and the swipe stack
//! - `inbound`: the screens, keys and the main loop
//! - `outbound`: the signed-in account and its saved session

use std::sync::Arc;

use anyhow::Result;
use ratatui_image::picker::Picker;
use zwipe_client::ZwipeClient;

use crate::domain::theme::Palette;
use crate::inbound::app::App;
use crate::inbound::event_loop::EventLoop;
use crate::outbound::account::{Account, SessionFile};
use crate::outbound::theme_file::ThemeFile;

/// Where the API is.
pub(crate) mod config;
/// Themes, the text form of a card, and the swipe stack.
pub(crate) mod domain;
/// The screens, keys and the main loop.
pub(crate) mod inbound;
/// The signed-in account and its saved session.
pub(crate) mod outbound;

#[cfg(test)]
mod test_support;

/// Turns card images off when set to `off`.
const IMAGES_VAR: &str = "ZWIPE_TUI_IMAGES";

/// How this terminal draws images, asked of the terminal itself, or `None` when `ZWIPE_TUI_IMAGES=off`. A terminal that does not answer gets half-block images.
///
/// The terminal must already be in the alternate screen, and nothing may read its events yet.
fn card_images() -> Option<Picker> {
    if std::env::var(IMAGES_VAR).is_ok_and(|v| v == "off") {
        return None;
    }
    Some(Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks()))
}

/// Runs zwipe-tui until the user quits, restoring the terminal however it ends.
///
/// # Errors
///
/// Fails when `ZWIPE_BACKEND_URL` is invalid, the system has no config directory, or the terminal cannot be set up, drawn to or read from.
pub async fn run() -> Result<()> {
    let client = ZwipeClient::new(config::backend_url()?);
    let account = Arc::new(Account::new(client, SessionFile::in_config_dir()?));
    let theme_file = ThemeFile::in_config_dir()?;
    let remembered = theme_file.load();
    let mut app = App::new(Palette::for_account(
        &remembered.theme,
        remembered.dark_mode,
    ));

    let mut terminal = ratatui::try_init()?;
    app.images = card_images();
    let result = EventLoop::new(account, Arc::new(theme_file))
        .run(&mut terminal, &mut app)
        .await;
    ratatui::try_restore()?;
    result
}
