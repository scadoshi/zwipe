//! Show more's ghost rows on the Commanders screen's lists, so a press answers at once: the ghosts
//! ease in at the foot of the list the moment Show more is pressed, and once its page has landed
//! and they have finished easing in, they lie over the page's first rows and fade as those rows
//! ease in over them. Under reduced motion every step is a fade.
//!
//! The list draws its rows held before Show more, the ghosts, then the rows its page dealt, all in
//! one run ([`MoreGhosts::layout`]): [`MoreGhosts::split`] says where the ghosts go. The dealt rows stay
//! hidden until the ghosts start to leave, so they ease in over the ghosts rather than under
//! them. The ghosts' element stays (hidden) once they have gone, as the marker the dealt rows
//! follow.

use uuid::Uuid;

/// How many ghost rows Show more eases in.
pub const GHOST_ROWS: usize = 3;

/// The ghosts' key among a list's keyed rows, which a row's printing id never matches.
pub const GHOSTS_KEY: &str = "more-ghosts";

/// The ghosts' ease in, which folds them open as they fade in.
pub const ENTER_ANIMATION: &str = "card-list-ghosts-enter";

/// The ghosts' ease in under reduced motion, a fade alone.
pub const FADE_IN_ANIMATION: &str = "card-list-ghosts-fade-in";

/// The ghosts' fade out over the dealt rows.
pub const LEAVE_ANIMATION: &str = "card-list-ghosts-leave";

/// Where the ghosts are.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Phase {
    /// Not drawn: Show more has not been pressed since the last fresh deal, or its page failed
    /// and the ghosts have gone.
    #[default]
    Off,
    /// Easing in at the foot of the list.
    Entering,
    /// Eased in, waiting on the page.
    Waiting,
    /// Over the dealt rows, fading out as they ease in.
    Leaving,
    /// Gone, their element kept hidden between the rows held before Show more and the rows it
    /// dealt.
    Gone,
}

/// One thing a list with Show more's ghost rows draws: a row, or the ghosts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MoreItem<T> {
    /// A row.
    Row(T),
    /// The ghost rows, between the rows held before Show more and the rows it dealt.
    Ghosts,
}

impl<T> MoreItem<T> {
    /// The item's key in the list's keyed loop: `row_key` of a row, [`GHOSTS_KEY`] for the
    /// ghosts. The client keys the loop body's first node with it, so a row keeps its own
    /// component as rows around it come and go.
    #[must_use]
    pub fn key(&self, row_key: impl Fn(&T) -> String) -> String {
        match self {
            Self::Row(row) => row_key(row),
            Self::Ghosts => GHOSTS_KEY.to_owned(),
        }
    }
}

/// Show more's ghost rows and the page they wait on. Starts [`Phase::Off`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MoreGhosts {
    phase: Phase,
    /// Whether the page has answered, rows or not.
    landed: bool,
    /// The oracle ids Show more's page dealt.
    dealt: Vec<Uuid>,
}

impl MoreGhosts {
    /// Off: no ghosts.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Where the ghosts are.
    #[must_use]
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Show more was pressed and its page is out: the ghosts ease in.
    pub fn ask(&mut self) {
        *self = Self {
            phase: Phase::Entering,
            ..Self::default()
        };
    }

    /// A page dealt the rows with oracle ids `dealt`. The first page to land after
    /// [`Self::ask`] is Show more's; any other changes nothing.
    pub fn landed(&mut self, dealt: Vec<Uuid>) {
        match self.phase {
            Phase::Entering if !self.landed => {
                self.landed = true;
                self.dealt = dealt;
            }
            Phase::Waiting => {
                self.landed = true;
                self.dealt = dealt;
                self.phase = Phase::Leaving;
            }
            Phase::Off | Phase::Entering | Phase::Leaving | Phase::Gone => {}
        }
    }

    /// Show more's page failed or had nothing to deal: the ghosts leave over no rows.
    pub fn failed(&mut self) {
        self.landed(Vec::new());
    }

    /// The list dealt afresh: the ghosts go at once.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// An animation named `animation` ended on the ghosts: their ease in or their fade out.
    /// Any other changes nothing.
    pub fn animation_ended(&mut self, animation: &str) {
        match self.phase {
            Phase::Entering if matches!(animation, ENTER_ANIMATION | FADE_IN_ANIMATION) => {
                self.phase = if self.landed {
                    Phase::Leaving
                } else {
                    Phase::Waiting
                };
            }
            Phase::Leaving if animation == LEAVE_ANIMATION => {
                self.phase = if self.dealt.is_empty() {
                    Phase::Off
                } else {
                    Phase::Gone
                };
            }
            Phase::Off | Phase::Entering | Phase::Waiting | Phase::Leaving | Phase::Gone => {}
        }
    }

