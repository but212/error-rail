use crate::types::BoxedComposableResult;
use crate::types::composable_error::ComposableError;
use crate::validation::core::Validation;

#[inline]
pub fn validation_to_result<T, E>(validation: Validation<E, T>) -> Result<T, E>
where
    E: Clone,
{
    match validation {
        Validation::Valid(value) => Ok(value),
        Validation::Invalid(errors) => {
            // Take the first error, or create a default if somehow empty
            Err(errors
                .into_iter()
                .next()
                .expect("Validation::Invalid should contain at least one error"))
        }
    }
}

#[inline]
pub fn result_to_validation<T, E>(result: Result<T, E>) -> Validation<E, T>
where
    E: Clone,
    T: Clone,
{
    match result {
        Ok(value) => Validation::Valid(value),
        Err(error) => Validation::invalid(error),
    }
}

#[inline]
pub fn composable_to_core<E>(composable: ComposableError<E>) -> E {
    composable.core_error
}

#[inline]
pub fn core_to_composable<E>(error: E) -> ComposableError<E> {
    error.into()
}

#[inline]
pub fn flatten_composable_result<T, E>(result: Result<T, ComposableError<E>>) -> Result<T, E> {
    result.map_err(composable_to_core)
}

#[inline]
#[allow(clippy::result_large_err)]
pub fn wrap_in_composable_result<T, E>(result: Result<T, E>) -> Result<T, ComposableError<E>> {
    result.map_err(core_to_composable)
}

#[inline]
pub fn wrap_in_composable_result_boxed<T, E>(result: Result<T, E>) -> BoxedComposableResult<T, E> {
    result.map_err(|e| Box::new(core_to_composable(e)))
}

#[inline]
pub fn collect_errors<E, I>(errors: I) -> Validation<E, ()>
where
    E: Clone,
    I: IntoIterator<Item = E>,
{
    let error_vec: Vec<E> = errors.into_iter().collect();
    if error_vec.is_empty() {
        Validation::Valid(())
    } else {
        Validation::invalid_many(error_vec)
    }
}

pub fn split_validation_errors<T, E>(validation: Validation<E, T>) -> Vec<Result<T, E>>
where
    T: Clone,
    E: Clone,
{
    match validation {
        Validation::Valid(value) => vec![Ok(value)],
        Validation::Invalid(errors) => errors.into_iter().map(|e| Err(e)).collect(),
    }
}
