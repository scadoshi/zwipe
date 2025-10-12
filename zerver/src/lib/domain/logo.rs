pub fn print() {
    println!("{}", logo());
}

pub fn logo() -> &'static str {
    include_str!("logo/1.txt")
}
