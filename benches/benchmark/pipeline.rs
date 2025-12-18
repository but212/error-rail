use criterion::{criterion_group, Criterion};
use error_rail::{ComposableError, ErrorContext, ErrorPipeline};
use std::hint::black_box;
use crate::common::{configure_criterion, simulate_auth_check, simulate_db_query, simulate_validation, DomainError, UserData};
use error_rail::context;

fn realistic_user_service(user_id: u64) -> Result<UserData, ComposableError<DomainError>> {
    ErrorPipeline::new(simulate_db_query(user_id))
        .with_context(context!("Fetching user {} from database", user_id))
        .and_then(|user| simulate_validation(user))
        .with_context(context!("Validating user data for user {}", user_id))
        .and_then(|user| simulate_auth_check(user))
        .with_context(context!("Authentication check for user {}", user_id))
        .finish()
}

pub fn bench_pipeline_vs_result_success(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline/vs_result");

    group.bench_function("pipeline_success", |b| {
        b.iter(|| {
            let result = realistic_user_service(42);
            let _ = black_box(result).is_ok();
        })
    });

    group.bench_function("result_with_context_success", |b| {
        b.iter(|| {
            let result = simulate_db_query(42)
                .and_then(|user| simulate_validation(user))
                .and_then(|user| simulate_auth_check(user));
            let result = result.map_err(|e| {
                ComposableError::<DomainError>::new(e)
                    .with_context(ErrorContext::new("User service operation failed"))
            });
            let _ = black_box(result).is_ok();
        })
    });

    group.bench_function("result_baseline_success", |b| {
        b.iter(|| {
            let result = simulate_db_query(42)
                .and_then(|user| simulate_validation(user))
                .and_then(|user| simulate_auth_check(user));
            let _ = black_box(result).is_ok();
        })
    });

    group.finish();
}

pub fn bench_pipeline_vs_result_error(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline/vs_result");

    group.bench_function("pipeline_error", |b| {
        b.iter(|| {
            let result = realistic_user_service(100); // Will fail at DB layer
            let _ = black_box(result).is_ok();
        })
    });

    group.bench_function("result_with_context_error", |b| {
        b.iter(|| {
            let result = simulate_db_query(100)
                .and_then(|user| simulate_validation(user))
                .and_then(|user| simulate_auth_check(user));
            let result = result.map_err(|e| {
                ComposableError::<DomainError>::new(e)
                    .with_context(ErrorContext::new("User service operation failed"))
            });
            let _ = black_box(result).is_ok();
        })
    });

    group.bench_function("result_baseline_error", |b| {
        b.iter(|| {
            let result = simulate_db_query(100)
                .and_then(|user| simulate_validation(user))
                .and_then(|user| simulate_auth_check(user));
            let _ = black_box(result).is_ok();
        })
    });

    group.finish();
}

criterion_group! {
    name = pipeline_benches;
    config = configure_criterion();
    targets =
        bench_pipeline_vs_result_success,
        bench_pipeline_vs_result_error,
}
