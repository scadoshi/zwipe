use chrono::{Duration, NaiveDateTime};

/// for seeing how long ago something was
pub trait WasAgo {
    fn was_a_week_ago(&self) -> bool;
    fn was_a_month_ago(&self) -> bool;
}

impl WasAgo for NaiveDateTime {
    fn was_a_week_ago(&self) -> bool {
        let a_week_ago = chrono::Utc::now().naive_utc() - Duration::days(7);
        self < &a_week_ago
    }

    fn was_a_month_ago(&self) -> bool {
        let a_month_ago = chrono::Utc::now().naive_utc() - Duration::days(30);
        self < &a_month_ago
    }
}
