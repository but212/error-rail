// benches/benchmark.rs
use criterion::{criterion_group, criterion_main, Criterion};
use error_rail::traits::ErrorOps;
use error_rail::validation::Validation;
use error_rail::{backtrace, context, ComposableError, ErrorContext, ErrorPipeline};
use std::{hint::black_box, sync::OnceLock};

#[derive(Debug)]
#[allow(dead_code)]
struct LargeStruct {
    data: Vec<String>,
}

impl LargeStruct {
    fn new() -> Self {
        let data = (0..100).map(|i| format!("value-{i}")).collect();
        Self { data }
    }
}

fn large_struct() -> &'static LargeStruct {
    static INSTANCE: OnceLock<LargeStruct> = OnceLock::new();
    INSTANCE.get_or_init(LargeStruct::new)
}

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

fn bench_context_lazy_vs_eager_success(c: &mut Criterion) {
    c.bench_function("context_lazy_success", |b| {
        b.iter(|| {
            let large = black_box(large_struct());
            let result: Result<(), &str> = Ok(());
            let _ = ErrorPipeline::new(result)
                .with_context(context!("computed: {:?}", large))
                .finish_boxed();
        })
    });

    c.bench_function("context_eager_success", |b| {
        b.iter(|| {
            let large = black_box(large_struct());
            let result: Result<(), &str> = Ok(());
            let message = format!("computed: {:?}", large);
            let _ = ErrorPipeline::new(result).with_context(message).finish();
        })
    });

    c.bench_function("context_baseline_success", |b| {
        b.iter(|| {
            let result: Result<(), &str> = Ok(());
            let _ = ErrorPipeline::new(result).finish();
        })
    });
}

fn bench_context_lazy_vs_eager_error(c: &mut Criterion) {
    c.bench_function("context_lazy_error", |b| {
        b.iter(|| {
            let large = black_box(large_struct());
            let result: Result<(), &str> = Err("fail");
            let _ = ErrorPipeline::new(result)
                .with_context(context!("computed: {:?}", large))
                .finish_boxed();
        })
    });

    c.bench_function("context_eager_error", |b| {
        b.iter(|| {
            let large = black_box(large_struct());
            let result: Result<(), &str> = Err("fail");
            let message = format!("computed: {:?}", large);
            let _ = ErrorPipeline::new(result).with_context(message).finish();
        })
    });

    c.bench_function("context_baseline_error", |b| {
        b.iter(|| {
            let result: Result<(), &str> = Err("fail");
            let _ = ErrorPipeline::new(result).finish();
        })
    });
}

fn bench_backtrace_lazy_success(c: &mut Criterion) {
    c.bench_function("backtrace_lazy_success", |b| {
        b.iter(|| {
            let result: Result<(), &str> = Ok(());
            let _ = ErrorPipeline::new(result)
                .with_context(backtrace!())
                .finish();
        })
    });
}

fn bench_backtrace_lazy_error(c: &mut Criterion) {
    c.bench_function("backtrace_lazy_error", |b| {
        b.iter(|| {
            let result: Result<(), &str> = Err("fail");
            let _ = ErrorPipeline::new(result)
                .with_context(backtrace!())
                .finish();
        })
    });
}

fn build_error_with_depth(depth: usize) -> ComposableError<&'static str> {
    let mut err = ComposableError::new("failure");
    for _ in 0..depth {
        err = err.with_context(ErrorContext::new("context"));
    }
    err
}

fn bench_context_depth(c: &mut Criterion) {
    c.bench_function("context_depth_1", |b| {
        b.iter(|| {
            let err = build_error_with_depth(1);
            black_box(err);
        })
    });

    c.bench_function("context_depth_3", |b| {
        b.iter(|| {
            let err = build_error_with_depth(3);
            black_box(err);
        })
    });

    c.bench_function("context_depth_10", |b| {
        b.iter(|| {
            let err = build_error_with_depth(10);
            black_box(err);
        })
    });

    c.bench_function("context_depth_30", |b| {
        b.iter(|| {
            let err = build_error_with_depth(30);
            black_box(err);
        })
    });
}

fn parse_ok() -> Result<i32, &'static str> {
    Ok(42)
}

fn parse_err() -> Result<i32, &'static str> {
    Err("parse error")
}

fn validate(v: i32) -> Result<i32, &'static str> {
    if v > 0 {
        Ok(v)
    } else {
        Err("validate error")
    }
}

