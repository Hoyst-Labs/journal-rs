use std::collections::HashSet;

pub static STOP_WORDS: &[&str] = &[
    "a", "an", "the", "to", "for", "of", "in", "on", "and", "or", "is", "are", "was", "were",
    "can", "now", "with", "that", "this",
];

pub fn tokenize(text: &str) -> Vec<String> {
    let stop_words: HashSet<&str> = STOP_WORDS.iter().copied().collect();

    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(light_normalize)
        .filter(|s| !s.is_empty())
        .filter(|s| !stop_words.contains(s.as_str()))
        .collect()
}

pub fn light_normalize(word: &str) -> String {
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
    fn tokenizes_basic_text() {
        let result = tokenize("Hello World");
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn removes_stop_words() {
        let result = tokenize("notes for the students");
        assert_eq!(result, vec!["note", "student"]);
    }

    #[test]
    fn normalizes_plurals() {
        assert_eq!(light_normalize("students"), "student");
        assert_eq!(light_normalize("notes"), "note");
        assert_eq!(light_normalize("entries"), "entry");
    }

    #[test]
    fn preserves_short_words_ending_in_s() {
        assert_eq!(light_normalize("bus"), "bus");
        assert_eq!(light_normalize("gas"), "gas");
    }

    #[test]
    fn all_stop_words_returns_empty() {
        let result = tokenize("the and or is for");
        assert!(result.is_empty());
    }

    #[test]
    fn empty_string_returns_empty() {
        let result = tokenize("");
        assert!(result.is_empty());
    }

    #[test]
    fn handles_mixed_case_and_punctuation() {
        let result = tokenize("Added notes, that the owner can send to students.");
        assert_eq!(result, vec!["added", "note", "owner", "send", "student"]);
    }

    #[test]
    fn splits_on_non_alphanumeric() {
        let result = tokenize("file-name_test.value");
        assert_eq!(result, vec!["file", "name", "test", "value"]);
    }
}
