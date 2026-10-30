//! Accent folding for card text search, the Rust twin of the database's
//! `f_unaccent`.
//!
//! The table is Postgres `unaccent`'s own rules for the Latin blocks
//! (U+00A0 to U+024F, the combining marks U+0300 to U+0362, U+1E00 to
//! U+1EFF and the Latin ligatures U+FB00 to U+FB06), limited to the entries
//! that fold to ASCII. Search strips everything outside `[a-zA-Z0-9 ]` after
//! folding, so a rule that maps to another non-ASCII letter changes nothing
//! and is left out. Keep it in step with the server: the parity test in
//! `zerver/tests/card_filter_parity.rs` folds the same samples both ways.

/// Folds accented Latin letters to ASCII the way Postgres `unaccent` does:
/// `Jötun` becomes `Jotun`, `Æther` becomes `AEther`, `Straße` becomes
/// `Strasse`. Combining marks are dropped, so decomposed input folds the same
/// as precomposed. Everything else passes through unchanged.
#[must_use]
pub fn fold_accents(text: &str) -> String {
    let mut folded = String::with_capacity(text.len());
    for c in text.chars() {
        match fold_char(c) {
            Some(ascii) => folded.push_str(ascii),
            None => folded.push(c),
        }
    }
    folded
}

/// The ASCII that `unaccent` puts in place of `c`, or `None` when it keeps
/// `c` (every ASCII character, and anything outside the table).
fn fold_char(c: char) -> Option<&'static str> {
    if c.is_ascii() {
        return None;
    }
    let ascii = match c {
        '¡' => "!",
        '©' => "(C)",
        'ª' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' | 'ǎ' | 'ǟ' | 'ǡ' | 'ǻ' | 'ȁ'
        | 'ȃ' | 'ȧ' | 'ḁ' | 'ẚ' | 'ạ' | 'ả' | 'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' | 'ắ' | 'ằ' | 'ẳ'
        | 'ẵ' | 'ặ' => "a",
        '«' => "<<",
        '\u{ad}' => "-",
        '®' => "(R)",
        '±' => "+/-",
        'º' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' | 'ơ' | 'ǒ' | 'ǫ' | 'ǭ' | 'ȍ'
        | 'ȏ' | 'ȫ' | 'ȭ' | 'ȯ' | 'ȱ' | 'ṍ' | 'ṏ' | 'ṑ' | 'ṓ' | 'ọ' | 'ỏ' | 'ố' | 'ồ' | 'ổ'
        | 'ỗ' | 'ộ' | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => "o",
        '»' => ">>",
        '¼' => " 1/4",
        '½' => " 1/2",
        '¾' => " 3/4",
        '¿' => "?",
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'Ā' | 'Ă' | 'Ą' | 'Ǎ' | 'Ǟ' | 'Ǡ' | 'Ǻ' | 'Ȁ' | 'Ȃ'
        | 'Ȧ' | 'Ⱥ' | 'Ḁ' | 'Ạ' | 'Ả' | 'Ấ' | 'Ầ' | 'Ẩ' | 'Ẫ' | 'Ậ' | 'Ắ' | 'Ằ' | 'Ẳ' | 'Ẵ'
        | 'Ặ' => "A",
        'Æ' => "AE",
        'Ç' | 'Ć' | 'Ĉ' | 'Ċ' | 'Č' | 'Ƈ' | 'Ȼ' | 'Ḉ' => "C",
        'È' | 'É' | 'Ê' | 'Ë' | 'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' | 'Ɛ' | 'Ȅ' | 'Ȇ' | 'Ȩ' | 'Ɇ' | 'Ḕ'
        | 'Ḗ' | 'Ḙ' | 'Ḛ' | 'Ḝ' | 'Ẹ' | 'Ẻ' | 'Ẽ' | 'Ế' | 'Ề' | 'Ể' | 'Ễ' | 'Ệ' => {
            "E"
        }
        'Ì' | 'Í' | 'Î' | 'Ï' | 'Ĩ' | 'Ī' | 'Ĭ' | 'Į' | 'İ' | 'Ɩ' | 'Ɨ' | 'Ǐ' | 'Ȉ' | 'Ȋ' | 'Ḭ'
        | 'Ḯ' | 'Ỉ' | 'Ị' => "I",
        'Ð' | 'Ď' | 'Đ' | 'Ɖ' | 'Ɗ' | 'Ƌ' | 'Ḋ' | 'Ḍ' | 'Ḏ' | 'Ḑ' | 'Ḓ' => "D",
        'Ñ' | 'Ń' | 'Ņ' | 'Ň' | 'Ŋ' | 'Ɲ' | 'Ǹ' | 'Ṅ' | 'Ṇ' | 'Ṉ' | 'Ṋ' => "N",
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'Ō' | 'Ŏ' | 'Ő' | 'Ơ' | 'Ǒ' | 'Ǫ' | 'Ǭ' | 'Ȍ' | 'Ȏ'
        | 'Ȫ' | 'Ȭ' | 'Ȯ' | 'Ȱ' | 'Ṍ' | 'Ṏ' | 'Ṑ' | 'Ṓ' | 'Ọ' | 'Ỏ' | 'Ố' | 'Ồ' | 'Ổ' | 'Ỗ'
        | 'Ộ' | 'Ớ' | 'Ờ' | 'Ở' | 'Ỡ' | 'Ợ' => "O",
        '×' => "*",
        'Ù' | 'Ú' | 'Û' | 'Ü' | 'Ũ' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' | 'Ų' | 'Ư' | 'Ǔ' | 'Ǖ' | 'Ǘ' | 'Ǚ'
        | 'Ǜ' | 'Ȕ' | 'Ȗ' | 'Ʉ' | 'Ṳ' | 'Ṵ' | 'Ṷ' | 'Ṹ' | 'Ṻ' | 'Ụ' | 'Ủ' | 'Ứ' | 'Ừ' | 'Ử'
        | 'Ữ' | 'Ự' => "U",
        'Ý' | 'Ŷ' | 'Ÿ' | 'Ƴ' | 'Ȳ' | 'Ɏ' | 'Ẏ' | 'Ỳ' | 'Ỵ' | 'Ỷ' | 'Ỹ' | 'Ỿ' => {
            "Y"
        }
        'Þ' => "TH",
        'ß' => "ss",
        'æ' => "ae",
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' | 'ƈ' | 'ȼ' | 'ḉ' => "c",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' | 'ȅ' | 'ȇ' | 'ȩ' | 'ɇ' | 'ḕ' | 'ḗ'
        | 'ḙ' | 'ḛ' | 'ḝ' | 'ẹ' | 'ẻ' | 'ẽ' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => {
            "e"
        }
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' | 'ǐ' | 'ȉ' | 'ȋ' | 'ḭ' | 'ḯ' | 'ỉ'
        | 'ị' => "i",
        'ð' | 'ď' | 'đ' | 'ƌ' | 'ȡ' | 'ḋ' | 'ḍ' | 'ḏ' | 'ḑ' | 'ḓ' => "d",
        'ñ' | 'ń' | 'ņ' | 'ň' | 'ŋ' | 'ƞ' | 'ǹ' | 'ȵ' | 'ṅ' | 'ṇ' | 'ṉ' | 'ṋ' => {
            "n"
        }
        '÷' => "/",
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' | 'ư' | 'ǔ' | 'ǖ' | 'ǘ' | 'ǚ'
        | 'ǜ' | 'ȕ' | 'ȗ' | 'ṳ' | 'ṵ' | 'ṷ' | 'ṹ' | 'ṻ' | 'ụ' | 'ủ' | 'ứ' | 'ừ' | 'ử' | 'ữ'
        | 'ự' => "u",
        'ý' | 'ÿ' | 'ŷ' | 'ƴ' | 'ȳ' | 'ɏ' | 'ẏ' | 'ẙ' | 'ỳ' | 'ỵ' | 'ỷ' | 'ỹ' | 'ỿ' => {
            "y"
        }
        'þ' => "th",
        'Ĝ' | 'Ğ' | 'Ġ' | 'Ģ' | 'Ɠ' | 'Ǥ' | 'Ǧ' | 'Ǵ' | 'Ḡ' => "G",
        'ĝ' | 'ğ' | 'ġ' | 'ģ' | 'ǥ' | 'ǧ' | 'ǵ' | 'ḡ' => "g",
        'Ĥ' | 'Ħ' | 'Ȟ' | 'Ḣ' | 'Ḥ' | 'Ḧ' | 'Ḩ' | 'Ḫ' => "H",
        'ĥ' | 'ħ' | 'ȟ' | 'ḣ' | 'ḥ' | 'ḧ' | 'ḩ' | 'ḫ' | 'ẖ' => "h",
        'Ĳ' => "IJ",
        'ĳ' => "ij",
        'Ĵ' | 'Ɉ' => "J",
        'ĵ' | 'ǰ' | 'ȷ' | 'ɉ' => "j",
        'Ķ' | 'Ƙ' | 'Ǩ' | 'Ḱ' | 'Ḳ' | 'Ḵ' => "K",
        'ķ' | 'ƙ' | 'ǩ' | 'ḱ' | 'ḳ' | 'ḵ' => "k",
        'ĸ' => "q",
        'Ĺ' | 'Ļ' | 'Ľ' | 'Ŀ' | 'Ł' | 'Ƚ' | 'Ḷ' | 'Ḹ' | 'Ḻ' | 'Ḽ' => "L",
        'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' | 'ƚ' | 'ȴ' | 'ḷ' | 'ḹ' | 'ḻ' | 'ḽ' => "l",
        'ŉ' => "'n",
        'Œ' => "OE",
        'œ' => "oe",
        'Ŕ' | 'Ŗ' | 'Ř' | 'Ȑ' | 'Ȓ' | 'Ɍ' | 'Ṙ' | 'Ṛ' | 'Ṝ' | 'Ṟ' => "R",
        'ŕ' | 'ŗ' | 'ř' | 'ȑ' | 'ȓ' | 'ɍ' | 'ṙ' | 'ṛ' | 'ṝ' | 'ṟ' => "r",
        'Ś' | 'Ŝ' | 'Ş' | 'Š' | 'Ș' | 'Ṡ' | 'Ṣ' | 'Ṥ' | 'Ṧ' | 'Ṩ' => "S",
        'ś' | 'ŝ' | 'ş' | 'š' | 'ſ' | 'ș' | 'ȿ' | 'ṡ' | 'ṣ' | 'ṥ' | 'ṧ' | 'ṩ' | 'ẜ' | 'ẝ' => {
            "s"
        }
        'Ţ' | 'Ť' | 'Ŧ' | 'Ƭ' | 'Ʈ' | 'Ț' | 'Ⱦ' | 'Ṫ' | 'Ṭ' | 'Ṯ' | 'Ṱ' => "T",
        'ţ' | 'ť' | 'ŧ' | 'ƫ' | 'ƭ' | 'ț' | 'ȶ' | 'ṫ' | 'ṭ' | 'ṯ' | 'ṱ' | 'ẗ' => {
            "t"
        }
        'Ŵ' | 'Ẁ' | 'Ẃ' | 'Ẅ' | 'Ẇ' | 'Ẉ' => "W",
        'ŵ' | 'ẁ' | 'ẃ' | 'ẅ' | 'ẇ' | 'ẉ' | 'ẘ' => "w",
        'Ź' | 'Ż' | 'Ž' | 'Ƶ' | 'Ȥ' | 'Ẑ' | 'Ẓ' | 'Ẕ' => "Z",
        'ź' | 'ż' | 'ž' | 'ƶ' | 'ȥ' | 'ɀ' | 'ẑ' | 'ẓ' | 'ẕ' => "z",
        'ƀ' | 'ƃ' | 'ḃ' | 'ḅ' | 'ḇ' => "b",
        'Ɓ' | 'Ƃ' | 'Ƀ' | 'Ḃ' | 'Ḅ' | 'Ḇ' => "B",
        'Ƒ' | 'Ḟ' => "F",
        'ƒ' | 'ḟ' => "f",
        'ƕ' => "hv",
        'Ƣ' => "OI",
        'ƣ' => "oi",
        'Ƥ' | 'Ṕ' | 'Ṗ' => "P",
        'ƥ' | 'ṕ' | 'ṗ' => "p",
        'Ʋ' | 'Ṽ' | 'Ṿ' | 'Ỽ' => "V",
        'Ǆ' | 'Ǳ' => "DZ",
        'ǅ' | 'ǲ' => "Dz",
        'ǆ' | 'ǳ' => "dz",
        'Ǉ' => "LJ",
        'ǈ' => "Lj",
        'ǉ' => "lj",
        'Ǌ' => "NJ",
        'ǋ' => "Nj",
        'ǌ' => "nj",
        'ȸ' => "db",
        'ȹ' => "qp",
        '\u{300}'..='\u{362}' => "",
        'Ḿ' | 'Ṁ' | 'Ṃ' => "M",
        'ḿ' | 'ṁ' | 'ṃ' => "m",
        'ṽ' | 'ṿ' | 'ỽ' => "v",
        'Ẋ' | 'Ẍ' => "X",
        'ẋ' | 'ẍ' => "x",
        'ẞ' => "SS",
        'Ỻ' => "LL",
        'ỻ' => "ll",
        'ﬀ' => "ff",
        'ﬁ' => "fi",
        'ﬂ' => "fl",
        'ﬃ' => "ffi",
        'ﬄ' => "ffl",
        'ﬅ' | 'ﬆ' => "st",
        _ => return None,
    };
    Some(ascii)
}

#[cfg(test)]
mod tests {
    use super::fold_accents;

    #[test]
    fn folds_accented_letters() {
        assert_eq!(fold_accents("Lim-Dûl"), "Lim-Dul");
        assert_eq!(fold_accents("Jötun Grunt"), "Jotun Grunt");
        assert_eq!(fold_accents("Dandân Séance"), "Dandan Seance");
    }

    #[test]
    fn expands_ligatures_and_sharp_s() {
        assert_eq!(fold_accents("Æther Œuvre Straße"), "AEther OEuvre Strasse");
        assert_eq!(fold_accents("ﬁre"), "fire");
    }

    #[test]
    fn drops_combining_marks() {
        assert_eq!(fold_accents("Du\u{302}l"), "Dul");
    }

    #[test]
    fn leaves_ascii_and_other_scripts_alone() {
        assert_eq!(fold_accents("Akroma's Will"), "Akroma's Will");
        assert_eq!(fold_accents("稲妻"), "稲妻");
    }
}
