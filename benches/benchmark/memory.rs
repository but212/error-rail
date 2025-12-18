use crate::common::{configure_criterion, DomainError};
use criterion::{criterion_group, Criterion};
use error_rail::{ComposableError, ErrorContext};
use std::hint::black_box;

pub fn bench_memory_allocation(c: &mut Criterion) {
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

criterion_group! {
    name = memory_benches;
    config = configure_criterion();
    targets = bench_memory_allocation,
}
