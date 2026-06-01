## Configuration

Configuration should be loaded once near the edge, validated, and passed inward.

```rust
use std::{env, path::PathBuf};

use crate::errors::AppError;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub data_dir: PathBuf,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, AppError> {
        let data_dir = env::var("APP_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./data"));

        let log_level = env::var("APP_LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());

        Ok(Self {
            data_dir,
            log_level,
        })
    }
}
```

Avoid reading environment variables from deep inside domain or application logic.

Bad:

```rust
pub fn find_data_dir() -> PathBuf {
    std::env::var("APP_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./data"))
}
```

Good:

```rust
pub fn build_repository(config: &AppConfig) -> FsDocumentRepository {
    FsDocumentRepository::new(config.data_dir.clone())
}
```

---

## Error Handling

Use a central application error type.

With no external crates:

```rust
use std::{fmt, io};

use crate::domain::DomainError;

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Domain(DomainError),
    InvalidConfig(String),
    NotFound(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Domain(error) => write!(formatter, "Domain error: {error:?}"),
            Self::InvalidConfig(message) => write!(formatter, "Invalid config: {message}"),
            Self::NotFound(id) => write!(formatter, "Not found: {id}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<DomainError> for AppError {
    fn from(value: DomainError) -> Self {
        Self::Domain(value)
    }
}
```

With `thiserror`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("domain error: {0}")]
    Domain(#[from] crate::domain::DomainError),

    #[error("invalid config: {0}")]
    InvalidConfig(String),

    #[error("not found: {0}")]
    NotFound(String),
}
```

With `anyhow`, use it mainly at binary boundaries, prototypes, tests, or top-level orchestration. Prefer typed errors for reusable library/application code.

Good split:

```rust
// lib/application code
pub fn run_use_case(&self) -> Result<UseCaseResult, AppError> {
    // ...
}

// binary edge
fn main() -> anyhow::Result<()> {
    let config = AppConfig::from_env()?;
    // ...
    Ok(())
}
```


## Result Types

Create aliases only when they improve clarity.

```rust
pub type AppResult<T> = Result<T, AppError>;
```

Then:

```rust
pub fn execute(&self, request: Request) -> AppResult<Response> {
    // ...
    Ok(Response::default())
}
```

Do not hide important error information behind `Option`:

```rust
// Bad
pub fn load_config() -> Option<AppConfig>;

// Good
pub fn load_config() -> Result<AppConfig, AppError>;
```

Use `Option` only when absence is expected and not an error.



## Data Modeling

Prefer domain-specific types.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(String);

impl UserId {
    pub fn parse(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();

        if value.trim().is_empty() {
            return Err(DomainError::InvalidUserId);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

Use enums for known states:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountStatus {
    Pending,
    Active,
    Suspended,
    Closed,
}
```

Avoid boolean traps:

```rust
// Bad
pub fn create_user(email: String, active: bool, admin: bool);

// Better
pub struct CreateUserRequest {
    pub email: EmailAddress,
    pub status: AccountStatus,
    pub role: UserRole,
}
```

Use request/response structs for use cases:

```rust
#[derive(Debug, Clone)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct CreateDocumentResponse {
    pub id: DocumentId,
}
```



## Formatting and Presentation

Formatting should be separate from application logic.

```rust
use crate::app::DocumentSummary;

pub fn render_summaries_text(summaries: &[DocumentSummary]) -> String {
    if summaries.is_empty() {
        return "No documents found.".to_string();
    }

    summaries
        .iter()
        .map(|summary| {
            let body = summary
                .summary
                .as_deref()
                .unwrap_or("[No summary found]");

            format!("{}\n{}", summary.title, body)
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}
```

For JSON, prefer serializable DTOs:

```rust
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DocumentSummaryDto {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
}
```

Do not make core application functions return formatted text unless the use case itself is specifically text generation.


## Logging and Tracing

Use structured logs at boundaries and meaningful transitions.

Recommended crates:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
```

Setup:

```rust
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
        )
        .init();
}
```

Use instrumentation:

```rust
#[tracing::instrument(skip(self))]
pub fn summarize_documents(
    &self,
    query: DocumentQuery,
) -> Result<Vec<DocumentSummary>, AppError> {
    tracing::debug!("summarizing documents");

    let documents = self.documents.find(query)?;

    tracing::info!(count = documents.len(), "documents loaded");

    // ...
    Ok(Vec::new())
}
```

Do not use logging as error handling. Return errors.



## Async Boundaries

Use async only when needed for IO, concurrency, or framework integration.

Async port:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait DocumentRepository {
    async fn find(&self, query: DocumentQuery) -> Result<Vec<Document>, AppError>;
}
```

Async app:

```rust
pub struct App<R> {
    repository: R,
}

impl<R> App<R>
where
    R: DocumentRepository + Send + Sync,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn summarize_documents(
        &self,
        query: DocumentQuery,
    ) -> Result<Vec<DocumentSummary>, AppError> {
        let documents = self.repository.find(query).await?;
        // ...
        Ok(Vec::new())
    }
}
```

Keep CPU-bound pure domain functions synchronous.

Avoid mixing sync blocking IO inside async request handlers. Use async IO adapters when running in async runtimes.



## Testing Strategy

Tests should be layered.

```txt
tests/
  integration_smoke.rs

src/
  domain/
    rules.rs         // unit tests here
  app/
    use_cases.rs     // app tests with fake ports
```

### Unit Test Pure Domain Logic

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_section_until_next_heading() {
        let content = "# Title\n\n## Summary\n\nHello\n\n## Details\n\nMore";

        let result = extract_section(content, "Summary");

        assert_eq!(result.as_deref(), Some("Hello"));
    }
}
```

### Test App Logic With Fakes

```rust
#[derive(Default)]
struct FakeDocumentRepository {
    documents: Vec<Document>,
}

impl DocumentRepository for FakeDocumentRepository {
    fn find(&self, _query: DocumentQuery) -> Result<Vec<Document>, AppError> {
        Ok(self.documents.clone())
    }

    fn get(&self, id: &str) -> Result<Option<Document>, AppError> {
        Ok(self
            .documents
            .iter()
            .find(|document| document.id.as_str() == id)
            .cloned())
    }
}

#[test]
fn summarizes_documents() {
    let repository = FakeDocumentRepository {
        documents: vec![Document {
            id: DocumentId::new("doc-1").unwrap(),
            title: "doc-1".to_string(),
            body: "## Summary\n\nHello".to_string(),
        }],
    };

    let app = App::new(repository);

    let result = app
        .summarize_documents(DocumentQuery::default())
        .unwrap();

    assert_eq!(result[0].summary.as_deref(), Some("Hello"));
}
```

### Temporary Directories Without Extra Crates

```rust
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn create_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let path = std::env::temp_dir().join(format!("{prefix}-{unique}"));
    fs::create_dir(&path).unwrap();
    path
}
```

### Prefer Deterministic Tests

Bad:

```rust
#[test]
fn creates_timestamp() {
    let id = create_id_from_current_time();
    assert!(id.starts_with("2026"));
}
```

Good:

```rust
struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_700_000_000)
    }
}
```
