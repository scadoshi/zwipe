mod about;
mod android;
mod changelog;
mod contribute;
mod discord;
mod guides;
mod home;
mod ios;
mod not_found;
mod privacy;
mod reset;
mod shared_deck;
mod verify;

pub use about::About;
pub use android::Android;
pub use changelog::Changelog;
pub use contribute::Contribute;
pub use discord::Discord;
pub use guides::{GuidePage, Guides};
// SSG prerender list only, which `#[server]` compiles under this feature.
#[cfg(feature = "server")]
pub(crate) use guides::slugs as guide_slugs;
pub use home::Home;
pub use ios::Ios;
pub use not_found::NotFound;
pub use privacy::Privacy;
pub use reset::Reset;
pub use shared_deck::SharedDeck;
pub use verify::Verify;
