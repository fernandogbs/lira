//! Redact CPF/CNPJ, cards (Luhn), emails, and common API tokens.

use regex::Regex;
use std::sync::OnceLock;

fn re_email() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}").unwrap())
}

fn re_cpf() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b\d{3}\.?\d{3}\.?\d{3}-?\d{2}\b").unwrap())
}

fn re_cnpj() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b\d{2}\.?\d{3}\.?\d{3}/?\d{4}-?\d{2}\b").unwrap())
}

fn re_card() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(?:\d[ -]*?){13,19}\b").unwrap())
}

fn re_tokens() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?x)
            \bsk-[A-Za-z0-9_\-]{10,}
            |\bghp_[A-Za-z0-9]{20,}
            |\beyJ[A-Za-z0-9_\-]+=*\.[A-Za-z0-9_\-]+=*\.[A-Za-z0-9_\-]+=*
            |\bAKIA[0-9A-Z]{16}
            |\bASIA[0-9A-Z]{16}
            ",
        )
        .unwrap()
    })
}

fn luhn_ok(digits: &str) -> bool {
    let mut sum = 0;
    let mut alt = false;
    for c in digits.chars().rev() {
        let Some(mut n) = c.to_digit(10) else {
            return false;
        };
        if alt {
            n *= 2;
            if n > 9 {
                n -= 9;
            }
        }
        sum += n;
        alt = !alt;
    }
    sum % 10 == 0
}

/// Redact sensitive patterns in OCR / screen text.
pub fn redact(text: &str) -> String {
    let mut out = re_tokens()
        .replace_all(text, "[REDACTED_TOKEN]")
        .into_owned();
    out = re_email()
        .replace_all(&out, "[REDACTED_EMAIL]")
        .into_owned();
    out = re_cnpj().replace_all(&out, "[REDACTED_CNPJ]").into_owned();
    out = re_cpf().replace_all(&out, "[REDACTED_CPF]").into_owned();

    out = re_card()
        .replace_all(&out, |caps: &regex::Captures| {
            let raw = &caps[0];
            let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
            if digits.len() >= 13 && digits.len() <= 19 && luhn_ok(&digits) {
                "[REDACTED_CARD]".to_string()
            } else {
                raw.to_string()
            }
        })
        .into_owned();

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_email_and_token() {
        let t = redact("mail me at a@b.com with sk-abcdefghijklmnop");
        assert!(t.contains("[REDACTED_EMAIL]"));
        assert!(t.contains("[REDACTED_TOKEN]"));
        assert!(!t.contains("a@b.com"));
    }

    #[test]
    fn redacts_valid_card() {
        // Visa test number
        let t = redact("card 4111 1111 1111 1111");
        assert!(t.contains("[REDACTED_CARD]"));
    }

    #[test]
    fn redacts_cpf() {
        let t = redact("cpf 529.982.247-25");
        assert!(t.contains("[REDACTED_CPF]"));
    }
}
