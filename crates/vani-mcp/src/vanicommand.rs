//! Rule-based vernacular command parser (MVP).
//!
//! Turns Hindi / Hinglish / English / Telugu phrases like
//! `"1 SOL Jupiter se USDC mein swap karo"` into a structured [`Intent`].
//! Deliberately simple — a real NLU pass (via rig + a small model) lands in
//! Weeks 1–2. This is a pure function so it's fully unit-testable.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intent {
    /// Detected action: `swap` | `balance` | `price` | `unknown`.
    pub action: String,
    /// First token mentioned, if any.
    pub source: Option<String>,
    /// Second token mentioned (usually the swap target).
    pub target: Option<String>,
    /// First number mentioned (e.g. 1.5), if any.
    pub amount: Option<f64>,
    /// The raw input, echoed back.
    pub raw: String,
}

/// Token needles (Latin + Devanagari) → canonical symbol. Kept small (MVP).
const TOKEN_NEEDLES: &[(&str, &str)] = &[
    ("sol", "sol"),
    ("सोल", "sol"),
    ("usdc", "usdc"),
    ("यूएसडीसी", "usdc"),
    ("usdt", "usdt"),
    ("यूएसडीटी", "usdt"),
    ("bonk", "bonk"),
    ("बोन्क", "bonk"),
    ("jup", "jup"),
    ("जुप", "jup"),
];

/// Devanagari digits ०-९ (Unicode Nd, same category as ASCII digits).
const DEVANAGARI_DIGITS: [char; 10] = ['०', '१', '२', '३', '४', '५', '६', '७', '८', '९'];

/// Common Hindi number words → value. MVP scope: 1–10 + 20.
const HINDI_NUMBERS: &[(&str, f64)] = &[
    ("एक", 1.0),
    ("दो", 2.0),
    ("तीन", 3.0),
    ("चार", 4.0),
    ("पांच", 5.0),
    ("छह", 6.0),
    ("सात", 7.0),
    ("आठ", 8.0),
    ("नौ", 9.0),
    ("दस", 10.0),
    ("बीस", 20.0),
];

/// Parse a vernacular command into an intent. Pure and total — never panics.
pub fn parse(raw: &str) -> Intent {
    let lower = raw.to_lowercase();
    let amount = extract_amount(&lower);
    let action = detect_action(&lower);
    let mentioned = mentioned_symbols(&lower);

    let (source, target) = if action == "swap" {
        (mentioned.first().cloned(), mentioned.get(1).cloned())
    } else {
        (mentioned.first().cloned(), None)
    };

    Intent {
        action: action.to_string(),
        source,
        target,
        amount,
        raw: raw.trim().to_string(),
    }
}

/// Canonical symbols mentioned in the input, in order of first appearance in
/// the *sentence*, not the needle-table order. Word-boundary matching keeps
/// "jup" from matching inside "Jupiter" and "sol" inside "solana". Without
/// position-based ordering, "USDC se SOL swap karo" would mis-assign source.
fn mentioned_symbols(lower: &str) -> Vec<String> {
    let mut found: Vec<(usize, &'static str)> = Vec::new();
    for (needle, canonical) in TOKEN_NEEDLES {
        if let Some(pos) = find_word(lower, needle) {
            if !found.iter().any(|(_, c)| *c == *canonical) {
                found.push((pos, canonical));
            }
        }
    }
    found.sort_by_key(|(pos, _)| *pos);
    found.into_iter().map(|(_, c)| String::from(c)).collect()
}

/// Byte index of the first standalone occurrence of `word` in `haystack`.
fn find_word(haystack: &str, word: &str) -> Option<usize> {
    let chars: Vec<char> = haystack.chars().collect();
    let w: Vec<char> = word.chars().collect();
    if w.is_empty() || w.len() > chars.len() {
        return None;
    }
    for i in 0..=chars.len() - w.len() {
        if chars[i..i + w.len()] == w[..]
            && (i == 0 || !is_continuation(chars[i - 1]))
            && (i + w.len() == chars.len() || !is_continuation(chars[i + w.len()]))
        {
            return Some(chars[..i].iter().map(|c| c.len_utf8()).sum());
        }
    }
    None
}

/// Word-boundary match so "एक" inside "एक्स" doesn't count as a number. Treats
/// Devanagari letters/digits and combining marks (matras, virama) as continuation.
fn contains_word(haystack: &str, word: &str) -> bool {
    find_word(haystack, word).is_some()
}

fn detect_action(lower: &str) -> &'static str {
    if contains_any(lower, &["swap", "स्वैप", "बदलें", "badal", "convert", "कनवर्ट"]) {
        "swap"
    } else if contains_any(lower, &["balance", "बैलेंस", "बैलेन्स", "kya hai mera", "enta"]) {
        "balance"
    } else if contains_any(lower, &["price", "कीमत", "rate", "भाव"]) {
        "price"
    } else {
        "unknown"
    }
}

fn contains_any(lower: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| contains(lower, n))
}

fn contains(haystack: &str, needle: &str) -> bool {
    haystack.contains(needle)
}

/// Extract the first amount: ASCII or Devanagari digits (optionally fractional),
/// falling back to a standalone Hindi number word (एक, दो, …).
fn extract_amount(lower: &str) -> Option<f64> {
    // Normalize Devanagari digits ०-९ → 0-9, then scan for the first number.
    let normalized: String = lower
        .chars()
        .map(|c| {
            DEVANAGARI_DIGITS
                .iter()
                .position(|d| *d == c)
                .and_then(|i| char::from_digit(i as u32, 10))
                .unwrap_or(c)
        })
        .collect();
    if let Some(n) = scan_number(&normalized) {
        return Some(n);
    }
    HINDI_NUMBERS
        .iter()
        .find(|(word, _)| contains_word(lower, word))
        .map(|(_, val)| *val)
}

