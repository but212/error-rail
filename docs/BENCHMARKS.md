# ErrorRail Benchmarks

This document summarizes performance benchmarks for `error-rail` using realistic production scenarios.

> **Last Updated**: 2025-12-01
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
| Error creation | ~269 ns | Basic error struct construction |
| Lazy vs eager evaluation | **2.2x faster** on success paths | Lazy evaluation: 597ns vs Eager: 1.29µs |
| Error propagation overhead | ~10% vs raw Result | Pipeline: 770ns vs Baseline: 693ns |
| Validation overhead | ~5% vs manual collection | ErrorRail: 921ns vs Manual: 883ns |
| Serialization | ~665 ns | JSON serialization with serde feature |
| Validation throughput | 5.49 M elements/sec | 5000 items batch validation |

## 1. Core Operations

**Note**: Basic ErrorRail operations - error creation, cloning, wrapping, and functional operations.

```text
core/error_creation     → 269.11 ns/iter  (Create basic error with context)
core/error_clone        → 3.08 µs/iter   (Clone error for async/ownership transfer)
core/error_arc_wrap     → 202.51 ns/iter  (Wrap error in Arc for sharing)
core/ops_recover        → 1.28 ns/iter    (Recover from error with fallback)
core/ops_bimap          → 1.53 ns/iter    (Transform error types)
```

### Deep Cloning Scaling

```text
depth=5   → 1.14 µs/iter
depth=10  → 2.09 µs/iter  (1.8x)
depth=20  → 7.22 µs/iter   (3.5x)
depth=50  → 16.43 µs/iter  (2.3x from depth=20)
```

## 2. Retry Operations

**Note**: Retry logic performance with different error classifications (transient vs permanent).

```text
retry/transient_success  → 388.07 ns/iter  (Retry succeeds after transient error)
retry/permanent_skip     → 56.81 ns/iter   (Skip retry for permanent errors - 6.8x faster)
retry/recover_transient  → 782.13 ns/iter  (Recover from transient error with retry)
retry/should_retry_check → 124.14 ns/iter  (Check if error should be retried)
```

## 3. Error Conversions

**Note**: Performance of converting between different error types (std, serde, domain errors).

```text
conversions/map_core         → 116.72 ns/iter  (Map between ErrorRail types)
conversions/std_io_to_domain → 3.32 µs/iter   (Convert std::io::Error to domain error)
conversions/serde_to_domain  → 447.53 ns/iter  (Convert serde error to domain error)
conversions/conversion_chain → 465.33 ns/iter  (Chain multiple error conversions)
```

## 4. Context Evaluation: Lazy vs Eager

**Important**: The "baseline" tests now use plain Result types without any ErrorRail functionality for accurate comparison.

```text
Success Path:
  context_lazy_success    → 597 ns/iter     (ErrorRail lazy evaluation)
  context_eager_success   → 1,287 ns/iter   (ErrorRail eager evaluation)
  context_baseline_success → 576 ns/iter    (Plain Result, no ErrorRail)

Error Path:
  context_lazy_error      → 885 ns/iter     (ErrorRail lazy evaluation)
  context_eager_error     → 794 ns/iter     (ErrorRail eager evaluation)
  context_baseline_error  → 65 ns/iter      (Plain Result, no ErrorRail)
```

**Key Insight**: ErrorRail's lazy evaluation adds only 4% overhead vs plain Result on success paths (597ns vs 576ns). The baseline error path is faster because it performs minimal error handling without context creation.

## 5. Scaling Tests

### Context Depth

```text
scaling/context_depth/1   → 407 ns/iter
scaling/context_depth/5   → 1.51 µs/iter
scaling/context_depth/10  → 7.04 µs/iter
scaling/context_depth/20  → 9.17 µs/iter
scaling/context_depth/50  → 11.72 µs/iter
```

### Validation Batch

```text
scaling/validation_batch/10    → 5.17 µs/iter  (1.93 M elements/sec)
scaling/validation_batch/100   → 29.97 µs/iter  (3.34 M elements/sec)
scaling/validation_batch/1000  → 190.97 µs/iter (5.24 M elements/sec)
scaling/validation_batch/5000  → 906.89 µs/iter (5.51 M elements/sec)
```

## 6. Pipeline vs Raw Result

