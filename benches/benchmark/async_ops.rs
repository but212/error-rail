use crate::common::configure_criterion;
use criterion::{criterion_group, Criterion};

#[cfg(feature = "async")]
use crate::common::{DomainError, UserData};
#[cfg(feature = "async")]
use error_rail::prelude_async::*;
#[cfg(feature = "async")]
use error_rail::Validation;
#[cfg(feature = "async")]
use tokio::runtime::Runtime;

#[cfg(feature = "async")]
pub fn bench_async_pipeline_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("async/pipeline");

    group.bench_function("success_path", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result: Result<UserData, DomainError> = Ok(UserData::new(1));
                let _ = AsyncErrorPipeline::new(async { result })
                    .with_context("operation context")
                    .finish()
                    .await;
            })
        })
    });

    group.bench_function("error_path", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result: Result<UserData, DomainError> =
                    Err(DomainError::Validation("invalid".to_string()));
                let _ = AsyncErrorPipeline::new(async { result })
                    .with_context("operation context")
                    .finish()
                    .await;
            })
        })
    });

    group.finish();
}

#[cfg(feature = "async")]
pub fn bench_async_context_lazy_vs_eager(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("async/context_evaluation");

    group.bench_function("lazy_success", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result: Result<UserData, DomainError> = Ok(UserData::new(1));
                let _ = async { result }.ctx("lazy context").await;
            })
        })
    });

    group.bench_function("eager_success", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result: Result<UserData, DomainError> = Ok(UserData::new(1));
                // Simulating eager by evaluating message before await
                let msg = "eager context".to_string();
                let _ = async { result }.ctx(msg).await;
            })
        })
    });

    group.finish();
}

#[cfg(feature = "async")]
pub fn bench_async_retry(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("async/retry");

    group.bench_function("transient_retry_1", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut attempts = 0;
                let _ = retry_with_policy(
                    || {
                        attempts += 1;
                        async move {
                            if attempts < 2 {
                                Err(DomainError::Network("timeout".to_string()))
                            } else {
                                Ok(UserData::new(1))
                            }
                        }
                    },
                    ExponentialBackoff::default().with_max_attempts(1),
                    |_| async {}, // No sleep for benchmark
                )
                .await;
            })
        })
    });

    group.finish();
}

#[cfg(feature = "async")]
pub fn bench_async_validation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("async/validation");

    group.bench_function("sequential_3", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _ = validate_all_async(
                    (1..=3).map(|i| async move { Validation::<DomainError, i32>::Valid(i) }),
                )
                .await;
            })
        })
    });

    group.finish();
}

#[cfg(feature = "async")]
criterion_group! {
    name = async_ops_benches;
    config = configure_criterion();
    targets =
        bench_async_pipeline_overhead,
        bench_async_context_lazy_vs_eager,
        bench_async_retry,
        bench_async_validation,
}

#[cfg(not(feature = "async"))]
criterion_group! {
    name = async_ops_benches;
    config = configure_criterion();
    targets = dummy
}

#[cfg(not(feature = "async"))]
fn dummy(_c: &mut Criterion) {}
