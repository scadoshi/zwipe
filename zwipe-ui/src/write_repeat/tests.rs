use uuid::Uuid;

use super::*;

fn add() -> WriteKey {
    WriteKey::add(Uuid::from_u128(1), Uuid::from_u128(2))
}

fn delete() -> WriteKey {
    WriteKey::delete(Uuid::from_u128(1), Uuid::from_u128(3))
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
            WriteKey::delete(Uuid::from_u128(1), Uuid::from_u128(2)),
            Answer::Status(404)
        ),
        Verdict::Failed,
        "a lost add isn't a lost delete of the same card"
    );
}

#[test]
fn a_success_clears_what_was_lost() {
    let mut writes = Unconfirmed::default();
    let _ = writes.settle(add(), Answer::Network);
    assert_eq!(writes.settle(add(), Answer::Done), Verdict::Done);
    assert_eq!(writes.settle(add(), Answer::Status(409)), Verdict::Failed);
}
