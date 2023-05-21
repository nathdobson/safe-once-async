#[cfg(feature = "lock")]
mod once_lock;
#[cfg(feature = "lock")]
mod state;
#[cfg(feature = "lock")]
mod thread_id;
#[cfg(feature = "async_lock")]
mod async_once_lock;

#[cfg(test)]
mod test;

#[cfg(feature = "lock")]
pub use once_lock::*;
use crate::const_box::ConstBoxFuture;

#[cfg(feature = "lock")]
pub type OnceLock<T> = crate::once::Once<RawOnceLock, T>;
#[cfg(feature = "lock")]
pub type LazyLock<T, F = fn() -> T> = crate::lazy::Lazy<RawOnceLock, T, F>;

#[cfg(feature = "async_lock")]
pub type AsyncOnceLock<T> = crate::once::Once<async_once_lock::AsyncRawOnceLock, T>;

#[cfg(feature = "async_lock")]
pub type AsyncLazyLock<T, F> = crate::async_lazy::AsyncLazy<async_once_lock::AsyncRawOnceLock, T, F>;

#[cfg(feature = "async_lock")]
pub type AsyncLazyStatic<T> = crate::async_lazy::AsyncLazy<async_once_lock::AsyncRawOnceLock, T, ConstBoxFuture<T>>;

