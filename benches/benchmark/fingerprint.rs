use crate::common::{configure_criterion, DomainError};
use criterion::{criterion_group, BenchmarkId, Criterion};
use error_rail::{ComposableError, ErrorContext};
use std::hint::black_box;

/// Benchmarks for fingerprint generation
pub fn bench_fingerprint_basic(c: &mut Criterion) {
    let mut group = c.benchmark_group("fingerprint/basic");

    let simple_err = ComposableError::new(DomainError::Database("timeout".to_string()))
        .with_context(ErrorContext::tag("db"))
        .set_code(504);

    let complex_err = ComposableError::new(DomainError::Network("connection refused".to_string()))
        .with_context(ErrorContext::tag("network"))
        .with_context(ErrorContext::tag("external_api"))
        .with_context(ErrorContext::metadata("host", "api.example.com"))
        .with_context(ErrorContext::metadata("port", "443"))
        .with_context(ErrorContext::metadata("retry_count", "3"))
        .set_code(503);

    // fingerprint() - returns u64
    group.bench_function("simple_u64", |b| {
        b.iter(|| {
            let fp = simple_err.fingerprint();
            let _ = black_box(fp);
        })
    });

    group.bench_function("complex_u64", |b| {
        b.iter(|| {
            let fp = complex_err.fingerprint();
            let _ = black_box(fp);
        })
    });

    // fingerprint_hex() - returns String
    group.bench_function("simple_hex", |b| {
        b.iter(|| {
            let fp = simple_err.fingerprint_hex();
            let _ = black_box(fp);
        })
    });

    group.bench_function("complex_hex", |b| {
        b.iter(|| {
            let fp = complex_err.fingerprint_hex();
            let _ = black_box(fp);
        })
    });

    group.finish();
}

/// Benchmarks for FingerprintConfig customization
pub fn bench_fingerprint_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("fingerprint/config");

    let err = ComposableError::new(DomainError::Validation("invalid email".to_string()))
        .with_context(ErrorContext::tag("validation"))
        .with_context(ErrorContext::metadata("field", "email"))
        .with_context(ErrorContext::metadata("value", "not-an-email"))
        .set_code(400);

    // Default config
    group.bench_function("default", |b| {
        b.iter(|| {
            let fp = err.fingerprint_config().compute();
            let _ = black_box(fp);
        })
    });

    // Exclude message (broader grouping)
    group.bench_function("exclude_message", |b| {
        b.iter(|| {
            let fp = err.fingerprint_config().include_message(false).compute();
            let _ = black_box(fp);
        })
    });

    // Include metadata
    group.bench_function("include_metadata", |b| {
        b.iter(|| {
            let fp = err.fingerprint_config().include_metadata(true).compute();
            let _ = black_box(fp);
        })
    });

    // Minimal (tags + code only)
    group.bench_function("minimal", |b| {
        b.iter(|| {
            let fp = err
                .fingerprint_config()
                .include_message(false)
                .include_metadata(false)
                .compute();
            let _ = black_box(fp);
        })
    });

    // compute_hex via config
    group.bench_function("config_hex", |b| {
        b.iter(|| {
            let fp = err.fingerprint_config().compute_hex();
            let _ = black_box(fp);
        })
    });

    group.finish();
}

/// Benchmarks for fingerprint consistency check
pub fn bench_fingerprint_consistency(c: &mut Criterion) {
    let mut group = c.benchmark_group("fingerprint/consistency");

    // Create two errors with same structure
    let err1 = ComposableError::new(DomainError::Database("timeout".to_string()))
        .with_context(ErrorContext::tag("db"))
        .set_code(504);

    let err2 = ComposableError::new(DomainError::Database("timeout".to_string()))
        .with_context(ErrorContext::tag("db"))
        .set_code(504);

    // Compare fingerprints
    group.bench_function("compare_equal", |b| {
        b.iter(|| {
            let fp1 = err1.fingerprint();
            let fp2 = err2.fingerprint();
            let _ = black_box(fp1 == fp2);
        })
    });

    group.finish();
}

/// Benchmarks for fingerprint scaling with context depth
pub fn bench_fingerprint_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("fingerprint/scaling");

    for depth in [1, 5, 10, 20] {
        let mut err = ComposableError::new(DomainError::Network("error".to_string()));
        for i in 0..depth {
            err = err.with_context(ErrorContext::tag(format!("tag_{}", i)));
        }
        err = err.set_code(500);

        group.bench_with_input(BenchmarkId::from_parameter(depth), &err, |b, err| {
            b.iter(|| black_box(err.fingerprint()))
        });
    }

    group.finish();
}

criterion_group! {
    name = fingerprint_benches;
    config = configure_criterion();
    targets =
        bench_fingerprint_basic,
        bench_fingerprint_config,
        bench_fingerprint_consistency,
        bench_fingerprint_scaling,
}
