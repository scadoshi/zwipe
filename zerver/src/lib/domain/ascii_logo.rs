pub fn print() {
    println!("{}", logo());
}

pub fn logo() -> &'static str {
    include_str!("ascii_logo/1.txt")
}
