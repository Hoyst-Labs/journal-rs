mod common;

use common::{TestDir, fixture_dir, run_journal, stderr, stdout};

const EXPECTED_HELP: &str = "Usage: journal [options]

Options:
  --summary                    Print each matching journal file's ## Summary section.
  --full                       Print each matching journal file's full contents.
  --type <heading>             Print each matching journal file's ## <heading> section.
  --files <query>              Filter files by date/timestamp prefix (YYYY-MM-DD or YYYY-MM-DD-HHmm).
  --since <value>              Include only files with prefix >= value.
  --between <start> <end>      Include only files with start <= prefix <= end.
  --filter <term|term|...>     Include files whose content matches any case-insensitive term.
  --search <query>             Ranked keyword search across entry summaries (scored by relevance + recency).
  --latest <count>             Show the <count> most recent entries (defaults to summary display).
  --help, -h                   Show this help message.
  help                         Show this help message.

Notes:
  --summary and --type are mutually exclusive.
  --since and --between are mutually exclusive.
  --search and --filter are mutually exclusive.

Journal Discovery:
  The tool looks only in the current working directory for:
  - ./.journal
  - ./journal

  If neither directory exists, the command prints:
  No journal entries found.

Examples:
  journal
  journal --summary
  journal --latest 3
  journal --latest 5 --full
  journal --type \"Next Actions\"
  journal --full --files 2026-04-04-1054
  journal --type \"Issues\" --since 2026-04-26
  journal --type \"Next Actions\" --between 2026-04-01 2026-04-30 --filter \"deploy|release\"
  journal --search \"notes for students\"
  journal --search \"recent auth changes\" --since 2026-04-01
  journal search \"deploy fixes\"
  journal files 2026-04-04-1054
  journal help
";

#[test]
fn help_aliases_preserve_exact_stdout() {
    let current_dir = fixture_dir("journal");
    for args in [&["help"][..], &["--help"][..], &["-h"][..]] {
        let output = run_journal(&current_dir, args);
        assert!(output.status.success());
        assert_eq!(stdout(&output), EXPECTED_HELP);
        assert_eq!(stderr(&output), "");
    }
}

#[test]
fn missing_store_is_a_successful_exact_empty_state() {
    let current_dir = TestDir::new("missing");
    let output = run_journal(current_dir.path(), &[]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "No journal entries found.\n");
    assert_eq!(stderr(&output), "");
}

