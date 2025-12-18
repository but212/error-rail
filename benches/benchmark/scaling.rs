use crate::common::{configure_criterion, validate_user_email, DomainError};
use criterion::{criterion_group, BenchmarkId, Criterion, Throughput};
use error_rail::validation::Validation;
use error_rail::{ComposableError, ErrorContext};
use std::hint::black_box;

pub fn bench_context_depth_scaling(c: &mut Criterion) {
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

pub fn bench_validation_batch_scaling(c: &mut Criterion) {
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

criterion_group! {
    name = scaling_benches;
    config = configure_criterion();
    targets =
        bench_context_depth_scaling,
        bench_validation_batch_scaling,
}
