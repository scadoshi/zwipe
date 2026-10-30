//! Copies text to the clipboard with the OSC 52 escape sequence, which the terminal itself carries out, over SSH too.
//!
//! Most current terminals honor it; some ask first or need it turned on, and tmux passes it along with `set-clipboard on`.

use std::io::{self, Write};

/// The standard base64 alphabet.
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64 with padding, as OSC 52 expects.
pub(crate) fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3).saturating_mul(4));
    let sextet = |n: u32, shift: u32| -> char {
        let index = usize::try_from((n >> shift) & 0x3f).unwrap_or(0);
        ALPHABET.get(index).copied().map_or('A', char::from)
    };
    for chunk in bytes.chunks(3) {
        let (a, b, c) = match *chunk {
            [a, b, c] => (a, Some(b), Some(c)),
            [a, b] => (a, Some(b), None),
            [a] => (a, None, None),
            _ => continue,
        };
        let n = u32::from(a) << 16 | u32::from(b.unwrap_or(0)) << 8 | u32::from(c.unwrap_or(0));
        out.push(sextet(n, 18));
        out.push(sextet(n, 12));
        out.push(if b.is_some() { sextet(n, 6) } else { '=' });
        out.push(if c.is_some() { sextet(n, 0) } else { '=' });
    }
    out
}

/// The escape sequence that puts `text` on the system clipboard.
pub(crate) fn sequence(text: &str) -> String {
    format!("\x1b]52;c;{}\x07", base64(text.as_bytes()))
}

/// Writes the sequence for `text` to `out` and flushes it.
///
/// # Errors
///
/// Fails when `out` cannot be written to.
pub(crate) fn copy_to(out: &mut impl Write, text: &str) -> io::Result<()> {
    out.write_all(sequence(text).as_bytes())?;
    out.flush()
}

/// Puts `text` on the clipboard through the terminal on stdout.
///
/// # Errors
///
/// Fails when stdout cannot be written to.
pub(crate) fn copy(text: &str) -> io::Result<()> {
    copy_to(&mut io::stdout(), text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_the_rfc_examples() {
        for (plain, encoded) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(plain.as_bytes()), encoded, "{plain:?}");
        }
        assert_eq!(base64("é".as_bytes()), "w6k=");
    }

    #[test]
    fn copying_writes_one_osc_52_sequence() {
        let mut out = Vec::new();
        copy_to(&mut out, "zwipe").unwrap();
        assert_eq!(out, b"\x1b]52;c;endpcGU=\x07");
    }
}
