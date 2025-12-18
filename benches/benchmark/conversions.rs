use crate::common::{configure_criterion, DomainError};
use criterion::{criterion_group, Criterion};
use error_rail::{ComposableError, ErrorContext};
use std::hint::black_box;

pub fn bench_error_conversions(c: &mut Criterion) {
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

criterion_group! {
    name = conversion_benches;
    config = configure_criterion();
    targets = bench_error_conversions,
}
