
mod async_once_cell;

pub use async_once_cell::AsyncRawOnceCell;

pub type AsyncOnceCell<T> = crate::async_once::AsyncOnce<async_once_cell::AsyncRawOnceCell, T>;
pub type AsyncLazyCell<T> = crate::async_lazy::AsyncLazy<async_once_cell::AsyncRawOnceCell, T>;
