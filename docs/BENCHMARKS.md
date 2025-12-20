# ErrorRail Benchmarks

This document summarizes performance benchmarks for `error-rail` using realistic production scenarios.

> **Last Updated**: 2025-12-21  
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
| Error creation | ~475 ns | Basic error struct construction |
| Lazy vs eager evaluation | **2.2x faster** on success paths | Lazy: 610ns vs Eager: 1.32µs |
| Error propagation overhead | ~4% vs raw Result | Pipeline: 737ns vs Baseline: 710ns |
| Validation overhead | ~6% vs manual collection | ErrorRail: 975ns vs Manual: 922ns |
| Serialization | ~731 ns | JSON serialization with serde feature |
| Validation throughput | 5.19 M elements/sec | 5000 items batch validation |

## 1. Core Operations

**Note**: Basic ErrorRail operations - error creation, cloning, wrapping, and functional operations.

```text
core/error_creation     → 475.56 ns/iter  (Create basic error with context)
core/error_clone        → 165.56 ns/iter  (Clone error for async/ownership transfer)
core/error_arc_wrap     → 232.77 ns/iter  (Wrap error in Arc for sharing)
core/ops_recover        → 2.57 ns/iter    (Recover from error with fallback)
core/ops_bimap          → 3.08 ns/iter    (Transform error types)
```

### Deep Cloning Scaling

```text
depth=5   → 1.28 µs/iter
depth=10  → 5.41 µs/iter  (4.2x)
depth=20  → 10.69 µs/iter  (2.0x)
depth=50  → 23.71 µs/iter (2.2x from depth=20)
```

## 2. Retry Operations

**Note**: Retry logic performance with different error classifications (transient vs permanent).

```text
retry/transient_success  → 381.54 ns/iter  (Retry succeeds after transient error)
retry/permanent_skip     → 62.29 ns/iter   (Skip retry for permanent errors - 6.1x faster)
retry/recover_transient  → 799.58 ns/iter  (Recover from transient error with retry)
retry/should_retry_check → 126.15 ns/iter  (Check if error should be retried)
```

## 3. Error Conversions

**Note**: Performance of converting between different error types (std, serde, domain errors).

```text
conversions/map_core         → 160.67 ns/iter  (Map between ErrorRail types)
conversions/std_io_to_domain → 414.01 ns/iter  (Convert std::io::Error to domain error)
conversions/serde_to_domain  → 563.32 ns/iter  (Convert serde error to domain error)
conversions/conversion_chain → 423.40 ns/iter  (Chain multiple error conversions)
```

## 4. Context Evaluation: Lazy vs Eager

**Important**: The "baseline" tests now use plain Result types without any ErrorRail functionality for accurate comparison.

```text
Success Path:
  context_lazy_success     → 610 ns/iter    (ErrorRail lazy evaluation)
  context_eager_success    → 1,320 ns/iter  (ErrorRail eager evaluation)
  context_baseline_success → 597 ns/iter    (Plain Result, no ErrorRail)

Error Path:
  context_lazy_error       → 835 ns/iter    (ErrorRail lazy evaluation)
  context_eager_error      → 757 ns/iter    (ErrorRail eager evaluation)
  context_baseline_error   → 66 ns/iter     (Plain Result, no ErrorRail)
```

**Key Insight**: ErrorRail's lazy evaluation adds only 2% overhead vs plain Result on success paths (610ns vs 597ns). The baseline error path is faster because it performs minimal error handling without context creation.

## 5. Scaling Tests

### Context Depth

```text
scaling/context_depth/1   → 540.67 ns/iter
scaling/context_depth/5   → 1.71 µs/iter
scaling/context_depth/10  → 2.98 µs/iter
scaling/context_depth/20  → 5.56 µs/iter
scaling/context_depth/50  → 19.05 µs/iter
```

### Validation Batch

```text
scaling/validation_batch/10    → 1.92 µs/iter  (5.19 M elements/sec)
scaling/validation_batch/100   → 31.31 µs/iter  (3.19 M elements/sec)
scaling/validation_batch/1000  → 239.56 µs/iter  (4.17 M elements/sec)
scaling/validation_batch/5000  → 961.13 µs/iter  (5.20 M elements/sec)
```

## 6. Pipeline vs Raw Result

**Note**: "result_with_context" uses ErrorRail's Result wrapper, "pipeline" uses ErrorRail's chain operations, "baseline" uses plain Result.

