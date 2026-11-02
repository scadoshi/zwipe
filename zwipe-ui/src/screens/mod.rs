//! Screen models: one per screen, holding what the screen knows, what a user can do on it and what each action asks the client to do.
//!
//! Each model is a `State` the client keeps for one visit, an `Action` for every tap, key or reply, and the `Effect`s [`deck_list::State::update`] and its siblings return. The model never fetches, spawns or navigates; the client performs each effect and feeds replies back in as actions. `context/screen-models.md` describes the pattern.

pub mod account;
pub mod activity;
pub mod commander_maybeboard;
pub mod config;
pub mod deck_export;
pub mod deck_import;
pub mod deck_list;
pub mod deck_view;
pub mod home;