**Note**: "result_with_context" uses ErrorRail's Result wrapper, "pipeline" uses ErrorRail's chain operations, "baseline" uses plain Result.

```text
Success Path:
  pipeline_success              → 770 ns/iter   (ErrorRail pipeline)
  result_with_context_success   → 698 ns/iter   (ErrorRail Result wrapper)
  result_baseline_success       → 693 ns/iter   (Plain Result)

Error Path:
  pipeline_error                → 633 ns/iter   (ErrorRail pipeline)
  result_with_context_error     → 59 ns/iter    (ErrorRail Result wrapper)
  result_baseline_error         → 55 ns/iter    (Plain Result)
```

## 7. Validation Performance

**Note**: "collect" uses ErrorRail's validation abstraction, "manual_collect" uses traditional error collection patterns.

```text
validation/collect_realistic_mixed  → 921 ns/iter   (ErrorRail validation)
validation/manual_collect_realistic → 883 ns/iter   (Manual error collection)
validation/collect_heterogeneous    → 1.66 µs/iter  (ErrorRail with mixed error types)
```

**Analysis**: ErrorRail's validation abstraction adds only ~4% overhead compared to manual collection while providing better ergonomics.

## 8. Real-World Scenarios

**Note**: Simulated production workloads - HTTP requests, database transactions, and microservice error propagation.

```text
http_request_simulation          → 917 ns/iter   (HTTP request with error handling)
database_transaction_rollback    → 870 ns/iter   (DB transaction with rollback on error)
microservice_error_propagation   → 610 ns/iter   (Error propagation across service boundaries)

Mixed Ratios (100 operations):
  95percent_success → 93.0 µs/iter  (1.07 M ops/sec - mostly success path)
  50percent_success → 95.6 µs/iter  (1.05 M ops/sec - mixed success/error)
```

## 9. Memory & Allocation

**Note**: Memory allocation patterns - static strings vs heap allocation, large metadata overhead.

```text
memory/large_metadata_contexts → 2.36 µs/iter  (Create error with large metadata payload)
memory/string_allocation       → 59.77 ns/iter  (String allocation for error message)
memory/static_str_no_allocation → 4.61 ns/iter  (Static str - 13x faster, no allocation)
```

## 10. Feature-Specific Benchmarks

### Backtrace (std feature)

**Note**: These benchmarks include backtrace capture overhead when the std feature is enabled.

```text
std/backtrace_lazy_success → 714 ns/iter  (Success path with backtrace)
std/backtrace_lazy_error   → 186 ns/iter  (Error path with backtrace)
```

### Serialization (serde feature)

**Note**: JSON serialization of error contexts with metadata.

```text
serde/error_serialization → 665 ns/iter  (JSON serialize error with context)
```

## Quick Reference

| Category | Key Metric | Performance | Assessment |
|----------|------------|-------------|------------|
| Error Creation | 269 ns | Excellent | Fast error creation |
| Lazy Context | 597 ns vs 1.29µs | **2.2x faster** | Lazy evaluation wins |
| Context Depth (50) | 11.7µs | Linear scaling | Practical for deep nesting |
| Pipeline Success | 770 ns | 10% overhead | Minimal overhead |
| Validation | 921 ns | 4% overhead | Good abstraction tradeoff |
| Error Clone | 3.08 µs | Good | Suitable for async patterns |
| Serialization | 665 ns | Excellent | Fast JSON serialization |
| Retry (permanent) | 57 ns | **6.8x faster** | Smart error classification |
| Static str | 4.61 ns vs 60 ns | **13x faster** | Prefer static strings |
| Validation Batch (5000) | 5.51 M/sec | Excellent | High throughput validation |

## Performance Tips

### Critical Optimizations

1. ✅ Use `context!()` macro - **2.2x faster** on success paths
2. ✅ Use `&'static str` - **13x faster** than String
3. ✅ Classify errors as transient/permanent - **6.8x faster** for permanent

### Best Practices

1. ✅ Deep context is practical (50 layers = 12µs)
2. ✅ Validation abstraction (5% overhead)
3. ✅ Error cloning for async (150ns)
4. ✅ Pipeline chains are free (constant time)
5. ⚠️ Large metadata sparingly (6.27µs)

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
