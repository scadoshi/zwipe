pub mod auth;

use crate::screens::auth::home::Home;
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Screen {
    #[route("/")]
    #[redirect("/", || Screen::Home {})]
    Home {},
}
