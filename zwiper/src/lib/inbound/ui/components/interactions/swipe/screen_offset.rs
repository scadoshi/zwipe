use dioxus::html::geometry::euclid::{Point2D, UnknownUnit};

pub type ScreenOffset = Point2D<i32, UnknownUnit>;

pub trait ScreenOffsetMethods {
    fn none() -> Self;
    fn left() -> Self;
    fn right() -> Self;
    fn up() -> Self;
    fn down() -> Self;
    fn left_again(&self) -> Self;
    fn right_again(&self) -> Self;
    fn up_again(&self) -> Self;
    fn down_again(&self) -> Self;
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
    fn left_again(&self) -> Self {
        Self::new(self.x - 1, self.y)
    }
    fn right_again(&self) -> Self {
        Self::new(self.x + 1, self.y)
    }
    fn up_again(&self) -> Self {
        Self::new(self.x, self.y - 1)
    }
    fn down_again(&self) -> Self {
        Self::new(self.x, self.y + 1)
    }
}
