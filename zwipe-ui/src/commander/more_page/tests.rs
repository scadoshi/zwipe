use super::*;
use crate::commander::more_ghosts::{ENTER_ANIMATION, LEAVE_ANIMATION, Phase};

fn ids(n: u128) -> Vec<Option<Uuid>> {
    (1..=n).map(|i| Some(Uuid::from_u128(i))).collect()
}

fn lens(page: &MorePage, rows: &[Option<Uuid>]) -> (usize, usize) {
    let (held, dealt) = page.runs(rows, |row| *row);
    (held.len(), dealt.len())
}

#[test]
fn it_shows_the_first_ten_and_offers_more_only_past_them() {
    let page = MorePage::new();
    assert_eq!(lens(&page, &ids(25)), (10, 0));
    assert!(page.offers_more(25));
    assert!(!page.offers_more(10), "ten rows have nothing more");
    assert_eq!(lens(&page, &ids(4)), (4, 0), "a short list shows whole");
}

#[test]
fn show_more_eases_ghosts_in_and_the_next_ten_in_over_them() {
    let rows = ids(25);
    let mut page = MorePage::new();
    page.show_more(&rows, |row| *row);
    assert_eq!(page.ghosts().phase(), Phase::Entering);
    assert_eq!(
        lens(&page, &rows),
        (10, 0),
        "the next ten wait on the ghosts"
    );
    page.animation_ended(ENTER_ANIMATION);
    assert_eq!(page.ghosts().phase(), Phase::Leaving);
    assert_eq!(
        lens(&page, &rows),
        (10, 10),
        "they ease in over the leaving ghosts"
    );
    page.animation_ended(LEAVE_ANIMATION);
    assert_eq!(page.ghosts().phase(), Phase::Gone);
    assert_eq!(lens(&page, &rows), (10, 10));
    assert!(page.offers_more(25));
}

#[test]
fn each_show_more_adds_ten_until_all_show() {
    let rows = ids(25);
    let mut page = MorePage::new();
    page.show_more(&rows, |row| *row);
    page.animation_ended(ENTER_ANIMATION);
    page.animation_ended(LEAVE_ANIMATION);
    page.show_more(&rows, |row| *row);
    page.animation_ended(ENTER_ANIMATION);
    assert_eq!(
        lens(&page, &rows),
        (20, 5),
        "the last five are this press's"
    );
    assert!(!page.offers_more(25), "nothing more once all show");
    page.show_more(&rows, |row| *row);
    assert_eq!(
        page.ghosts().phase(),
        Phase::Leaving,
        "a press with nothing more does nothing"
    );
}

#[test]
fn a_reset_goes_back_to_the_first_ten_with_no_ghosts() {
    let rows = ids(25);
    let mut page = MorePage::new();
    page.show_more(&rows, |row| *row);
    page.reset();
    assert_eq!(page.ghosts().phase(), Phase::Off);
    assert_eq!(lens(&page, &rows), (10, 0));
}

#[test]
fn it_knows_which_press_shows_the_rest() {
    let page = MorePage::new();
    assert!(!page.next_shows_the_rest(25), "fifteen still hide after it");
    assert!(page.next_shows_the_rest(20));
    assert!(page.next_shows_the_rest(13));
}
