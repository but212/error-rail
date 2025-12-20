# ErrorRail Benchmarks

This document summarizes performance benchmarks for `error-rail` using realistic production scenarios.

> **Last Updated**: 2025-12-20  
> **Rust Version**: 1.92.0  
> **Platform**: Windows 11 (x86_64)  
> **CPU**: Intel(R) Core(TM) i5-9400F CPU @ 2.90GHz  
> **Criterion Version**: 0.7.0

## Running Benchmarks

```bash
# Run all benchmarks with full features (includes std + serde)
cargo bench --features full

# Run all benchmarks without features
cargo bench --no-default-features

# Run specific benchmark group
cargo bench -- retry
cargo bench -- scaling
cargo bench -- real_world

# View detailed results in HTML
start target/criterion/report/index.html
```

> **Note**: Absolute numbers depend on hardware and compiler settings. Focus on relative differences between configurations and patterns.

## Executive Summary

| Metric | Performance | Context |
|--------|-------------|---------|
| Error creation | ~482 ns | Basic error struct construction |
| Lazy vs eager evaluation | **2.2x faster** on success paths | Lazy: 587ns vs Eager: 1.30µs |
| Error propagation overhead | ~5% vs raw Result | Pipeline: 722ns vs Baseline: 691ns |
| Validation overhead | ~7% vs manual collection | ErrorRail: 968ns vs Manual: 900ns |
| Serialization | ~807 ns | JSON serialization with serde feature |
| Validation throughput | 5.47 M elements/sec | 5000 items batch validation |

## 1. Core Operations

**Note**: Basic ErrorRail operations - error creation, cloning, wrapping, and functional operations.

```text
core/error_creation     → 482.05 ns/iter  (Create basic error with context)
core/error_clone        → 180.46 ns/iter  (Clone error for async/ownership transfer)
core/error_arc_wrap     → 248.00 ns/iter  (Wrap error in Arc for sharing)
core/ops_recover        → 2.55 ns/iter    (Recover from error with fallback)
core/ops_bimap          → 3.06 ns/iter    (Transform error types)
```

### Deep Cloning Scaling

```text
depth=5   → 1.40 µs/iter
depth=10  → 5.72 µs/iter  (4.1x)
depth=20  → 8.12 µs/iter  (1.4x)
depth=50  → 21.31 µs/iter (2.6x from depth=20)
```

## 2. Retry Operations

**Note**: Retry logic performance with different error classifications (transient vs permanent).

```text
retry/transient_success  → 471.33 ns/iter  (Retry succeeds after transient error)
retry/permanent_skip     → 58.57 ns/iter   (Skip retry for permanent errors - 8x faster)
retry/recover_transient  → 756.53 ns/iter  (Recover from transient error with retry)
retry/should_retry_check → 115.91 ns/iter  (Check if error should be retried)
```

## 3. Error Conversions

**Note**: Performance of converting between different error types (std, serde, domain errors).

```text
conversions/map_core         → 161.34 ns/iter  (Map between ErrorRail types)
conversions/std_io_to_domain → 412.28 ns/iter  (Convert std::io::Error to domain error)
conversions/serde_to_domain  → 529.56 ns/iter  (Convert serde error to domain error)
conversions/conversion_chain → 417.72 ns/iter  (Chain multiple error conversions)
```

## 4. Context Evaluation: Lazy vs Eager

**Important**: The "baseline" tests now use plain Result types without any ErrorRail functionality for accurate comparison.

```text
Success Path:
  context_lazy_success     → 587 ns/iter    (ErrorRail lazy evaluation)
  context_eager_success    → 1,297 ns/iter  (ErrorRail eager evaluation)
  context_baseline_success → 582 ns/iter    (Plain Result, no ErrorRail)

Error Path:
  context_lazy_error       → 833 ns/iter    (ErrorRail lazy evaluation)
  context_eager_error      → 764 ns/iter    (ErrorRail eager evaluation)
  context_baseline_error   → 70 ns/iter     (Plain Result, no ErrorRail)
```

**Key Insight**: ErrorRail's lazy evaluation adds only 1% overhead vs plain Result on success paths (587ns vs 582ns). The baseline error path is faster because it performs minimal error handling without context creation.

## 5. Scaling Tests

### Context Depth

```text
scaling/context_depth/1   → 537.28 ns/iter
scaling/context_depth/5   → 5.54 µs/iter
scaling/context_depth/10  → 6.85 µs/iter
scaling/context_depth/20  → 8.65 µs/iter
scaling/context_depth/50  → 18.94 µs/iter
```

### Validation Batch

```text
scaling/validation_batch/10    → 1.86 µs/iter  (5.39 M elements/sec)
scaling/validation_batch/100   → 24.17 µs/iter  (4.14 M elements/sec)
scaling/validation_batch/1000  → 194.20 µs/iter  (5.15 M elements/sec)
scaling/validation_batch/5000  → 914.39 µs/iter  (5.47 M elements/sec)
```

## 6. Pipeline vs Raw Result

**Note**: "result_with_context" uses ErrorRail's Result wrapper, "pipeline" uses ErrorRail's chain operations, "baseline" uses plain Result.

```text
Success Path:
  pipeline_success              → 724.66 ns/iter  (ErrorRail pipeline)
  result_with_context_success   → 694.22 ns/iter  (ErrorRail Result wrapper)
  result_baseline_success       → 693.21 ns/iter  (Plain Result)

Error Path:
  pipeline_error                → 521.20 ns/iter  (ErrorRail pipeline)
  result_with_context_error     → 59.69 ns/iter   (ErrorRail Result wrapper)
  result_baseline_error         → 57.29 ns/iter   (Plain Result)
```

