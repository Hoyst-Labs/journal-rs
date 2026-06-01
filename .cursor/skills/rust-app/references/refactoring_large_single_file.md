## Refactoring Large Single-File Apps

When refactoring a large Rust file, follow this order:

1. **Identify responsibilities.**
   - Input parsing
   - Configuration
   - Domain parsing/rules
   - Application orchestration
   - Filesystem/database/network IO
   - Formatting/output
   - Error handling
   - Tests

2. **Move pure functions first.**
   - Put pure business rules in `domain`.
   - Add or preserve unit tests.

3. **Create request/response structs.**
   - Replace loose argument lists with named structs.

4. **Move side effects behind ports/adapters.**
   - Filesystem access becomes a repository or gateway.
   - Time access becomes a clock.
   - External calls become clients/adapters.

5. **Create an application service.**
   - App service coordinates the use case.
   - App service depends on ports or injected concrete adapters.

6. **Move formatting to output/presentation.**
   - App returns structured values.
   - Output modules render text/JSON/etc.

7. **Shrink the entry point.**
   - Entry point wires dependencies.
   - Entry point calls the app.
   - Entry point handles platform-specific output and exit behavior.

8. **Clean up tests.**
   - Domain tests test pure logic.
   - App tests use fakes.
   - Adapter tests may touch real filesystem/database when appropriate.



## Example Refactor Target

From single-file mixed responsibilities:

```rust
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let directory = discover_directory();
    let files = read_files(directory);
    let filtered = filter_files(files, args);
    println!("{}", format_files(filtered));
}
```

To layered structure:

```rust
// main.rs
fn main() -> Result<(), AppError> {
    let request = EntryRequest::from_env()?;
    let config = AppConfig::from_env()?;

    let app = App::new(FsDocumentRepository::new(config.data_dir));

    let response = app.list_documents(request.into_app_request())?;

    println!("{}", render_document_list(&response));

    Ok(())
}
```

```rust
// app/use_cases.rs
impl<R> App<R>
where
    R: DocumentRepository,
{
    pub fn list_documents(
        &self,
        request: ListDocumentsRequest,
    ) -> Result<ListDocumentsResponse, AppError> {
        let documents = self.repository.find(request.query)?;

        Ok(ListDocumentsResponse { documents })
    }
}
```

```rust
// output/text.rs
pub fn render_document_list(response: &ListDocumentsResponse) -> String {
    response
        .documents
        .iter()
        .map(|document| document.title.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}
```
