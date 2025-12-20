#![allow(clippy::print_stdout)]

pub const ZWIPE: &str = include_str!("zwipe.txt");
pub struct Zwipe;

impl Zwipe {
    pub fn print() {
        println!("{ZWIPE}");
    }
}

pub const ZERVER: &str = include_str!("zerver.txt");
pub struct Zerver;

impl Zerver {
    pub fn print() {
        println!("{ZERVER}");
    }
}

pub const ZERVICE: &str = include_str!("zervice.txt");
pub struct Zervice;

impl Zervice {
    pub fn print() {
        println!("{ZERVICE}");
    }
}

pub const ZWIPER: &str = include_str!("zwiper.txt");
pub struct Zwiper;

impl Zwiper {
    pub fn print() {
        println!("{ZWIPER}");
    }
}
