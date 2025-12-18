# ErrorRail Benchmarks

This document summarizes performance benchmarks for `error-rail` using realistic production scenarios.

> **Last Updated**: 2025-12-18  
> **Rust Version**: 1.90.0  
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
| Error creation | ~281 ns | Basic error struct construction |
| Lazy vs eager evaluation | **2.1x faster** on success paths | Lazy: 638ns vs Eager: 1.35µs |
| Error propagation overhead | ~8% vs raw Result | Pipeline: 759ns vs Baseline: 703ns |
| Validation overhead | ~4% vs manual collection | ErrorRail: 939ns vs Manual: 902ns |
| Serialization | ~802 ns | JSON serialization with serde feature |
| Validation throughput | 4.84 M elements/sec | 5000 items batch validation |

## 1. Core Operations

**Note**: Basic ErrorRail operations - error creation, cloning, wrapping, and functional operations.

```text
core/error_creation     → 281.37 ns/iter  (Create basic error with context)
core/error_clone        → 136.56 ns/iter  (Clone error for async/ownership transfer)
core/error_arc_wrap     → 202.46 ns/iter  (Wrap error in Arc for sharing)
core/ops_recover        → 2.13 ns/iter    (Recover from error with fallback)
core/ops_bimap          → 3.13 ns/iter    (Transform error types)
```

### Deep Cloning Scaling

```text
depth=5   → 1.10 µs/iter
depth=10  → 2.05 µs/iter  (1.9x)
depth=20  → 7.17 µs/iter  (3.5x)
depth=50  → 15.75 µs/iter (2.2x from depth=20)
```

## 2. Retry Operations

**Note**: Retry logic performance with different error classifications (transient vs permanent).

```text
retry/transient_success  → 402.81 ns/iter  (Retry succeeds after transient error)
retry/permanent_skip     → 59.19 ns/iter   (Skip retry for permanent errors - 7x faster)
retry/recover_transient  → 792.13 ns/iter  (Recover from transient error with retry)
retry/should_retry_check → 125.90 ns/iter  (Check if error should be retried)
```

## 3. Error Conversions

**Note**: Performance of converting between different error types (std, serde, domain errors).

```text
conversions/map_core         → 124.64 ns/iter  (Map between ErrorRail types)
conversions/std_io_to_domain → 328.38 ns/iter  (Convert std::io::Error to domain error)
conversions/serde_to_domain  → 461.81 ns/iter  (Convert serde error to domain error)
conversions/conversion_chain → 466.80 ns/iter  (Chain multiple error conversions)
```

## 4. Context Evaluation: Lazy vs Eager

**Important**: The "baseline" tests now use plain Result types without any ErrorRail functionality for accurate comparison.

```text
Success Path:
  context_lazy_success     → 638 ns/iter    (ErrorRail lazy evaluation)
  context_eager_success    → 1,355 ns/iter  (ErrorRail eager evaluation)
  context_baseline_success → 600 ns/iter    (Plain Result, no ErrorRail)

Error Path:
  context_lazy_error       → 927 ns/iter    (ErrorRail lazy evaluation)
  context_eager_error      → 825 ns/iter    (ErrorRail eager evaluation)
  context_baseline_error   → 67 ns/iter     (Plain Result, no ErrorRail)
```

**Key Insight**: ErrorRail's lazy evaluation adds only 6% overhead vs plain Result on success paths (638ns vs 600ns). The baseline error path is faster because it performs minimal error handling without context creation.

## 5. Scaling Tests

### Context Depth

```text
scaling/context_depth/1   → 421.49 ns/iter
scaling/context_depth/5   → 1.51 µs/iter
scaling/context_depth/10  → 2.70 µs/iter
scaling/context_depth/20  → 5.02 µs/iter
scaling/context_depth/50  → 11.98 µs/iter
```

### Validation Batch

```text
scaling/validation_batch/10    → 5.27 µs/iter  (1.90 M elements/sec)
scaling/validation_batch/100   → 35.4 µs/iter  (2.82 M elements/sec)
scaling/validation_batch/1000  → 223 µs/iter   (4.48 M elements/sec)
scaling/validation_batch/5000  → 1.03 ms/iter  (4.84 M elements/sec)
```

## 6. Pipeline vs Raw Result

**Note**: "result_with_context" uses ErrorRail's Result wrapper, "pipeline" uses ErrorRail's chain operations, "baseline" uses plain Result.

```text
Success Path:
  pipeline_success              → 759.23 ns/iter  (ErrorRail pipeline)
  result_with_context_success   → 700.83 ns/iter  (ErrorRail Result wrapper)
  result_baseline_success       → 703.02 ns/iter  (Plain Result)

Error Path:
  pipeline_error                → 629.48 ns/iter  (ErrorRail pipeline)
  result_with_context_error     → 60.61 ns/iter   (ErrorRail Result wrapper)
  result_baseline_error         → 57.05 ns/iter   (Plain Result)
```

