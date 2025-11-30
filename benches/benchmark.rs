use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
#[cfg(feature = "std")]
use error_rail::backtrace;
use error_rail::traits::{ErrorOps, TransientError};
use error_rail::validation::Validation;
use error_rail::{context, ComposableError, ErrorContext, ErrorPipeline};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{hint::black_box, sync::OnceLock, time::Duration};

// ============================================================================
// Test Data & Domain Types
// ============================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct UserData {
    user_id: u64,
    username: String,
    email: String,
    metadata: std::collections::HashMap<String, String>,
}

impl UserData {
    fn new(id: u64) -> Self {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("role".to_string(), "user".to_string());
        metadata.insert("department".to_string(), "engineering".to_string());
        metadata.insert("location".to_string(), "seoul".to_string());

        Self {
            user_id: id,
            username: format!("user_{id}"),
            email: format!("user{id}@company.com"),
            metadata,
        }
    }
}

fn realistic_user_data() -> &'static Vec<UserData> {
    static INSTANCE: OnceLock<Vec<UserData>> = OnceLock::new();
    INSTANCE.get_or_init(|| (0..1000).map(UserData::new).collect())
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
enum DomainError {
    Database(String),
    Network(String),
    Validation(String),
    Authentication(String),
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::Database(msg) => write!(f, "Database error: {msg}"),
            DomainError::Network(msg) => write!(f, "Network error: {msg}"),
            DomainError::Validation(msg) => write!(f, "Validation error: {msg}"),
            DomainError::Authentication(msg) => write!(f, "Authentication error: {msg}"),
        }
    }
}

impl TransientError for DomainError {
    fn is_transient(&self) -> bool {
        matches!(self, DomainError::Network(_) | DomainError::Database(_))
    }

    fn retry_after_hint(&self) -> Option<Duration> {
        match self {
            DomainError::Network(_) => Some(Duration::from_secs(1)),
            DomainError::Database(_) => Some(Duration::from_millis(100)),
            _ => None,
        }
    }

    fn max_retries_hint(&self) -> Option<u32> {
        match self {
            DomainError::Network(_) => Some(3),
            DomainError::Database(_) => Some(5),
            _ => None,
        }
    }
}

// ============================================================================
// Simulation Functions
// ============================================================================

fn simulate_db_query(user_id: u64) -> Result<UserData, DomainError> {
    if user_id % 100 == 0 {
        Err(DomainError::Database("Connection timeout".to_string()))
    } else {
        Ok(UserData::new(user_id))
    }
}

fn simulate_validation(user: UserData) -> Result<UserData, DomainError> {
    if user.user_id % 50 == 0 {
        Err(DomainError::Validation("Invalid email format".to_string()))
    } else {
        Ok(user)
    }
}

fn simulate_auth_check(user: UserData) -> Result<UserData, DomainError> {
    if user.user_id % 25 == 0 {
        Err(DomainError::Authentication("Token expired".to_string()))
    } else {
        Ok(user)
    }
}

fn validate_user_email(email: &str) -> Result<String, DomainError> {
    if email.contains('@') {
        Ok(email.to_string())
    } else {
        Err(DomainError::Validation("Invalid email format".to_string()))
    }
}

fn validate_user_age(age: i32) -> Result<i32, DomainError> {
    if age >= 18 && age <= 120 {
        Ok(age)
    } else {
        Err(DomainError::Validation(
            "Age out of valid range".to_string(),
        ))
    }
}

fn validate_user_name(name: &str) -> Result<String, DomainError> {
    if name.len() >= 2 && name.len() <= 50 {
        Ok(name.to_string())
    } else {
        Err(DomainError::Validation("Name length invalid".to_string()))
    }
}

fn realistic_user_service(user_id: u64) -> Result<UserData, ComposableError<DomainError>> {
    ErrorPipeline::new(simulate_db_query(user_id))
        .with_context(context!("Fetching user {} from database", user_id))
        .and_then(|user| simulate_validation(user))
        .with_context(context!("Validating user data for user {}", user_id))
        .and_then(|user| simulate_auth_check(user))
        .with_context(context!("Authentication check for user {}", user_id))
        .finish()
}

// ============================================================================
// Group 1: Core Error Operations
// ============================================================================

fn bench_composable_error_creation(c: &mut Criterion) {
    c.bench_function("core/error_creation", |b| {
        b.iter(|| {
            black_box(
                ComposableError::new(DomainError::Database(
                    "Connection pool exhausted".to_string(),
                ))
                .with_context(ErrorContext::tag("database"))
                .with_context(ErrorContext::metadata("query", "SELECT * FROM users"))
                .with_context(ErrorContext::metadata(
                    "host",
                    "db-primary-01.company.local",
                ))
                .with_context(ErrorContext::metadata("retry_count", "3"))
                .set_code(503),
            )
        })
    });
}

