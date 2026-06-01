## Minimal Rust CLI Template

Use this as a compact, CLI-focused starting point.

```rust
// src/main.rs
use clap::{Parser, Subcommand, ValueEnum};

fn main() {
    let cli = Cli::parse();

    let result = run(cli);
    std::process::exit(result.code());
}

#[derive(Debug, Parser)]
#[command(name = "my-cli", version, about = "Example Rust CLI")]
struct Cli {
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    List,
    Show { id: String },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy)]
enum Exit {
    Ok,
    InvalidUsage,
    Failure,
}

impl Exit {
    fn code(self) -> i32 {
        match self {
            Self::Ok => 0,
            Self::InvalidUsage => 2,
            Self::Failure => 1,
        }
    }
}

fn run(cli: Cli) -> Exit {
    match execute(cli) {
        Ok(output) => {
            println!("{output}");
            Exit::Ok
        }
        Err(error) => {
            eprintln!("{error}");
            match error.kind {
                CliErrorKind::InvalidUsage => Exit::InvalidUsage,
                CliErrorKind::Failure => Exit::Failure,
            }
        }
    }
}

fn execute(cli: Cli) -> Result<String, CliError> {
    match cli.command {
        Command::List => render_list(cli.format),
        Command::Show { id } => render_item(id, cli.format),
    }
}

fn render_list(format: Format) -> Result<String, CliError> {
    let items = vec!["a", "b", "c"];

    match format {
        Format::Text => Ok(items.join("\n")),
        Format::Json => Ok(format!("[\"{}\"]", items.join("\",\""))),
    }
}

fn render_item(id: String, format: Format) -> Result<String, CliError> {
    if id.trim().is_empty() {
        return Err(CliError::invalid_usage("id cannot be empty"));
    }

    match format {
        Format::Text => Ok(format!("item: {id}")),
        Format::Json => Ok(format!("{{\"id\":\"{id}\"}}")),
    }
}

#[derive(Debug)]
struct CliError {
    kind: CliErrorKind,
    message: String,
}

impl CliError {
    fn invalid_usage(message: impl Into<String>) -> Self {
        Self {
            kind: CliErrorKind::InvalidUsage,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "error: {}", self.message)
    }
}

impl std::error::Error for CliError {}

#[derive(Debug, Clone, Copy)]
enum CliErrorKind {
    InvalidUsage,
    Failure,
}
```

Template properties:

- Parsing and execution are separated.
- `stdout` and `stderr` responsibilities are clear.
- Exit-code mapping is explicit.
- Human and machine output modes are modeled.
