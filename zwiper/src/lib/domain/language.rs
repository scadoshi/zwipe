/// Extension trait to convert language codes to full display names
pub trait LanguageCodeToFullName {
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
