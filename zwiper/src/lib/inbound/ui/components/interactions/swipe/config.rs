use crate::inbound::ui::components::interactions::swipe::{
    direction::Direction, screen_offset::ScreenOffset,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SwipeConfig {
    // swipe in which directions result in screen navigation
    pub navigation_swipes: Vec<Direction>,
    // swipe in which direction results in form submission
    pub submission_swipe: Option<Direction>,
    // relative position from main interface screen
    pub from_main_screen: ScreenOffset,
}

impl SwipeConfig {
    pub fn blank() -> Self {
        Self {
            navigation_swipes: vec![],
            submission_swipe: None,
            from_main_screen: ScreenOffset::origin(),
        }
    }
}
