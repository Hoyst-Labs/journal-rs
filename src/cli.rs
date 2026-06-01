use crate::journal::{is_valid_date, is_valid_date_or_time};
use crate::model::{DisplayMode, QueryParams};

pub fn parse_args(args: &[String]) -> Result<QueryParams, String> {
    let positional_command = args.first().map(String::as_str);
    let help_requested =
        args.iter().any(|arg| arg == "--help" || arg == "-h") || positional_command == Some("help");

    if help_requested {
        return Ok(QueryParams {
            help_requested: true,
            ..QueryParams::default()
        });
    }

    let mut summary_requested = positional_command == Some("summary");
    let mut full_requested = positional_command == Some("full");
    let mut type_heading: Option<String> = None;
    let mut params = QueryParams::default();

    if matches!(positional_command, Some("files" | "full")) {
        params.files_query = args
            .get(1)
            .filter(|value| !value.starts_with("--"))
            .cloned();
    }

    let has_known_command = matches!(
        positional_command,
        Some("help" | "summary" | "full" | "files")
    );
    let mut index = if has_known_command { 1 } else { 0 };

    while index < args.len() {
        match args[index].as_str() {
            "--summary" => {
                summary_requested = true;
                index += 1;
            }
            "--full" => {
                full_requested = true;
                index += 1;
            }
            "--type" => {
                let heading = args
                    .get(index + 1)
                    .filter(|value| !value.starts_with("--"))
                    .cloned()
                    .ok_or_else(|| "--type requires a heading value.".to_string())?;
                type_heading = Some(heading);
                index += 2;
            }
            "--files" => {
                let query = args
                    .get(index + 1)
                    .filter(|value| !value.starts_with("--"))
                    .cloned()
                    .ok_or_else(|| "--files requires a date or timestamp query.".to_string())?;
                params.files_query = Some(query);
                index += 2;
            }
            "--since" => {
                let value = args
                    .get(index + 1)
                    .filter(|candidate| !candidate.starts_with("--"))
                    .cloned()
                    .ok_or_else(|| {
                        "--since requires a value in YYYY-MM-DD or YYYY-MM-DD-HHmm format."
                            .to_string()
                    })?;

                if !is_valid_date_or_time(&value) {
                    return Err("--since value must be YYYY-MM-DD or YYYY-MM-DD-HHmm.".to_string());
                }

                params.since = Some(value);
                index += 2;
            }
            "--between" => {
                let start = args
                    .get(index + 1)
                    .filter(|candidate| !candidate.starts_with("--"))
                    .cloned()
                    .ok_or_else(|| "--between requires a start value.".to_string())?;
                let end = args
                    .get(index + 2)
                    .filter(|candidate| !candidate.starts_with("--"))
                    .cloned()
                    .ok_or_else(|| "--between requires an end value.".to_string())?;

                if !is_valid_date_or_time(&start) || !is_valid_date_or_time(&end) {
                    return Err(
                        "--between values must be YYYY-MM-DD or YYYY-MM-DD-HHmm.".to_string()
                    );
                }

                params.between = Some((start, end));
                index += 3;
            }
            "--filter" => {
                let pattern = args
                    .get(index + 1)
                    .filter(|value| !value.starts_with("--"))
                    .cloned()
                    .ok_or_else(|| "--filter requires a pipe-delimited value.".to_string())?;
                let terms = parse_filter_terms(&pattern)?;
                params.filter_terms = Some(terms);
                index += 2;
            }
            token if token.starts_with("--") => {
                return Err(format!("Unknown option: {token}"));
            }
            _ => {
                index += 1;
            }
        }
    }

    if !has_known_command
        && !args.is_empty()
        && !args[0].starts_with("--")
        && params.files_query.is_none()
    {
        params.files_query = Some(args[0].clone());
    }

    if summary_requested && type_heading.is_some() {
        return Err("Cannot use --summary with --type. Use one display selector.".to_string());
    }

    if params.since.is_some() && params.between.is_some() {
        return Err("Cannot use --since with --between.".to_string());
    }

    if let Some((start, end)) = params.between.as_ref() {
        if normalize_range_start(start) > normalize_range_end(end) {
            return Err("Invalid --between range: start must be <= end.".to_string());
        }
    }

    params.display_mode = if let Some(heading) = type_heading {
        DisplayMode::TypeSection(heading)
    } else if summary_requested {
        DisplayMode::Summary
    } else if full_requested {
        DisplayMode::Full
    } else {
        DisplayMode::List
    };

    Ok(params)
}

fn parse_filter_terms(pattern: &str) -> Result<Vec<String>, String> {
    let terms: Vec<String> = pattern
        .split('|')
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .map(|term| term.to_ascii_lowercase())
        .collect();

    if terms.is_empty() {
        return Err("--filter must contain at least one non-empty term.".to_string());
    }

    Ok(terms)
}

fn normalize_range_start(value: &str) -> String {
    if is_valid_date(value) {
        format!("{value}-0000")
    } else {
        value.to_string()
    }
}

fn normalize_range_end(value: &str) -> String {
    if is_valid_date(value) {
        format!("{value}-9999")
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_summary_and_type_together() {
        let args = vec![
            "--summary".to_string(),
            "--type".to_string(),
            "Summary".to_string(),
        ];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn rejects_since_and_between_together() {
        let args = vec![
            "--since".to_string(),
            "2026-05-01".to_string(),
            "--between".to_string(),
            "2026-04-01".to_string(),
            "2026-04-30".to_string(),
        ];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn parses_type_since_and_filter() {
        let args = vec![
            "--type".to_string(),
            "Next Actions".to_string(),
            "--since".to_string(),
            "2026-04-26".to_string(),
            "--filter".to_string(),
            "deploy|release".to_string(),
        ];
        let params = parse_args(&args).unwrap();

        assert_eq!(
            params.display_mode,
            DisplayMode::TypeSection("Next Actions".to_string())
        );
        assert_eq!(params.since.as_deref(), Some("2026-04-26"));
        assert_eq!(
            params.filter_terms,
            Some(vec!["deploy".to_string(), "release".to_string()])
        );
    }

    #[test]
    fn parses_positional_files_command() {
        let args = vec!["files".to_string(), "2026-04-04-1054".to_string()];
        let params = parse_args(&args).unwrap();
        assert_eq!(params.files_query.as_deref(), Some("2026-04-04-1054"));
        assert_eq!(params.display_mode, DisplayMode::List);
    }

    #[test]
    fn parses_bare_query_as_files_filter() {
        let args = vec!["2026-04-04".to_string()];
        let params = parse_args(&args).unwrap();
        assert_eq!(params.files_query.as_deref(), Some("2026-04-04"));
    }

    #[test]
    fn rejects_between_with_invalid_order() {
        let args = vec![
            "--between".to_string(),
            "2026-05-02".to_string(),
            "2026-05-01".to_string(),
        ];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn accepts_between_with_mixed_precision_bounds() {
        let args = vec![
            "--between".to_string(),
            "2026-04-04-1500".to_string(),
            "2026-04-04".to_string(),
        ];
        assert!(parse_args(&args).is_ok());
    }
}
