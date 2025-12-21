use crate::common::{configure_criterion, DomainError, UserData};
use criterion::{criterion_group, Criterion};
use error_rail::{
    context, group, ComposableError, ErrorContext, ErrorPipeline, LazyContext, LazyGroupContext,
};
use std::hint::black_box;

/// Benchmarks for LazyContext creation and evaluation
pub fn bench_lazy_context_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("lazy_context/creation");

    // LazyContext construction (closure not called)
    group.bench_function("lazy_context_construct", |b| {
        let user_id = 42u64;
        b.iter(|| {
            let ctx = LazyContext::new(move || format!("user_id: {}", user_id));
            let _ = black_box(ctx);
        })
    });

    // LazyGroupContext construction
    group.bench_function("lazy_group_context_construct", |b| {
        b.iter(|| {
            let ctx = LazyGroupContext::new(|| {
                ErrorContext::builder()
                    .tag("dynamic_tag")
                    .metadata("key", "value")
                    .build()
            });
            let _ = black_box(ctx);
        })
    });

    group.finish();
}

/// Benchmarks comparing lazy vs eager context with success path
pub fn bench_lazy_vs_eager_success(c: &mut Criterion) {
    let mut group = c.benchmark_group("lazy_context/success_path");

    let user_id = 42u64;
    let username = "alice";

    // Lazy context! macro - success path (closure NOT called)
    group.bench_function("context_macro", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> = Ok(UserData::new(1));
            let _ = ErrorPipeline::new(result)
                .with_context(context!("user {} ({})", user_id, username))
                .finish();
        })
    });

    // Eager format! - success path (format ALWAYS called)
    group.bench_function("format_eager", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> = Ok(UserData::new(1));
            let msg = format!("user {} ({})", user_id, username);
            let _ = ErrorPipeline::new(result).with_context(msg).finish();
        })
    });

    // Static string - success path (baseline)
    group.bench_function("static_str", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> = Ok(UserData::new(1));
            let _ = ErrorPipeline::new(result)
                .with_context("static context")
                .finish();
        })
    });

    group.finish();
}

/// Benchmarks comparing lazy vs eager context with error path
pub fn bench_lazy_vs_eager_error(c: &mut Criterion) {
    let mut group = c.benchmark_group("lazy_context/error_path");

    let user_id = 42u64;
    let username = "alice";

    // Lazy context! macro - error path (closure IS called)
    group.bench_function("context_macro", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> =
                Err(DomainError::Validation("invalid".to_string()));
            let _ = ErrorPipeline::new(result)
                .with_context(context!("user {} ({})", user_id, username))
                .finish();
        })
    });

    // Eager format! - error path
    group.bench_function("format_eager", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> =
                Err(DomainError::Validation("invalid".to_string()));
            let msg = format!("user {} ({})", user_id, username);
            let _ = ErrorPipeline::new(result).with_context(msg).finish();
        })
    });

    group.finish();
}

/// Benchmarks for group! macro with lazy evaluation
pub fn bench_lazy_group_context(c: &mut Criterion) {
    let mut group = c.benchmark_group("lazy_context/group_macro");

    // group! macro with complex context - success path
    group.bench_function("group_success", |b| {
        let query = "SELECT * FROM users WHERE id = ?";
        b.iter(|| {
            let result: Result<UserData, DomainError> = Ok(UserData::new(1));
            let _ = ErrorPipeline::new(result)
                .with_context(group!(
                    tag("database"),
                    message("executing query: {}", query),
                    metadata("table", "users")
                ))
                .finish();
        })
    });

    // group! macro with complex context - error path
    group.bench_function("group_error", |b| {
        let query = "SELECT * FROM users WHERE id = ?";
        b.iter(|| {
            let result: Result<UserData, DomainError> =
                Err(DomainError::Database("timeout".to_string()));
            let _ = ErrorPipeline::new(result)
                .with_context(group!(
                    tag("database"),
                    message("executing query: {}", query),
                    metadata("table", "users")
                ))
                .finish();
        })
    });

    // ComposableError with LazyGroupContext
    group.bench_function("composable_lazy_group", |b| {
        b.iter(|| {
            let err = ComposableError::new(DomainError::Network("timeout".to_string()))
                .with_context(LazyGroupContext::new(|| {
                    ErrorContext::builder()
                        .tag("network")
                        .metadata("host", "api.example.com")
                        .metadata("timeout_ms", "5000")
                        .build()
                }));
            let _ = black_box(err);
        })
    });

    group.finish();
}

criterion_group! {
    name = lazy_context_benches;
    config = configure_criterion();
    targets =
        bench_lazy_context_creation,
        bench_lazy_vs_eager_success,
        bench_lazy_vs_eager_error,
        bench_lazy_group_context,
}
