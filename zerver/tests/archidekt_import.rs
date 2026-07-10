//! LIVE Archidekt fetch + parse against a real, permanent deck.
//!
//! `#[ignore]`d on purpose: it hits archidekt.com over the network, so it must
//! never run in CI (external dependency = flaky). Run it on demand when touching
//! the Archidekt client or when Archidekt might have changed their (undocumented)
//! JSON shape:
//!
//!   cargo test -p zerver --test archidekt_import -- --ignored
//!
//! The deck — https://archidekt.com/decks/11493358/satya — is owned by the
//! maintainer and will not be deleted. The `extract_deck_id` parsing and the
//! null-field handling already have offline unit tests in
//! `outbound::archidekt::tests`; this guards the one thing those can't: that the
//! live API still returns the shape the parser expects.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stderr
)]

use zwipe::outbound::archidekt::ArchidektClient;

const DECK_URL: &str = "https://archidekt.com/decks/11493358/satya";
const DECK_ID: i64 = 11493358;

#[tokio::test]
#[ignore = "hits archidekt.com live; run with `-- --ignored`"]
async fn fetches_and_parses_the_real_satya_deck() {
    // The URL parses to the expected id (also covered offline, asserted here so
    // the live path is exercised end to end from a real URL).
    let id = ArchidektClient::extract_deck_id(DECK_URL).expect("parse a deck id from the url");
    assert_eq!(id, DECK_ID);

    // contact_url feeds a courteous User-Agent; any real URL is fine.
    let cards = ArchidektClient::new("https://zwipe.net")
        .fetch_deck(id)
        .await
        .expect("public deck should fetch + parse");

    eprintln!("fetched {} distinct entries from Archidekt deck {DECK_ID}", cards.len());

    assert!(!cards.is_empty(), "a real deck should return cards");
    assert!(cards.len() >= 10, "a real commander deck should have many cards, got {}", cards.len());
    assert!(cards.iter().all(|c| c.quantity >= 1), "every entry has quantity >= 1");
    assert!(cards.iter().all(|c| !c.name.is_empty()), "every entry has a name");

    // Unparseable printings become nil ids (surfaced as unresolved on import),
    // but a real, mostly-standard deck should resolve the majority.
    let resolved = cards.iter().filter(|c| !c.scryfall_id.is_nil()).count();
    assert!(resolved > 0, "at least some printings should resolve to Scryfall ids");
    eprintln!("{resolved}/{} entries resolved a Scryfall id", cards.len());
}
