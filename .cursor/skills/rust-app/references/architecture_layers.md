## Recommended Project Shape

A solid general-purpose Rust app commonly starts with this shape:

```txt
my-app/
  Cargo.toml
  src/
    lib.rs
    main.rs

    app/
      mod.rs
      use_cases.rs
      services.rs

    domain/
      mod.rs
      models.rs
      rules.rs

    ports/
      mod.rs
      repositories.rs
      clock.rs

    adapters/
      mod.rs
      fs_repository.rs
      system_clock.rs

    config/
      mod.rs

    errors.rs
    output/
      mod.rs
      text.rs
      json.rs
```

For smaller apps, collapse modules carefully:

```txt
src/
  lib.rs
  main.rs
  app.rs
  domain.rs
  ports.rs
  adapters.rs
  errors.rs
```

For larger apps, split by feature:

```txt
src/
  lib.rs
  features/
    journal/
      mod.rs
      app.rs
      domain.rs
      ports.rs
      adapters.rs
      output.rs
    users/
      mod.rs
      app.rs
      domain.rs
      ports.rs
      adapters.rs
```

Prefer feature-first organization when the app has several independent business areas.



## `main.rs` Should Stay Thin

`main.rs` is an adapter. It should not contain business rules.

Good shape:

```rust
use my_app::{App, AppConfig, FsDocumentRepository, SystemClock};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env()?;

    let repository = FsDocumentRepository::new(config.data_dir.clone());
    let clock = SystemClock;

    let app = App::new(repository, clock);

    let result = app.run_default()?;
    println!("{}", result.to_text());

    Ok(())
}
```

Better for production apps with richer errors:

```rust
use my_app::{
    adapters::{FsDocumentRepository, SystemClock},
    app::App,
    config::AppConfig,
    errors::AppError,
};

fn main() -> Result<(), AppError> {
    let config = AppConfig::from_env()?;

    let app = App::new(
        FsDocumentRepository::new(config.data_dir),
        SystemClock,
    );

    let result = app.run_default()?;
    println!("{}", result.render_text());

    Ok(())
}
```

Bad shape:

```rust
fn main() {
    // Bad:
    // - parses args
    // - discovers files
    // - reads files
    // - filters domain data
    // - formats output
    // - handles errors
    // - contains tests for private behavior
}
```



## `lib.rs` Should Expose the App Surface

Use `lib.rs` to expose the public application surface. Keep it intentional.

```rust
pub mod adapters;
pub mod app;
pub mod config;
pub mod domain;
pub mod errors;
pub mod output;
pub mod ports;

pub use app::App;
pub use config::AppConfig;
pub use errors::AppError;
```

Avoid dumping everything into the public API:

```rust
// Avoid:
pub mod everything;
pub use everything::*;
```



## Domain Layer

The domain layer contains business concepts and rules. It should be as pure as possible.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub id: DocumentId,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentId(String);

impl DocumentId {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();

        if value.trim().is_empty() {
            return Err(DomainError::EmptyDocumentId);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    EmptyDocumentId,
}
```

Domain rules should be independent of storage and presentation:

```rust
pub fn extract_section(markdown: &str, heading: &str) -> Option<String> {
    let normalized = markdown.replace("\r\n", "\n");
    let lines: Vec<&str> = normalized.lines().collect();

    let target = format!("## {heading}");
    let start = lines.iter().position(|line| line.trim() == target)?;

    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, line)| line.starts_with("## ").then_some(index))
        .unwrap_or(lines.len());

    Some(lines[start + 1..end].join("\n").trim().to_string())
}
```

Bad domain code:

```rust
pub fn extract_summary_from_file(path: &Path) -> String {
    // Bad:
    // - reads filesystem
    // - swallows errors
    // - hardcodes presentation fallback
    // - mixes IO with parsing
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| extract_section(&content, "Summary"))
        .unwrap_or_else(|| "[missing]".to_string())
}
```


## Application Layer

The application layer coordinates use cases. It can call ports, domain functions, and services.

```rust
use crate::{
    domain::{extract_section, Document},
    errors::AppError,
    ports::DocumentRepository,
};

pub struct App<R> {
    documents: R,
}

impl<R> App<R>
where
    R: DocumentRepository,
{
    pub fn new(documents: R) -> Self {
        Self { documents }
    }

    pub fn summarize_documents(&self, query: DocumentQuery) -> Result<Vec<DocumentSummary>, AppError> {
        let documents = self.documents.find(query)?;

        let summaries = documents
            .into_iter()
            .map(|document| {
                let summary = extract_section(&document.body, "Summary");

                DocumentSummary {
                    id: document.id,
                    title: document.title,
                    summary,
                }
            })
            .collect();

        Ok(summaries)
    }
}

