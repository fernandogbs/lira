/// Normalize reference/hypothesis text for WER (accents preserved).
pub fn normalize(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();
    for c in lower.chars() {
        if c.is_alphanumeric() || c == '\'' {
            current.push(c);
        } else if !current.is_empty() {
            words.push(current);
            current = String::new();
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words.join(" ")
}

fn words(s: &str) -> Vec<String> {
    normalize(s)
        .split_whitespace()
        .map(|w| w.to_string())
        .collect()
}

fn word_edit_distance(a: &[String], b: &[String]) -> usize {
    let n = a.len();
    let m = b.len();
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr = vec![0usize; m + 1];

    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

/// Corpus-level WER: total word edits / total reference words.
pub fn wer(refs: &[String], hyps: &[String]) -> f64 {
    assert_eq!(refs.len(), hyps.len());
    let mut edits = 0usize;
    let mut ref_words = 0usize;
    for (r, h) in refs.iter().zip(hyps) {
        let rw = words(r);
        let hw = words(h);
        edits += word_edit_distance(&rw, &hw);
        ref_words += rw.len();
    }
    if ref_words == 0 {
        return 0.0;
    }
    edits as f64 / ref_words as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_zero() {
        let refs = vec!["Hello world".to_string()];
        let hyps = vec!["hello world".to_string()];
        assert_eq!(wer(&refs, &hyps), 0.0);
    }

    #[test]
    fn one_substitution() {
        let refs = vec!["a b c".to_string()];
        let hyps = vec!["a x c".to_string()];
        assert!((wer(&refs, &hyps) - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn insertion_and_deletion() {
        let refs = vec!["one two".to_string()];
        let ins = vec!["one two three".to_string()];
        assert!((wer(&refs, &ins) - 0.5).abs() < 1e-9);

        let del = vec!["one".to_string()];
        assert!((wer(&refs, &del) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn punctuation_and_case_ignored() {
        let refs = vec!["Hello, World!".to_string()];
        let hyps = vec!["hello world".to_string()];
        assert_eq!(wer(&refs, &hyps), 0.0);
    }

    #[test]
    fn accents_preserved() {
        let refs = vec!["café".to_string()];
        let hyps = vec!["cafe".to_string()];
        assert_eq!(wer(&refs, &hyps), 1.0);
    }
}
