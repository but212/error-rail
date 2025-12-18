use crate::common::{configure_criterion, DomainError, UserData};
use criterion::{criterion_group, Criterion};
use error_rail::{ComposableError, ErrorContext, ErrorPipeline};
use std::hint::black_box;

#[cfg(feature = "std")]
use error_rail::backtrace;

#[cfg(feature = "std")]
pub fn bench_backtrace_lazy_success(c: &mut Criterion) {
    c.bench_function("std/backtrace_lazy_success", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> = Ok(UserData::new(42));
            let _ = ErrorPipeline::new(result)
                .with_context(backtrace!())
                .finish();
        })
    });
}

#[cfg(feature = "std")]
pub fn bench_backtrace_lazy_error(c: &mut Criterion) {
    c.bench_function("std/backtrace_lazy_error", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> =
                Err(DomainError::Network("Connection refused".to_string()));
            let _ = ErrorPipeline::new(result)
                .with_context(backtrace!())
                .finish();
        })
    });
}

#[cfg(feature = "serde")]
pub fn bench_composable_error_serialization(c: &mut Criterion) {
    let err = ComposableError::new(DomainError::Network("API rate limit exceeded".to_string()))
        .with_context(ErrorContext::tag("external_api"))
        .with_context(ErrorContext::metadata("endpoint", "/api/v2/users"))
        .with_context(ErrorContext::metadata("retry_after", "60"))
        .with_context(ErrorContext::metadata("quota_limit", "1000"))
        .set_code(429);

    c.bench_function("serde/error_serialization", |b| {
        b.iter(|| black_box(serde_json::to_string(&err).unwrap()))
    });
}

#[cfg(feature = "std")]
criterion_group! {
    name = std_benches;
    config = configure_criterion();
    targets =
        bench_backtrace_lazy_success,
        bench_backtrace_lazy_error,
}

#[cfg(feature = "serde")]
criterion_group! {
    name = serde_benches;
    config = configure_criterion();
    targets = bench_composable_error_serialization,
}
