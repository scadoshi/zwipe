pub mod auth;
pub mod card;
pub mod deck;
pub mod user;

pub fn print_logo() {
    let logo = include_str!("domain/logos/deck_builder/ansi_shadow.txt");
    println!("{}", logo);
}
