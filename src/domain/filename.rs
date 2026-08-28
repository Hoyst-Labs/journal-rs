use super::EntryName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalMoment {
    value: String,
    precision: Precision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Precision {
    Date,
    DateTime,
}

impl JournalMoment {
    pub fn parse(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        let precision = if is_valid_date(&value) {
            Precision::Date
        } else if is_valid_date_time(&value) {
            Precision::DateTime
        } else {
            return None;
        };

        Some(Self { value, precision })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn normalized_start(&self) -> String {
        match self.precision {
            Precision::Date => format!("{}-0000", self.value),
            Precision::DateTime => self.value.clone(),
        }
    }

    pub fn normalized_end(&self) -> String {
        match self.precision {
            Precision::Date => format!("{}-9999", self.value),
            Precision::DateTime => self.value.clone(),
        }
    }

    pub fn matches_since(&self, entry: &EntryName) -> bool {
        match self.precision {
            Precision::Date => entry.date_prefix() >= self.value.as_str(),
            Precision::DateTime => comparable_datetime(entry) >= self.value,
        }
    }

    pub fn contains_as_start(&self, entry: &EntryName) -> bool {
        self.matches_since(entry)
    }

    pub fn contains_as_end(&self, entry: &EntryName) -> bool {
        match self.precision {
            Precision::Date => entry.date_prefix() <= self.value.as_str(),
            Precision::DateTime => comparable_datetime(entry) <= self.value,
        }
    }
}

fn comparable_datetime(entry: &EntryName) -> String {
    entry
        .timestamp_prefix()
        .map(str::to_string)
        .unwrap_or_else(|| format!("{}-0000", entry.date_prefix()))
}

pub fn extract_date_prefix(value: &str) -> Option<&str> {
    let prefix = value.get(0..10)?;
    is_valid_date(prefix).then_some(prefix)
}

pub fn extract_timestamp_prefix(value: &str) -> Option<&str> {
    let prefix = value.get(0..15)?;
    is_valid_date_time(prefix).then_some(prefix)
}

pub fn is_valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
}

pub fn is_valid_date_time(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 15
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7 | 10) || byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(value: &str) -> EntryName {
        EntryName::parse(value).unwrap()
    }

    #[test]
    fn parses_date_and_datetime_moments() {
        assert!(JournalMoment::parse("2026-05-01").is_some());
        assert!(JournalMoment::parse("2026-05-01-0930").is_some());
        assert!(JournalMoment::parse("2026/05/01").is_none());
    }

    #[test]
    fn compares_mixed_precision_bounds() {
        let value = entry("2026-04-04-1525-entry.md");
        assert!(
            JournalMoment::parse("2026-04-04")
                .unwrap()
                .matches_since(&value)
        );
        assert!(
            JournalMoment::parse("2026-04-04-1500")
                .unwrap()
                .matches_since(&value)
        );
        assert!(
            JournalMoment::parse("2026-04-04")
                .unwrap()
                .contains_as_end(&value)
        );
        assert!(
            !JournalMoment::parse("2026-04-04-1500")
                .unwrap()
                .contains_as_end(&value)
        );
    }
}
