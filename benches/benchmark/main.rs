use criterion::criterion_main;

mod common;
mod context;
mod conversions;
mod core;
mod features;
mod memory;
mod pipeline;
mod retry;
mod scaling;
mod scenarios;
mod validation;

mod async_ops;
mod tower;

// Main benchmark groups based on available features
#[cfg(all(feature = "std", feature = "serde"))]
criterion_main!(
    core::core_benches,
    retry::retry_benches,
    conversions::conversion_benches,
    context::context_benches,
    scaling::scaling_benches,
    pipeline::pipeline_benches,
    validation::validation_benches,
    scenarios::real_world_benches,
    memory::memory_benches,
    features::std_benches,
    features::serde_benches,
    async_ops::async_ops_benches,
    tower::tower_benches,
);

#[cfg(all(feature = "std", not(feature = "serde")))]
criterion_main!(
    core::core_benches,
    retry::retry_benches,
    conversions::conversion_benches,
    context::context_benches,
    scaling::scaling_benches,
    pipeline::pipeline_benches,
    validation::validation_benches,
    scenarios::real_world_benches,
    memory::memory_benches,
    features::std_benches,
    async_ops::async_ops_benches,
    tower::tower_benches,
);

#[cfg(all(feature = "serde", not(feature = "std")))]
criterion_main!(
    core::core_benches,
    retry::retry_benches,
    conversions::conversion_benches,
    context::context_benches,
    scaling::scaling_benches,
    pipeline::pipeline_benches,
    validation::validation_benches,
    scenarios::real_world_benches,
    memory::memory_benches,
    features::serde_benches,
    async_ops::async_ops_benches,
    tower::tower_benches,
);

#[cfg(not(any(feature = "std", feature = "serde")))]
criterion_main!(
    core::core_benches,
    retry::retry_benches,
    conversions::conversion_benches,
    context::context_benches,
    scaling::scaling_benches,
    pipeline::pipeline_benches,
    validation::validation_benches,
    scenarios::real_world_benches,
    memory::memory_benches,
    async_ops::async_ops_benches,
    tower::tower_benches,
);
