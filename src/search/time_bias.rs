#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeBias {
    Normal,
    Recent,
    Old,
}

const RECENT_KEYWORDS: &[&str] = &["recent", "latest", "newest"];
const OLD_KEYWORDS: &[&str] = &["old", "older", "earliest"];

pub fn detect_time_bias(query: &str) -> (TimeBias, String) {
    let words: Vec<&str> = query.split_whitespace().collect();
    let mut bias = TimeBias::Normal;
    let mut cleaned_words: Vec<&str> = Vec::new();

    for word in &words {
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
    fn detects_recent_bias() {
        let (bias, cleaned) = detect_time_bias("recent auth changes");
        assert_eq!(bias, TimeBias::Recent);
        assert_eq!(cleaned, "auth changes");
    }

    #[test]
    fn detects_latest_bias() {
        let (bias, cleaned) = detect_time_bias("latest deploy notes");
        assert_eq!(bias, TimeBias::Recent);
        assert_eq!(cleaned, "deploy notes");
    }

    #[test]
    fn detects_newest_bias() {
        let (bias, cleaned) = detect_time_bias("newest updates");
        assert_eq!(bias, TimeBias::Recent);
        assert_eq!(cleaned, "updates");
    }

    #[test]
    fn detects_old_bias() {
        let (bias, cleaned) = detect_time_bias("old auth setup");
        assert_eq!(bias, TimeBias::Old);
        assert_eq!(cleaned, "auth setup");
    }

    #[test]
    fn detects_older_bias() {
        let (bias, cleaned) = detect_time_bias("older migration notes");
        assert_eq!(bias, TimeBias::Old);
        assert_eq!(cleaned, "migration notes");
    }

    #[test]
    fn detects_earliest_bias() {
        let (bias, cleaned) = detect_time_bias("earliest entries");
        assert_eq!(bias, TimeBias::Old);
        assert_eq!(cleaned, "entries");
    }

    #[test]
    fn no_bias_returns_normal() {
        let (bias, cleaned) = detect_time_bias("deploy notes");
        assert_eq!(bias, TimeBias::Normal);
        assert_eq!(cleaned, "deploy notes");
    }

    #[test]
    fn keyword_mid_sentence() {
        let (bias, cleaned) = detect_time_bias("notes from recent deploys");
        assert_eq!(bias, TimeBias::Recent);
        assert_eq!(cleaned, "notes from deploys");
    }

    #[test]
    fn multiple_bias_keywords_last_wins() {
        let (bias, cleaned) = detect_time_bias("recent old stuff");
        assert_eq!(bias, TimeBias::Old);
        assert_eq!(cleaned, "stuff");
    }
}