```text
Success Path:
  pipeline_success              → 737.46 ns/iter  (ErrorRail pipeline)
  result_with_context_success   → 705.69 ns/iter  (ErrorRail Result wrapper)
  result_baseline_success       → 710.00 ns/iter  (Plain Result)

Error Path:
  pipeline_error                → 533.02 ns/iter  (ErrorRail pipeline)
  result_with_context_error     → 62.03 ns/iter   (ErrorRail Result wrapper)
  result_baseline_error         → 58.60 ns/iter   (Plain Result)
```

## 7. Validation Performance

**Note**: "collect" uses ErrorRail's validation abstraction, "manual_collect" uses traditional error collection patterns.

```text
validation/collect_realistic_mixed  → 975 ns/iter   (ErrorRail validation)
validation/manual_collect_realistic → 922 ns/iter   (Manual error collection)
validation/collect_heterogeneous    → 1.73 µs/iter  (ErrorRail with mixed error types)
```

**Analysis**: ErrorRail's validation abstraction adds only ~6% overhead compared to manual collection while providing better ergonomics.

## 8. Real-World Scenarios

**Note**: Simulated production workloads - HTTP requests, database transactions, and microservice error propagation.

```text
http_request_simulation          → 1.066 µs/iter  (HTTP request with error handling)
database_transaction_rollback    → 1.012 µs/iter  (DB transaction with rollback on error)
microservice_error_propagation   → 752 ns/iter   (Error propagation across service boundaries)

Mixed Ratios (100 operations):
  95percent_success → 115.92 µs/iter  (862 K ops/sec - mostly success path)
  50percent_success → 129.17 µs/iter  (774 K ops/sec - mixed success/error)
```

## 9. Memory & Allocation

**Note**: Memory allocation patterns - static strings vs heap allocation, large metadata overhead.

```text
memory/large_metadata_contexts  → 2.45 µs/iter  (Create error with large metadata payload)
memory/string_allocation        → 62.27 ns/iter (String allocation for error message)
memory/static_str_no_allocation → 4.34 ns/iter  (Static str - 14x faster, no allocation)
```

## 10. Feature-Specific Benchmarks

### Backtrace (std feature)

**Note**: These benchmarks include backtrace capture overhead when the std feature is enabled.

```text
std/backtrace_lazy_success → 723.06 ns/iter  (Success path with backtrace)
std/backtrace_lazy_error   → 164.47 ns/iter  (Error path with backtrace)
```

### Serialization (serde feature)

**Note**: JSON serialization of error contexts with metadata.

```text
serde/error_serialization → 3.76 µs/iter  (JSON serialize error with context)
```

## 11. Async Benchmarks

**Note**: Async operations performance with Tokio runtime.

```text
async/pipeline/success_path         → 819.78 ns/iter  (Async pipeline success)
async/pipeline/error_path           → 166.60 ns/iter  (Async pipeline error)
async/context_evaluation/lazy       → 851.75 ns/iter  (Lazy context in async)
async/context_evaluation/eager      → 929.93 ns/iter  (Eager context in async)
async/retry/transient_retry_1       → 926.69 ns/iter  (Retry with 1 transient error)
async/validation/sequential_3       → 250.76 ns/iter  (Sequential validation of 3 items)
```

## 12. Tower Integration

**Note**: Tower middleware layer performance overhead.

```text
tower/layer/baseline_raw_service    → 833.83 ns/iter  (Raw Tower service)
tower/layer/error_rail_layer_success → 826.83 ns/iter (ErrorRail layer - 0.8% overhead)
```

## Quick Reference

| Category | Key Metric | Performance | Assessment |
|----------|------------|-------------|------------|
| Error Creation | 476 ns | Excellent | Fast error creation |
| Lazy Context | 610 ns vs 1.32µs | **2.2x faster** | Lazy evaluation wins |
| Context Depth (50) | 19.1µs | Linear scaling | Practical for deep nesting |
| Pipeline Success | 737 ns | 4% overhead | Minimal overhead |
| Validation | 975 ns | 6% overhead | Good abstraction tradeoff |
| Error Clone | 166 ns | Excellent | Cheap for async patterns |
| Serialization | 3.76 µs | Good | JSON serialization overhead |
| Retry (permanent) | 59 ns | **7.8x faster** | Smart error classification |
| Static str | 4.34 ns vs 62 ns | **14x faster** | Prefer static strings |
| Validation Batch (5000) | 5.20 M/sec | Excellent | High throughput validation |
| Tower Layer | 827 ns | 0.8% overhead | Minimal middleware cost |
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
