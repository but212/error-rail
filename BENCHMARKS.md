# error-rail Benchmarks

This document summarizes micro-benchmarks for `error-rail` on a local machine.

> **Note**: Absolute numbers depend on hardware and compiler settings. Focus on
> relative differences between configurations.

## A0. ComposableError Construction & Serialization

### Setup - A0

- Error type: `ComposableError<&'static str, u32>`
- Operations:
  - `composable_error_creation`: create error, add 2 contexts, set code.
  - `composable_error_serialization`: serialize the above error with `serde_json`.

### Results (approximate medians) - A0

- `composable_error_creation`        → ~0.35 µs/iter
- `composable_error_serialization`   → ~0.38 µs/iter

### Interpretation - A0

- Creating a moderately annotated `ComposableError` is sub-microsecond.
- JSON serialization of such an error is also sub-microsecond and only slightly
  slower than construction itself.

## A1. Lazy vs Eager Context

### Setup - A1

- Payload: `LargeStruct` with 100 `String` entries
- Pipeline: `ErrorPipeline::new(result)` with and without `with_context`
- Operations:
  - **lazy**: `with_context(context!("computed: {:?}", large_struct))`
  - **eager**: `with_context(format!("computed: {:?}", large_struct))`
  - **baseline**: no context

### Results (approximate medians) - A1

- **Success path (`Ok`)**
  - `context_baseline_success`  → ~3.87 ns/iter
  - `context_lazy_success`      → ~3.92 ns/iter
  - `context_eager_success`     → ~6.92 µs/iter

- **Error path (`Err`)**
  - `context_baseline_error`    → ~95.7 ns/iter
  - `context_lazy_error`        → ~7.25 µs/iter
  - `context_eager_error`       → ~7.20 µs/iter

### Interpretation - A1

- On the **success path**, lazy context is effectively free:
  - Baseline vs lazy differ by only a few percent, within noise.
  - Eager formatting is ~1,800× slower for the same payload.
- On the **error path**, lazy and eager are similar:
  - Both must call `format!`, so they have comparable cost.

## A2. Context Depth Scaling

### Setup - A2

- Start from `ComposableError::new("failure")`.
- Chain `with_context(ErrorContext::new("context"))` `N` times.

### Results (approximate medians) - A2

- `context_depth_1`   → ~94 ns/iter
- `context_depth_3`   → ~245 ns/iter
- `context_depth_10`  → ~0.99 µs/iter
- `context_depth_30`  → ~6.00 µs/iter

### Interpretation - A2

- Cost scales roughly linearly with context depth.
- Even at 30 frames of context, total overhead is only a few microseconds.

## B1. ErrorPipeline vs Raw Result

### Setup - B1

- Simple 3-step pipeline: `parse -> validate -> transform`
- Variants:
  - **Pipeline**: `ErrorPipeline::new(parse(..))` + `with_context(context!(..))` + `and_then`
  - **Result + context**: pure `Result` chain, then wrap error in `ComposableError` with context
  - **Baseline**: pure `Result` chain, no context or `ComposableError`

### Results (approximate medians) - B1

- **Success path**
  - `pipeline_success`              → ~2.83 ns/iter
  - `result_with_context_success`   → ~0.51 ns/iter
  - `result_baseline_success`       → ~0.51 ns/iter

- **Error path**
  - `pipeline_error`                → ~100.7 ns/iter
  - `result_with_context_error`     → ~70.6 ns/iter
  - `result_baseline_error`         → ~0.51 ns/iter

### Interpretation - B1

- Wrapping errors in `ComposableError` + context on top of a `Result` chain is
  **almost zero overhead** relative to a plain `Result`.
- `ErrorPipeline` trades a small amount of performance for ergonomics:
  - Success path: a few extra ns/iter vs raw `Result`.
  - Error path: about 1.4–1.5× the cost of a manual `Result` + context chain.
- In realistic code dominated by IO or heavy work, these differences are
  typically negligible.

## C1. Validation Collect vs Manual Collect (N = 10)

### Setup - C1

- Inputs: 10 fields (`0..10`)
- Variants:
  - **Validation collect**: `fields.iter().map(..).collect::<Validation<_, Vec<_>>>()`
  - **Manual collect**: loop with `values.push(..)` / `errors.push(..)`
- Scenarios:
  - **All valid**
  - **Half invalid** (odds fail)
  - **All invalid**

### Results (approximate medians) - C1

- **All valid**
  - `validation_collect_all_valid`   → ~232 ns/iter
  - `manual_collect_all_valid`       → ~3.09 µs/iter

- **Half invalid**
  - `validation_collect_half_invalid`→ ~234 ns/iter
  - `manual_collect_half_invalid`    → ~315 ns/iter

- **All invalid**
  - `validation_collect_all_invalid` → ~173 ns/iter
  - `manual_collect_all_invalid`     → ~248 ns/iter

### Interpretation - C1

- For small N (10):
  - `Validation::collect()` is as fast as, and often significantly faster than,
    manual error aggregation.
  - When many fields are invalid, `Validation` is ~25–45% faster than a
    handwritten `Vec`-based loop.
  - Even when all fields are valid, `Validation` remains competitive despite
    additional abstraction.

## D1. ErrorOps: `recover` and `bimap_result`

### Setup - D1

- `error_ops_recover`: `Err::<i32, &str>("missing").recover(|_| Ok(42))`
- `error_ops_bimap`: `Ok::<i32, &str>(21).bimap_result(|x| x * 2, |e| e.to_uppercase())`

### Results (approximate medians) - D1

- `error_ops_recover` → ~1.29 ns/iter
- `error_ops_bimap`   → ~1.55 ns/iter

### Interpretation - D1

- The core `ErrorOps` helpers add sub-nanosecond to low-nanosecond overhead and
  are effectively free compared to real-world workloads.
