use uuid::Uuid;

use super::*;

fn add() -> WriteKey {
    WriteKey::add(Uuid::from_u128(1), Uuid::from_u128(2))
}

fn delete() -> WriteKey {
    WriteKey::delete_deck(Uuid::from_u128(1))
}

fn remove() -> WriteKey {
    WriteKey::remove_card(Uuid::from_u128(1), Uuid::from_u128(3))
}

#[test]
fn an_answer_is_done_and_a_failure_is_a_failure() {
    let mut writes = Unconfirmed::default();
    assert_eq!(writes.settle(add(), Answer::Done), Verdict::Done);
    assert_eq!(writes.settle(add(), Answer::Network), Verdict::Failed);
    assert_eq!(writes.settle(add(), Answer::Failed), Verdict::Failed);
}

#[test]
fn a_409_after_a_lost_add_is_that_add_landing() {
    let mut writes = Unconfirmed::default();
    assert_eq!(writes.settle(add(), Answer::Network), Verdict::Failed);
    assert_eq!(
        writes.settle(add(), Answer::Status(409)),
        Verdict::LandedEarlier
    );
    assert_eq!(
        writes.settle(add(), Answer::Status(409)),
        Verdict::Failed,
        "once settled, a later 409 is a real duplicate"
    );
}

#[test]
fn a_409_with_no_lost_add_is_a_real_duplicate() {
    let mut writes = Unconfirmed::default();
    assert_eq!(writes.settle(add(), Answer::Status(409)), Verdict::Failed);
}

#[test]
fn a_404_after_a_lost_delete_is_that_delete_landing() {
    let mut writes = Unconfirmed::default();
    let _ = writes.settle(delete(), Answer::Network);
    assert_eq!(
        writes.settle(delete(), Answer::Status(404)),
        Verdict::LandedEarlier
    );
}

/// The server answers a second removal of a deck card 422 "deck card not found".
#[test]
fn a_422_after_a_lost_card_removal_is_that_removal_landing() {
    let mut writes = Unconfirmed::default();
    let _ = writes.settle(remove(), Answer::Network);
    assert_eq!(
        writes.settle(remove(), Answer::Status(404)),
        Verdict::Failed
    );
    assert_eq!(
        writes.settle(remove(), Answer::Status(422)),
        Verdict::LandedEarlier
    );
}

#[test]
fn only_the_status_a_repeat_answers_counts() {
    let mut writes = Unconfirmed::default();
    let _ = writes.settle(add(), Answer::Network);
    assert_eq!(
        writes.settle(add(), Answer::Status(404)),
        Verdict::Failed,
        "an add's repeat answers 409"
    );
    assert_eq!(writes.settle(add(), Answer::Status(500)), Verdict::Failed);
    assert_eq!(
        writes.settle(add(), Answer::Status(409)),
        Verdict::LandedEarlier,
        "another failure leaves the lost add unconfirmed"
    );
    let _ = writes.settle(delete(), Answer::Network);
    assert_eq!(
        writes.settle(delete(), Answer::Status(409)),
        Verdict::Failed,
        "a delete's repeat answers 404"
    );
}

#[test]
fn a_lost_write_is_keyed_by_what_it_acts_on() {
    let mut writes = Unconfirmed::default();
    let _ = writes.settle(add(), Answer::Network);
    let other_card = WriteKey::add(Uuid::from_u128(1), Uuid::from_u128(9));
    let other_deck = WriteKey::add(Uuid::from_u128(8), Uuid::from_u128(2));
    assert_eq!(
        writes.settle(other_card, Answer::Status(409)),
        Verdict::Failed
    );
    assert_eq!(
        writes.settle(other_deck, Answer::Status(409)),
        Verdict::Failed
    );
    assert_eq!(
        writes.settle(
            WriteKey::remove_card(Uuid::from_u128(1), Uuid::from_u128(2)),
            Answer::Status(422)
        ),
        Verdict::Failed,
        "a lost add isn't a lost removal of the same card"
    );
}

