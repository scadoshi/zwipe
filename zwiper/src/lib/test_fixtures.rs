//! Domain values for unit tests. zwipe-core's own builders sit behind its
//! `cfg(test)`, so this crate deserializes a fixture instead.

use std::str::FromStr;

use chrono::{Duration, Utc};
use uuid::Uuid;
use zwipe_core::domain::auth::models::access_token::{AccessToken, Jwt};
use zwipe_core::domain::auth::models::refresh_token::RefreshToken;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::DeckEntry;
use zwipe_core::domain::deck::quantity::Quantity;
use zwipe_core::domain::user::models::User;
use zwipe_core::domain::user::models::email::Email;
use zwipe_core::domain::user::models::preferences::UserPreferences;
use zwipe_core::domain::user::models::username::Username;

/// A deck entry for `name` holding `quantity` copies on the main deck, with
/// fresh card ids so entries never collide.
pub(crate) fn deck_entry(name: &str, quantity: i32) -> DeckEntry {
    let mut entry: DeckEntry = serde_json::from_str(include_str!("test_fixtures/deck_entry.json"))
        .expect("the fixture matches DeckEntry");
    let id = Uuid::new_v4();
    entry.card.card_profile.scryfall_data_id = id;
    entry.card.scryfall_data.id = id;
    entry.deck_card.scryfall_data_id = id;
    name.clone_into(&mut entry.card.scryfall_data.name);
    entry.deck_card.quantity = Quantity::new(quantity).expect("a valid test quantity");
    entry
}

/// A live session for a made-up user; the tokens are well formed but sign
/// nothing.
pub(crate) fn session() -> Session {
    Session {
        user: User::new(
            Uuid::new_v4(),
            Username::new("alice").expect("a valid username"),
            Email::from_str("alice@example.com").expect("a valid email"),
        ),
        access_token: AccessToken {
            value: Jwt::from_str("header.payload.signature").expect("a well-formed jwt"),
            expires_at: Utc::now() + Duration::hours(24),
        },
        refresh_token: RefreshToken {
            value: "x".repeat(64),
            expires_at: Utc::now() + Duration::days(14),
        },
        preferences: UserPreferences::default(),
    }
}
