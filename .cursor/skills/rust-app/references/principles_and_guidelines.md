## SOLID Applied to Rust

### Single Responsibility

Each module/type should have one main reason to change.

```rust
// Good
DocumentRepository     // persistence
DocumentParser         // parsing
DocumentFormatter      // presentation
DocumentService        // use-case orchestration
```

Avoid:

```rust
DocumentManager // vague type that reads, parses, filters, formats, logs, and prints
```

### Open/Closed

Add new behavior through new adapters or strategy implementations instead of editing core logic.

```rust
pub trait Exporter {
    fn export(&self, report: &Report) -> Result<Vec<u8>, AppError>;
}

pub struct JsonExporter;
pub struct MarkdownExporter;
pub struct CsvExporter;
```

### Liskov Substitution

Any implementation of a port should behave according to the same contract.

```rust
pub trait UserRepository {
    /// Returns Ok(None) when the user does not exist.
    /// Returns Err only when the lookup fails.
    fn get_by_id(&self, id: &UserId) -> Result<Option<User>, AppError>;
}
```

Do not make one implementation return an error for “not found” while another returns `Ok(None)`.

### Interface Segregation

Prefer small focused traits.

```rust
pub trait UserReader {
    fn get_user(&self, id: &UserId) -> Result<Option<User>, AppError>;
}

pub trait UserWriter {
    fn save_user(&self, user: &User) -> Result<(), AppError>;
}
```

Avoid one huge trait:

```rust
pub trait Database {
    fn get_user(...);
    fn save_user(...);
    fn delete_user(...);
    fn list_orders(...);
    fn send_email(...);
}
```

### Dependency Inversion

Application logic depends on abstractions or injected dependencies, not concrete infrastructure.

```rust
pub struct RegisterUser<R, N> {
    users: R,
    notifier: N,
}

impl<R, N> RegisterUser<R, N>
where
    R: UserWriter,
    N: Notifier,
{
    pub fn execute(&self, request: RegisterUserRequest) -> Result<RegisterUserResponse, AppError> {
        // ...
        Ok(RegisterUserResponse { user_id })
    }
}
```



## DRY Without Over-Abstracting

DRY means “do not duplicate knowledge,” not “never duplicate syntax.”

Acceptable duplication:

```rust
#[test]
fn active_user_can_login() {
    // explicit setup is fine when it improves test clarity
}

#[test]
fn suspended_user_cannot_login() {
    // similar setup is fine
}
```

Bad abstraction:

```rust
fn magic_user_test(status: &str, expected: bool, mode: i32, flags: Vec<&str>) {
    // hides intent
}
```

Good abstraction:

```rust
fn active_user() -> User {
    User {
        status: AccountStatus::Active,
        ..User::default()
    }
}
```

Create helpers when they clarify intent, not merely to reduce line count.


## Ownership and Borrowing Guidelines

Prefer borrowed input when a function does not need ownership.

```rust
pub fn normalize_title(title: &str) -> String {
    title.trim().to_lowercase()
}
```

Take ownership when storing the value.

```rust
pub struct Document {
    title: String,
}

impl Document {
    pub fn new(title: String) -> Self {
        Self { title }
    }
}
```

Use `impl Into<String>` for ergonomic constructors.

```rust
impl Document {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
        }
    }
}
```

Return references when exposing internal data without transfer.

```rust
impl Document {
    pub fn title(&self) -> &str {
        &self.title
    }
}
```

Avoid cloning to silence borrow checker errors. Clone intentionally.



## Collections

Use the collection that matches the behavior.

```rust
Vec<T>          // ordered list
HashMap<K, V>   // fast key lookup, unordered
BTreeMap<K, V>  // sorted keys, deterministic output
HashSet<T>      // uniqueness
BTreeSet<T>     // sorted uniqueness
VecDeque<T>     // queue behavior
```

For deterministic CLI/API/test output, prefer `BTreeMap`/`BTreeSet` when sorted keys matter.



## Iterators

Prefer iterator chains when they remain readable.

```rust
let names = users
    .iter()
    .filter(|user| user.status == AccountStatus::Active)
    .map(|user| user.name.clone())
    .collect::<Vec<_>>();
```

