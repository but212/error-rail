#[macro_export]
macro_rules! context {
    ($($arg:tt)*) => {
        $crate::types::LazyContext::new(move || format!($($arg)*))
    };
}

#[macro_export]
macro_rules! location {
    () => {
        $crate::types::ErrorContext::location(file!(), line!())
    };
}

#[macro_export]
macro_rules! tag {
    ($tag:expr) => {
        $crate::types::ErrorContext::tag($tag)
    };
}

#[macro_export]
macro_rules! metadata {
    ($key:expr, $value:expr) => {
        $crate::types::ErrorContext::metadata($key, $value)
    };
}
