//! Swipe direction enum.

/// The four cardinal directions a swipe can resolve to.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    /// Converts the direction to a signed integer (-1 for Left/Up, 1 for Right/Down).
    pub fn as_i32(&self) -> i32 {
        match self {
            Direction::Left | Direction::Up => -1,
            Direction::Right | Direction::Down => 1,
        }
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Left => write!(f, "Left"),
            Self::Right => write!(f, "Right"),
            Self::Up => write!(f, "Up"),
            Self::Down => write!(f, "Down"),
        }
    }
}
