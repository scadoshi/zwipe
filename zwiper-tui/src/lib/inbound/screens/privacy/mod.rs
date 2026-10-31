//! The Privacy policy screen: the policy compiled into `zwipe_core::legal`, as text in cards that scroll, links printed with their addresses. No network, so no loading or error states.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use super::Screen;
use crate::inbound::app::Context;

/// How far the policy is scrolled.
#[derive(Debug, Default)]
pub(crate) struct PrivacyView {
    pub(crate) scroll: u16,
}

/// Opens the Privacy policy at the top.
pub(crate) fn open(cx: &mut Context<'_>) {
    cx.push(Screen::Privacy(Box::default()));
}
