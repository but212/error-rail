use crate::common::{configure_criterion, DomainError, UserData};
use criterion::{criterion_group, Criterion};
use error_rail::ErrorPipeline;
use std::hint::black_box;
use std::time::Duration;

pub fn bench_retry_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("retry");

    // Transient error with successful retry
    group.bench_function("transient_success", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> =
                Err(DomainError::Network("Timeout".to_string()));
            let pipeline = ErrorPipeline::new(result)
                .retry()
                .max_retries(3)
                .after_hint(Duration::from_secs(1));
            black_box(pipeline.is_transient());
        })
    });

    // Permanent error should not retry
    group.bench_function("permanent_skip", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> =
                Err(DomainError::Validation("Invalid input".to_string()));
            let pipeline = ErrorPipeline::new(result);
            black_box(pipeline.should_retry());
        })
    });

    // Recover transient errors
    group.bench_function("recover_transient", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> =
                Err(DomainError::Network("Connection refused".to_string()));
            let recovered = ErrorPipeline::new(result)
                .recover_transient(|_| Ok(UserData::new(1)))
                .finish();
            let _ = black_box(recovered);
        })
    });

    // Should retry check
    group.bench_function("should_retry_check", |b| {
        b.iter(|| {
            let transient: Result<(), DomainError> =
                Err(DomainError::Database("Deadlock".to_string()));
            let permanent: Result<(), DomainError> =
                Err(DomainError::Validation("Bad input".to_string()));

            let t_pipeline = ErrorPipeline::new(transient);
            let p_pipeline = ErrorPipeline::new(permanent);

            black_box((t_pipeline.should_retry(), p_pipeline.should_retry()));
        })
    });

    group.finish();
}

criterion_group! {
    name = retry_benches;
    config = configure_criterion();
    targets = bench_retry_operations,
}
