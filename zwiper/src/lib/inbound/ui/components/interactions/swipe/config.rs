use crate::inbound::ui::components::interactions::swipe::{
    direction::Direction, screen_offset::ScreenOffset,
};
use dioxus::html::geometry::euclid::{Point2D, UnknownUnit};

type DeltaPoint = Point2D<f64, UnknownUnit>;

#[derive(Debug, Clone, PartialEq)]
pub struct SwipeConfig {
    // swipe in which directions result in screen navigation
    pub navigation_swipes: Vec<Direction>,
    // swipe in which direction results in form submission
    pub submission_swipe: Option<Direction>,
    // relative position from main interface screen
    pub from_main_screen: ScreenOffset,
}
