# ErrorRail Benchmarks

This document summarizes performance benchmarks for `error-rail` using realistic production scenarios.

> **Last Updated**: 2025-12-23  
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
| Error creation | ~468 ns | Basic error struct construction |
| Lazy vs eager evaluation | **2.2x faster** on success paths | Lazy: 621ns vs Eager: 1.34µs |
| Error propagation overhead | ~6% vs raw Result | Pipeline: 756ns vs Baseline: 710ns |
| Validation overhead | ~8% vs manual collection | ErrorRail: 978ns vs Manual: 898ns |
| Serialization | ~791 ns | JSON serialization with serde feature |
| Validation throughput | 5.18 M elements/sec | 5000 items batch validation |

## 1. Core Operations

**Note**: Basic ErrorRail operations - error creation, cloning, wrapping, and functional operations.

```text
core/error_creation     → 467.88 ns/iter  (Create basic error with context)
core/error_clone        → 166.99 ns/iter  (Clone error for async/ownership transfer)
core/error_arc_wrap     → 231.32 ns/iter  (Wrap error in Arc for sharing)
core/ops_recover        → 2.58 ns/iter    (Recover from error with fallback)
core/ops_bimap          → 3.11 ns/iter    (Transform error types)
```

### Deep Cloning Scaling

```text
depth=5   → 1.28 µs/iter
depth=10  → 5.40 µs/iter  (4.2x)
depth=20  → 7.81 µs/iter  (1.4x)
depth=50  → 23.93 µs/iter (3.1x from depth=20)
```

## 2. Retry Operations

**Note**: Retry logic performance with different error classifications (transient vs permanent).

```text
retry/transient_success  → 361.10 ns/iter  (Retry succeeds after transient error)
retry/permanent_skip     → 58.52 ns/iter   (Skip retry for permanent errors - 6.2x faster)
retry/recover_transient  → 705.47 ns/iter  (Recover from transient error with retry)
retry/should_retry_check → 119.42 ns/iter  (Check if error should be retried)
```

## 3. Error Conversions

**Note**: Performance of converting between different error types (std, serde, domain errors).

```text
conversions/map_core         → 163.82 ns/iter  (Map between ErrorRail types)
conversions/std_io_to_domain → 416.88 ns/iter  (Convert std::io::Error to domain error)
conversions/serde_to_domain  → 530.25 ns/iter  (Convert serde error to domain error)
conversions/conversion_chain → 430.23 ns/iter  (Chain multiple error conversions)
```

## 4. Context Evaluation: Lazy vs Eager

**Important**: The "baseline" tests now use plain Result types without any ErrorRail functionality for accurate comparison.

```text
Success Path:
  context_lazy_success     → 621 ns/iter    (ErrorRail lazy evaluation)
  context_eager_success    → 1,344 ns/iter  (ErrorRail eager evaluation)
  context_baseline_success → 596 ns/iter    (Plain Result, no ErrorRail)

Error Path:
  context_lazy_error       → 863 ns/iter    (ErrorRail lazy evaluation)
  context_eager_error      → 800 ns/iter    (ErrorRail eager evaluation)
  context_baseline_error   → 67 ns/iter     (Plain Result, no ErrorRail)
```

**Key Insight**: ErrorRail's lazy evaluation adds only ~4% overhead vs plain Result on success paths (621ns vs 596ns). The baseline error path is faster because it performs minimal error handling without context creation.

## 5. Scaling Tests

### Context Depth

```text
scaling/context_depth/1   → 539.49 ns/iter
scaling/context_depth/5   → 1.71 µs/iter
scaling/context_depth/10  → 3.06 µs/iter
scaling/context_depth/20  → 5.60 µs/iter
scaling/context_depth/50  → 19.32 µs/iter
```

### Validation Batch

```text
scaling/validation_batch/10    → 1.93 µs/iter  (5.18 M elements/sec)
scaling/validation_batch/100   → 20.80 µs/iter  (4.81 M elements/sec)
scaling/validation_batch/1000  → 238.59 µs/iter  (4.19 M elements/sec)
scaling/validation_batch/5000  → 971.85 µs/iter  (5.14 M elements/sec)
```

## 6. Pipeline vs Raw Result

**Note**: "result_with_context" uses ErrorRail's Result wrapper, "pipeline" uses ErrorRail's chain operations, "baseline" uses plain Result.

