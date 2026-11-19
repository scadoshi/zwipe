//! Entities, value objects and requests, grouped by business area.

pub mod auth;
pub mod card;
pub mod collection;
pub mod deck;
pub mod logo;
pub mod moderation;
pub mod share_link;
pub mod site;
pub mod user;

pub use user::models::{Email, InvalidEmail};
