use dioxus::html::geometry::euclid::{Point2D, UnknownUnit};

pub type ScreenOffset = Point2D<i32, UnknownUnit>;

pub trait ScreenOffsetMethods {
    fn none() -> Self;
    fn left() -> Self;
    fn right() -> Self;
    fn up() -> Self;
    fn down() -> Self;
    fn and_left(&self) -> Self;
    fn and_right(&self) -> Self;
    fn and_up(&self) -> Self;
    fn and_down(&self) -> Self;
}

impl ScreenOffsetMethods for ScreenOffset {
    fn none() -> Self {
        Self::new(0, 0)
    }
    fn left() -> Self {
        Self::new(-1, 0)
    }
    fn right() -> Self {
        Self::new(1, 0)
    }
    fn up() -> Self {
        Self::new(0, -1)
    }
    fn down() -> Self {
        Self::new(0, 1)
    }
    fn and_left(&self) -> Self {
        Self::new(self.x - 1, self.y)
    }
    fn and_right(&self) -> Self {
        Self::new(self.x + 1, self.y)
    }
    fn and_up(&self) -> Self {
        Self::new(self.x, self.y - 1)
    }
    fn and_down(&self) -> Self {
        Self::new(self.x, self.y + 1)
    }
}
