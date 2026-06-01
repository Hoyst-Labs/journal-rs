
## Minimal Template

Use this as a starting point for small Rust apps.

```rust
// src/lib.rs
pub mod adapters;
pub mod app;
pub mod domain;
pub mod errors;
pub mod output;
pub mod ports;

pub use app::App;
pub use errors::AppError;
```

```rust
// src/app.rs
use crate::{errors::AppError, ports::Repository};

pub struct App<R> {
    repository: R,
}

impl<R> App<R>
where
    R: Repository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn execute(&self, request: AppRequest) -> Result<AppResponse, AppError> {
        let items = self.repository.list()?;

        Ok(AppResponse { items })
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppRequest;

#[derive(Debug, Clone)]
pub struct AppResponse {
    pub items: Vec<String>,
}
```

```rust
// src/ports.rs
use crate::errors::AppError;

pub trait Repository {
    fn list(&self) -> Result<Vec<String>, AppError>;
}
```

```rust
// src/adapters.rs
use crate::{errors::AppError, ports::Repository};

pub struct InMemoryRepository {
    items: Vec<String>,
}

impl InMemoryRepository {
    pub fn new(items: Vec<String>) -> Self {
        Self { items }
    }
}

impl Repository for InMemoryRepository {
    fn list(&self) -> Result<Vec<String>, AppError> {
        Ok(self.items.clone())
    }
}
```

```rust
// src/output.rs
use crate::app::AppResponse;

pub fn render_text(response: &AppResponse) -> String {
    response.items.join("\n")
}
```

```rust
// src/errors.rs
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Message(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for AppError {}
```

```rust
// src/main.rs
use my_app::{adapters::InMemoryRepository, output::render_text, App};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new(InMemoryRepository::new(vec![
        "first".to_string(),
        "second".to_string(),
    ]));

    let response = app.execute(Default::default())?;

    println!("{}", render_text(&response));

    Ok(())
}
```

---
