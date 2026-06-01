pub fn extract_section(content: &str, heading: &str) -> Option<String> {
    let target_heading = normalize_heading_alias(heading);
    if target_heading.is_empty() {
        return None;
    }

    let normalized_content = content.replace("\r\n", "\n");
    let lines: Vec<&str> = normalized_content.lines().collect();
    let start = lines.iter().position(|line| {
        strip_heading_prefix(line)
            .is_some_and(|value| value.trim().eq_ignore_ascii_case(&target_heading))
    })?;

    let mut end = lines.len();
    for (index, line) in lines.iter().enumerate().skip(start + 1) {
        if is_heading(line) {
            end = index;
            break;
        }
    }

    Some(lines[start + 1..end].join("\n").trim().to_string())
}

fn strip_heading_prefix(line: &str) -> Option<&str> {
    line.strip_prefix("## ")
        .or_else(|| line.strip_prefix("# "))
}

fn is_heading(line: &str) -> bool {
    line.starts_with("## ") || line.starts_with("# ")
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

    #[test]
    fn extracts_h1_section() {
        let content = "# Summary\n\n- done\n- shipped\n\n# Context\n\nmore stuff";
        let extracted = extract_section(content, "Summary");
        assert_eq!(extracted.as_deref(), Some("- done\n- shipped"));
    }

    #[test]
    fn extracts_h1_section_at_end_of_file() {
        let content = "# Context\n\nsome context\n\n# Summary\n\nfinal notes";
        let extracted = extract_section(content, "Summary");
        assert_eq!(extracted.as_deref(), Some("final notes"));
    }

    #[test]
    fn h1_issues_alias_works() {
        let content = "# Issues / Unknowns\n\n- blocked on API";
        let extracted = extract_section(content, "Issues");
        assert_eq!(extracted.as_deref(), Some("- blocked on API"));
    }

    #[test]
    fn h2_boundary_stops_h1_section() {
        let content = "# Summary\n\nstuff\n\n## Details\n\nmore";
        let extracted = extract_section(content, "Summary");
        assert_eq!(extracted.as_deref(), Some("stuff"));
    }

    #[test]
    fn h1_boundary_stops_h2_section() {
        let content = "## Summary\n\nstuff\n\n# Next\n\nmore";
        let extracted = extract_section(content, "Summary");
        assert_eq!(extracted.as_deref(), Some("stuff"));
    }
}
