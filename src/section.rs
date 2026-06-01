pub fn extract_section(content: &str, heading: &str) -> Option<String> {
    let target_heading = normalize_heading_alias(heading);
    if target_heading.is_empty() {
        return None;
    }

    let normalized_content = content.replace("\r\n", "\n");
    let lines: Vec<&str> = normalized_content.lines().collect();
    let start = lines.iter().position(|line| {
        line.strip_prefix("## ")
            .is_some_and(|value| value.trim().eq_ignore_ascii_case(&target_heading))
    })?;

    let mut end = lines.len();
    for (index, line) in lines.iter().enumerate().skip(start + 1) {
        if line.starts_with("## ") {
            end = index;
            break;
        }
    }

    Some(lines[start + 1..end].join("\n").trim().to_string())
}

fn normalize_heading_alias(heading: &str) -> String {
    let trimmed = heading.trim();
    if trimmed.eq_ignore_ascii_case("issues") || trimmed.eq_ignore_ascii_case("unknowns") {
        "Issues / Unknowns".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_section_until_next_heading() {
        let content = "# Title\n\n## Summary\n\n- first\n- second\n\n## Why\n\nbecause";
        let summary = extract_section(content, "Summary");
        assert_eq!(summary.as_deref(), Some("- first\n- second"));
    }

    #[test]
    fn matches_section_case_insensitively() {
        let content = "## Next Actions\n\n- one";
        let extracted = extract_section(content, "next actions");
        assert_eq!(extracted.as_deref(), Some("- one"));
    }

    #[test]
    fn supports_issues_alias_from_issues_keyword() {
        let content = "## Issues / Unknowns\n\n- blocked";
        let extracted = extract_section(content, "Issues");
        assert_eq!(extracted.as_deref(), Some("- blocked"));
    }

    #[test]
    fn supports_issues_alias_from_unknowns_keyword() {
        let content = "## Issues / Unknowns\n\n- blocked";
        let extracted = extract_section(content, "Unknowns");
        assert_eq!(extracted.as_deref(), Some("- blocked"));
    }

    #[test]
    fn returns_none_when_heading_is_missing() {
        assert_eq!(extract_section("# Title\n\nNothing else", "Summary"), None);
    }

    #[test]
    fn handles_heading_at_end_of_file() {
        let content = "## Verification\n\nPass";
        let extracted = extract_section(content, "Verification");
        assert_eq!(extracted.as_deref(), Some("Pass"));
    }
}
