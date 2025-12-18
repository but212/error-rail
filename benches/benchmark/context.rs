use crate::common::{configure_criterion, realistic_user_data, DomainError, UserData};
use criterion::{criterion_group, Criterion};
use error_rail::{context, ErrorPipeline};
use std::hint::black_box;

pub fn bench_context_lazy_vs_eager_success(c: &mut Criterion) {
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
        let user = UserData::new(1); // Pre-allocate to match other benchmarks
        b.iter(|| {
            let result: Result<UserData, &str> = Ok(user.clone());
            let _ = black_box(result);
        })
    });

    group.finish();
}

pub fn bench_context_lazy_vs_eager_error(c: &mut Criterion) {
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
        let error = DomainError::Validation("Email format invalid".to_string()); // Pre-allocate
        b.iter(|| {
            let result: Result<UserData, DomainError> = Err(error.clone());
            let _ = black_box(result);
        })
    });

    group.finish();
}

criterion_group! {
    name = context_benches;
    config = configure_criterion();
    targets =
        bench_context_lazy_vs_eager_success,
        bench_context_lazy_vs_eager_error,
}