/// First number in a string (digits with an optional single decimal point).
fn scan_number(s: &str) -> Option<f64> {
    let mut number = String::new();
    let mut seen_dot = false;
    let mut started = false;

    for ch in s.chars() {
        match ch {
            '0'..='9' => {
                started = true;
                number.push(ch);
            }
            '.' if started && !seen_dot => {
                seen_dot = true;
                number.push(ch);
            }
            _ if started => break,
            _ => continue,
        }
    }

    if number.is_empty() || number == "." {
        None
    } else {
        number.parse().ok()
    }
}

fn is_continuation(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, 'ं' | 'ः' | 'ँ' | '़' | '्')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swaps_in_hinglish() {
        let i = parse("1 SOL Jupiter se USDC mein swap karo");
        assert_eq!(i.action, "swap");
        assert_eq!(i.source.as_deref(), Some("sol"));
        assert_eq!(i.target.as_deref(), Some("usdc"));
        assert_eq!(i.amount, Some(1.0));
    }

    #[test]
    fn swap_direction_preserved_when_target_comes_first() {
        // "USDC se SOL" — USDC is mentioned first in the sentence, so it must
        // be the source even though "sol" precedes "usdc" in the needle table.
        let i = parse("USDC se SOL mein swap karo");
        assert_eq!(i.action, "swap");
        assert_eq!(i.source.as_deref(), Some("usdc"));
        assert_eq!(i.target.as_deref(), Some("sol"));
    }

    #[test]
    fn solana_is_not_a_token_mention() {
        // "solana" contains "sol" but isn't a token mention.
        let i = parse("solana ki price kya hai");
        assert_eq!(i.action, "price");
        assert_eq!(i.source, None);
    }

    #[test]
    fn swap_direction_preserved_with_devanagari() {
        let i = parse("एक सोल स्वैप यूएसडीसी");
        assert_eq!(i.action, "swap");
        assert_eq!(i.source.as_deref(), Some("sol"));
        assert_eq!(i.target.as_deref(), Some("usdc"));
    }

    #[test]
    fn swap_negative_fraction() {
        let i = parse("0.5 ETH convert karo USDC");
        assert_eq!(i.action, "swap");
        assert_eq!(i.amount, Some(0.5));
        // ETH isn't in our token list; the only recognized token (usdc) is the
        // swap's first mention, so it lands in `source` and there is no `target`.
        assert_eq!(i.source.as_deref(), Some("usdc"));
        assert_eq!(i.target, None);
    }

    #[test]
    fn balance_in_hindi() {
        let i = parse("mera balance kya hai?");
        assert_eq!(i.action, "balance");
        assert_eq!(i.amount, None);
    }

    #[test]
    fn balance_in_telugu() {
        let i = parse("na balance enta?");
        assert_eq!(i.action, "balance");
    }

    #[test]
    fn price_in_hindi() {
        let i = parse("SOL ki price kya hai");
        assert_eq!(i.action, "price");
        assert_eq!(i.source.as_deref(), Some("sol"));
    }

    #[test]
    fn amount_at_start() {
        let i = parse("10 SOL balance karo");
        assert_eq!(i.action, "balance"); // "karo" isn't swap; balance wins
        assert_eq!(i.amount, Some(10.0));
        assert_eq!(i.source.as_deref(), Some("sol"));
    }

    #[test]
    fn empty_input_is_unknown() {
        let i = parse("");
        assert_eq!(i.action, "unknown");
        assert_eq!(i.amount, None);
    }

    #[test]
    fn no_leading_digits_does_not_misparse() {
        assert_eq!(extract_amount("abc"), None);
        assert_eq!(extract_amount("abc 2.5 sol"), Some(2.5));
    }

    #[test]
    fn devanagari_digits_amount() {
        let i = parse("१ सोल स्वैप करो");
        assert_eq!(i.action, "swap");
        assert_eq!(i.amount, Some(1.0));
        assert_eq!(i.source.as_deref(), Some("sol"));
    }

    #[test]
    fn hindi_number_word_amount() {
        let i = parse("एक सोल स्वैप करो");
        assert_eq!(i.action, "swap");
        assert_eq!(i.amount, Some(1.0));
        assert_eq!(i.source.as_deref(), Some("sol"));
    }

    #[test]
    fn hindi_tokens_parse() {
        let i = parse("पांच यूएसडीसी कीमत क्या है");
        assert_eq!(i.action, "price");
        assert_eq!(i.amount, Some(5.0));
        assert_eq!(i.source.as_deref(), Some("usdc"));
    }

    #[test]
    fn devanagari_fraction() {
        let i = parse("0.५ सोल कनवर्ट करो यूएसडीटी");
        assert_eq!(i.action, "swap");
        assert_eq!(i.amount, Some(0.5));
        assert_eq!(i.source.as_deref(), Some("sol"));
        assert_eq!(i.target.as_deref(), Some("usdt"));
    }

    #[test]
    fn hindi_word_not_substring() {
        // "एक्स" (X) must not parse as the number एक.
        assert_eq!(extract_amount("एक्स बोलो"), None);
    }
}
