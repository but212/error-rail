# ErrorRail Benchmarks

This document summarizes performance benchmarks for `error-rail` using realistic production scenarios.

> **Last Updated**: 2025-11-30
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
| Error creation | ~296 ns | Basic error struct construction |
| Lazy vs eager evaluation | **7x faster** on success paths | Lazy evaluation: 613ns vs Eager: 4.28µs |
| Error propagation overhead | ~9% vs raw Result | Pipeline: 770ns vs Baseline: 706ns |
| Validation overhead | ~5% vs manual collection | ErrorRail: 4.97µs vs Manual: 4.72µs |
| Serialization | ~684 ns | JSON serialization with serde feature |
| Validation throughput | 5.37 M elements/sec | 5000 items batch validation |

## 1. Core Operations

**Note**: Basic ErrorRail operations - error creation, cloning, wrapping, and functional operations.

```text
core/error_creation     → 295.92 ns/iter  (Create basic error with context)
core/error_clone        → 149.89 ns/iter  (Clone error for async/ownership transfer)
core/error_arc_wrap     → 208.26 ns/iter  (Wrap error in Arc for sharing)
core/ops_recover        → 1.28 ns/iter    (Recover from error with fallback)
core/ops_bimap          → 1.54 ns/iter    (Transform error types)
```

### Deep Cloning Scaling

```text
depth=5   → 1.18 µs/iter
depth=10  → 2.16 µs/iter  (2.0x)
depth=20  → 10.3 µs/iter  (4.8x)
depth=50  → 16.7 µs/iter  (1.6x from depth=20)
```

## 2. Retry Operations

**Note**: Retry logic performance with different error classifications (transient vs permanent).

```text
retry/transient_success  → 400.29 ns/iter  (Retry succeeds after transient error)
retry/permanent_skip     → 57.99 ns/iter   (Skip retry for permanent errors - 7x faster)
retry/recover_transient  → 778.69 ns/iter  (Recover from transient error with retry)
retry/should_retry_check → 126.61 ns/iter  (Check if error should be retried)
```

## 3. Error Conversions

**Note**: Performance of converting between different error types (std, serde, domain errors).

```text
conversions/map_core         → 123.71 ns/iter  (Map between ErrorRail types)
conversions/std_io_to_domain → 319.58 ns/iter  (Convert std::io::Error to domain error)
conversions/serde_to_domain  → 455.15 ns/iter  (Convert serde error to domain error)
conversions/conversion_chain → 452.73 ns/iter  (Chain multiple error conversions)
```

## 4. Context Evaluation: Lazy vs Eager

**Important**: The "baseline" tests now use plain Result types without any ErrorRail functionality for accurate comparison.

```text
Success Path:
  context_lazy_success    → 605 ns/iter     (ErrorRail lazy evaluation)
  context_eager_success   → 1,291 ns/iter   (ErrorRail eager evaluation)
  context_baseline_success → 572 ns/iter    (Plain Result, no ErrorRail)

Error Path:
  context_lazy_error      → 894 ns/iter     (ErrorRail lazy evaluation)
  context_eager_error     → 796 ns/iter     (ErrorRail eager evaluation)
  context_baseline_error  → 65 ns/iter      (Plain Result, no ErrorRail)
```

**Key Insight**: ErrorRail's lazy evaluation adds only 6% overhead vs plain Result on success paths (605ns vs 572ns). The baseline error path is faster because it performs minimal error handling without context creation.

## 5. Scaling Tests

### Context Depth

```text
scaling/context_depth/1   → 422.92 ns/iter
scaling/context_depth/5   → 1.5 µs/iter
scaling/context_depth/10  → 2.70 µs/iter
scaling/context_depth/20  → 5.08 µs/iter
scaling/context_depth/50  → 12.02 µs/iter
```

### Validation Batch

```text
scaling/validation_batch/10    → 4.91 µs/iter  (2.04 M elements/sec)
scaling/validation_batch/100   → 30.9 µs/iter  (3.24 M elements/sec)
scaling/validation_batch/1000  → 194 µs/iter   (5.15 M elements/sec)
scaling/validation_batch/5000  → 929 µs/iter   (5.37 M elements/sec)
```

