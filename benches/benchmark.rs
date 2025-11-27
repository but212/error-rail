use criterion::{criterion_group, criterion_main, Criterion};
use error_rail::traits::ErrorOps;
use error_rail::validation::Validation;
use error_rail::{backtrace, context, ComposableError, ErrorContext, ErrorPipeline};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{hint::black_box, sync::OnceLock};

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

// Simulate realistic error propagation through multiple layers
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

// 1. Construction benchmark - realistic error with domain context
fn bench_composable_error_creation(c: &mut Criterion) {
    c.bench_function("composable_error_creation", |b| {
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

#[cfg(feature = "serde")]
// 2. Serialization benchmark - realistic complex error
fn bench_composable_error_serialization(c: &mut Criterion) {
    let err = ComposableError::new(DomainError::Network("API rate limit exceeded".to_string()))
        .with_context(ErrorContext::tag("external_api"))
        .with_context(ErrorContext::metadata("endpoint", "/api/v2/users"))
        .with_context(ErrorContext::metadata("retry_after", "60"))
        .with_context(ErrorContext::metadata("quota_limit", "1000"))
        .set_code(429);
    c.bench_function("composable_error_serialization", |b| {
        b.iter(|| black_box(serde_json::to_string(&err).unwrap()))
    });
}

// 3. ErrorOps benchmark (recover & bimap)
fn bench_error_ops_recover(c: &mut Criterion) {
    c.bench_function("error_ops_recover", |b| {
        b.iter(|| black_box(Err::<i32, &str>("missing").recover(|_| Ok(42))))
    });
}

fn bench_error_ops_bimap(c: &mut Criterion) {
    c.bench_function("error_ops_bimap", |b| {
        b.iter(|| black_box(Ok::<i32, &str>(21).bimap_result(|x| x * 2, |e| e.to_uppercase())))
    });
}

fn bench_context_lazy_vs_eager_success(c: &mut Criterion) {
    let users = realistic_user_data();

    c.bench_function("context_lazy_success", |b| {
        b.iter(|| {
            let user = black_box(&users[0]);
            let result: Result<UserData, &str> = Ok(user.clone());
            let _ = ErrorPipeline::new(result)
                .with_context(context!("user_data: {:?}", user))
                .finish_boxed();
        })
    });

    c.bench_function("context_eager_success", |b| {
        b.iter(|| {
            let user = black_box(&users[0]);
            let result: Result<UserData, &str> = Ok(user.clone());
            let message = format!("user_data: {:?}", user);
            let _ = ErrorPipeline::new(result).with_context(message).finish();
        })
    });

    c.bench_function("context_baseline_success", |b| {
        b.iter(|| {
            let result: Result<UserData, &str> = Ok(UserData::new(1));
            let _ = ErrorPipeline::new(result).finish();
        })
    });
}

fn bench_context_lazy_vs_eager_error(c: &mut Criterion) {
    let users = realistic_user_data();

    c.bench_function("context_lazy_error", |b| {
        b.iter(|| {
            let user = black_box(&users[0]);
            let result: Result<UserData, DomainError> =
                Err(DomainError::Validation("Email format invalid".to_string()));
            let _ = ErrorPipeline::new(result)
                .with_context(context!("failed_user: {:?}", user))
                .finish_boxed();
        })
    });

    c.bench_function("context_eager_error", |b| {
        b.iter(|| {
            let user = black_box(&users[0]);
            let result: Result<UserData, DomainError> =
                Err(DomainError::Validation("Email format invalid".to_string()));
            let message = format!("failed_user: {:?}", user);
            let _ = ErrorPipeline::new(result).with_context(message).finish();
        })
    });

    c.bench_function("context_baseline_error", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> =
                Err(DomainError::Validation("Email format invalid".to_string()));
            let _ = ErrorPipeline::new(result).finish();
        })
    });
}

fn bench_backtrace_lazy_success(c: &mut Criterion) {
    c.bench_function("backtrace_lazy_success", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> = Ok(UserData::new(42));
            let _ = ErrorPipeline::new(result)
                .with_context(backtrace!())
                .finish();
        })
    });
}

fn bench_backtrace_lazy_error(c: &mut Criterion) {
    c.bench_function("backtrace_lazy_error", |b| {
        b.iter(|| {
            let result: Result<UserData, DomainError> =
                Err(DomainError::Network("Connection refused".to_string()));
            let _ = ErrorPipeline::new(result)
                .with_context(backtrace!())
                .finish();
        })
    });
}

fn build_realistic_error_with_depth(depth: usize) -> ComposableError<DomainError> {
    let mut err = ComposableError::new(DomainError::Database("Query failed".to_string()))
        .with_context(ErrorContext::tag("database"))
        .with_context(ErrorContext::metadata("query_id", "12345"));

    for i in 0..depth {
        err = err
            .with_context(ErrorContext::new(format!("layer_{}", i)))
            .with_context(ErrorContext::metadata("depth", i.to_string()));
    }
    err
}

fn bench_context_depth(c: &mut Criterion) {
    c.bench_function("context_depth_1", |b| {
        b.iter(|| {
            let err = build_realistic_error_with_depth(1);
            let _ = black_box(err);
        })
    });

    c.bench_function("context_depth_3", |b| {
        b.iter(|| {
            let err = build_realistic_error_with_depth(3);
            let _ = black_box(err);
        })
    });

    c.bench_function("context_depth_10", |b| {
        b.iter(|| {
            let err = build_realistic_error_with_depth(10);
            let _ = black_box(err);
        })
    });

    c.bench_function("context_depth_30", |b| {
        b.iter(|| {
            let err = build_realistic_error_with_depth(30);
            let _ = black_box(err);
        })
    });
}