```text
Success Path:
  pipeline_success              → 756.13 ns/iter  (ErrorRail pipeline)
  result_with_context_success   → 713.35 ns/iter  (ErrorRail Result wrapper)
  result_baseline_success       → 710.41 ns/iter  (Plain Result)

Error Path:
  pipeline_error                → 541.03 ns/iter  (ErrorRail pipeline)
  result_with_context_error     → 61.45 ns/iter   (ErrorRail Result wrapper)
  result_baseline_error         → 57.27 ns/iter   (Plain Result)
```

## 7. Validation Performance

**Note**: "collect" uses ErrorRail's validation abstraction, "manual_collect" uses traditional error collection patterns.

```text
validation/collect_realistic_mixed  → 978 ns/iter   (ErrorRail validation)
validation/manual_collect_realistic → 898 ns/iter   (Manual error collection)
validation/collect_heterogeneous    → 1.73 µs/iter  (ErrorRail with mixed error types)
```

**Analysis**: ErrorRail's validation abstraction adds only ~8% overhead compared to manual collection while providing better ergonomics.

## 8. Real-World Scenarios

**Note**: Simulated production workloads - HTTP requests, database transactions, and microservice error propagation.

```text
http_request_simulation          → 953 ns/iter   (HTTP request with error handling)
database_transaction_rollback    → 882 ns/iter   (DB transaction with rollback on error)
microservice_error_propagation   → 762 ns/iter   (Error propagation across service boundaries)

Mixed Ratios (100 operations):
  95percent_success → 87.85 µs/iter  (1.14 M ops/sec - mostly success path)
  50percent_success → 99.36 µs/iter  (1.00 M ops/sec - mixed success/error)
```

## 9. Memory & Allocation

**Note**: Memory allocation patterns - static strings vs heap allocation, large metadata overhead.

```text
memory/large_metadata_contexts  → 2.45 µs/iter  (Create error with large metadata payload)
memory/string_allocation        → 60.76 ns/iter (String allocation for error message)
memory/static_str_no_allocation → 4.78 ns/iter  (Static str - 13x faster, no allocation)
```

## 10. Feature-Specific Benchmarks

### Backtrace (std feature)

**Note**: These benchmarks include backtrace capture overhead when the std feature is enabled.

```text
std/backtrace_lazy_success → 721.62 ns/iter  (Success path with backtrace)
std/backtrace_lazy_error   → 155.65 ns/iter  (Error path with backtrace)
```

### Serialization (serde feature)

**Note**: JSON serialization of error contexts with metadata.

```text
serde/error_serialization → 791.48 ns/iter  (JSON serialize error with context)
```

## 11. Async Benchmarks

**Note**: Async operations performance with Tokio runtime.

```text
async/pipeline/success_path         → 829.17 ns/iter  (Async pipeline success)
async/pipeline/error_path           → 167.46 ns/iter  (Async pipeline error)
async/context_evaluation/lazy       → 829.54 ns/iter  (Lazy context in async)
async/context_evaluation/eager      → 873.05 ns/iter  (Eager context in async)
async/retry/transient_retry_1       → 902.75 ns/iter  (Retry with 1 transient error)
async/validation/sequential_3       → 169.45 ns/iter  (Sequential validation of 3 items)
```

## 12. Tower Integration

**Note**: Tower middleware layer performance overhead.

```text
tower/layer/baseline_raw_service    → 818.86 ns/iter  (Raw Tower service)
tower/layer/error_rail_layer_success → 817.68 ns/iter (ErrorRail layer - negligible overhead)
```

## 13. Core Traits

**Note**: Benchmarks for the extension traits that provide ErrorRail's fluent API.

```text
traits/result_ext/ctx_success         → 721.80 ns/iter  (Add context on success path)
traits/result_ext/ctx_error           → 122.80 ns/iter  (Add context on error path)
traits/result_ext/ctx_with_lazy_success → 723.26 ns/iter (Lazy context on success)
traits/result_ext/ctx_with_lazy_error   → 207.72 ns/iter (Lazy context on error)

traits/boxed_result_ext/ctx_boxed_chain      → 121.19 ns/iter (Add context to boxed error)
traits/boxed_result_ext/ctx_boxed_with_chain → 185.09 ns/iter (Add lazy context to boxed error)
traits/boxed_result_ext/ctx_boxed_depth_3     → 201.24 ns/iter (Deeply nested boxed context)

traits/with_error/fmap_error      → 68.86 ns/iter (Map error using WithError trait)
traits/with_error/to_result_first → 515.36 ps/iter (Instant conversion to Result)
traits/with_error/to_result_all   → 1.04 ns/iter (Full conversion to Result)

traits/into_error_context/from_static_str     → 2.06 ns/iter  (Zero-copy context from static str)
traits/into_error_context/from_string         → 59.47 ns/iter (Context from heap String)
traits/into_error_context/from_error_context  → 75.67 ns/iter (Identity conversion)
```

