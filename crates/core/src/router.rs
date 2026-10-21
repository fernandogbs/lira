//! Rule-based intent router (pt-BR + en). No LLM classifier in M2.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    Status {
        session: Option<String>,
    },
    Summarize {
        session: Option<String>,
    },
    Approve {
        target: String,
    },
    Cancel,
    Instruct {
        session: Option<String>,
        message: String,
    },
}

/// Parse a transcribed utterance into an intent.
pub fn route(text: &str) -> Intent {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Intent::Instruct {
            session: None,
            message: String::new(),
        };
    }

    let lower = trimmed.to_lowercase();

    if matches_exact(&lower, &["cancela", "cancel", "cancelar"]) {
        return Intent::Cancel;
    }

    if let Some(rest) = strip_prefix_ci(&lower, trimmed, &["aprova ", "approve ", "aprovar "]) {
        return Intent::Approve {
            target: rest.trim().to_string(),
        };
    }

    let (session, body) = split_session_prefix(trimmed);
    let body_lower = body.to_lowercase();

    if matches_exact(
        &body_lower,
        &[
            "status",
            "como tá",
            "como esta",
            "como está",
            "como ta",
            "how is it",
            "how's it going",
        ],
    ) {
        return Intent::Status { session };
    }

    if matches_exact(
        &body_lower,
        &[
            "resume",
            "summarize",
            "resumo",
            "resume o que mudou",
            "o que mudou",
        ],
    ) || body_lower.starts_with("resume ")
        || body_lower.starts_with("summarize ")
        || body_lower.starts_with("resumo ")
    {
        return Intent::Summarize { session };
    }

    Intent::Instruct {
        session,
        message: body.trim().to_string(),
    }
}

fn matches_exact(lower: &str, options: &[&str]) -> bool {
    options.contains(&lower)
}

fn strip_prefix_ci<'a>(lower: &str, original: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    for p in prefixes {
        if lower.starts_with(p) {
            return Some(&original[p.len()..]);
        }
    }
    None
}

/// `"backend, run tests"` → (Some("backend"), "run tests")
fn split_session_prefix(text: &str) -> (Option<String>, String) {
    if let Some((head, rest)) = text.split_once(',') {
        let name = head.trim();
        if !name.is_empty()
            && name.len() <= 40
            && !name.contains(' ')
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return (Some(name.to_string()), rest.trim().to_string());
        }
    }
    (None, text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_pt() {
        assert_eq!(route("como tá"), Intent::Status { session: None });
    }

    #[test]
    fn session_prefix_instruct() {
        assert_eq!(
            route("backend, roda os testes"),
            Intent::Instruct {
                session: Some("backend".into()),
                message: "roda os testes".into(),
            }
        );
    }

    #[test]
    fn approve() {
        assert_eq!(
            route("aprova os testes"),
            Intent::Approve {
                target: "os testes".into()
            }
        );
    }

    #[test]
    fn cancel() {
        assert_eq!(route("cancel"), Intent::Cancel);
    }

    #[test]
    fn summarize() {
        assert_eq!(
            route("frontend, resume"),
            Intent::Summarize {
                session: Some("frontend".into())
            }
        );
    }
}
