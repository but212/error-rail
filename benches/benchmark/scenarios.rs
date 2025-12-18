use crate::common::{
    configure_criterion, simulate_auth_check, simulate_db_query, simulate_validation, DomainError,
    UserData,
};
use criterion::{criterion_group, Criterion, Throughput};
use error_rail::context;
use error_rail::{ComposableError, ErrorContext, ErrorPipeline};
use std::hint::black_box;
use std::time::Duration;

fn realistic_user_service(user_id: u64) -> Result<UserData, ComposableError<DomainError>> {
    ErrorPipeline::new(simulate_db_query(user_id))
        .with_context(context!("Fetching user {} from database", user_id))
        .and_then(|user| simulate_validation(user))
        .with_context(context!("Validating user data for user {}", user_id))
        .and_then(|user| simulate_auth_check(user))
        .with_context(context!("Authentication check for user {}", user_id))
        .finish()
}

pub fn bench_real_world_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world");

    // Simulated HTTP request with error handling
    group.bench_function("http_request_simulation", |b| {
        b.iter(|| {
            let result: Result<String, DomainError> =
                Err(DomainError::Network("503 Service Unavailable".to_string()));

            let response = ErrorPipeline::new(result)
                .with_context(ErrorContext::tag("http"))
                .with_context(ErrorContext::metadata("method", "GET"))
                .with_context(ErrorContext::metadata("url", "/api/v1/users"))
                .with_context(ErrorContext::metadata("status", "503"))
                .retry()
                .max_retries(3)
                .after_hint(Duration::from_secs(1))
                .to_error_pipeline()
                .finish();

            let _ = black_box(response);
        })
    });

    // Database transaction error
    group.bench_function("database_transaction_rollback", |b| {
        b.iter(|| {
            let tx_result: Result<(), DomainError> =
                Err(DomainError::Database("Deadlock detected".to_string()));

            let result = ErrorPipeline::new(tx_result)
                .with_context(ErrorContext::tag("transaction"))
                .with_context(ErrorContext::metadata("isolation", "serializable"))
                .with_context(ErrorContext::metadata("table", "users"))
                .retry()
                .max_retries(5)
                .after_hint(Duration::from_millis(100))
                .to_error_pipeline()
                .finish();

            let _ = black_box(result);
        })
    });

    // Microservice error propagation
    group.bench_function("microservice_error_propagation", |b| {
        b.iter(|| {
            let auth_result: Result<(), DomainError> =
                Err(DomainError::Authentication("Invalid token".to_string()));

            let result = ErrorPipeline::new(auth_result)
                .with_context(ErrorContext::tag("auth-service"))
                .with_context(ErrorContext::metadata("service", "auth-ms-01"))
                .and_then(|_| Err(DomainError::Network("Connection timeout".to_string())))
                .with_context(ErrorContext::tag("user-service"))
                .with_context(ErrorContext::metadata("service", "user-ms-02"))
                .and_then(|_: ()| Ok::<(), DomainError>(()))
                .with_context(ErrorContext::tag("api-gateway"))
                .finish()
                .map_err(|e| e.set_code(500));

            let _ = black_box(result);
        })
    });

    group.finish();
}

pub fn bench_mixed_success_error_ratios(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world/mixed_ratios");
    group.throughput(Throughput::Elements(100));

    group.bench_function("95percent_success", |b| {
        b.iter(|| {
            let results: Vec<Result<UserData, ComposableError<DomainError>>> =
                (0..100).map(|i| realistic_user_service(i)).collect();
            let success_count = results.iter().filter(|r| r.is_ok()).count();
            black_box(success_count);
        })
    });

    group.bench_function("50percent_success", |b| {
        b.iter(|| {
            let results: Vec<Result<UserData, ComposableError<DomainError>>> = (0..100)
                .map(|i| realistic_user_service(i * 2)) // Higher failure rate
                .collect();
            let success_count = results.iter().filter(|r| r.is_ok()).count();
            success_count
        })
    });

    group.finish();
}

criterion_group! {
    name = real_world_benches;
    config = configure_criterion();
    targets =
        bench_real_world_scenarios,
        bench_mixed_success_error_ratios,
}
