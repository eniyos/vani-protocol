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

const TOKENS: &[&str] = &["sol", "usdc", "usdt", "bonk", "jup"];

/// Parse a vernacular command into an intent. Pure and total — never panics.
pub fn parse(raw: &str) -> Intent {
    let lower = raw.to_lowercase();
    let amount = extract_amount(&lower);
    let action = detect_action(&lower);
    let mentioned: Vec<String> = TOKENS
        .iter()
        .filter(|t| contains(&lower, t))
        .map(|t| t.to_string())
        .collect();

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

/// Extract the first number (digits with optional single decimal point).
fn extract_amount(lower: &str) -> Option<f64> {
    let mut number = String::new();
    let mut seen_dot = false;
    let mut started = false;

    for ch in lower.chars() {
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
}
