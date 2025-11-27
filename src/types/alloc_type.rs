#[cfg(feature = "std")]
pub type Cow<'a, B> = std::borrow::Cow<'a, B>;
#[cfg(not(feature = "std"))]
pub type Cow<'a, B> = alloc::borrow::Cow<'a, B>;

#[cfg(feature = "std")]
pub type Box<T> = std::boxed::Box<T>;
#[cfg(not(feature = "std"))]
pub type Box<T> = alloc::boxed::Box<T>;

#[cfg(feature = "std")]
pub type Vec<T> = std::vec::Vec<T>;
#[cfg(not(feature = "std"))]
pub type Vec<T> = alloc::vec::Vec<T>;

#[cfg(feature = "std")]
pub type String = std::string::String;
#[cfg(not(feature = "std"))]
pub type String = alloc::string::String;
