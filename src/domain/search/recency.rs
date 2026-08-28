use crate::domain::EntryName;

use super::time_bias::TimeBias;

pub fn recency_score(timestamp_unix: i64, now_unix: i64, half_life_days: f64) -> f64 {
    let age_seconds = (now_unix - timestamp_unix).max(0) as f64;
    let age_days = age_seconds / 86_400.0;
    100.0 * (-age_days / half_life_days).exp()
}

pub fn final_score(text_score: f64, recency: f64, time_bias: TimeBias) -> f64 {
    if text_score <= 0.0 {
        return 0.0;
    }

    let recency_weight = match time_bias {
        TimeBias::Recent => {
            if text_score >= 80.0 {
                0.18
            } else if text_score >= 40.0 {
                0.30
            } else {
                0.40
            }
        }
        TimeBias::Old => 0.0,
        TimeBias::Normal => {
            if text_score >= 80.0 {
                0.08
            } else if text_score >= 40.0 {
                0.15
            } else {
                0.25
            }
        }
    };
    text_score * (1.0 - recency_weight) + recency * recency_weight
}

pub fn entry_to_unix_timestamp(entry: &EntryName) -> Option<i64> {
    let filename = entry.as_str();
    let year: i64 = filename.get(0..4)?.parse().ok()?;
    let month: i64 = filename.get(5..7)?.parse().ok()?;
    let day: i64 = filename.get(8..10)?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let (hour, minute) = match entry.timestamp_prefix() {
        Some(timestamp) => {
            let hour: i64 = timestamp.get(11..13)?.parse().ok()?;
            let minute: i64 = timestamp.get(13..15)?.parse().ok()?;
            if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) {
                return None;
            }
            (hour, minute)
        }
        None => (0, 0),
    };

    Some(naive_utc_to_unix(year, month, day, hour, minute))
}

fn naive_utc_to_unix(year: i64, month: i64, day: i64, hour: i64, minute: i64) -> i64 {
    days_from_epoch(year, month, day) * 86_400 + hour * 3600 + minute * 60
}

fn days_from_epoch(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let month = if month <= 2 { month + 9 } else { month - 3 };
    let era = year / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(value: &str) -> EntryName {
        EntryName::parse(value).unwrap()
    }

    #[test]
    fn preserves_recency_decay_and_weighting() {
        let now = 1_700_000_000;
        assert!((recency_score(now, now, 60.0) - 100.0).abs() < 0.01);
        assert!((recency_score(now - 60 * 86_400, now, 60.0) - 36.79).abs() < 0.5);
        assert!((final_score(50.0, 100.0, TimeBias::Normal) - 57.5).abs() < 0.01);
        assert!((final_score(50.0, 100.0, TimeBias::Recent) - 65.0).abs() < 0.01);
        assert!((final_score(50.0, 100.0, TimeBias::Old) - 50.0).abs() < 0.01);
    }

    #[test]
    fn parses_known_entry_timestamps() {
        assert_eq!(
            entry_to_unix_timestamp(&entry("1970-01-01-0000-test.md")),
            Some(0)
        );
        assert!(
            entry_to_unix_timestamp(&entry("2026-05-31-1430-test.md")).unwrap() > 1_700_000_000
        );
    }
}
