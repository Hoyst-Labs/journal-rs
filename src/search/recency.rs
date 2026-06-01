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

    let text_weight = 1.0 - recency_weight;
    text_score * text_weight + recency * recency_weight
}

pub fn filename_to_unix_timestamp(filename: &str) -> Option<i64> {
    if filename.len() < 10 {
        return None;
    }

    let year: i64 = filename[0..4].parse().ok()?;
    if filename.as_bytes().get(4) != Some(&b'-') {
        return None;
    }
    let month: i64 = filename[5..7].parse().ok()?;
    if filename.as_bytes().get(7) != Some(&b'-') {
        return None;
    }
    let day: i64 = filename[8..10].parse().ok()?;

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let (hour, minute) = if filename.len() >= 15
        && filename.as_bytes().get(10) == Some(&b'-')
        && filename[11..15].chars().all(|c| c.is_ascii_digit())
    {
        let h: i64 = filename[11..13].parse().ok()?;
        let m: i64 = filename[13..15].parse().ok()?;
        if !(0..=23).contains(&h) || !(0..=59).contains(&m) {
            return None;
        }
        (h, m)
    } else {
        (0, 0)
    };

    Some(naive_utc_to_unix(year, month, day, hour, minute))
}

fn naive_utc_to_unix(year: i64, month: i64, day: i64, hour: i64, minute: i64) -> i64 {
    let days = days_from_epoch(year, month, day);
    days * 86_400 + hour * 3600 + minute * 60
}

fn days_from_epoch(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let m = if month <= 2 { month + 9 } else { month - 3 };

    let era = y / 400;
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;

    era * 146097 + doe - 719468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recency_today_is_100() {
        let now = 1_700_000_000;
        let score = recency_score(now, now, 60.0);
        assert!((score - 100.0).abs() < 0.01);
    }

    #[test]
    fn recency_at_half_life() {
        let now = 1_700_000_000;
        let sixty_days_ago = now - 60 * 86_400;
        let score = recency_score(sixty_days_ago, now, 60.0);
        // e^(-1) ≈ 0.368
        assert!((score - 36.79).abs() < 0.5);
    }

    #[test]
    fn recency_at_120_days() {
        let now = 1_700_000_000;
        let old = now - 120 * 86_400;
        let score = recency_score(old, now, 60.0);
        // e^(-2) ≈ 0.135
        assert!((score - 13.53).abs() < 0.5);
    }

    #[test]
    fn final_score_zero_text_returns_zero() {
        assert_eq!(final_score(0.0, 100.0, TimeBias::Normal), 0.0);
    }

    #[test]
    fn final_score_high_text_low_recency_weight() {
        let result = final_score(100.0, 100.0, TimeBias::Normal);
        // 100 * 0.92 + 100 * 0.08 = 100
        assert!((result - 100.0).abs() < 0.01);
    }

    #[test]
    fn final_score_mid_text_normal_weight() {
        let result = final_score(50.0, 100.0, TimeBias::Normal);
        // 50 * 0.85 + 100 * 0.15 = 42.5 + 15 = 57.5
        assert!((result - 57.5).abs() < 0.01);
    }

    #[test]
    fn final_score_low_text_higher_recency_weight() {
        let result = final_score(30.0, 100.0, TimeBias::Normal);
        // 30 * 0.75 + 100 * 0.25 = 22.5 + 25 = 47.5
        assert!((result - 47.5).abs() < 0.01);
    }

    #[test]
    fn final_score_recent_bias_amplifies_recency() {
        let result = final_score(50.0, 100.0, TimeBias::Recent);
        // 50 * 0.70 + 100 * 0.30 = 35 + 30 = 65
        assert!((result - 65.0).abs() < 0.01);
    }

    #[test]
    fn final_score_old_bias_ignores_recency() {
        let result = final_score(50.0, 100.0, TimeBias::Old);
        // 50 * 1.0 + 100 * 0.0 = 50
        assert!((result - 50.0).abs() < 0.01);
    }

    #[test]
    fn filename_to_unix_timestamp_full_datetime() {
        let ts = filename_to_unix_timestamp("2026-05-31-1430-something.md");
        assert!(ts.is_some());
        let ts = ts.unwrap();
        // 2026-05-31 14:30 UTC - just verify it's a reasonable value
        assert!(ts > 1_700_000_000);
    }

    #[test]
    fn filename_to_unix_timestamp_date_only() {
        let ts = filename_to_unix_timestamp("2026-05-31-something.md");
        assert!(ts.is_some());
        let ts = ts.unwrap();
        assert!(ts > 1_700_000_000);
    }

    #[test]
    fn filename_to_unix_timestamp_invalid() {
        assert!(filename_to_unix_timestamp("not-a-date.md").is_none());
        assert!(filename_to_unix_timestamp("short").is_none());
    }

    #[test]
    fn known_epoch_date() {
        // 1970-01-01 should be 0
        let ts = filename_to_unix_timestamp("1970-01-01-0000-test.md");
        assert_eq!(ts, Some(0));
    }
}
