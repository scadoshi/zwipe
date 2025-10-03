use std::collections::HashSet;

use once_cell::sync::Lazy;

const SUBSTRING_BANNED: &[&str] = &[
    // slurs
    "nigger",
    "nigga",
    "faggot",
    "kike",
    "chink",
    "wetback",
    "beaner",
    "sandnigger",
    // abuse
    "rapist",
    "pedophile",
    "molest",
    "molester",
    "molestation",
    "incest",
    "childmolester",
    "childabuse",
    "sexoffender",
    "sexualassault",
    "sexualabuse",
    // sexual
    "pussy",
    "cock",
    "clit",
    "dildo",
    "jizz",
    "fuck",
    "jackoff",
    "jerkoff",
    "milf",
    "masturbate",
    "masturbater",
    "masturbation",
    "masturbating",
    "cumdumpster",
    "cumslut",
    "fuckface",
    "motherfucker",
    "fucktard",
];

const EXACT_MATCH_BANNED: &[&str] = &[
    // common vulgarities (can appear in compound words like "grass" or "classic")
    "shit", "ass", "dick", "bitch", "asshole", "cunt", "slut", "whore", "cum",
    // body parts
    "tit", "tits", "titty", "titties", "tiddies", "boob", "boobs",
    // slurs (less severe than substring-banned)
    "fag", "retard", "dyke", "tard", "homo", "lesbo", // compound insults
    "shithead", "dickhead", "dumbass", "jackass",
];

pub static SUBSTRING_BANNED_SET: Lazy<HashSet<&'static str>> =
    Lazy::new(|| SUBSTRING_BANNED.iter().cloned().collect());

pub static EXACT_MATCH_BANNED_SET: Lazy<HashSet<&'static str>> =
    Lazy::new(|| EXACT_MATCH_BANNED.iter().cloned().collect());

pub trait ContainsBadWord {
    fn contains_bad_word(&self) -> bool;
}

impl ContainsBadWord for &str {
    fn contains_bad_word(&self) -> bool {
        let s = self.trim().to_lowercase();
        EXACT_MATCH_BANNED_SET.contains(s.as_str())
            || SUBSTRING_BANNED_SET.iter().any(|w| s.contains(w))
    }
}
