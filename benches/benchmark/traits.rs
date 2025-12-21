use crate::common::{configure_criterion, DomainError, UserData};
use criterion::{criterion_group, Criterion};
use error_rail::prelude::*;
use std::hint::black_box;

/// Benchmarks for ResultExt trait methods
pub fn bench_result_ext(c: &mut Criterion) {
    let mut group = c.benchmark_group("traits/result_ext");

    // ResultExt::ctx() - Success path (should be near zero-cost)
    group.bench_function("ctx_success", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> = Ok(UserData::new(1));
            let _ = black_box(result.ctx("operation context"));
        })
    });

    // ResultExt::ctx() - Error path
    group.bench_function("ctx_error", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> =
                Err(DomainError::Database("connection failed".to_string()));
            let _ = black_box(result.ctx("database operation"));
        })
    });

    // ResultExt::ctx_with() - Success path (lazy, should skip closure)
    group.bench_function("ctx_with_lazy_success", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> = Ok(UserData::new(1));
            let user_id = 42u64;
            let _ = black_box(result.ctx_with(|| format!("fetching user {}", user_id)));
        })
    });

    // ResultExt::ctx_with() - Error path
    group.bench_function("ctx_with_lazy_error", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> =
                Err(DomainError::Network("timeout".to_string()));
            let user_id = 42u64;
            let _ = black_box(result.ctx_with(|| format!("fetching user {}", user_id)));
        })
    });

    group.finish();
}

/// Benchmarks for BoxedResultExt trait methods
pub fn bench_boxed_result_ext(c: &mut Criterion) {
    let mut group = c.benchmark_group("traits/boxed_result_ext");

    // BoxedResultExt::ctx_boxed() - Chaining on BoxedResult
    group.bench_function("ctx_boxed_chain", |b| {
        b.iter(|| {
            let inner: BoxedResult<UserData, DomainError> =
                Err(DomainError::Validation("invalid".to_string())).ctx("inner context");
            let _ = black_box(inner.ctx_boxed("outer context"));
        })
    });

    // BoxedResultExt::ctx_boxed_with() - Lazy chaining
    group.bench_function("ctx_boxed_with_chain", |b| {
        b.iter(|| {
            let inner: BoxedResult<UserData, DomainError> =
                Err(DomainError::Validation("invalid".to_string())).ctx("inner context");
            let _ = black_box(inner.ctx_boxed_with(|| "lazy outer context".to_string()));
        })
    });

    // Multiple chain depth comparison
    group.bench_function("ctx_boxed_depth_3", |b| {
        b.iter(|| {
            let result: BoxedResult<UserData, DomainError> =
                Err(DomainError::Database("error".to_string())).ctx("level 1");
            let result = result.ctx_boxed("level 2");
            let result = result.ctx_boxed("level 3");
            let _ = black_box(result);
        })
    });

    group.finish();
}

/// Benchmarks for WithError trait methods
pub fn bench_with_error(c: &mut Criterion) {
    use error_rail::traits::WithError;

    let mut group = c.benchmark_group("traits/with_error");

    // fmap_error
    group.bench_function("fmap_error", |b| {
        b.iter(|| {
            let result: Result<i32, &str> = Err("error");
            let mapped = result.fmap_error(|e| e.to_uppercase());
            let _ = black_box(mapped);
        })
    });

    // to_result_first
    group.bench_function("to_result_first", |b| {
        b.iter(|| {
            let result: Result<i32, &str> = Err("error");
            let _ = black_box(result.to_result_first());
        })
    });

    // to_result_all
    group.bench_function("to_result_all", |b| {
        b.iter(|| {
            let result: Result<i32, &str> = Err("error");
            let _ = black_box(result.to_result_all());
        })
    });

    group.finish();
}

/// Benchmarks for IntoErrorContext implementations
pub fn bench_into_error_context(c: &mut Criterion) {
    use error_rail::traits::IntoErrorContext;

    let mut group = c.benchmark_group("traits/into_error_context");

    // &'static str
    group.bench_function("from_static_str", |b| {
        b.iter(|| {
            let ctx = "static context".into_error_context();
            let _ = black_box(ctx);
        })
    });

    // String
    group.bench_function("from_string", |b| {
        b.iter(|| {
            let ctx = "dynamic context".to_string().into_error_context();
            let _ = black_box(ctx);
        })
    });

    // ErrorContext passthrough
    group.bench_function("from_error_context", |b| {
        b.iter(|| {
            let ec = ErrorContext::tag("test");
            let ctx = ec.into_error_context();
            let _ = black_box(ctx);
        })
    });

    group.finish();
}

criterion_group! {
    name = traits_benches;
    config = configure_criterion();
    targets =
        bench_result_ext,
        bench_boxed_result_ext,
        bench_with_error,
        bench_into_error_context,
}
