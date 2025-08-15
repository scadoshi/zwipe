pub mod auth;
pub mod models;
pub mod ports;
pub mod services;

pub fn print_logo() {
    let logo = include_str!("logos/deck_builder/ansi_shadow.txt");
    println!("{}", logo);
}
