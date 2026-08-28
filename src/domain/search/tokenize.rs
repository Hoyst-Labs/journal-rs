use std::collections::HashSet;

const STOP_WORDS: &[&str] = &[
    "a", "an", "the", "to", "for", "of", "in", "on", "and", "or", "is", "are", "was", "were",
    "can", "now", "with", "that", "this",
];

pub fn tokenize(text: &str) -> Vec<String> {
    let stop_words: HashSet<&str> = STOP_WORDS.iter().copied().collect();
    text.to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|value| !value.is_empty())
        .map(light_normalize)
        .filter(|value| !value.is_empty())
        .filter(|value| !stop_words.contains(value.as_str()))
        .collect()
}

fn light_normalize(word: &str) -> String {
    if word.len() > 4 && word.ends_with("ies") {
        format!("{}y", &word[..word.len() - 3])
    } else if word.len() > 3 && word.ends_with('s') {
        word[..word.len() - 1].to_string()
    } else {
        word.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_normalizes_and_removes_stop_words() {
        assert_eq!(
            tokenize("Added notes, that the owner can send to students."),
            vec!["added", "note", "owner", "send", "student"]
        );
        assert_eq!(
            tokenize("file-name_test.value"),
            vec!["file", "name", "test", "value"]
        );
    }

    #[test]
    fn normalization_preserves_short_words() {
        assert_eq!(light_normalize("entries"), "entry");
        assert_eq!(light_normalize("notes"), "note");
        assert_eq!(light_normalize("bus"), "bus");
        assert_eq!(light_normalize("gas"), "gas");
    }

    #[test]
    fn empty_and_stop_word_only_text_returns_empty() {
        assert!(tokenize("").is_empty());
        assert!(tokenize("the and or is for").is_empty());
    }
}
