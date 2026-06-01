pub const HELP_TEXT: &str = "Usage: journal [options]

Options:
  --summary                    Print each matching journal file's ## Summary section.
  --full                       Print each matching journal file's full contents.
  --type <heading>             Print each matching journal file's ## <heading> section.
  --files <query>              Filter files by date/timestamp prefix (YYYY-MM-DD or YYYY-MM-DD-HHmm).
  --since <value>              Include only files with prefix >= value.
  --between <start> <end>      Include only files with start <= prefix <= end.
  --filter <term|term|...>     Include files whose content matches any case-insensitive term.
  --latest <count>             Show the <count> most recent entries (defaults to summary display).
  --help, -h                   Show this help message.
  help                         Show this help message.

Notes:
  --summary and --type are mutually exclusive.
  --since and --between are mutually exclusive.

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
  journal files 2026-04-04-1054
  journal help";
