//! MTG card language code conversions.
//!
//! Provides conversions from Scryfall language codes (e.g., "en", "ja") to
//! full display names for UI presentation.

/// Extension trait to convert language codes to full display names.
///
/// # Supported Languages
///
/// - Standard languages: English, Spanish, French, German, Italian, Portuguese, Japanese, Korean, Russian
/// - Chinese variants: Simplified Chinese (zhs), Traditional Chinese (zht)
/// - Historical/fantasy languages: Ancient Greek, Hebrew, Latin, Phyrexian, Quenya, Sanskrit
///
/// # Fallback Behavior
///
/// Unknown language codes return the code itself (e.g., "xy" → "xy").
///
/// # Example
///
/// ```rust,ignore
/// use zwiper::domain::language::LanguageCodeToFullName;
///
/// assert_eq!("en".language_code_to_full_name(), "English");
/// assert_eq!("ja".language_code_to_full_name(), "Japanese");
/// assert_eq!("ph".language_code_to_full_name(), "Phyrexian");
/// assert_eq!("unknown".language_code_to_full_name(), "unknown"); // Fallback
/// ```
pub trait LanguageCodeToFullName {
    /// Converts a language code to its full display name.
    ///
    /// Returns the full language name if recognized, otherwise returns the code itself.
    fn language_code_to_full_name(&self) -> &str;
}

/// Implements language code to name conversion for string slices.
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
