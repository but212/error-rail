use crate::common::{
    configure_criterion, validate_user_age, validate_user_email, validate_user_name, DomainError,
};
use criterion::{criterion_group, Criterion};
use error_rail::validation::Validation;
use std::hint::black_box;

pub fn bench_validation_collect_realistic(c: &mut Criterion) {
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

criterion_group! {
    name = validation_benches;
    config = configure_criterion();
    targets = bench_validation_collect_realistic,
}