## 14. Formatting Benchmarks

**Note**: Performance of turning `ComposableError` into human-readable strings.

```text
formatting/error_chain/shallow   → 910.79 ns/iter (Format simple error chain)
formatting/error_chain/deep      → 2.18 µs/iter   (Format complex/nested error chain)

formatting/builder/default           → 1.55 µs/iter (Standard builder formatting)
formatting/builder/custom_separator  → 1.56 µs/iter (Builder with custom separator)
formatting/builder/hide_code         → 1.36 µs/iter (Formatting without error codes)
formatting/builder/cascaded          → 1.62 µs/iter (Indented cascade style)
formatting/builder/pretty            → 1.74 µs/iter (Multi-line tree style)

formatting/scaling/1   → 1.20 µs/iter
formatting/scaling/5   → 2.83 µs/iter
formatting/scaling/10  → 4.81 µs/iter
formatting/scaling/20  → 8.92 µs/iter

formatting/display/display_normal    → 917.48 ns/iter (Through Display trait)
formatting/display/display_alternate → 1.05 µs/iter   (Through alternate {:#} Display)
```

## 15. Fingerprint Benchmarks

**Note**: Performance of generating unique error fingerprints for tracking.

```text
fingerprint/basic/simple_u64   → 108.88 ns/iter (Raw u64 fingerprint)
fingerprint/basic/complex_u64  → 154.87 ns/iter (Complex context u64 fingerprint)
fingerprint/basic/simple_hex   → 203.28 ns/iter (Hex string fingerprint)
fingerprint/basic/complex_hex  → 245.12 ns/iter (Complex hex fingerprint)

fingerprint/config/default          → 125.88 ns/iter
fingerprint/config/exclude_message  → 87.89 ns/iter  (Skip message for performance)
fingerprint/config/include_metadata → 237.54 ns/iter (Include all metadata in hash)
fingerprint/config/minimal          → 88.92 ns/iter  (Minimal stable fingerprint)
fingerprint/config/config_hex       → 216.73 ns/iter (Custom config to hex)

fingerprint/consistency/compare_equal → 217.87 ns/iter (Time to compare two fingerprints)

fingerprint/scaling/1   → 168.21 ns/iter
fingerprint/scaling/5   → 491.99 ns/iter
fingerprint/scaling/10  → 864.97 ns/iter
fingerprint/scaling/20  → 2.10 µs/iter
```

## 16. Lazy Context In-depth

**Note**: Detailed benchmarks for lazy evaluation internals.

```text
lazy_context/creation/lazy_context_construct         → 258.03 ps/iter (Constructing the wrapper)
lazy_context/creation/lazy_group_context_construct   → 257.92 ps/iter

lazy_context/success_path/context_macro → 716.96 ns/iter (Using contextual macros)
lazy_context/success_path/format_eager  → 812.21 ns/iter (Comparison: eager formatting)
lazy_context/success_path/static_str    → 706.76 ns/iter (Comparison: static string)

lazy_context/error_path/context_macro   → 188.26 ns/iter
lazy_context/error_path/format_eager    → 183.60 ns/iter

lazy_context/group_macro/group_success        → 715.03 ns/iter (Group context macro)
lazy_context/group_macro/group_error          → 349.50 ns/iter
lazy_context/group_macro/composable_lazy_group → 151.71 ns/iter
```

## Quick Reference

| Category | Key Metric | Performance | Assessment |
|----------|------------|-------------|------------|
| Error Creation | 468 ns | Excellent | Fast error creation |
| Lazy Context | 621 ns vs 1.34µs | **2.2x faster** | Lazy evaluation wins |
| Context Depth (50) | 19.3µs | Linear scaling | Practical for deep nesting |
| Pipeline Success | 756 ns | 6% overhead | Minimal overhead |
| Validation | 978 ns | 8% overhead | Good abstraction tradeoff |
| Error Clone | 167 ns | Excellent | Cheap for async patterns |
| Serialization | 791 ns | Excellent | JSON serialization overhead |
| Retry (permanent) | 59 ns | **6.2x faster** | Smart error classification |
| Static str | 4.78 ns vs 61 ns | **13x faster** | Prefer static strings |
| Validation Batch (5000) | 5.14 M/sec | Excellent | High throughput validation |
| Tower Layer | 818 ns | Negligible | Minimal middleware cost |
| Async Pipeline | 829 ns | Excellent | Fast async error handling |

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
