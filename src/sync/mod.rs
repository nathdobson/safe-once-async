mod async_once_lock;

#[cfg(test)]
mod test;

pub use async_once_lock::AsyncRawOnceLock;

pub type AsyncOnceLock<T> = crate::async_once::AsyncOnce<async_once_lock::AsyncRawOnceLock, T>;
pub type AsyncLazyLock<T> = crate::async_lazy::AsyncLazy<async_once_lock::AsyncRawOnceLock, T>;
pub type AsyncStaticLock<T> = crate::async_static::AsyncStatic<async_once_lock::AsyncRawOnceLock, T>;
