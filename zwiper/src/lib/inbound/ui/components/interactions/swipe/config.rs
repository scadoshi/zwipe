use crate::inbound::ui::components::interactions::swipe::direction::Direction;

#[derive(Debug, Clone, PartialEq)]
pub struct SwipeConfig {
    // swipe in which directions result in screen navigation
    pub navigation_swipes: Vec<Direction>,
    // swipe in which direction results in form submission
    pub submission_swipe: Option<Direction>,
    // relative position from main interface screen
    pub from_main_screen: Option<Direction>,
}

impl SwipeConfig {
    pub fn new(
        navigation_swipes: Vec<Direction>,
        submission_swipe: Option<Direction>,
        from_main_screen: Option<Direction>,
    ) -> Self {
        Self {
            navigation_swipes,
            submission_swipe,
            from_main_screen,
        }
    }
}
