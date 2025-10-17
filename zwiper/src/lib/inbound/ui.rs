pub mod components;

use crate::inbound::ui::components::screens::{
    app::home::Home as AppHome, auth::home::Home as AuthHome,
};
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Screen {
    #[route("/")]
    AppHome {},
    #[route("/auth")]
    AuthHome {},
}
