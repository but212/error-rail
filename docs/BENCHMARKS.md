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

| Metric | Performance |
|--------|-------------|
| Error creation | ~296 ns |
| Lazy vs eager evaluation | **7x faster** on success paths |
| Error propagation overhead | ~9% vs raw Result |
| Validation overhead | ~5% vs manual collection |
| Serialization | ~684 ns |
| Retry (permanent errors) | 58 ns (**7x faster** than transient) |
| Validation throughput | 5.37 M elements/sec (5000 items) |

## 1. Core Operations

```text
core/error_creation     → 295.92 ns/iter
core/error_clone        → 149.89 ns/iter
core/error_arc_wrap     → 208.26 ns/iter
core/ops_recover        → 1.28 ns/iter
core/ops_bimap          → 1.54 ns/iter
```

### Deep Cloning Scaling

```text
depth=5   → 1.18 µs/iter
depth=10  → 2.16 µs/iter  (2.0x)
depth=20  → 10.3 µs/iter  (4.8x)
depth=50  → 16.7 µs/iter  (1.6x from depth=20)
```

## 2. Retry Operations

```text
retry/transient_success  → 400.29 ns/iter
retry/permanent_skip     → 57.99 ns/iter  (7x faster)
retry/recover_transient  → 778.69 ns/iter
retry/should_retry_check → 126.61 ns/iter
```

## 3. Error Conversions

```text
conversions/map_core         → 123.71 ns/iter
conversions/std_io_to_domain → 319.58 ns/iter
conversions/serde_to_domain  → 455.15 ns/iter
conversions/conversion_chain → 452.73 ns/iter
```

## 4. Context Evaluation: Lazy vs Eager

```text
Success Path:
  context_lazy_success    → 612.98 ns/iter
  context_eager_success   → 4,277.2 ns/iter  (7.0x slower)
  context_baseline_success → 4,457.7 ns/iter

Error Path:
  context_lazy_error      → 914.37 ns/iter
  context_eager_error     → 801.70 ns/iter
  context_baseline_error  → 84.18 ns/iter
```

## 5. Scaling Tests

### Context Depth

```text
scaling/context_depth/1   → 422.92 ns/iter
scaling/context_depth/5   → 5.53 µs/iter
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

### Pipeline Chain

```text
scaling/pipeline_chain/2   → 3.09 ns/iter
scaling/pipeline_chain/5   → 3.09 ns/iter
scaling/pipeline_chain/10  → 3.09 ns/iter
scaling/pipeline_chain/20  → 3.09 ns/iter
```

## 6. Pipeline vs Raw Result

```text
Success Path:
  pipeline_success              → 770.06 ns/iter
  result_with_context_success   → 703.63 ns/iter
  result_baseline_success       → 706.17 ns/iter

Error Path:
  pipeline_error                → 630.16 ns/iter
  result_with_context_error     → 61.95 ns/iter
  result_baseline_error         → 57.08 ns/iter
```

## 7. Validation Performance

```text
validation/collect_realistic_mixed  → 4.97 µs/iter
validation/manual_collect_realistic → 4.72 µs/iter  (5% faster)
validation/collect_heterogeneous    → 5.53 µs/iter
```

## 8. Real-World Scenarios

```text
http_request_simulation          → 933.41 ns/iter
database_transaction_rollback    → 881.27 ns/iter
microservice_error_propagation   → 607.67 ns/iter

Mixed Ratios (100 operations):
  95percent_success → 93.02 µs/iter  (1.07 M ops/sec)
  50percent_success → 105.5 µs/iter  (945 K ops/sec)
```

## 9. Memory & Allocation

```text
memory/large_metadata_contexts → 6.27 µs/iter
memory/string_allocation       → 63.36 ns/iter
memory/static_str_no_allocation → 4.66 ns/iter  (13x faster)
```

## 10. Feature-Specific Benchmarks

### Backtrace (std feature)

```text
std/backtrace_lazy_success → 704.60 ns/iter
std/backtrace_lazy_error   → 193.74 ns/iter
```

### Serialization (serde feature)

```text
serde/error_serialization → 683.95 ns/iter
```

## Quick Reference

| Category | Key Metric | Performance |
|----------|------------|-------------|
| Error Creation | 296 ns | Excellent |
| Lazy Context | 613 ns vs 4.28µs | **7x faster** |
| Context Depth (50) | 12.0µs | Linear scaling |
| Pipeline Success | 770 ns | 9% overhead |
| Validation | 4.97 µs | 5% overhead |
| Error Clone | 150 ns | Excellent |
| Serialization | 684 ns | Excellent |
| Retry (permanent) | 58 ns | **7x faster** |
| Static str | 4.66 ns vs 63 ns | **13x faster** |
| Validation Batch (5000) | 5.37 M/sec | Excellent |

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
- **Compiler**: rustc 1.90.0 (release profile)
- **Framework**: Criterion.rs 0.7.0
- **Config**: 100 samples, 3s warm-up, 5s measurement

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