#[test]
fn default_listing_is_grouped_and_newest_first() {
    let output = run_journal(&fixture_dir("journal"), &[]);

    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        concat!(
            "2026-05-03\n",
            " - 2026-05-03-1100-third.md\n\n",
            "2026-05-02\n",
            " - 2026-05-02-1000-second.md\n\n",
            "2026-05-01\n",
            " - 2026-05-01-0900-first.md\n"
        )
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn summary_and_latest_preserve_block_formatting() {
    let current_dir = fixture_dir("journal");
    let summary = run_journal(&current_dir, &["summary", "2026-05-02"]);
    let latest = run_journal(&current_dir, &["--latest", "2"]);

    assert!(summary.status.success());
    assert_eq!(
        stdout(&summary),
        concat!(
            "2026-05-03-1100-third.md\n",
            "Deployed the service to production.\n\n",
            "2026-05-02-1000-second.md\n",
            "Second entry summary.\n\n",
            "2026-05-01-0900-first.md\n",
            "First entry summary.\n"
        )
    );
    assert_eq!(stderr(&summary), "");

    assert!(latest.status.success());
    assert_eq!(
        stdout(&latest),
        concat!(
            "2026-05-03-1100-third.md\n",
            "Deployed the service to production.\n\n",
            "2026-05-02-1000-second.md\n",
            "Second entry summary.\n"
        )
    );
    assert_eq!(stderr(&latest), "");
}

#[test]
fn full_and_named_section_preserve_aliases_and_filters() {
    let current_dir = fixture_dir("journal");
    let full = run_journal(&current_dir, &["full", "2026-05-01-0900"]);
    let section = run_journal(
        &current_dir,
        &[
            "--type",
            "Next Actions",
            "--since",
            "2026-05-02",
            "--filter",
            "deploy|missing",
        ],
    );

    assert!(full.status.success());
    assert_eq!(
        stdout(&full),
        concat!(
            "2026-05-01-0900-first.md\n",
            "# First Entry\n\n",
            "## Summary\n\n",
            "First entry summary.\n\n",
            "## Next Actions\n\n",
            "- archive docs\n"
        )
    );
    assert_eq!(stderr(&full), "");

    assert!(section.status.success());
    assert_eq!(
        stdout(&section),
        concat!(
            "2026-05-03-1100-third.md\n",
            "- verify production\n\n",
            "2026-05-02-1000-second.md\n",
            "- deploy release\n"
        )
    );
    assert_eq!(stderr(&section), "");
}

#[test]
fn search_alias_has_stable_score_and_preview_format() {
    let output = run_journal(
        &fixture_dir("journal"),
        &["search", "old deploy production"],
    );

    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        concat!(
            "[12.00] 2026-05-03-1100-third.md\n",
            "  Deployed the service to production.\n"
        )
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn no_search_matches_is_a_successful_exact_empty_state() {
    let output = run_journal(&fixture_dir("journal"), &["--search", "unfindable"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "No matching entries found.\n");
    assert_eq!(stderr(&output), "");
}

#[test]
fn invalid_usage_writes_only_to_stderr_and_exits_one() {
    let current_dir = fixture_dir("journal");
    let unknown = run_journal(&current_dir, &["--unknown"]);
    let conflict = run_journal(&current_dir, &["--search", "deploy", "--filter", "deploy"]);
    let range = run_journal(&current_dir, &["--between", "2026-05-03", "2026-05-01"]);

    assert_eq!(unknown.status.code(), Some(1));
    assert_eq!(stdout(&unknown), "");
    assert_eq!(stderr(&unknown), "Unknown option: --unknown\n");

    assert_eq!(conflict.status.code(), Some(1));
    assert_eq!(stdout(&conflict), "");
    assert_eq!(
        stderr(&conflict),
        "--search and --filter cannot be used together.\n"
    );

    assert_eq!(range.status.code(), Some(1));
    assert_eq!(stdout(&range), "");
    assert_eq!(
        stderr(&range),
        "Invalid --between range: start must be <= end.\n"
    );
}

#[test]
fn missing_values_preserve_exact_diagnostics() {
    let current_dir = fixture_dir("journal");
    let cases = [
        (&["--type"][..], "--type requires a heading value.\n"),
        (
            &["--since"][..],
            "--since requires a value in YYYY-MM-DD or YYYY-MM-DD-HHmm format.\n",
        ),
        (
            &["--between", "2026-05-01"][..],
            "--between requires an end value.\n",
        ),
        (&["--latest"][..], "--latest requires a positive integer.\n"),
        (&["--search"][..], "--search requires a query string.\n"),
    ];

    for (args, expected) in cases {
        let output = run_journal(&current_dir, args);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(stdout(&output), "");
        assert_eq!(stderr(&output), expected);
    }
}

#[test]
fn hidden_journal_takes_precedence_over_plain_journal() {
    let current_dir = TestDir::new("precedence");
    current_dir.write(
        ".journal/2026-05-03-1000-hidden.md",
        "## Summary\n\nHidden store\n",
    );
    current_dir.write(
        "journal/2026-05-04-1000-plain.md",
        "## Summary\n\nPlain store\n",
    );

    let output = run_journal(current_dir.path(), &["--summary"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "2026-05-03-1000-hidden.md\nHidden store\n");
    assert_eq!(stderr(&output), "");
}
