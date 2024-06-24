mod async_fused_lock;

#[cfg(test)]
mod test;

pub use async_fused_lock::AsyncRawFusedLock;

pub type AsyncOnceLock<T> = crate::async_once::AsyncOnce<async_fused_lock::AsyncRawFusedLock, T>;
pub type AsyncLazyLock<T> = crate::async_lazy::AsyncLazy<async_fused_lock::AsyncRawFusedLock, T>;
// pub type AsyncStaticLock<T> =
//     crate::async_static::AsyncStatic<async_fused_lock::AsyncRawFusedLock, T>;