#[test]
fn a_success_clears_what_was_lost() {
    let mut writes = Unconfirmed::default();
    let _ = writes.settle(add(), Answer::Network);
    assert_eq!(writes.settle(add(), Answer::Done), Verdict::Done);
    assert_eq!(writes.settle(add(), Answer::Status(409)), Verdict::Failed);
}

#[test]
fn a_landed_add_s_deck_card_is_the_one_its_request_made() {
    use zwipe_core::domain::deck::Board;
    use zwipe_core::http::contracts::deck_card::HttpCreateDeckCard;

    let deck = Uuid::from_u128(1);
    let sent = HttpCreateDeckCard {
        scryfall_data_id: Uuid::from_u128(2).to_string(),
        oracle_id: Uuid::from_u128(3).to_string(),
        quantity: 1,
        board: Some(Board::Maybeboard.display_name().to_owned()),
        mvp_at: None,
    };
    let card = landed_deck_card(deck, &sent).unwrap();
    assert_eq!(card.deck_id, deck);
    assert_eq!(card.scryfall_data_id, Uuid::from_u128(2));
    assert_eq!(card.oracle_id, Uuid::from_u128(3));
    assert_eq!(*card.quantity, 1);
    assert_eq!(card.board, Board::Maybeboard);
    assert!(!card.flagged);

    let main = HttpCreateDeckCard {
        board: None,
        ..sent
    };
    assert_eq!(
        landed_deck_card(deck, &main).unwrap().board,
        Board::Deck,
        "no board is the main deck"
    );
}

#[test]
fn a_landed_write_reads_as_done_for_a_caller_that_needs_a_value() {
    assert_eq!(Written::Done(3).or_landed(|| 7), Some(3));
    assert_eq!(Written::LandedEarlier.or_landed(|| 7), Some(7));
    assert_eq!(Written::<i32>::Failed.or_landed(|| 7), None);
}

fn form() -> FormRepeat<&'static str> {
    FormRepeat::default()
}

#[test]
fn a_form_s_lost_answer_says_so_and_its_refused_repeat_says_it_may_have_landed() {
    let mut form = form();
    assert_eq!(form.settle("a", FormAnswer::Network), Some(FormNote::Lost));
    assert_eq!(
        form.settle("a", FormAnswer::RepeatRefused),
        Some(FormNote::Landed)
    );
    assert_eq!(
        form.settle("a", FormAnswer::RepeatRefused),
        None,
        "once said, a later refusal is the server's own"
    );
}

#[test]
fn a_refusal_with_no_lost_try_is_the_server_s_own() {
    let mut form = form();
    assert_eq!(form.settle("a", FormAnswer::RepeatRefused), None);
    assert_eq!(form.settle("a", FormAnswer::Other), None);
}

/// A retry with a field typed differently, say a mistyped current password, isn't the lost try.
#[test]
fn a_changed_form_s_refusal_is_its_own() {
    let mut form = form();
    let _ = form.settle("a", FormAnswer::Network);
    assert_eq!(form.settle("b", FormAnswer::RepeatRefused), None);
    assert_eq!(
        form.settle("a", FormAnswer::RepeatRefused),
        None,
        "only the last form sent is the lost one's repeat"
    );
}

#[test]
fn a_success_or_another_error_after_a_lost_try() {
    let mut form = form();
    let _ = form.settle("a", FormAnswer::Network);
    assert_eq!(
        form.settle("a", FormAnswer::Other),
        None,
        "another error is its own"
    );
    assert_eq!(
        form.settle("a", FormAnswer::RepeatRefused),
        Some(FormNote::Landed),
        "the lost try still may have landed"
    );
    let _ = form.settle("a", FormAnswer::Network);
    assert_eq!(form.settle("a", FormAnswer::Done), None);
    assert_eq!(
        form.settle("a", FormAnswer::RepeatRefused),
        None,
        "a success clears it"
    );
}
