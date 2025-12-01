# Async Quick Start Guide

This guide covers async error handling with error-rail.

## Installation

```toml
[dependencies]
error-rail = { version = "0.8", features = ["async"] }
tokio = { version = "1", features = ["full"] }
```

## Basic Usage

### Import the Async Prelude

```rust
use error_rail::prelude_async::*;
```

This imports everything from the sync prelude plus async-specific items:

- `FutureResultExt` trait for `.ctx()` and `.with_ctx()` on futures
- `AsyncErrorPipeline` for chainable async error handling
- `rail_async!` and `ctx_async!` macros

### Attaching Context to Async Operations

```rust
use error_rail::prelude_async::*;

async fn fetch_user(id: u64) -> BoxedResult<User, DbError> {
    // .ctx() works on any Future<Output = Result<T, E>>
    database.get_user(id)
        .ctx("fetching user from database")
        .await
        .map_err(Box::new)
}
```

### Lazy Context Evaluation

Just like sync code, context is only evaluated when an error occurs:

```rust
async fn process_order(order_id: u64) -> BoxedResult<Order, OrderError> {
    // The context! is only called if the future returns Err
    load_order(order_id)
        .with_ctx(|| context!("processing order {}", order_id))
        .await
        .map_err(Box::new)
}
```

## AsyncErrorPipeline

For more complex scenarios, use `AsyncErrorPipeline`:

```rust
use error_rail::prelude_async::*;
use error_rail::context;

async fn create_user(req: CreateUserRequest) -> BoxedResult<User, ApiError> {
    AsyncErrorPipeline::new(database.insert_user(&req))
        .with_context("inserting user")
        .with_context(context!("email: {}", req.email))
        .finish_boxed()
        .await
}
```

Or use the `rail_async!` macro:

```rust
async fn create_user(req: CreateUserRequest) -> BoxedResult<User, ApiError> {
    rail_async!(database.insert_user(&req))
        .with_context("inserting user")
        .with_context(context!("email: {}", req.email))
        .finish_boxed()
        .await
}
```

## Macros

### `ctx_async!`

Shorthand for attaching context to a future:

```rust
// Static message
let user = ctx_async!(fetch_user(id), "fetching user").await?;

// With formatting (lazy)
let order = ctx_async!(fetch_order(id), "order {}", id).await?;
```

### `rail_async!`

Creates an AsyncErrorPipeline:

```rust
let result = rail_async!(some_async_operation())
    .with_context("operation context")
    .finish_boxed()
    .await;
```

## Chaining Multiple Contexts

Contexts can be chained naturally:

```rust
async fn complex_operation() -> BoxedResult<Data, Error> {
    fetch_data()
        .ctx("fetching data")
        .ctx("in complex operation")
        .ctx("at application layer")
        .await
        .map_err(Box::new)
}
```

The error chain will show:

```text
at application layer -> in complex operation -> fetching data -> [original error]
```

## Cancel Safety

`ContextFuture` is cancel-safe:

- If the inner future is cancel-safe, `ContextFuture` is also cancel-safe
- The context closure is only called when the future completes with an error
- Dropping the future mid-execution won't evaluate the context

## Best Practices

1. **Use lazy context for expensive formatting**

   ```rust
   // Good: context! only called on error
   .with_ctx(|| context!("data: {:?}", large_data))
   
   // Avoid: context! always called
   .ctx(context!("data: {:?}", large_data))
   ```

2. **Prefer `finish_boxed()` for public APIs**

   ```rust
   // Recommended: 8-byte stack footprint
   pub async fn api_fn() -> BoxedResult<T, E> { ... }
   ```

3. **Chain contexts from specific to general**

   ```rust
   fetch_user(id)
       .ctx("from database")           // Most specific
       .ctx("in user service")         // More general
       .ctx("handling API request")    // Most general
   ```

## Complete Example

```rust
use error_rail::prelude_async::*;

#[derive(Debug)]
struct User { id: u64, name: String }

#[derive(Debug)]
struct DbError(String);

async fn get_user_by_id(id: u64) -> Result<User, DbError> {
    // Simulated database call
    if id == 0 {
        Err(DbError("user not found".into()))
    } else {
        Ok(User { id, name: "Alice".into() })
    }
}

async fn fetch_user_profile(user_id: u64) -> BoxedResult<User, DbError> {
    get_user_by_id(user_id)
        .with_ctx(|| context!("fetching profile for user {}", user_id))
        .await
        .map_err(Box::new)
}

#[tokio::main]
async fn main() {
    match fetch_user_profile(0).await {
        Ok(user) => println!("Found user: {:?}", user),
        Err(e) => println!("Error: {}", e.error_chain()),
    }
}
```

Output:

```text
Error: fetching profile for user 0 -> DbError("user not found")
