//! The frame around every screen: the top nav, with the screens on authed
//! routes and `Log in` and `Create profile` on public ones, the app's menu
//! footer below the breakpoint, and the screen header.

mod frame;
mod menu_footer;
mod menu_theme;
pub mod nav;
mod top_nav;

pub use frame::{OpenFrame, PublicFrame, ScreenFrame};
pub use nav::MenuOpen;
pub use top_nav::Shell;