    /// Whether the ghosts' element is drawn, between the rows held before Show more and the rows
    /// it dealt.
    #[must_use]
    pub fn drawn(&self) -> bool {
        self.phase != Phase::Off
    }

    /// The ghosts' classes, for the phase they are in.
    #[must_use]
    pub fn class(&self) -> &'static str {
        match self.phase {
            Phase::Off | Phase::Waiting => "card-list-ghosts",
            Phase::Entering => "card-list-ghosts card-list-ghosts-entering",
            Phase::Leaving => "card-list-ghosts card-list-ghosts-leaving",
            Phase::Gone => "card-list-ghosts card-list-ghosts-gone",
        }
    }

    /// Where `rows` splits into the rows held before Show more and the rows its page dealt:
    /// the first dealt row's index, else the end. `oracle_id` reads a row's oracle id.
    #[must_use]
    pub fn split<T>(&self, rows: &[T], oracle_id: impl Fn(&T) -> Option<Uuid>) -> usize {
        rows.iter()
            .position(|row| oracle_id(row).is_some_and(|id| self.dealt.contains(&id)))
            .unwrap_or(rows.len())
    }

    /// Whether the dealt rows are drawn: not while the ghosts ease in or wait, so the rows
    /// ease in over them once they leave.
    #[must_use]
    pub fn shows_dealt(&self) -> bool {
        !matches!(self.phase, Phase::Entering | Phase::Waiting)
    }

    /// What the list draws, in order: the rows held before Show more, the ghosts while they are
    /// drawn, then the rows its page dealt once they show ([`Self::split`],
    /// [`Self::shows_dealt`]). One run, so a client drawing it in one keyed loop moves no row to
    /// another parent when a later press moves the ghosts. `oracle_id` reads a row's oracle id.
    #[must_use]
    pub fn layout<T: Clone>(
        &self,
        rows: &[T],
        oracle_id: impl Fn(&T) -> Option<Uuid>,
    ) -> Vec<MoreItem<T>> {
        let (held, dealt) = rows.split_at(self.split(rows, oracle_id));
        let mut items: Vec<MoreItem<T>> = held.iter().cloned().map(MoreItem::Row).collect();
        if self.drawn() {
            items.push(MoreItem::Ghosts);
        }
        if self.shows_dealt() {
            items.extend(dealt.iter().cloned().map(MoreItem::Row));
        }
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    fn rows(ids: &[u128]) -> Vec<Option<Uuid>> {
        ids.iter().map(|n| Some(id(*n))).collect()
    }

    fn split(ghosts: &MoreGhosts, ids: &[Option<Uuid>]) -> usize {
        ghosts.split(ids, |row| *row)
    }

    #[test]
    fn show_more_eases_the_ghosts_in_at_once() {
        let mut ghosts = MoreGhosts::new();
        assert!(!ghosts.drawn(), "no ghosts before Show more");
        ghosts.ask();
        assert_eq!(ghosts.phase(), Phase::Entering);
        assert!(
            ghosts.drawn(),
            "the ghosts show as soon as Show more is pressed"
        );
    }

    #[test]
    fn a_page_that_lands_after_the_ease_in_sends_the_ghosts_over_it() {
        let mut ghosts = MoreGhosts::new();
        ghosts.ask();
        ghosts.animation_ended(ENTER_ANIMATION);
        assert_eq!(ghosts.phase(), Phase::Waiting);
        assert!(!ghosts.shows_dealt());
        ghosts.landed(vec![id(11), id(12)]);
        assert_eq!(ghosts.phase(), Phase::Leaving);
        assert!(
            ghosts.shows_dealt(),
            "the dealt rows ease in as the ghosts leave"
        );
    }

    #[test]
    fn a_page_that_lands_mid_ease_in_waits_for_it() {
        let mut ghosts = MoreGhosts::new();
        ghosts.ask();
        ghosts.landed(vec![id(11)]);
        assert_eq!(
            ghosts.phase(),
            Phase::Entering,
            "the ghosts finish easing in"
        );
        assert!(!ghosts.shows_dealt(), "the dealt rows hold until then");
        ghosts.animation_ended(FADE_IN_ANIMATION);
        assert_eq!(ghosts.phase(), Phase::Leaving);
    }

    #[test]
    fn the_ghosts_stay_hidden_between_the_runs_once_gone() {
        let mut ghosts = MoreGhosts::new();
        ghosts.ask();
        ghosts.animation_ended(ENTER_ANIMATION);
        ghosts.landed(vec![id(11)]);
        ghosts.animation_ended(LEAVE_ANIMATION);
        assert_eq!(ghosts.phase(), Phase::Gone);
        assert!(
            ghosts.drawn(),
            "the element keeps the dealt rows in their run"
        );
        assert!(ghosts.shows_dealt());
    }

    #[test]
    fn a_failed_page_takes_the_ghosts_away_entirely() {
        let mut ghosts = MoreGhosts::new();
        ghosts.ask();
        ghosts.animation_ended(ENTER_ANIMATION);
        ghosts.failed();
        assert_eq!(ghosts.phase(), Phase::Leaving, "they fade out over nothing");
        ghosts.animation_ended(LEAVE_ANIMATION);
        assert_eq!(ghosts.phase(), Phase::Off);
        assert!(!ghosts.drawn());
    }

    #[test]
    fn only_the_first_page_after_show_more_is_its_page() {
        let mut ghosts = MoreGhosts::new();
        ghosts.landed(vec![id(5)]);
        assert_eq!(
            ghosts.phase(),
            Phase::Off,
            "a top up with no Show more draws nothing"
        );
        ghosts.ask();
        ghosts.animation_ended(ENTER_ANIMATION);
        ghosts.landed(vec![id(11)]);
        ghosts.animation_ended(LEAVE_ANIMATION);
        ghosts.landed(vec![id(21)]);
        let list = rows(&[1, 11, 21]);
        assert_eq!(
            split(&ghosts, &list),
            1,
            "a later top up joins the dealt run"
        );
    }

    #[test]
    fn other_animations_change_nothing() {
        let mut ghosts = MoreGhosts::new();
        ghosts.ask();
        ghosts.animation_ended("row-enter");
        ghosts.animation_ended("skeleton-pulse");
        assert_eq!(ghosts.phase(), Phase::Entering);
        ghosts.animation_ended(LEAVE_ANIMATION);
        assert_eq!(
            ghosts.phase(),
            Phase::Entering,
            "a leave before any page lands"
        );
    }

    #[test]
    fn the_split_is_the_first_dealt_row() {
        let mut ghosts = MoreGhosts::new();
        let list = rows(&[1, 2, 3]);
        assert_eq!(split(&ghosts, &list), 3, "no page, no second run");
        ghosts.ask();
        ghosts.landed(vec![id(11), id(12)]);
        let list = rows(&[1, 2, 3, 11, 12]);
        assert_eq!(split(&ghosts, &list), 3);
        let dropped_above = rows(&[1, 3, 11, 12]);
        assert_eq!(
            split(&ghosts, &dropped_above),
            2,
            "a row dropped above moves it up"
        );
        let first_dealt_dropped = rows(&[1, 2, 3, 12]);
        assert_eq!(split(&ghosts, &first_dealt_dropped), 3);
    }

    #[test]
    fn a_fresh_deal_takes_the_ghosts_away_at_once() {
        let mut ghosts = MoreGhosts::new();
        ghosts.ask();
        ghosts.landed(vec![id(11)]);
        ghosts.reset();
        assert_eq!(ghosts, MoreGhosts::new());
        let list = rows(&[11]);
        assert_eq!(split(&ghosts, &list), 1);
    }

    #[test]
    fn each_phase_has_its_class() {
        let mut ghosts = MoreGhosts::new();
        ghosts.ask();
        assert_eq!(ghosts.class(), "card-list-ghosts card-list-ghosts-entering");
        ghosts.animation_ended(ENTER_ANIMATION);
        assert_eq!(ghosts.class(), "card-list-ghosts");
        ghosts.landed(vec![id(11)]);
        assert_eq!(ghosts.class(), "card-list-ghosts card-list-ghosts-leaving");
        ghosts.animation_ended(LEAVE_ANIMATION);
        assert_eq!(ghosts.class(), "card-list-ghosts card-list-ghosts-gone");
    }
}
