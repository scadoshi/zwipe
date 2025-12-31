#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn as_i32(&self) -> i32 {
        match self {
            Direction::Left | Direction::Up => -1,
            Direction::Right | Direction::Down => 1,
        }
    }
}
