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
    let futures: Vec<_> = validations.into_iter().collect();
    let mut results = Vec::with_capacity(futures.len());

    // Sequential execution (parallel requires runtime, deferred to Phase 3)
    for fut in futures {
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
    let mut current = Validation::Valid(initial);

    for validator in validators {
        match current {
            Validation::Valid(v) => {
                current = validator(v).await;
            }
            invalid => return invalid,
        }
    }

    current
}

/// Collects validation results into a single `Validation`.
///
/// - If all results are `Valid`, returns `Valid(Vec<T>)` with all values.
/// - If any result is `Invalid`, returns `Invalid` with all accumulated errors.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_all_valid() {
        let results = vec![
            Validation::<&str, _>::Valid(1),
            Validation::Valid(2),
            Validation::Valid(3),
        ];
        let collected = collect_validation_results(results);
        assert_eq!(collected, Validation::Valid(vec![1, 2, 3]));
    }

    #[test]
    fn collect_some_invalid() {
        let results = vec![
            Validation::Valid(1),
            Validation::invalid("error 1"),
            Validation::Valid(3),
            Validation::invalid("error 2"),
        ];
        let collected: Validation<&str, Vec<i32>> = collect_validation_results(results);
        match collected {
            Validation::Invalid(errs) => {
                assert_eq!(errs.len(), 2);
            }
            _ => panic!("expected Invalid"),
        }
    }
}
