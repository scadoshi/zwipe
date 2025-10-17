use crate::inbound::ui::components::interactions::swipe::direction::Direction;

#[derive(Debug, Clone)]
pub struct Delta {
    pub from_start: f64,
    pub direction_from_start: Option<Direction>,
    pub from_previous: f64,
    pub milliseconds_from_previous: f64,
}

impl Default for Delta {
    fn default() -> Self {
        Self {
            from_start: 0.0,
            direction_from_start: None,
            from_previous: 0.0,
            milliseconds_from_previous: 0.0,
        }
    }
}

impl Delta {
    pub fn new(
        from_start: f64,
        from_previous: f64,
        direction_from_start: Option<Direction>,
        milliseconds_from_previous: f64,
    ) -> Self {
        Self {
            from_start,
            from_previous,
            direction_from_start,
            milliseconds_from_previous,
        }
    }

    pub fn speed(&self) -> Option<f64> {
        if self.milliseconds_from_previous < 0.001 {
            return None;
        }
        Some(self.from_previous / self.milliseconds_from_previous)
    }
}
