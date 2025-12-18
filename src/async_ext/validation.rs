//! Async validation utilities.
//!
//! Provides functions for running multiple async validations and collecting
//! all errors, mirroring the sync `Validation` type's accumulating behavior.

use core::future::Future;

use crate::types::alloc_type::Vec;
use crate::Validation;

/// Runs multiple async validations sequentially and collects all errors.
///
/// Unlike `Result` which short-circuits on the first error, this function
/// executes all validations and accumulates any errors that occur.
///
/// # Note
///
/// Validations are executed **sequentially** (not in parallel) to maintain
/// runtime neutrality. For parallel execution, use a runtime-specific
/// combinator like `futures::join_all` or `tokio::join!`.
///
/// # Example
///
/// ```rust,ignore
/// use error_rail::prelude_async::*;
///
/// async fn validate_user(user: &User) -> Validation<ValidationError, ()> {
///     validate_all_async([
///         validate_email(&user.email),
///         validate_username(&user.username),
///         check_user_exists(&user.id),
///     ])
///     .await
///     .map(|_| ())
/// }
/// ```
pub async fn validate_all_async<T, E, Fut, I>(validations: I) -> Validation<E, Vec<T>>
where
    I: IntoIterator<Item = Fut>,
    Fut: Future<Output = Validation<E, T>>,
{
    let mut results = Vec::new();

    for fut in validations {
        results.push(fut.await);
    }

    collect_validation_results(results)
}

/// Runs async validations sequentially, where each validation depends on
/// the previous result.
///
/// Stops at the first invalid result and returns accumulated errors.
///
/// # Example
///
/// ```rust,ignore
/// use error_rail::prelude_async::*;
///
/// async fn validate_order(order_id: u64) -> Validation<OrderError, Order> {
///     validate_seq_async(
///         order_id,
///         [
///             |id| async move { fetch_order(id).await },
///             |order| async move { validate_inventory(&order).await },
///             |order| async move { validate_payment(&order).await },
///         ],
///     )
///     .await
/// }
/// ```
pub async fn validate_seq_async<T, E, F, Fut>(
    initial: T,
    validators: impl IntoIterator<Item = F>,
) -> Validation<E, T>
where
    F: FnOnce(T) -> Fut,
    Fut: Future<Output = Validation<E, T>>,
{
    let mut current = initial;

    for validator in validators {
        match validator(current).await {
            Validation::Valid(v) => current = v,
            invalid => return invalid,
        }
    }

    Validation::Valid(current)
}

/// Collects validation results into a single `Validation`.
///
/// This helper function aggregates multiple validation results, accumulating
/// all errors rather than short-circuiting on the first failure.
///
/// # Behavior
///
/// - If **all** results are `Valid`, returns `Valid(Vec<T>)` containing all values
///   in their original order.
/// - If **any** result is `Invalid`, returns `Invalid` with all accumulated errors
///   from all invalid results combined.
///
/// # Arguments
///
/// * `results` - A vector of `Validation<E, T>` results to collect
///
/// # Returns
///
/// A single `Validation<E, Vec<T>>` that either contains all success values
/// or all accumulated errors.
fn collect_validation_results<T, E>(results: Vec<Validation<E, T>>) -> Validation<E, Vec<T>> {
    let mut errors: Vec<E> = Vec::new();
    let mut values = Vec::with_capacity(results.len());

    for result in results {
        match result {
            Validation::Valid(v) => values.push(v),
            Validation::Invalid(errs) => errors.extend(errs),
        }
    }

    if errors.is_empty() {
        Validation::Valid(values)
    } else {
        Validation::invalid_many(errors)
    }
}