Prefer loops when error handling, branching, or mutation becomes clearer.

```rust
let mut documents = Vec::new();

for path in paths {
    if !path.is_file() {
        continue;
    }

    let document = load_document(path)?;
    documents.push(document);
}
```

Do not force everything into iterator chains.



## Module Visibility

Default to private. Expose only what other modules need.

```rust
pub struct App {
    repository: Repository,
}

impl App {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }
}

fn internal_helper() {
    // private by default
}
```

Use `pub(crate)` for internal cross-module APIs.

```rust
pub(crate) fn normalize_key(value: &str) -> String {
    value.trim().to_lowercase()
}
```

Avoid making fields public unless the type is a simple DTO.

```rust
pub struct User {
    id: UserId,
    email: EmailAddress,
}

impl User {
    pub fn id(&self) -> &UserId {
        &self.id
    }
}
```



## Constants

Use constants for real constants, not configuration.

```rust
pub const DEFAULT_PAGE_SIZE: usize = 50;
pub const MAX_PAGE_SIZE: usize = 500;
```

Use config for environment-specific values.

```rust
pub struct AppConfig {
    pub page_size: usize,
    pub database_url: String,
}
```

Large text blocks belong in a dedicated module or asset.

```rust
pub const HELP_TEXT: &str = include_str!("../assets/help.txt");
```



## Validation

