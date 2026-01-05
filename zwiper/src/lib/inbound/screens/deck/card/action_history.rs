#[derive(Clone, Debug, PartialEq)]
pub enum SwipeAction {
    Skip, // Left swipe
    Do,   // Right swipe - affirmative action
}

// Maximum cards to keep in memory before requiring refresh
pub const MAX_CARDS_IN_STACK: usize = 1000;

// Warning threshold - show toast when approaching limit
pub const CARDS_WARNING_THRESHOLD: usize = 500;
