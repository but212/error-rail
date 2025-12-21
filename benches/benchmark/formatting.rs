use crate::common::{configure_criterion, DomainError};
use criterion::{criterion_group, BenchmarkId, Criterion};
use error_rail::{ComposableError, ErrorContext};
use std::hint::black_box;

/// Benchmarks for error formatting operations
pub fn bench_error_chain_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("formatting/error_chain");

    // Create error with varying context depths
    let shallow_err = ComposableError::new(DomainError::Database("query failed".to_string()))
        .with_context(ErrorContext::tag("db"))
        .set_code(500);

    let deep_err = ComposableError::new(DomainError::Network("connection refused".to_string()))
        .with_context(ErrorContext::tag("network"))
        .with_context(ErrorContext::metadata("host", "api.example.com"))
        .with_context(ErrorContext::metadata("port", "443"))
        .with_context(ErrorContext::metadata("retry_count", "3"))
        .with_context("establishing connection")
        .set_code(503);

    // Basic error_chain()
    group.bench_function("shallow", |b| {
        b.iter(|| {
            let chain = shallow_err.error_chain();
            let _ = black_box(chain);
        })
    });

    group.bench_function("deep", |b| {
        b.iter(|| {
            let chain = deep_err.error_chain();
            let _ = black_box(chain);
        })
    });

    group.finish();
}

/// Benchmarks for ErrorFormatBuilder options
pub fn bench_format_builder(c: &mut Criterion) {
    let mut group = c.benchmark_group("formatting/builder");

    let err = ComposableError::new(DomainError::Validation("invalid input".to_string()))
        .with_context(ErrorContext::tag("validation"))
        .with_context(ErrorContext::metadata("field", "email"))
        .with_context("user registration")
        .set_code(400);

    // Default formatting
    group.bench_function("default", |b| {
        b.iter(|| {
            let formatted = err.fmt().to_string();
            let _ = black_box(formatted);
        })
    });

    // Custom separator
    group.bench_function("custom_separator", |b| {
        b.iter(|| {
            let formatted = err.fmt().with_separator(" | ").to_string();
            let _ = black_box(formatted);
        })
    });

    // Hide code
    group.bench_function("hide_code", |b| {
        b.iter(|| {
            let formatted = err.fmt().show_code(false).to_string();
            let _ = black_box(formatted);
        })
    });

    // Cascaded format
    group.bench_function("cascaded", |b| {
        b.iter(|| {
            let formatted = err.fmt().cascaded().to_string();
            let _ = black_box(formatted);
        })
    });

    // Pretty format
    group.bench_function("pretty", |b| {
        b.iter(|| {
            let formatted = err.fmt().pretty().to_string();
            let _ = black_box(formatted);
        })
    });

    group.finish();
}

/// Benchmarks for scaling with context depth
pub fn bench_format_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("formatting/scaling");

    for depth in [1, 5, 10, 20] {
        let mut err = ComposableError::new(DomainError::Database("error".to_string()));
        for i in 0..depth {
            err = err
                .with_context(ErrorContext::new(format!("context_{}", i)))
                .with_context(ErrorContext::metadata("depth", i.to_string()));
        }
        err = err.set_code(500);

        group.bench_with_input(BenchmarkId::from_parameter(depth), &err, |b, err| {
            b.iter(|| black_box(err.error_chain()))
        });
    }

    group.finish();
}

/// Benchmarks for Display trait implementation
pub fn bench_display_trait(c: &mut Criterion) {
    let mut group = c.benchmark_group("formatting/display");

    let err = ComposableError::new(DomainError::Authentication("token expired".to_string()))
        .with_context(ErrorContext::tag("auth"))
        .set_code(401);

    // Normal Display
    group.bench_function("display_normal", |b| {
        b.iter(|| {
            let s = format!("{}", err);
            let _ = black_box(s);
        })
    });

    // Alternate Display (cascaded)
    group.bench_function("display_alternate", |b| {
        b.iter(|| {
            let s = format!("{:#}", err);
            let _ = black_box(s);
        })
    });

    group.finish();
}

criterion_group! {
    name = formatting_benches;
    config = configure_criterion();
    targets =
        bench_error_chain_formatting,
        bench_format_builder,
        bench_format_scaling,
        bench_display_trait,
}
