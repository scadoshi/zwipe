//! Deck-aware filter data extraction.
//!
//! When a `DeckCards` context is provided (by view/remove screens), filter components
//! derive selectable values from the deck's cards instead of fetching from the server.

use dioxus::prelude::*;
use zwipe::domain::card::models::Card;
use zwipe::domain::card::models::search_card::stop_words::{ORACLE_STOP_WORDS, TYPE_STOP_WORDS};

/// Newtype so `try_use_context` doesn't collide with other `Signal<Vec<Card>>` contexts.
#[derive(Clone, Copy)]
pub struct DeckCards(pub Signal<Vec<Card>>);

/// Extracts distinct type words from deck cards' `type_line` fields.
pub fn extract_type_words(cards: &[Card]) -> Vec<String> {
    let mut words: Vec<String> = cards
        .iter()
        .filter_map(|c| c.scryfall_data.type_line.as_deref())
        .flat_map(|line| line.split_whitespace())
        .map(|w| w.trim_matches(|c: char| matches!(c, ':' | '-' | '?' | ',' | ' ')))
        .map(|w| w.to_lowercase())
        .filter(|w| !w.is_empty() && !TYPE_STOP_WORDS.contains(&w.as_str()))
        .collect();
    words.sort();
    words.dedup();
    words
}

/// Extracts distinct words from deck cards' `oracle_text` fields.
pub fn extract_oracle_words(cards: &[Card]) -> Vec<String> {
    let mut words: Vec<String> = cards
        .iter()
        .filter_map(|c| c.scryfall_data.oracle_text.as_deref())
        .flat_map(|text| text.split_whitespace())
        .map(|w| {
            w.chars()
                .filter(|c| c.is_ascii_alphabetic())
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|w| !w.is_empty() && !ORACLE_STOP_WORDS.contains(&w.as_str()))
        .collect();
    words.sort();
    words.dedup();
    words
}

/// Extracts distinct keywords from deck cards' `keywords` arrays.
pub fn extract_keywords(cards: &[Card]) -> Vec<String> {
    let mut keywords: Vec<String> = cards
        .iter()
        .filter_map(|c| c.scryfall_data.keywords.as_ref())
        .flat_map(|kws| kws.iter())
        .map(|k| k.trim().to_lowercase())
        .filter(|k| !k.is_empty())
        .collect();
    keywords.sort();
    keywords.dedup();
    keywords
}

/// Extracts distinct artist names from deck cards.
pub fn extract_artists(cards: &[Card]) -> Vec<String> {
    let mut artists: Vec<String> = cards
        .iter()
        .filter_map(|c| c.scryfall_data.artist.as_deref())
        .filter(|a| !a.is_empty())
        .map(|a| a.to_string())
        .collect();
    artists.sort();
    artists.dedup();
    artists
}

/// Extracts distinct set names from deck cards.
pub fn extract_sets(cards: &[Card]) -> Vec<String> {
    let mut sets: Vec<String> = cards
        .iter()
        .map(|c| c.scryfall_data.set_name.clone())
        .collect();
    sets.sort();
    sets.dedup();
    sets
}
