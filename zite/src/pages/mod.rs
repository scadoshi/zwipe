mod about;
mod android;
mod changelog;
mod community;
mod confirm_email;
mod contribute;
mod guides;
mod home;
mod ios;
mod not_found;
mod privacy;
mod reset;
mod shared_deck;
mod verify;

pub(crate) use about::About;
pub(crate) use android::Android;
pub(crate) use changelog::Changelog;
pub(crate) use community::Community;
pub(crate) use confirm_email::ConfirmEmail;
pub(crate) use contribute::Contribute;
// SSG prerender list only, which `#[server]` compiles under this feature.
#[cfg(feature = "server")]
pub(crate) use guides::slugs as guide_slugs;
pub(crate) use guides::{GuidePage, Guides};
pub(crate) use home::Home;
pub(crate) use ios::Ios;
pub(crate) use not_found::NotFound;
pub(crate) use privacy::Privacy;
pub(crate) use reset::Reset;
pub(crate) use shared_deck::SharedDeck;
pub(crate) use verify::Verify;
