use smallvec::SmallVec;

pub mod composable_error;
pub mod error_context;
pub mod lazy_context;

pub use composable_error::*;
pub use error_context::*;
pub use lazy_context::*;

pub type ErrorVec<E> = SmallVec<[E; 8]>;
pub type ComposableResult<T, E> = Result<T, ComposableError<E>>;
pub type BoxedComposableError<E> = Box<ComposableError<E>>;
pub type BoxedComposableResult<T, E> = Result<T, BoxedComposableError<E>>;
