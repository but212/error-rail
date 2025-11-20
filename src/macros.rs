#[macro_export]
macro_rules! context {
    ($($arg:tt)*) => {
        $crate::error::types::LazyContext::new(move || format!($($arg)*))
    };
}
