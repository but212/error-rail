use crate::common::{configure_criterion, DomainError};
use criterion::{criterion_group, BenchmarkId, Criterion};
use error_rail::{ComposableError, ErrorContext, ErrorOps};
use std::hint::black_box;

pub fn bench_composable_error_creation(c: &mut Criterion) {
    c.bench_function("core/error_creation", |b| {
        b.iter(|| {
            black_box(
                ComposableError::new(DomainError::Database(
                    "Connection pool exhausted".to_string(),
                ))
                .with_context(ErrorContext::tag("database"))
                .with_context(ErrorContext::metadata("query", "SELECT * FROM users"))
                .with_context(ErrorContext::metadata("host", "db-primary-01.company.local"))
                .with_context(ErrorContext::metadata("retry_count", "3"))
                .set_code(503),
            )
        })
    });
}

pub fn bench_error_cloning_and_arc(c: &mut Criterion) {
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

pub fn bench_error_cloning_deep(c: &mut Criterion) {
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

pub fn bench_error_ops_recover(c: &mut Criterion) {
    c.bench_function("core/ops_recover", |b| {
        b.iter(|| black_box(Err::<i32, &str>("missing").recover(|_| Ok(42))))
    });
}

pub fn bench_error_ops_bimap(c: &mut Criterion) {
    c.bench_function("core/ops_bimap", |b| {
        b.iter(|| black_box(Ok::<i32, &str>(21).bimap_result(|x| x * 2, |e| e.to_uppercase())))
    });
}

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
