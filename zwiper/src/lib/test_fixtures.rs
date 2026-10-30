//! Domain values for unit tests. zwipe-core's own builders sit behind its
//! `cfg(test)`, so this crate deserializes a fixture instead.

use std::str::FromStr;

use chrono::{Duration, Utc};
use uuid::Uuid;
use zwipe_core::domain::auth::models::access_token::{AccessToken, Jwt};
use zwipe_core::domain::auth::models::refresh_token::RefreshToken;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::DeckEntry;
use zwipe_core::domain::deck::deck_name::DeckName;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
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

/// An empty, formatless deck profile with no command zone and no cards.
pub(crate) fn deck_profile() -> DeckProfile {
    DeckProfile {
        id: Uuid::new_v4(),
        name: DeckName::new("Test deck").expect("a valid deck name"),
        commander_id: None,
        partner_commander_id: None,
        background_id: None,
        signature_spell_id: None,
        format: None,
        tags: Vec::new(),
        power_level: None,
        other_tags: Vec::new(),
        oracle_tags: Vec::new(),
        land_target: None,
        price_target: None,
        price_target_currency: None,
        share_token: None,
        user_id: Uuid::new_v4(),
        card_count: 0,
        commander_name: None,
        partner_commander_name: None,
        background_name: None,
        signature_spell_name: None,
        color_identity: Vec::new(),
        commander_art_url: None,
        partner_commander_art_url: None,
        background_art_url: None,
        signature_spell_art_url: None,
    }
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
