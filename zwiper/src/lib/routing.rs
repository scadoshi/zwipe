pub mod home;
pub mod login;
pub mod register;

use crate::components::layout::Layout;
use crate::routing::home::Home;
use crate::routing::login::Login;
use crate::routing::register::Register;
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Layout)]
    #[route("/login")]
    Login {},
    #[route("/register")]
    Register {},
    #[route("/")]
    #[redirect("/", || Route::Home {})]
    Home {},
}
