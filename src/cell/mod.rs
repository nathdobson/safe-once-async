#[cfg(feature = "cell")]
mod once_cell;

#[cfg(feature = "cell")]
pub use once_cell::*;
#[cfg(feature = "cell")]
use crate::lazy::Lazy;
#[cfg(feature = "cell")]
use crate::once::Once;

#[cfg(feature = "cell")]
pub type OnceCell<T> = Once<RawOnceCell, T>;
#[cfg(feature = "cell")]
pub type LazyCell<T, F = fn() -> T> = Lazy<RawOnceCell, T, F>;

#[cfg(feature = "async_cell")]
mod async_once_cell;

#[cfg(feature = "async_cell")]
use crate::async_lazy::AsyncLazy;
#[cfg(feature = "async_cell")]
use crate::cell::async_once_cell::AsyncRawOnceCell;

#[cfg(feature = "async_cell")]
pub type AsyncOnceCell<T> = Once<AsyncRawOnceCell, T>;
#[cfg(feature = "async_cell")]
pub type AsyncLazyCell<T, F> = AsyncLazy<AsyncRawOnceCell, T, F>;