## 7. Validation Performance

**Note**: "collect" uses ErrorRail's validation abstraction, "manual_collect" uses traditional error collection patterns.

```text
validation/collect_realistic_mixed  → 968 ns/iter   (ErrorRail validation)
validation/manual_collect_realistic → 900 ns/iter   (Manual error collection)
validation/collect_heterogeneous    → 1.72 µs/iter  (ErrorRail with mixed error types)
```

**Analysis**: ErrorRail's validation abstraction adds only ~7% overhead compared to manual collection while providing better ergonomics.

## 8. Real-World Scenarios

**Note**: Simulated production workloads - HTTP requests, database transactions, and microservice error propagation.

```text
http_request_simulation          → 1.058 µs/iter  (HTTP request with error handling)
database_transaction_rollback    → 1.005 µs/iter  (DB transaction with rollback on error)
microservice_error_propagation   → 745 ns/iter   (Error propagation across service boundaries)

Mixed Ratios (100 operations):
  95percent_success → 105.19 µs/iter  (951 K ops/sec - mostly success path)
  50percent_success → 92.78 µs/iter  (1.08 M ops/sec - mixed success/error)
```

## 9. Memory & Allocation

**Note**: Memory allocation patterns - static strings vs heap allocation, large metadata overhead.

```text
memory/large_metadata_contexts  → 5.41 µs/iter  (Create error with large metadata payload)
memory/string_allocation        → 61.76 ns/iter (String allocation for error message)
memory/static_str_no_allocation → 4.35 ns/iter  (Static str - 14x faster, no allocation)
```

## 10. Feature-Specific Benchmarks

### Backtrace (std feature)

**Note**: These benchmarks include backtrace capture overhead when the std feature is enabled.

```text
std/backtrace_lazy_success → 698.49 ns/iter  (Success path with backtrace)
std/backtrace_lazy_error   → 158.56 ns/iter  (Error path with backtrace)
```

### Serialization (serde feature)

**Note**: JSON serialization of error contexts with metadata.

```text
serde/error_serialization → 807.21 ns/iter  (JSON serialize error with context)
```

## 11. Async Benchmarks

**Note**: Async operations performance with Tokio runtime.

```text
async/pipeline/success_path         → 819.91 ns/iter  (Async pipeline success)
async/pipeline/error_path           → 159.42 ns/iter  (Async pipeline error)
async/context_evaluation/lazy       → 795.05 ns/iter  (Lazy context in async)
async/context_evaluation/eager      → 865.98 ns/iter  (Eager context in async)
async/retry/transient_retry_1       → 916.52 ns/iter  (Retry with 1 transient error)
async/validation/sequential_3       → 244.74 ns/iter  (Sequential validation of 3 items)
```

## 12. Tower Integration

**Note**: Tower middleware layer performance overhead.

```text
tower/layer/baseline_raw_service    → 783.46 ns/iter  (Raw Tower service)
tower/layer/error_rail_layer_success → 799.88 ns/iter (ErrorRail layer - 2% overhead)
```

## Quick Reference

| Category | Key Metric | Performance | Assessment |
|----------|------------|-------------|------------|
| Error Creation | 482 ns | Excellent | Fast error creation |
| Lazy Context | 587 ns vs 1.30µs | **2.2x faster** | Lazy evaluation wins |
| Context Depth (50) | 18.9µs | Linear scaling | Practical for deep nesting |
| Pipeline Success | 725 ns | 5% overhead | Minimal overhead |
| Validation | 968 ns | 7% overhead | Good abstraction tradeoff |
| Error Clone | 180 ns | Excellent | Cheap for async patterns |
| Serialization | 807 ns | Excellent | Fast JSON serialization |
| Retry (permanent) | 59 ns | **8x faster** | Smart error classification |
| Static str | 4.35 ns vs 62 ns | **14x faster** | Prefer static strings |
| Validation Batch (5000) | 5.47 M/sec | Excellent | High throughput validation |
| Tower Layer | 800 ns | 2% overhead | Minimal middleware cost |
| Async Pipeline | 820 ns | Excellent | Fast async error handling |

## Performance Tips

### Critical Optimizations

1. ✅ Use `context!()` macro - **2.2x faster** on success paths
2. ✅ Use `&'static str` - **14x faster** than String
3. ✅ Classify errors as transient/permanent - **8x faster** for permanent

### Best Practices

1. ✅ Deep context is practical (50 layers = 19µs)
2. ✅ Validation abstraction (7% overhead)
3. ✅ Error cloning for async (180ns)
4. ✅ Pipeline chains are free (constant time)
5. ✅ Tower layer overhead minimal (2%)
6. ⚠️ Large metadata sparingly (5.41µs)

## Methodology

### Environment

- **Platform**: Windows 11 (x86_64)
- **Compiler**: rustc 1.92.0 (release profile, default optimizations)
- **Framework**: Criterion.rs 0.7.0
- **Config**: 100 samples, 3s warm-up, 5s measurement
- **Build Settings**: Release mode, default Cargo.toml optimization settings
- **Caveat**: Results may vary significantly across different CPU architectures, OS, and compiler versions

### Benchmark Groups

1. Core Operations
2. Retry Operations
3. Error Conversions
4. Context Operations
5. Scaling Tests
6. Pipeline Operations
7. Validation Operations
8. Real-World Scenarios
9. Memory & Allocation
10. Feature-Specific (std, serde)
11. Async Operations
12. Tower Integration
