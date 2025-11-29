# Error Handling Patterns with error-rail

This guide provides practical patterns for using error-rail in different scenarios. Each pattern includes complete, working examples that you can adapt to your needs.

> **Note**: All patterns use `ErrorPipeline` for proper error composition. The examples in this documentation are also available as runnable files in the `examples/` directory.

## Table of Contents

- [Pattern 1: Service Layer Error Handling](#pattern-1-service-layer-error-handling)
- [Pattern 2: HTTP API Error Responses](#pattern-2-http-api-error-responses)
- [Pattern 3: CLI Applications](#pattern-3-cli-applications)
- [Pattern 4: Library Development](#pattern-4-library-development)
- [Anti-patterns](#anti-patterns)

---

## Pattern 1: Service Layer Error Handling

### Use Case - Service Layer Error Handling

Converting domain-specific errors into service-layer errors with contextual information about the operation being performed.

### Example - Service Layer Error Handling

```rust
use error_rail::{context, ComposableError, ErrorPipeline};

// Domain layer error
#[derive(Debug)]
enum DbError {
    ConnectionFailed,
    QueryFailed(String),
    NotFound,
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DbError::ConnectionFailed => write!(f, "database connection failed"),
            DbError::QueryFailed(q) => write!(f, "query failed: {}", q),
            DbError::NotFound => write!(f, "record not found"),
        }
    }
}

impl std::error::Error for DbError {}

// Service layer functions
fn fetch_user_from_db(_user_id: u64) -> Result<String, DbError> {
    // Simulate database operation
    Err(DbError::NotFound)
}

fn process_user_request(user_id: u64) -> Result<String, Box<ComposableError<DbError>>> {
    let user_id_str = user_id.to_string();
    ErrorPipeline::new(fetch_user_from_db(user_id))
        .with_context(context!("processing user request for user_id: {}", user_id_str))
        .with_context(context!("fetching user profile for user_id: {}", user_id))
        .with_context("formatting profile data")
        .map(|data| format!("Profile: {}", data))
        .finish_boxed()
}

// Usage
fn main() {
    match process_user_request(42) {
        Ok(profile) => println!("{}", profile),
        Err(e) => {
            eprintln!("Error: {}", e.error_chain());
            // Output: processing user request -> fetching user profile for user_id: 42 -> record not found
        }
    }
}
```

### Key Takeaways - Service Layer Error Handling

- Use `ErrorPipeline` for proper error composition with `finish_boxed()`
- Add multiple contexts in a single pipeline chain
- Use `context!()` macro for dynamic, lazy-evaluated context
- Keep context messages focused on **what operation** was being performed, not **how** it failed

---

## Pattern 2: HTTP API Error Responses

### Use Case - HTTP API Error Responses

Converting internal errors to structured HTTP responses with appropriate status codes.

### Example - HTTP API Error Responses

```rust
use error_rail::{context, ComposableError, ErrorPipeline};

#[derive(Debug)]
enum ApiError {
    NotFound,
    BadRequest(String),
    Unauthorized,
    Internal(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ApiError::NotFound => write!(f, "resource not found"),
            ApiError::Unauthorized => write!(f, "unauthorized"),
            ApiError::BadRequest(msg) => write!(f, "bad request: {}", msg),
            ApiError::Internal(msg) => write!(f, "internal error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

// Map error to HTTP status code
fn error_to_status_code(err: &ComposableError<ApiError>) -> u16 {
    match err.core_error() {
        ApiError::NotFound => 404,
        ApiError::Unauthorized => 401,
        ApiError::BadRequest(_) => 400,
        ApiError::Internal(_) => 500,
    }
}

// API handler
fn get_resource(resource_id: &str) -> Result<String, Box<ComposableError<ApiError>>> {
    if resource_id.is_empty() {
        return ErrorPipeline::new(Err(ApiError::BadRequest(
            "resource_id cannot be empty".into(),
        )))
        .with_context("validating resource_id")
        .finish_boxed();
    }
    
    // Simulate resource fetch
    ErrorPipeline::new(Err(ApiError::NotFound))
        .with_context(context!("fetching resource: {}", resource_id))
        .finish_boxed()
}

// Convert to HTTP response
fn handle_request(resource_id: &str) -> (u16, String) {
    match get_resource(resource_id) {
        Ok(data) => (200, data),
        Err(e) => {
            let status = error_to_status_code(&*e);
            let body = if status >= 500 {
                // Include full error chain for 5xx errors (for debugging)
                e.error_chain()
            } else {
                // Only show core error for 4xx errors (security)
                e.core_error().to_string()
            };
            (status, body)
        }
    }
}

fn main() {
    let (status, body) = handle_request("");
    println!("Status: {}, Body: {}", status, body);
    // Output: Status: 400, Body: bad request: resource_id cannot be empty
}
```

### Key Takeaways - HTTP API Error Responses

- Use error codes to map internal errors to HTTP status codes
- Be careful about exposing error chains in production (potential security leak)
- Different error visibility for 4xx vs 5xx errors

---

## Pattern 3: CLI Applications

### Use Case - CLI Applications

Providing helpful, user-friendly error messages in command-line tools.

### Example - CLI Applications

```rust
use error_rail::{context, ComposableError, ErrorPipeline};

/// Custom error type for parsing operations that implements std::error::Error
#[derive(Debug)]
struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseError {}

fn read_config_file(path: &str) -> Result<String, Box<ComposableError<std::io::Error>>> {
    ErrorPipeline::new(std::fs::read_to_string(path))
        .with_context(context!("reading configuration from '{}'", path))
        .finish_boxed()
}

fn parse_config(content: &str) -> Result<(), Box<ComposableError<ParseError>>> {
    if content.is_empty() {
        return ErrorPipeline::new(Err(ParseError("configuration is empty".to_string())))
            .with_context("parsing configuration")
            .finish_boxed();
    }
    Ok(())
}

fn load_and_parse_config(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = read_config_file(path).map_err(|e| e as Box<dyn std::error::Error>)?;
    parse_config(&content)?;
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config_path = args.get(1).map(|s| s.as_str()).unwrap_or("config.toml");
    
    match load_and_parse_config(config_path) {
        Ok(_) => println!("✓ Configuration loaded successfully"),
        Err(e) => {
            eprintln!("✗ Error: {}", e);
            
            // In debug mode, show full error chain
            if std::env::var("DEBUG").is_ok() {
                eprintln!("\nDebug trace:");
                eprintln!("{:#}", e);
            } else {
                eprintln!("\n💡 Hint: Run with DEBUG=1 for detailed error trace");
            }
            
            std::process::exit(1);
        }
    }
}
```

### Key Takeaways - CLI Applications

- Provide user-friendly error messages by default
- Offer verbose/debug mode for detailed error chains
- Use emojis and formatting to make errors more scannable
- Suggest next steps or hints when possible

---

## Pattern 4: Library Development

### Use Case - Library Development

Designing error types for a public library while using error-rail internally.

### Example - Library Development

```rust
use error_rail::{ComposableError, ErrorPipeline};

// Public error type (opaque to users)
#[derive(Debug, Clone)]
pub enum MyLibError {
    InvalidInput(String),
    ProcessingFailed,
}

impl std::fmt::Display for MyLibError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MyLibError::InvalidInput(msg) => write!(f, "invalid input: {}", msg),
            MyLibError::ProcessingFailed => write!(f, "processing failed"),
        }
    }
}

impl std::error::Error for MyLibError {}

// Internal error handling with error-rail
// Note: InternalResult type alias kept for documentation purposes
type InternalResult<T> = Result<T, Box<ComposableError<MyLibError>>>;

fn validate_input(input: &str) -> Result<(), MyLibError> {
    if input.is_empty() {
        return Err(MyLibError::InvalidInput("input cannot be empty".into()));
    }
    Ok(())
}

fn process_data(input: &str) -> Result<String, Box<ComposableError<MyLibError>>> {
    // Simulate processing
    if input.len() > 100 {
        return ErrorPipeline::new(Err(MyLibError::ProcessingFailed))
            .with_context("processing data - input too large")
            .finish_boxed();
    }
    
    ErrorPipeline::new(validate_input(input))
        .with_context("validating input")
        .with_context("preparing data for processing")
        .map(|_| format!("Processed: {}", input))
        .finish_boxed()
}

// Public API - hides error-rail implementation
pub fn process_user_data(input: &str) -> Result<String, MyLibError> {
    match process_data(input) {
        Ok(result) => Ok(result),
        Err(boxed_error) => Err(boxed_error.core_error().clone()),
    }
}

fn main() {
    match process_user_data("ab") {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Key Takeaways - Library Development

- Use `ErrorPipeline` internally for ergonomic error handling
- Convert to simple `Result<T, E>` at public API boundaries
- Log full error chains internally, but expose only core errors publicly
- Use `.core_error().clone()` to extract the underlying error from `ComposableError`

---

## Anti-patterns

### ❌ Anti-pattern 1: Excessive Context Nesting

**Problem**: Adding context at every single line creates deep, redundant error chains.

```rust
// DON'T DO THIS
fn bad_example() -> Result<String, Box<ComposableError<std::io::Error>>> {
    let file = std::fs::File::open("data.txt")
        .map_err(|e| ErrorPipeline::new(Err(e))
            .with_context("opening file")
            .finish_boxed())?;  // ❌
    
    let mut content = String::new();
    std::io::Read::read_to_string(&mut file, &mut content)
        .map_err(|e| ErrorPipeline::new(Err(e))
            .with_context("reading file")
            .finish_boxed())?;  // ❌
    
    let trimmed = content.trim();
    Ok(trimmed.to_string())
}
```

**Solution**: Add context only at meaningful boundaries in a single ErrorPipeline chain.

```rust
// DO THIS
fn good_example() -> Result<String, Box<ComposableError<std::io::Error>>> {
    let content = std::fs::read_to_string("data.txt")
        .map_err(|e| ErrorPipeline::new(Err(e))
            .with_context("loading data file")
            .finish_boxed())?;  // ✅ One clear context
    
    Ok(content.trim().to_string())
}
```

### ❌ Anti-pattern 2: Eager String Formatting

**Problem**: Using `format!()` directly defeats the lazy evaluation benefit.

```rust
// DON'T DO THIS
use error_rail::ErrorContext;
let expensive_data = vec![1, 2, 3, 4, 5];
ErrorPipeline::new(Err("some error"))
    .with_context(ErrorContext::new(format!("data: {:?}", expensive_data)))  // ❌ Always evaluated
    .finish_boxed()
```

**Solution**: Use `context!()` macro for lazy evaluation.

```rust
// DO THIS
let expensive_data = vec![1, 2, 3, 4, 5];
ErrorPipeline::new(Err("some error"))
    .with_context(context!("data: {:?}", expensive_data))  // ✅ Only evaluated on error
    .finish_boxed()
```

### ❌ Anti-pattern 3: Losing Error Type Information

**Problem**: Converting to `Box<dyn Error>` too early loses type information.

```rust
// DON'T DO THIS
fn bad() -> Result<(), Box<dyn std::error::Error>> {
    ErrorPipeline::new(std::fs::read_to_string("file.txt"))
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)  // ❌
        .with_context("reading file")
        .finish_boxed()
}
```

**Solution**: Keep concrete types as long as possible.

```rust
// DO THIS
fn good() -> Result<(), Box<ComposableError<std::io::Error>>> {
    ErrorPipeline::new(std::fs::read_to_string("file.txt"))
        .with_context("reading file")  // ✅ Preserves io::Error type
        .finish_boxed()
}
```

### ❌ Anti-pattern 4: Mixing Error Types Carelessly

**Problem**: Creating confusing error type hierarchies.

```rust
// DON'T DO THIS
fn confusing() -> Result<(), Box<dyn std::error::Error>> {
    let _ = std::fs::read_to_string("file.txt")?;  // io::Error
    let _ = "123".parse::<i32>()?;  // ParseIntError
    Err("custom error")?;  // &str
    Ok(())
}
```

**Solution**: Define clear error types for your domain.

```rust
// DO THIS
#[derive(Debug)]
enum AppError {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
    Custom(String),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(e: std::num::ParseIntError) -> Self {
        AppError::Parse(e)
    }
}

fn clear() -> Result<(), Box<ComposableError<AppError>>> {
    ErrorPipeline::new((|| {
        let _ = std::fs::read_to_string("file.txt")?;
        let _ = "123".parse::<i32>()?;
        Ok::<(), AppError>(())
    })())
    .with_context("processing files and data")
    .finish_boxed()
}
```

---

## Summary

- **Service Layer**: Use `ErrorPipeline` with multiple `.with_context()` calls in a single chain
- **HTTP APIs**: Map errors to status codes, careful with error exposure, use proper dereferencing
- **CLI Apps**: User-friendly by default, verbose mode for debugging, handle error type conversions properly
- **Libraries**: Use `ErrorPipeline` internally, convert at API boundaries with `.core_error().clone()`
- **Avoid**: Excessive nesting, eager formatting, early type erasure, mixing error types carelessly

For more information, see the [API documentation](https://docs.rs/error-rail) and [Quick Start Guide](QUICK_START.md).
