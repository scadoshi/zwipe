use crate::inbound::ui::components::interactions::swipe::time_point::TimePoint;
use crate::inbound::ui::components::interactions::swipe::{config::SwipeConfig, state::SwipeState};
use chrono::Utc;
use dioxus::html::geometry::ClientPoint;

// functionality shared by ontouch- and onmouse-
pub trait OnSwipe {
    fn onswipestart(&mut self, start_point: ClientPoint);
    fn onswipemove(&mut self, current_point: ClientPoint);
    fn onswipeend(&mut self, end_point: ClientPoint, config: &SwipeConfig);
}

impl OnSwipe for SwipeState {
    fn onswipestart(&mut self, start_point: ClientPoint) {
        let time_point = TimePoint::new(start_point, Utc::now().naive_utc());

        self.start_point = Some(time_point.clone());
        self.current_point = Some(time_point.clone());
        self.previous_point = Some(time_point);

        self.is_swiping = true;

        tracing::trace!("swipe start={:?}", self.current_point);
    }

    fn onswipemove(&mut self, current_point: ClientPoint) {
        self.return_animation_seconds = 0.0;
        let time_point = TimePoint::new(current_point, Utc::now().naive_utc());

        if self.traversing_axis.is_none() {
            self.set_traversing_axis();
        }

        self.previous_point = self.current_point.clone();
        self.current_point = Some(time_point);
    }

    fn onswipeend(&mut self, end_point: ClientPoint, config: &SwipeConfig) {
        self.calculate_return_animation_seconds();
        let time_point = TimePoint::new(end_point, Utc::now().naive_utc());

        self.previous_point = self.current_point.clone();
        self.current_point = Some(time_point);

        self.is_swiping = false;

        self.set_latest_swipe(config);

        tracing::trace!("swipe end={:?}", self.current_point);
        self.reset();
    }
}
