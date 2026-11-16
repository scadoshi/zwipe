//! Core swipe event processing.
//!
//! Defines the [`OnSwipe`](crate::inbound::components::interactions::swipe::onswipe::OnSwipe) trait with shared functionality used by both
//! touch and mouse event handlers.

use chrono::Utc;
use dioxus::html::geometry::ClientPoint;

use crate::inbound::components::interactions::swipe::config::SwipeConfig;
use crate::inbound::components::interactions::swipe::state::SwipeState;
use crate::inbound::components::interactions::swipe::time_point::TimePoint;

/// Core swipe event handlers shared by touch and mouse implementations.
#[expect(
    missing_docs,
    reason = "each method handles the swipe phase it is named after"
)]
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
        self.release_delta = None;

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
        self.end_at(TimePoint::new(end_point, Utc::now().naive_utc()), config);
    }
}
