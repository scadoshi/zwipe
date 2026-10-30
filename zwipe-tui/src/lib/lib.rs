//! ZWIPE in the terminal: sign in, pick a deck, and swipe cards into it with the keyboard.
//!
//! - `config`: where the API is
//! - `domain`: themes, the text form of a card, and the swipe stack
//! - `inbound`: the screens, keys and the main loop
//! - `outbound`: the signed-in account and its saved session

use std::sync::Arc;

use anyhow::Result;
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
    let result = EventLoop::new(account, Arc::new(theme_file))
        .run(&mut terminal, &mut app)
        .await;
    ratatui::try_restore()?;
    result
}
