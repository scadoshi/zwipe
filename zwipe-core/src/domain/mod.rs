/// Sessions, tokens, passwords and the client platform.
pub mod auth;
/// Cards, their Scryfall data and the search filter.
pub mod card;
/// Decks, their cards, tags and draw odds.
pub mod deck;
pub mod logo;
pub mod moderation;
pub mod site;
/// Accounts, emails, usernames and preferences.
pub mod user;

pub use user::models::{Email, InvalidEmail};
