mod async_fused_lock;

#[cfg(test)]
mod test;

use crate::detached::DetachedFuture;
pub use async_fused_lock::AsyncRawFusedLock;
use std::future::Future;
use std::pin::Pin;

pub type AsyncOnceLock<F> =
    crate::async_once::AsyncOnce<async_fused_lock::AsyncRawFusedLock, F>;
pub type AsyncLazyLock<F> =
    crate::async_lazy::AsyncLazy<async_fused_lock::AsyncRawFusedLock, F>;