## 7. Validation Performance

**Note**: "collect" uses ErrorRail's validation abstraction, "manual_collect" uses traditional error collection patterns.

```text
validation/collect_realistic_mixed  → 939 ns/iter   (ErrorRail validation)
validation/manual_collect_realistic → 902 ns/iter   (Manual error collection)
validation/collect_heterogeneous    → 1.71 µs/iter  (ErrorRail with mixed error types)
```

**Analysis**: ErrorRail's validation abstraction adds only ~4% overhead compared to manual collection while providing better ergonomics.

## 8. Real-World Scenarios

**Note**: Simulated production workloads - HTTP requests, database transactions, and microservice error propagation.

```text
http_request_simulation          → 928.42 ns/iter  (HTTP request with error handling)
database_transaction_rollback    → 884.21 ns/iter  (DB transaction with rollback on error)
microservice_error_propagation   → 607.99 ns/iter  (Error propagation across service boundaries)

Mixed Ratios (100 operations):
  95percent_success → 103.9 µs/iter  (962 K ops/sec - mostly success path)
  50percent_success → 105.0 µs/iter  (953 K ops/sec - mixed success/error)
```

## 9. Memory & Allocation

**Note**: Memory allocation patterns - static strings vs heap allocation, large metadata overhead.

```text
memory/large_metadata_contexts  → 5.33 µs/iter  (Create error with large metadata payload)
memory/string_allocation        → 62.95 ns/iter (String allocation for error message)
memory/static_str_no_allocation → 4.65 ns/iter  (Static str - 14x faster, no allocation)
```

## 10. Feature-Specific Benchmarks

### Backtrace (std feature)

**Note**: These benchmarks include backtrace capture overhead when the std feature is enabled.

```text
std/backtrace_lazy_success → 708.69 ns/iter  (Success path with backtrace)
std/backtrace_lazy_error   → 193.40 ns/iter  (Error path with backtrace)
```

### Serialization (serde feature)

**Note**: JSON serialization of error contexts with metadata.

```text
serde/error_serialization → 801.53 ns/iter  (JSON serialize error with context)
```

## 11. Async Benchmarks

**Note**: Async operations performance with Tokio runtime.

```text
async/pipeline/success_path         → 832.28 ns/iter  (Async pipeline success)
async/pipeline/error_path           → 168.07 ns/iter  (Async pipeline error)
async/context_evaluation/lazy       → 831.54 ns/iter  (Lazy context in async)
async/context_evaluation/eager      → 876.44 ns/iter  (Eager context in async)
async/retry/transient_retry_1       → 929.82 ns/iter  (Retry with 1 transient error)
async/validation/sequential_3       → 257.05 ns/iter  (Sequential validation of 3 items)
```

## 12. Tower Integration

**Note**: Tower middleware layer performance overhead.

```text
tower/layer/baseline_raw_service    → 788.73 ns/iter  (Raw Tower service)
tower/layer/error_rail_layer_success → 821.47 ns/iter (ErrorRail layer - 4% overhead)
```

## Quick Reference

| Category | Key Metric | Performance | Assessment |
|----------|------------|-------------|------------|
| Error Creation | 281 ns | Excellent | Fast error creation |
| Lazy Context | 638 ns vs 1.35µs | **2.1x faster** | Lazy evaluation wins |
| Context Depth (50) | 12.0µs | Linear scaling | Practical for deep nesting |
| Pipeline Success | 759 ns | 8% overhead | Minimal overhead |
| Validation | 939 ns | 4% overhead | Good abstraction tradeoff |
| Error Clone | 137 ns | Excellent | Cheap for async patterns |
| Serialization | 802 ns | Excellent | Fast JSON serialization |
| Retry (permanent) | 59 ns | **7x faster** | Smart error classification |
| Static str | 4.65 ns vs 63 ns | **14x faster** | Prefer static strings |
| Validation Batch (5000) | 4.84 M/sec | Excellent | High throughput validation |
| Tower Layer | 821 ns | 4% overhead | Minimal middleware cost |
| Async Pipeline | 832 ns | Excellent | Fast async error handling |

## Performance Tips

### Critical Optimizations

1. ✅ Use `context!()` macro - **2x faster** on success paths
2. ✅ Use `&'static str` - **14x faster** than String
3. ✅ Classify errors as transient/permanent - **7x faster** for permanent

### Best Practices

1. ✅ Deep context is practical (50 layers = 12µs)
2. ✅ Validation abstraction (4% overhead)
3. ✅ Error cloning for async (137ns)
4. ✅ Pipeline chains are free (constant time)
5. ✅ Tower layer overhead minimal (4%)
6. ⚠️ Large metadata sparingly (5.33µs)

## Methodology

### Environment

- **Platform**: Windows 11 (x86_64)
- **Compiler**: rustc 1.90.0 (release profile, default optimizations)
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
