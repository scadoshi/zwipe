pub const ZWIPE: &str = include_str!("logo/zwipe.txt");
pub struct Zwipe;

impl Zwipe {
    pub fn print() {
        tracing::info!("{ZWIPE}");
    }
}

pub const ZERVER: &str = include_str!("logo/zerver.txt");
pub struct Zerver;

impl Zerver {
    pub fn print() {
        tracing::info!("{ZERVER}");
    }
}

pub const ZERVICE: &str = include_str!("logo/zervice.txt");
pub struct Zervice;

impl Zervice {
    pub fn print() {
        tracing::info!("{ZERVICE}");
    }
}

pub const ZWIPER: &str = include_str!("logo/zwiper.txt");
pub struct Zwiper;

impl Zwiper {
    pub fn print() {
        tracing::info!("{ZWIPER}");
    }
}