fn transform(v: i32) -> Result<i32, &'static str> {
    Ok(v * 2)
}

fn bench_pipeline_vs_result_success(c: &mut Criterion) {
    c.bench_function("pipeline_success", |b| {
        b.iter(|| {
            let result = ErrorPipeline::new(parse_ok())
                .with_context(context!("parsing"))
                .and_then(|v| validate(v))
                .and_then(|v| transform(v))
                .finish();
            let _ = black_box(result).is_ok();
        })
    });

    c.bench_function("result_with_context_success", |b| {
        b.iter(|| {
            let result = parse_ok()
                .and_then(|v| validate(v))
                .and_then(|v| transform(v));
            let result = result.map_err(|e| {
                ComposableError::<&'static str>::new(e).with_context(ErrorContext::new("parsing"))
            });
            let _ = black_box(result).is_ok();
        })
    });

    c.bench_function("result_baseline_success", |b| {
        b.iter(|| {
            let result = parse_ok()
                .and_then(|v| validate(v))
                .and_then(|v| transform(v));
            let _ = black_box(result).is_ok();
        })
    });
}

fn bench_pipeline_vs_result_error(c: &mut Criterion) {
    c.bench_function("pipeline_error", |b| {
        b.iter(|| {
            let result = ErrorPipeline::new(parse_err())
                .with_context(context!("parsing"))
                .and_then(|v| validate(v))
                .and_then(|v| transform(v))
                .finish();
            let _ = black_box(result).is_ok();
        })
    });

    c.bench_function("result_with_context_error", |b| {
        b.iter(|| {
            let result = parse_err()
                .and_then(|v| validate(v))
                .and_then(|v| transform(v));
            let result = result.map_err(|e| {
                ComposableError::<&'static str>::new(e).with_context(ErrorContext::new("parsing"))
            });
            let _ = black_box(result).is_ok();
        })
    });

    c.bench_function("result_baseline_error", |b| {
        b.iter(|| {
            let result = parse_err()
                .and_then(|v| validate(v))
                .and_then(|v| transform(v));
            let _ = black_box(result).is_ok();
        })
    });
}

fn bench_validation_collect_small_n(c: &mut Criterion) {
    let fields: Vec<i32> = (0..10).collect();

    c.bench_function("validation_collect_all_valid", |b| {
        b.iter(|| {
            let result: Validation<&'static str, Vec<i32>> =
                fields.iter().map(|&i| Ok::<i32, &'static str>(i)).collect();
            black_box(&result);
        })
    });

    c.bench_function("manual_collect_all_valid", |b| {
        b.iter(|| {
            let mut values = Vec::new();
            let mut errors = Vec::new();
            for &i in &fields {
                let r: Result<i32, &'static str> = Ok(i);
                match r {
                    Ok(v) => values.push(v),
                    Err(e) => errors.push(e),
                }
            }
            black_box((&values, &errors));
        })
    });

    c.bench_function("validation_collect_half_invalid", |b| {
        b.iter(|| {
            let result: Validation<&'static str, Vec<i32>> = fields
                .iter()
                .map(|&i| {
                    if i % 2 == 0 {
                        Ok::<i32, &'static str>(i)
                    } else {
                        Err("invalid")
                    }
                })
                .collect();
            black_box(&result);
        })
    });

    c.bench_function("manual_collect_half_invalid", |b| {
        b.iter(|| {
            let mut values = Vec::new();
            let mut errors = Vec::new();
            for &i in &fields {
                let r: Result<i32, &'static str> = if i % 2 == 0 { Ok(i) } else { Err("invalid") };
                match r {
                    Ok(v) => values.push(v),
                    Err(e) => errors.push(e),
                }
            }
            black_box((&values, &errors));
        })
    });

    c.bench_function("validation_collect_all_invalid", |b| {
        b.iter(|| {
            let result: Validation<&'static str, Vec<i32>> = fields
                .iter()
                .map(|_| Err::<i32, &'static str>("invalid"))
                .collect();
            black_box(&result);
        })
    });

    c.bench_function("manual_collect_all_invalid", |b| {
        b.iter(|| {
            let mut values = Vec::new();
            let mut errors = Vec::new();
            for _ in &fields {
                let r: Result<i32, &'static str> = Err("invalid");
                match r {
                    Ok(v) => values.push(v),
                    Err(e) => errors.push(e),
                }
            }
            black_box((&values, &errors));
        })
    });
}

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
    bench_validation_collect_small_n
);
criterion_main!(benches);