#[derive(Debug, Clone, Default)]
pub struct DocumentQuery {
    pub text: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSummary {
    pub id: crate::domain::DocumentId,
    pub title: String,
    pub summary: Option<String>,
}
```

Keep orchestration here. Keep low-level details elsewhere.



## Ports

Ports are traits that describe what the application needs from the outside world.

```rust
use crate::{
    app::DocumentQuery,
    domain::Document,
    errors::AppError,
};

pub trait DocumentRepository {
    fn find(&self, query: DocumentQuery) -> Result<Vec<Document>, AppError>;
    fn get(&self, id: &str) -> Result<Option<Document>, AppError>;
}
```

A clock port:

```rust
use std::time::SystemTime;

pub trait Clock {
    fn now(&self) -> SystemTime;
}
```

An ID generator port:

```rust
pub trait IdGenerator {
    fn next_id(&self) -> String;
}
```

A notification port:

```rust
use crate::errors::AppError;

pub trait Notifier {
    fn notify(&self, message: NotificationMessage) -> Result<(), AppError>;
}

#[derive(Debug, Clone)]
pub struct NotificationMessage {
    pub subject: String,
    pub body: String,
}
```

Use traits when the dependency is external, volatile, expensive, stateful, or should be mocked in tests.

Do not create traits for every tiny helper function. Rust does not need Java-style interface explosion.



## Adapters

Adapters implement ports using concrete technologies.

Filesystem adapter:

```rust
use std::{fs, path::PathBuf};

use crate::{
    app::DocumentQuery,
    domain::{Document, DocumentId},
    errors::AppError,
    ports::DocumentRepository,
};

pub struct FsDocumentRepository {
    root: PathBuf,
}

impl FsDocumentRepository {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl DocumentRepository for FsDocumentRepository {
    fn find(&self, query: DocumentQuery) -> Result<Vec<Document>, AppError> {
        let mut documents = Vec::new();

        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };

            if !file_name.ends_with(".md") {
                continue;
            }

            if let Some(text) = &query.text {
                if !file_name.contains(text) {
                    continue;
                }
            }

            let body = fs::read_to_string(&path)?;
            let id = DocumentId::new(file_name.to_string())?;

            documents.push(Document {
                id,
                title: file_name.to_string(),
                body,
            });
        }

        if let Some(limit) = query.limit {
            documents.truncate(limit);
        }

        Ok(documents)
    }

    fn get(&self, id: &str) -> Result<Option<Document>, AppError> {
        let path = self.root.join(id);

        if !path.exists() {
            return Ok(None);
        }

        let body = fs::read_to_string(&path)?;
        let id = DocumentId::new(id.to_string())?;

        Ok(Some(Document {
            title: id.as_str().to_string(),
            id,
            body,
        }))
    }
}
```

System clock adapter:

```rust
use std::time::SystemTime;

use crate::ports::Clock;

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}
```

Adapters may use real systems. Domain and application logic should not.


## Dependency Injection Patterns

### Generic Dependencies

Best when dependencies are known at compile time and the app does not need dynamic plugin behavior.

```rust
pub struct App<R, C> {
    repository: R,
    clock: C,
}

impl<R, C> App<R, C>
where
    R: DocumentRepository,
    C: Clock,
{
    pub fn new(repository: R, clock: C) -> Self {
        Self { repository, clock }
    }
}
```

### Trait Objects

Useful when dependencies need to be chosen dynamically at runtime.

```rust
pub struct App {
    repository: Box<dyn DocumentRepository>,
    clock: Box<dyn Clock>,
}

impl App {
    pub fn new(
        repository: Box<dyn DocumentRepository>,
        clock: Box<dyn Clock>,
    ) -> Self {
        Self { repository, clock }
    }
}
```

### Shared Dependencies

Use `Arc` when dependencies are shared across tasks, threads, handlers, or services.

```rust
use std::sync::Arc;

pub struct AppState<R> {
    pub repository: Arc<R>,
}

impl<R> Clone for AppState<R> {
    fn clone(&self) -> Self {
        Self {
            repository: Arc::clone(&self.repository),
        }
    }
}
```

Do not reach for `Arc<Mutex<T>>` by default. Prefer immutable dependencies, message passing, database transactions, or interior mutability only when necessary.
