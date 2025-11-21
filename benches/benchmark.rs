// benches/benchmark.rs
use criterion::{criterion_group, criterion_main, Criterion};
use error_rail::traits::ErrorOps;
use error_rail::{ComposableError, ErrorContext};
use std::hint::black_box;

// 1. Construction benchmark
fn bench_composable_error_creation(c: &mut Criterion) {
    c.bench_function("composable_error_creation", |b| {
        b.iter(|| {
            black_box(
                ComposableError::new("failure")
                    .with_context(ErrorContext::tag("db"))
                    .with_context(ErrorContext::metadata("host", "db-primary-01"))
                    .set_code(503),
            )
        })
    });
}

// 2. Serialization benchmark
fn bench_composable_error_serialization(c: &mut Criterion) {
    let err = ComposableError::new("failure")
        .with_context(ErrorContext::tag("db"))
        .with_context(ErrorContext::metadata("host", "db-primary-01"))
        .set_code(503);
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

criterion_group!(
    benches,
    bench_composable_error_creation,
    bench_composable_error_serialization,
    bench_error_ops_recover,
    bench_error_ops_bimap
);
criterion_main!(benches);
