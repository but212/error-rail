use smallvec::SmallVec;

pub mod composable_error;
pub mod error_context;
pub mod lazy_context;

pub use composable_error::*;
pub use error_context::*;
pub use lazy_context::*;

pub type ErrorVec<E> = SmallVec<[E; 8]>;
pub type ComposableResult<T, E, C = u32> = Result<T, ComposableError<E, C>>;
pub type BoxedComposableError<E, C = u32> = Box<ComposableError<E, C>>;
pub type BoxedComposableResult<T, E, C = u32> = Result<T, BoxedComposableError<E, C>>;
