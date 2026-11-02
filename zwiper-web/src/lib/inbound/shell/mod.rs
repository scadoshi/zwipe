//! The frame around every screen: the top nav, with the screens on authed
//! routes and `Log in` and `Create profile` on public ones, and the screen
//! header.

mod frame;
pub mod nav;
mod top_nav;

pub use frame::{OpenFrame, PublicFrame, ScreenFrame};
pub use top_nav::Shell;