// Realistic service layer simulation with mixed success/error ratios
fn realistic_user_service(user_id: u64) -> Result<UserData, ComposableError<DomainError>> {
    ErrorPipeline::new(simulate_db_query(user_id))
        .with_context(context!("Fetching user {} from database", user_id))
        .and_then(|user| simulate_validation(user))
        .with_context(context!("Validating user data for user {}", user_id))
        .and_then(|user| simulate_auth_check(user))
        .with_context(context!("Authentication check for user {}", user_id))
        .finish()
}

fn bench_pipeline_vs_result_success(c: &mut Criterion) {
    c.bench_function("pipeline_success", |b| {
        b.iter(|| {
            let result = realistic_user_service(42);
            let _ = black_box(result).is_ok();
        })
    });

    c.bench_function("result_with_context_success", |b| {
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

    c.bench_function("result_baseline_success", |b| {
        b.iter(|| {
            let result = simulate_db_query(42)
                .and_then(|user| simulate_validation(user))
                .and_then(|user| simulate_auth_check(user));
            let _ = black_box(result).is_ok();
        })
    });
}

fn bench_pipeline_vs_result_error(c: &mut Criterion) {
    c.bench_function("pipeline_error", |b| {
        b.iter(|| {
            let result = realistic_user_service(100); // Will fail at DB layer
            let _ = black_box(result).is_ok();
        })
    });

    c.bench_function("result_with_context_error", |b| {
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

    c.bench_function("result_baseline_error", |b| {
        b.iter(|| {
            let result = simulate_db_query(100)
                .and_then(|user| simulate_validation(user))
                .and_then(|user| simulate_auth_check(user));
            let _ = black_box(result).is_ok();
        })
    });
}

// Realistic validation with heterogeneous error types
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

    c.bench_function("validation_collect_realistic_mixed", |b| {
        b.iter(|| {
            let result: Validation<DomainError, Vec<String>> = test_emails
                .iter()
                .map(|email| validate_user_email(email))
                .collect();
            black_box(&result);
        })
    });

    c.bench_function("manual_collect_realistic_mixed", |b| {
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

    // Test with different validation types
    c.bench_function("validation_collect_heterogeneous", |b| {
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
}

// Additional realistic benchmarks for production scenarios

// Benchmark error cloning and Arc wrapping (common in async/concurrent contexts)
fn bench_error_cloning_and_arc(c: &mut Criterion) {
    let err = ComposableError::new(DomainError::Network("Service unavailable".to_string()))
        .with_context(ErrorContext::tag("external_service"))
        .set_code(503);

    c.bench_function("error_clone", |b| {
        b.iter(|| {
            let cloned = black_box(err.clone());
            let _ = black_box(cloned);
        })
    });

    c.bench_function("error_arc_wrap", |b| {
        b.iter(|| {
            let arc_err = black_box(std::sync::Arc::new(err.clone()));
            black_box(arc_err);
        })
    });
}

// Benchmark mixed success/error ratios (95% success, 5% error - typical production)
fn bench_mixed_success_error_ratios(c: &mut Criterion) {
    c.bench_function("mixed_95percent_success", |b| {
        b.iter(|| {
            let results: Vec<Result<UserData, ComposableError<DomainError>>> =
                (0..100).map(|i| realistic_user_service(i)).collect();
            let success_count = results.iter().filter(|r| r.is_ok()).count();
            black_box(success_count);
        })
    });

    c.bench_function("mixed_50percent_success", |b| {
        b.iter(|| {
            let results: Vec<Result<UserData, ComposableError<DomainError>>> = (0..100)
                .map(|i| realistic_user_service(i * 2)) // Higher failure rate
                .collect();
            let success_count = results.iter().filter(|r| r.is_ok()).count();
            black_box(success_count);
        })
    });
}

// Benchmark error type conversion scenarios
fn bench_error_type_conversions(c: &mut Criterion) {
    c.bench_function("std_io_error_conversion", |b| {
        b.iter(|| {
            let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "config file not found");
            let converted = ComposableError::new(DomainError::Database(io_err.to_string()))
                .with_context(ErrorContext::tag("file_system"))
                .with_context(ErrorContext::metadata("path", "/etc/app/config.toml"));
            let _ = black_box(converted);
        })
    });

    c.bench_function("serde_error_conversion", |b| {
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
}

#[cfg(not(feature = "serde"))]
criterion_group!(
    benches,
    bench_composable_error_creation,
    bench_error_ops_recover,
    bench_error_ops_bimap,
    bench_context_lazy_vs_eager_success,
    bench_context_lazy_vs_eager_error,
    bench_backtrace_lazy_success,
    bench_backtrace_lazy_error,
    bench_context_depth,
    bench_pipeline_vs_result_success,
    bench_pipeline_vs_result_error,
    bench_validation_collect_realistic,
    bench_error_cloning_and_arc,
    bench_mixed_success_error_ratios,
    bench_error_type_conversions
);

#[cfg(feature = "serde")]
criterion_group!(
    benches,
    bench_composable_error_creation,
    bench_composable_error_serialization,
    bench_error_ops_recover,
    bench_error_ops_bimap,
    bench_context_lazy_vs_eager_success,
    bench_context_lazy_vs_eager_error,
    bench_backtrace_lazy_success,
    bench_backtrace_lazy_error,
    bench_context_depth,
    bench_pipeline_vs_result_success,
    bench_pipeline_vs_result_error,
    bench_validation_collect_realistic,
    bench_error_cloning_and_arc,
    bench_mixed_success_error_ratios,
    bench_error_type_conversions
);
criterion_main!(benches);
