use crate::inbound::components::interactions::swipe::direction::Direction;

#[derive(Debug, Clone, PartialEq)]
pub struct SwipeConfig {
    /// Which swipe directions are valid for this swipeable entity
    pub allowed_directions: Vec<Direction>,
    /// Distance threshold in pixels to commit a swipe (default: 100.0)
    pub distance_threshold: f64,
    /// Speed threshold in pixels/millisecond to commit a swipe (default: 5.0)
    pub speed_threshold: f64,
}

impl SwipeConfig {
    pub fn new(allowed_directions: Vec<Direction>, distance_threshold: f64, speed_threshold: f64) -> Self {
        Self {
            allowed_directions,
            distance_threshold,
            speed_threshold,
        }
    }
}

impl Default for SwipeConfig {
    fn default() -> Self {
        Self {
            allowed_directions: vec![Direction::Left, Direction::Right, Direction::Up, Direction::Down],
            distance_threshold: 100.0,
            speed_threshold: 5.0,
        }
    }
}
