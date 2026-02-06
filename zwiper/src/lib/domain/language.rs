//! MTG card language code conversions.
//!
//! Provides conversions from Scryfall language codes (e.g., "en", "ja") to
//! full display names for UI presentation.

/// Extension trait to convert language codes to full display names.
///
/// # Fallback Behavior
///
/// Unknown language codes return the code itself (e.g., "xy" → "xy").
pub trait LanguageCodeToFullName {
    /// Converts a language code to its full display name.
    fn language_code_to_full_name(&self) -> &str;
}

impl LanguageCodeToFullName for str {
    fn language_code_to_full_name(&self) -> &str {
        match self {
            "en" => "English",
            "es" => "Spanish",
            "fr" => "French",
            "de" => "German",
            "it" => "Italian",
            "pt" => "Portuguese",
            "ja" => "Japanese",
            "ko" => "Korean",
            "ru" => "Russian",
            "zhs" => "Simplified Chinese",
            "zht" => "Traditional Chinese",
            "ar" => "Arabic",
            "grc" => "Ancient Greek",
            "he" => "Hebrew",
            "la" => "Latin",
            "ph" => "Phyrexian",
            "qya" => "Quenya",
            "sa" => "Sanskrit",
            _ => self, // Fallback: show the code itself for unknown languages
        }
    }
}