Validate at boundaries and create safe domain types.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn parse(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();

        if !value.contains('@') {
            return Err(DomainError::InvalidEmail);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

Then use the safe type internally:

```rust
pub struct User {
    pub id: UserId,
    pub email: EmailAddress,
}
```

Avoid validating the same primitive string everywhere.



## Builder Pattern

Use builders when constructors become unclear.

```rust
#[derive(Debug, Clone)]
pub struct ReportRequest {
    pub start_date: String,
    pub end_date: String,
    pub include_archived: bool,
    pub limit: usize,
}

#[derive(Default)]
pub struct ReportRequestBuilder {
    start_date: Option<String>,
    end_date: Option<String>,
    include_archived: bool,
    limit: Option<usize>,
}

impl ReportRequestBuilder {
    pub fn start_date(mut self, value: impl Into<String>) -> Self {
        self.start_date = Some(value.into());
        self
    }

    pub fn end_date(mut self, value: impl Into<String>) -> Self {
        self.end_date = Some(value.into());
        self
    }

    pub fn include_archived(mut self, value: bool) -> Self {
        self.include_archived = value;
        self
    }

    pub fn limit(mut self, value: usize) -> Self {
        self.limit = Some(value);
        self
    }

    pub fn build(self) -> Result<ReportRequest, AppError> {
        Ok(ReportRequest {
            start_date: self
                .start_date
                .ok_or_else(|| AppError::InvalidConfig("start_date is required".to_string()))?,
            end_date: self
                .end_date
                .ok_or_else(|| AppError::InvalidConfig("end_date is required".to_string()))?,
            include_archived: self.include_archived,
            limit: self.limit.unwrap_or(100),
        })
    }
}
```



## Service Pattern

Use services for reusable application operations.

```rust
pub struct DocumentService<R> {
    repository: R,
}

impl<R> DocumentService<R>
where
    R: DocumentRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn find_summaries(
        &self,
        query: DocumentQuery,
    ) -> Result<Vec<DocumentSummary>, AppError> {
        let documents = self.repository.find(query)?;

        Ok(documents
            .into_iter()
            .map(DocumentSummary::from)
            .collect())
    }
}
```

Do not create services that only wrap one function with no added meaning.



## Repository Pattern

Repositories should deal in domain types, not raw database rows or filesystem details.

```rust
pub trait UserRepository {
    fn save(&self, user: &User) -> Result<(), AppError>;
    fn get_by_id(&self, id: &UserId) -> Result<Option<User>, AppError>;
}
```

Adapter converts infrastructure data:

```rust
struct UserRow {
    id: String,
    email: String,
    status: String,
}

impl TryFrom<UserRow> for User {
    type Error = AppError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        Ok(User {
            id: UserId::parse(row.id)?,
            email: EmailAddress::parse(row.email)?,
            status: row.status.parse()?,
        })
    }
}
```



## DTO Mapping

Keep external DTOs separate from domain models when boundaries matter.

```rust
#[derive(Debug, serde::Deserialize)]
pub struct CreateUserDto {
    pub email: String,
    pub name: String,
}

impl TryFrom<CreateUserDto> for CreateUserRequest {
    type Error = AppError;

    fn try_from(value: CreateUserDto) -> Result<Self, Self::Error> {
        Ok(Self {
            email: EmailAddress::parse(value.email)?,
            name: value.name,
        })
    }
}
```

This prevents external API shape from polluting domain logic.



## Serialization

Use serde at boundaries.

```rust
#[derive(Debug, serde::Serialize)]
pub struct UserResponseDto {
    pub id: String,
    pub email: String,
}

impl From<User> for UserResponseDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id().as_str().to_string(),
            email: user.email().as_str().to_string(),
        }
    }
}
```

Avoid deriving `Serialize`/`Deserialize` on every domain type by default. It is fine for simple apps, but for serious boundaries, use DTOs.


## Feature Flags

Use Cargo features to keep optional integrations separate.

```toml
[features]
default = []
fs = []
json = ["serde", "serde_json"]

[dependencies]
serde = { version = "1", features = ["derive"], optional = true }
serde_json = { version = "1", optional = true }
```

Code:

```rust
#[cfg(feature = "json")]
pub mod json_output;
```



## Cargo Workspace Pattern

For larger systems, split crates by responsibility.

```txt
workspace/
  Cargo.toml
  crates/
    app-domain/
    app-core/
    app-adapters/
    app-cli/
    app-web/
```

Workspace `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
  "crates/app-domain",
  "crates/app-core",
  "crates/app-adapters",
  "crates/app-cli",
  "crates/app-web",
]
```

Use this when the app is large enough that crate-level boundaries are valuable. Do not start with many crates for a tiny app.



## Recommended Crates

Use only what the project needs.

Common app-quality crates:

```toml
[dependencies]
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

For async apps:

```toml
[dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
async-trait = "0.1"
```

For CLI-specific apps, use a separate CLI skill. Do not include CLI-specific requirements here.

For web-specific apps, use a separate web/API skill.



## Documentation

Use doc comments for public types and non-obvious behavior.

```rust
/// Coordinates document-related use cases.
///
/// This type does not know where documents are stored or how results are rendered.
pub struct App<R> {
    repository: R,
}
```

Document trait contracts carefully:

```rust
pub trait DocumentRepository {
    /// Returns matching documents.
    ///
    /// An empty result is not an error.
    /// Infrastructure failures should be returned as `Err`.
    fn find(&self, query: DocumentQuery) -> Result<Vec<Document>, AppError>;
}
```



## Comments

Use comments to explain why, not what.

Good:

```rust
// Keep deterministic ordering so output is stable in tests and snapshots.
let mut grouped = BTreeMap::new();
```

Bad:

```rust
// Create a new BTreeMap.
let mut grouped = BTreeMap::new();
```



## Performance Guidelines

Prefer clarity first, then optimize measured bottlenecks.

General rules:

- Avoid unnecessary clones.
- Prefer streaming/iterators for large data.
- Avoid loading huge files into memory unless the use case requires it.
- Use `BufReader` for large file reads.
- Use `BTreeMap` only when sorted keys are needed; otherwise `HashMap` is usually faster.
- Do not add `Arc`, `Mutex`, channels, async, or threads unless there is a clear need.

Buffered file example:

```rust
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

pub fn read_lines(path: &Path) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    reader.lines().collect()
}
```



## Concurrency

Prefer simple ownership and immutable data.

Thread example:

```rust
use std::thread;

pub fn process_items(items: Vec<String>) -> Vec<String> {
    let handles = items
        .into_iter()
        .map(|item| {
            thread::spawn(move || {
                item.to_uppercase()
            })
        })
        .collect::<Vec<_>>();

    handles
        .into_iter()
        .map(|handle| handle.join().expect("worker thread panicked"))
        .collect()
}
```

Async task example:

```rust
pub async fn process_all<R>(
    repository: &R,
    ids: Vec<String>,
) -> Result<Vec<Document>, AppError>
where
    R: DocumentRepository + Sync,
{
    let mut documents = Vec::new();

    for id in ids {
        if let Some(document) = repository.get(&id).await? {
            documents.push(document);
        }
    }

    Ok(documents)
}
```

Avoid concurrency unless the workload benefits from it.



## State Management

Prefer explicit state structs.

```rust
#[derive(Debug, Default)]
pub struct ImportState {
    pub processed: usize,
    pub skipped: usize,
    pub failed: usize,
}

impl ImportState {
    pub fn record_processed(&mut self) {
        self.processed += 1;
    }

    pub fn record_skipped(&mut self) {
        self.skipped += 1;
    }

    pub fn record_failed(&mut self) {
        self.failed += 1;
    }
}
```

Avoid global mutable state.



## File and Path Handling

Use `Path` and `PathBuf`, not raw strings.

```rust
use std::path::{Path, PathBuf};

pub fn resolve_data_file(root: &Path, file_name: &str) -> PathBuf {
    root.join(file_name)
}
```

Validate file names when accepting user-controlled paths.

```rust
pub fn safe_child_path(root: &Path, file_name: &str) -> Result<PathBuf, AppError> {
    if file_name.contains('/') || file_name.contains('\\') || file_name == ".." {
        return Err(AppError::InvalidConfig("invalid file name".to_string()));
    }

    Ok(root.join(file_name))
}
```



## Parsing

Parsing should return structured values.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatePrefix {
    value: String,
}

impl DatePrefix {
    pub fn parse(file_name: &str) -> Option<Self> {
        let prefix = file_name.get(0..10)?;
        let bytes = prefix.as_bytes();

        if bytes.len() != 10 {
            return None;
        }

        if bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }

        let valid = bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit());

        valid.then(|| Self {
            value: prefix.to_string(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}
```

Avoid mixing parsing with IO or printing.



## Sorting and Grouping

Make sorting/grouping explicit and testable.

```rust
use std::collections::BTreeMap;

pub fn group_by_date(files: Vec<DocumentFile>) -> BTreeMap<String, Vec<DocumentFile>> {
    let mut grouped = BTreeMap::new();

    for file in files {
        grouped
            .entry(file.date.clone())
            .or_insert_with(Vec::new)
            .push(file);
    }

    grouped
}
```

Sort near the behavior that requires sorted output:

```rust
pub fn newest_first(mut files: Vec<DocumentFile>) -> Vec<DocumentFile> {
    files.sort_by(|left, right| right.timestamp.cmp(&left.timestamp));
    files
}
```




## Naming Guidelines

Prefer names that say what the type does.

Good:

```rust
DocumentRepository
DocumentSummary
ListDocumentsRequest
ListDocumentsResponse
MarkdownSectionParser
FsDocumentRepository
SystemClock
```

Avoid vague names:

```rust
Manager
Processor
Handler
Helper
Utils
Common
Stuff
Engine
```

Use `Handler` only at interface boundaries where it truly handles an external event or request.

Use `Service` when it coordinates meaningful application behavior.

Use `Repository` for persistence-like access to domain entities.

Use `Client` for external APIs.

Use `Gateway` for external system boundaries when repository/client is not precise enough.



## What Not To Do

Do not create this kind of structure:

```txt
src/
  main.rs       // 900 lines
  utils.rs      // random unrelated helpers
  helpers.rs    // more random unrelated helpers
  common.rs     // unclear shared dumping ground
```

Do not do this:

```rust
pub fn run(args: Vec<String>) {
    // app logic tied directly to CLI args
}
```

Prefer this:

```rust
pub fn run(request: RunRequest) -> Result<RunResponse, AppError> {
    // app logic independent of caller
}
```

Do not do this:

```rust
pub fn list_files() -> String {
    // reads files and returns formatted output
}
```

Prefer this:

```rust
pub fn list_documents(request: ListDocumentsRequest) -> Result<ListDocumentsResponse, AppError>;
pub fn render_document_list(response: &ListDocumentsResponse) -> String;
```
