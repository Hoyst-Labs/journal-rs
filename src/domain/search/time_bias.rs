#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeBias {
    Normal,
    Recent,
    Old,
}

const RECENT_KEYWORDS: &[&str] = &["recent", "latest", "newest"];
const OLD_KEYWORDS: &[&str] = &["old", "older", "earliest"];

pub fn detect_time_bias(query: &str) -> (TimeBias, String) {
    let mut bias = TimeBias::Normal;
    let mut cleaned_words = Vec::new();

    for word in query.split_whitespace() {
        let lower = word.to_lowercase();
        if RECENT_KEYWORDS.contains(&lower.as_str()) {
            bias = TimeBias::Recent;
        } else if OLD_KEYWORDS.contains(&lower.as_str()) {
            bias = TimeBias::Old;
        } else {
            cleaned_words.push(word);
        }
    }

    (bias, cleaned_words.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_and_removes_time_bias_keywords() {
        assert_eq!(
            detect_time_bias("recent auth changes"),
            (TimeBias::Recent, "auth changes".to_string())
        );
        assert_eq!(
            detect_time_bias("earliest deploy notes"),
            (TimeBias::Old, "deploy notes".to_string())
        );
        assert_eq!(
            detect_time_bias("deploy notes"),
            (TimeBias::Normal, "deploy notes".to_string())
        );
    }

    #[test]
    fn last_time_bias_keyword_wins() {
        assert_eq!(
            detect_time_bias("recent old stuff"),
            (TimeBias::Old, "stuff".to_string())
        );
    }
}
