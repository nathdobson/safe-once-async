mod async_fused_cell;
mod condvar;

use crate::detached::DetachedFuture;
pub use async_fused_cell::AsyncRawFusedCell;
use futures::future::LocalBoxFuture;
use std::future::Future;
use std::pin::Pin;

pub type AsyncOnceCell<F> = crate::async_once::AsyncOnce<async_fused_cell::AsyncRawFusedCell, F>;

pub type AsyncLazyCell<F> = crate::async_lazy::AsyncLazy<async_fused_cell::AsyncRawFusedCell, F>;
