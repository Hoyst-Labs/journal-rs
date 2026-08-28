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

    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, line)| is_heading(line).then_some(index))
        .unwrap_or(lines.len());

    Some(lines[start + 1..end].join("\n").trim().to_string())
}

fn strip_heading_prefix(line: &str) -> Option<&str> {
    line.strip_prefix("## ").or_else(|| line.strip_prefix("# "))
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
    fn extracts_sections_across_h1_h2_and_crlf() {
        let content = "# Summary\r\n\r\n- first\r\n- second\r\n\r\n## Why\r\n\r\nbecause";
        assert_eq!(
            extract_section(content, "summary").as_deref(),
            Some("- first\n- second")
        );
    }

    #[test]
    fn supports_issues_aliases() {
        let content = "## Issues / Unknowns\n\n- blocked";
        assert_eq!(
            extract_section(content, "Issues").as_deref(),
            Some("- blocked")
        );
        assert_eq!(
            extract_section(content, "Unknowns").as_deref(),
            Some("- blocked")
        );
    }

    #[test]
    fn stops_at_either_heading_level() {
        assert_eq!(
            extract_section("# Summary\n\nstuff\n\n## Details\n\nmore", "Summary").as_deref(),
            Some("stuff")
        );
        assert_eq!(
            extract_section("## Summary\n\nstuff\n\n# Next\n\nmore", "Summary").as_deref(),
            Some("stuff")
        );
    }

    #[test]
    fn missing_or_empty_heading_returns_none() {
        assert_eq!(extract_section("# Title\n\nNothing", "Summary"), None);
        assert_eq!(extract_section("# Summary\n\nValue", "  "), None);
    }
}
