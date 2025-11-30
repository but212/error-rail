//! API Layering Example (New in 0.7.0)
//!
//! This example demonstrates the 3-level API hierarchy:
//! - Beginner: prelude module
//! - Intermediate: intermediate module  
//! - Advanced: advanced module
//!
//! Run with: cargo run --example api_layering

// =============================================================================
// Beginner API - Start here!
// =============================================================================

mod beginner_example {
    use error_rail::prelude::*;

    pub fn example() {
        println!("=== Beginner API (prelude) ===\n");

        // Simple error handling with .ctx()
        let result: BoxedResult<String, std::io::Error> =
            std::fs::read_to_string("config.toml").ctx("loading config");

        if let Err(e) = result {
            println!("Error: {}", e.error_chain());
        }

        // Using rail! macro
        let result2 = rail!(std::fs::read_to_string("data.json"));
        if let Err(e) = result2 {
            println!("Error with rail!: {}", e.error_chain());
        }

        // Using group! for structured context
        let err = ComposableError::new("connection failed").with_context(group!(
            tag("database"),
            location(file!(), line!()),
            metadata("host", "localhost")
        ));

        println!("Structured error: {}\n", err.error_chain());
    }
}

// =============================================================================
// Intermediate API - Advanced patterns
// =============================================================================

mod intermediate_example {
    use error_rail::intermediate::*;
    use std::time::Duration;

    #[derive(Debug)]
    pub enum ApiError {
        Timeout,
        #[allow(dead_code)]
        RateLimited(u64),
        NotFound,
    }

    impl TransientError for ApiError {
        fn is_transient(&self) -> bool {
            matches!(self, ApiError::Timeout | ApiError::RateLimited(_))
        }

        fn retry_after_hint(&self) -> Option<Duration> {
            match self {
                ApiError::RateLimited(secs) => Some(Duration::from_secs(*secs)),
                ApiError::Timeout => Some(Duration::from_millis(100)),
                _ => None,
            }
        }
    }

    pub fn example() {
        println!("=== Intermediate API ===\n");

        // Transient error classification
        let timeout_err = ApiError::Timeout;
        let not_found_err = ApiError::NotFound;

        println!("Timeout is transient: {}", timeout_err.is_transient());
        println!("NotFound is transient: {}", not_found_err.is_transient());

        if let Some(delay) = timeout_err.retry_after_hint() {
            println!("Retry after: {:?}", delay);
        }

        // Custom error formatting
        use error_rail::ComposableError;
        let err = ComposableError::new("database error")
            .with_context(error_rail::context!("query failed"))
            .set_code(500);

        let formatted = err.fmt().pretty().show_code(true).to_string();
        println!("\nFormatted error:\n{}\n", formatted);
    }
}

// =============================================================================
// Advanced API - Library authors
// =============================================================================

mod advanced_example {
    use error_rail::advanced::*;
    use error_rail::ErrorContext;
    use error_rail::IntoErrorContext;

    pub fn example() {
        println!("=== Advanced API ===\n");

        // Direct access to ErrorVec
        let mut vec: ErrorVec<&str> = ErrorVec::new();
        vec.push("error 1");
        vec.push("error 2");
        println!("ErrorVec contents: {:?}", vec);

        // Using ErrorContextBuilder
        let ctx = ErrorContextBuilder::new()
            .message("custom context")
            .tag("advanced")
            .metadata("level", "expert")
            .build();

        println!("Built context: {}", ctx.message());

        // LazyContext for deferred evaluation
        let lazy = LazyContext::new(|| {
            format!(
                "computed at: {}",
                std::time::SystemTime::now().elapsed().unwrap().as_secs()
            )
        });
        println!(
            "LazyContext created (not yet evaluated): {}",
            lazy.into_error_context().message()
        );

        // GroupContext via ErrorContext::builder
        let group = ErrorContext::builder()
            .message("operation failed")
            .tag("system")
            .metadata("component", "core")
            .build();

        println!("GroupContext: {}\n", group.message());
    }
}

// =============================================================================
// Main
// =============================================================================

fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║   error-rail API Layering Examples    ║");
    println!("╚════════════════════════════════════════╝\n");

    beginner_example::example();
    intermediate_example::example();
    advanced_example::example();

    println!("✓ All examples completed!");
}