fn bench_error_cloning_and_arc(c: &mut Criterion) {
    let err = ComposableError::new(DomainError::Network("Service unavailable".to_string()))
        .with_context(ErrorContext::tag("external_service"))
        .set_code(503);

    c.bench_function("core/error_clone", |b| {
        b.iter(|| {
            let cloned = black_box(err.clone());
            let _ = black_box(cloned);
        })
    });

    c.bench_function("core/error_arc_wrap", |b| {
        b.iter(|| {
            let arc_err = black_box(std::sync::Arc::new(err.clone()));
            black_box(arc_err);
        })
    });
}

fn bench_error_cloning_deep(c: &mut Criterion) {
    let mut group = c.benchmark_group("core/error_clone_deep");

    for depth in [5, 10, 20, 50] {
        let mut err = ComposableError::new(DomainError::Database("Query failed".to_string()));
        for i in 0..depth {
            err = err
                .with_context(ErrorContext::new(format!("layer_{}", i)))
                .with_context(ErrorContext::metadata("depth", i.to_string()));
        }

        group.bench_with_input(BenchmarkId::from_parameter(depth), &err, |b, err| {
            b.iter(|| black_box(err.clone()))
        });
    }
    group.finish();
}

fn bench_error_ops_recover(c: &mut Criterion) {
    c.bench_function("core/ops_recover", |b| {
        b.iter(|| black_box(Err::<i32, &str>("missing").recover(|_| Ok(42))))
    });
}

fn bench_error_ops_bimap(c: &mut Criterion) {
    c.bench_function("core/ops_bimap", |b| {
        b.iter(|| black_box(Ok::<i32, &str>(21).bimap_result(|x| x * 2, |e| e.to_uppercase())))
    });
}

// ============================================================================
// Group 2: Retry Operations (NEW)
// ============================================================================

fn bench_retry_operations(c: &mut Criterion) {
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

// ============================================================================
// Group 3: Error Conversion Patterns (NEW)
// ============================================================================

fn bench_error_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("conversions");

    // Map core error type
    group.bench_function("map_core", |b| {
        b.iter(|| {
            let err = ComposableError::new("io error")
                .with_context(ErrorContext::tag("fs"))
                .set_code(500);
            let mapped = err.map_core(|e| format!("wrapped: {}", e));
            let _ = black_box(mapped);
        })
    });

    // std::io::Error conversion
    group.bench_function("std_io_to_domain", |b| {
        b.iter(|| {
            let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "config file not found");
            let converted = ComposableError::new(DomainError::Database(io_err.to_string()))
                .with_context(ErrorContext::tag("file_system"))
                .with_context(ErrorContext::metadata("path", "/etc/app/config.toml"));
            let _ = black_box(converted);
        })
    });

    // Serde error conversion
    group.bench_function("serde_to_domain", |b| {
        b.iter(|| {
            let json_str = "{invalid json}";
            let serde_result: Result<serde_json::Value, serde_json::Error> =
                serde_json::from_str(json_str);
            let converted = match serde_result {
                Ok(_) => unreachable!(),
                Err(e) => ComposableError::new(DomainError::Validation(e.to_string()))
                    .with_context(ErrorContext::tag("json_parsing"))
                    .with_context(ErrorContext::metadata("input", json_str)),
            };
            let _ = black_box(converted);
        })
    });

    // Error type conversion chain
    group.bench_function("conversion_chain", |b| {
        b.iter(|| {
            let err = ComposableError::new("initial")
                .map_core(|e| format!("step1: {}", e))
                .map_core(|e| format!("step2: {}", e))
                .map_core(|e| format!("step3: {}", e));
            let _ = black_box(err);
        })
    });

    group.finish();
}

// ============================================================================
// Group 4: Context Operations
// ============================================================================

fn bench_context_lazy_vs_eager_success(c: &mut Criterion) {
    let users = realistic_user_data();
    let mut group = c.benchmark_group("context/lazy_vs_eager");

    group.bench_function("lazy_success", |b| {
        b.iter(|| {
            let user = black_box(&users[0]);
            let result: Result<UserData, &str> = Ok(user.clone());
            let _ = ErrorPipeline::new(result)
                .with_context(context!("user_data: {:?}", user))
                .finish_boxed();
        })
    });

    group.bench_function("eager_success", |b| {
        b.iter(|| {
            let user = black_box(&users[0]);
            let result: Result<UserData, &str> = Ok(user.clone());
            let message = format!("user_data: {:?}", user);
            let _ = ErrorPipeline::new(result).with_context(message).finish();
        })
    });

    group.bench_function("baseline_success", |b| {
        b.iter(|| {
            let result: Result<UserData, &str> = Ok(UserData::new(1));
            let _ = ErrorPipeline::new(result).finish();
        })
    });

    group.finish();
}

