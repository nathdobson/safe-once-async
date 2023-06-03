
mod async_fused_cell;

pub use async_fused_cell::AsyncRawFusedCell;

pub type AsyncOnceCell<T> = crate::async_once::AsyncOnce<async_fused_cell::AsyncRawFusedCell, T>;
pub type AsyncLazyCell<T> = crate::async_lazy::AsyncLazy<async_fused_cell::AsyncRawFusedCell, T>;
