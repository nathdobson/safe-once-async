use std::cell::Cell;
use std::future::Future;
use crate::const_box::{ConstBox, ConstBoxFuture};
use crate::once::Once;
use crate::raw::AsyncRawOnce;
use crate::RawOnce;

pub struct AsyncLazy<R: RawOnce, T, F> {
    once: Once<R, T>,
    init: Cell<Option<F>>,
}

impl<R: RawOnce, T, F: Future<Output=T>> AsyncLazy<R, T, F> {
    pub const fn new(init: F) -> Self {
        AsyncLazy { once: Once::new(), init: Cell::new(Some(init)) }
    }
    pub async fn get(&self) -> &T where R: AsyncRawOnce {
        self.once.get_or_init_async(async move {
            self.init.take().unwrap().await
        }).await
    }
}

impl<R: RawOnce, T> AsyncLazy<R, T, ConstBoxFuture<T>> {
    pub const fn new_static<F: Future<Output=T> + Send + 'static>(x: F) -> Self {
        let x: ConstBoxFuture<T> = ConstBox::pin(x);
        Self::new(x)
    }
}

unsafe impl<R: RawOnce, T, F> Send for AsyncLazy<R, T, F> where R: Send, F: Send, T: Send {}

unsafe impl<R: RawOnce, T, F> Sync for AsyncLazy<R, T, F> where Once<R, T>: Sync {}