## 6. Pipeline vs Raw Result

**Note**: "result_with_context" uses ErrorRail's Result wrapper, "pipeline" uses ErrorRail's chain operations, "baseline" uses plain Result.

```text
Success Path:
  pipeline_success              → 770.06 ns/iter  (ErrorRail pipeline)
  result_with_context_success   → 703.63 ns/iter  (ErrorRail Result wrapper)
  result_baseline_success       → 706.17 ns/iter  (Plain Result)

Error Path:
  pipeline_error                → 630.16 ns/iter  (ErrorRail pipeline)
  result_with_context_error     → 61.95 ns/iter   (ErrorRail Result wrapper)
  result_baseline_error         → 57.08 ns/iter   (Plain Result)
```

## 7. Validation Performance

**Note**: "collect" uses ErrorRail's validation abstraction, "manual_collect" uses traditional error collection patterns.

```text
validation/collect_realistic_mixed  → 4.97 µs/iter  (ErrorRail validation)
validation/manual_collect_realistic → 4.72 µs/iter  (Manual error collection)
validation/collect_heterogeneous    → 5.53 µs/iter  (ErrorRail with mixed error types)
```

**Analysis**: ErrorRail's validation abstraction adds only ~5% overhead compared to manual collection while providing better ergonomics.

## 8. Real-World Scenarios

**Note**: Simulated production workloads - HTTP requests, database transactions, and microservice error propagation.

```text
http_request_simulation          → 933.41 ns/iter  (HTTP request with error handling)
database_transaction_rollback    → 881.27 ns/iter  (DB transaction with rollback on error)
microservice_error_propagation   → 607.67 ns/iter  (Error propagation across service boundaries)

Mixed Ratios (100 operations):
  95percent_success → 93.02 µs/iter  (1.07 M ops/sec - mostly success path)
  50percent_success → 105.5 µs/iter  (945 K ops/sec - mixed success/error)
```

## 9. Memory & Allocation

**Note**: Memory allocation patterns - static strings vs heap allocation, large metadata overhead.

```text
memory/large_metadata_contexts → 6.27 µs/iter  (Create error with large metadata payload)
memory/string_allocation       → 63.36 ns/iter  (String allocation for error message)
memory/static_str_no_allocation → 4.66 ns/iter  (Static str - 13x faster, no allocation)
```

## 10. Feature-Specific Benchmarks

### Backtrace (std feature)

**Note**: These benchmarks include backtrace capture overhead when the std feature is enabled.

```text
std/backtrace_lazy_success → 704.60 ns/iter  (Success path with backtrace)
std/backtrace_lazy_error   → 193.74 ns/iter  (Error path with backtrace)
```

### Serialization (serde feature)

**Note**: JSON serialization of error contexts with metadata.

```text
serde/error_serialization → 683.95 ns/iter  (JSON serialize error with context)
```

## Quick Reference

| Category | Key Metric | Performance | Assessment |
|----------|------------|-------------|------------|
| Error Creation | 296 ns | Excellent | Fast error creation |
| Lazy Context | 613 ns vs 4.28µs | **7x faster** | Lazy evaluation wins |
| Context Depth (50) | 12.0µs | Linear scaling | Practical for deep nesting |
| Pipeline Success | 770 ns | 9% overhead | Minimal overhead |
| Validation | 4.97 µs | 5% overhead | Good abstraction tradeoff |
| Error Clone | 150 ns | Excellent | Cheap for async patterns |
| Serialization | 684 ns | Excellent | Fast JSON serialization |
| Retry (permanent) | 58 ns | **7x faster** | Smart error classification |
| Static str | 4.66 ns vs 63 ns | **13x faster** | Prefer static strings |
| Validation Batch (5000) | 5.37 M/sec | Excellent | High throughput validation |

## Performance Tips

### Critical Optimizations

1. ✅ Use `context!()` macro - **7x faster** on success paths
2. ✅ Use `&'static str` - **13x faster** than String
3. ✅ Classify errors as transient/permanent - **7x faster** for permanent

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
