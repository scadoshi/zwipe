pub mod auth;
pub mod main;

use crate::screens::{auth::home::Home as AuthHome, main::home::Home as MainHome};
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Screen {
    #[route("/")]
    MainHome {},
    #[route("/auth")]
    AuthHome {},
}