fn bench_context_lazy_vs_eager_error(c: &mut Criterion) {
    let users = realistic_user_data();
    let mut group = c.benchmark_group("context/lazy_vs_eager");

    group.bench_function("lazy_error", |b| {
        b.iter(|| {
            let user = black_box(&users[0]);
            let result: Result<UserData, DomainError> =
                Err(DomainError::Validation("Email format invalid".to_string()));
            let _ = ErrorPipeline::new(result)
                .with_context(context!("failed_user: {:?}", user))
                .finish_boxed();
        })
    });

    group.bench_function("eager_error", |b| {
        b.iter(|| {
            let user = black_box(&users[0]);
            let result: Result<UserData, DomainError> =
                Err(DomainError::Validation("Email format invalid".to_string()));
            let message = format!("failed_user: {:?}", user);
            let _ = ErrorPipeline::new(result).with_context(message).finish();
        })
    });

    group.bench_function("baseline_error", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> =
                Err(DomainError::Validation("Email format invalid".to_string()));
            let _ = ErrorPipeline::new(result).finish();
        })
    });

    group.finish();
}

// ============================================================================
// Group 5: Scaling Tests (Parameterized)
// ============================================================================

fn bench_context_depth_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling/context_depth");

    for depth in [1, 5, 10, 20, 50] {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &depth| {
            b.iter(|| {
                let mut err =
                    ComposableError::new(DomainError::Database("Query failed".to_string()))
                        .with_context(ErrorContext::tag("database"))
                        .with_context(ErrorContext::metadata("query_id", "12345"));

                for i in 0..depth {
                    err = err
                        .with_context(ErrorContext::new(format!("layer_{}", i)))
                        .with_context(ErrorContext::metadata("depth", i.to_string()));
                }
                let _ = black_box(err);
            })
        });
    }

    group.finish();
}

fn bench_validation_batch_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling/validation_batch");

    for size in [10, 100, 1000, 5000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let emails: Vec<String> = (0..size)
                    .map(|i| {
                        if i % 5 == 0 {
                            format!("invalid{}", i)
                        } else {
                            format!("user{}@test.com", i)
                        }
                    })
                    .collect();

                let result: Validation<DomainError, Vec<String>> = emails
                    .iter()
                    .map(|email| validate_user_email(email))
                    .collect();
                let _ = black_box(result);
            })
        });
    }

    group.finish();
}

// ============================================================================
// Group 6: Pipeline Operations
// ============================================================================

fn bench_pipeline_vs_result_success(c: &mut Criterion) {
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

fn bench_pipeline_vs_result_error(c: &mut Criterion) {
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

// ============================================================================
// Group 7: Validation Operations
// ============================================================================

fn bench_validation_collect_realistic(c: &mut Criterion) {
    let test_emails = vec![
        "user1@company.com",
        "invalid-email",
        "user3@company.com",
        "user4@company.com",
        "another-invalid",
        "user6@company.com",
        "user7@company.com",
        "bad-email-format",
        "user9@company.com",
        "user10@company.com",
    ];

    let mut group = c.benchmark_group("validation");

    group.bench_function("collect_realistic_mixed", |b| {
        b.iter(|| {
            let result: Validation<DomainError, Vec<String>> = test_emails
                .iter()
                .map(|email| validate_user_email(email))
                .collect();
            black_box(&result);
        })
    });

    group.bench_function("manual_collect_realistic_mixed", |b| {
        b.iter(|| {
            let mut values = Vec::new();
            let mut errors = Vec::new();
            for email in &test_emails {
                match validate_user_email(email) {
                    Ok(v) => values.push(v),
                    Err(e) => errors.push(e),
                }
            }
            black_box((&values, &errors));
        })
    });

    group.bench_function("collect_heterogeneous", |b| {
        b.iter(|| {
            let email_result: Validation<DomainError, Vec<String>> = test_emails
                .iter()
                .map(|email| validate_user_email(email))
                .collect();
            let age_result: Validation<DomainError, Vec<i32>> =
                (18..28).map(|age| validate_user_age(age)).collect();
            let name_result: Validation<DomainError, Vec<String>> =
                ["Alice", "B", "Charlie", "D", "Eve"]
                    .iter()
                    .map(|name| validate_user_name(name))
                    .collect();

            black_box((&email_result, &age_result, &name_result));
        })
    });

    group.finish();
}

// ============================================================================
// Group 8: Real-World Scenarios (NEW)
// ============================================================================

fn bench_real_world_scenarios(c: &mut Criterion) {
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

fn bench_mixed_success_error_ratios(c: &mut Criterion) {
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
            black_box(success_count);
        })
    });

    group.finish();
}

// ============================================================================
// Group 9: Memory & Allocation (NEW)
// ============================================================================

fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    // Large metadata contexts
    group.bench_function("large_metadata_contexts", |b| {
        b.iter(|| {
            let large_json = serde_json::json!({
                "user_id": 12345,
                "request_id": "abc-def-ghi-jkl-mno",
                "timestamp": "2024-01-01T00:00:00Z",
                "metadata": {
                    "ip": "192.168.1.1",
                    "user_agent": "Mozilla/5.0...",
                    "session": "session-token-here"
                }
            })
            .to_string();

            let err = ComposableError::new(DomainError::Network("Timeout".to_string()))
                .with_context(ErrorContext::metadata("request", large_json))
                .with_context(ErrorContext::tag("api"))
                .set_code(408);

            let _ = black_box(err);
        })
    });

    // String vs static str contexts
    group.bench_function("string_allocation", |b| {
        b.iter(|| {
            let dynamic = format!("Error at timestamp: {}", 1234567890);
            let err = ComposableError::new("error").with_context(dynamic);
            let _ = black_box(err);
        })
    });

    group.bench_function("static_str_no_allocation", |b| {
        b.iter(|| {
            let err = ComposableError::new("error").with_context("Static error context");
            let _ = black_box(err);
        })
    });

    group.finish();
}

// ============================================================================
// Feature-Gated Benchmarks
// ============================================================================

#[cfg(feature = "std")]
fn bench_backtrace_lazy_success(c: &mut Criterion) {
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
fn bench_backtrace_lazy_error(c: &mut Criterion) {
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
fn bench_composable_error_serialization(c: &mut Criterion) {
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

// ============================================================================
// Benchmark Group Configuration
// ============================================================================

fn configure_criterion() -> Criterion {
    Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(5))
        .noise_threshold(0.05)
}

// Base benchmarks (no_std compatible)
criterion_group! {
    name = core_benches;
    config = configure_criterion();
    targets =
        bench_composable_error_creation,
        bench_error_cloning_and_arc,
        bench_error_cloning_deep,
        bench_error_ops_recover,
        bench_error_ops_bimap,
}

criterion_group! {
    name = retry_benches;
    config = configure_criterion();
    targets = bench_retry_operations,
}

criterion_group! {
    name = conversion_benches;
    config = configure_criterion();
    targets = bench_error_conversions,
}

criterion_group! {
    name = context_benches;
    config = configure_criterion();
    targets =
        bench_context_lazy_vs_eager_success,
        bench_context_lazy_vs_eager_error,
}

criterion_group! {
    name = scaling_benches;
    config = configure_criterion();
    targets =
        bench_context_depth_scaling,
        bench_validation_batch_scaling,
}

criterion_group! {
    name = pipeline_benches;
    config = configure_criterion();
    targets =
        bench_pipeline_vs_result_success,
        bench_pipeline_vs_result_error,
}

criterion_group! {
    name = validation_benches;
    config = configure_criterion();
    targets = bench_validation_collect_realistic,
}

criterion_group! {
    name = real_world_benches;
    config = configure_criterion();
    targets =
        bench_real_world_scenarios,
        bench_mixed_success_error_ratios,
}

criterion_group! {
    name = memory_benches;
    config = configure_criterion();
    targets = bench_memory_allocation,
}

// std-specific benchmarks
#[cfg(feature = "std")]
criterion_group! {
    name = std_benches;
    config = configure_criterion();
    targets =
        bench_backtrace_lazy_success,
        bench_backtrace_lazy_error,
}

// serde-specific benchmarks
#[cfg(feature = "serde")]
criterion_group! {
    name = serde_benches;
    config = configure_criterion();
    targets = bench_composable_error_serialization,
}

// Main benchmark groups based on available features
#[cfg(all(feature = "std", feature = "serde"))]
criterion_main!(
    core_benches,
    retry_benches,
    conversion_benches,
    context_benches,
    scaling_benches,
    pipeline_benches,
    validation_benches,
    real_world_benches,
    memory_benches,
    std_benches,
    serde_benches
);

#[cfg(all(feature = "std", not(feature = "serde")))]
criterion_main!(
    core_benches,
    retry_benches,
    conversion_benches,
    context_benches,
    scaling_benches,
    pipeline_benches,
    validation_benches,
    real_world_benches,
    memory_benches,
    std_benches
);

#[cfg(all(feature = "serde", not(feature = "std")))]
criterion_main!(
    core_benches,
    retry_benches,
    conversion_benches,
    context_benches,
    scaling_benches,
    pipeline_benches,
    validation_benches,
    real_world_benches,
    memory_benches,
    serde_benches
);

#[cfg(not(any(feature = "std", feature = "serde")))]
criterion_main!(
    core_benches,
    retry_benches,
    conversion_benches,
    context_benches,
    scaling_benches,
    pipeline_benches,
    validation_benches,
    real_world_benches,
    memory_benches
);
