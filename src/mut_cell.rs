use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct MutCell<T>(T);

impl<T> AsMut<T> for MutCell<T> {
    fn as_mut(&mut self) -> &mut T { &mut self.0 }
}

impl<T> MutCell<T> {
    pub const fn new(x: T) -> Self { MutCell(x) }
    pub fn into_inner(self) -> T { self.0 }
}

impl<T> From<T> for MutCell<T> {
    fn from(value: T) -> Self { MutCell(value) }
}

unsafe impl<T: Send> Sync for MutCell<T> {}

unsafe impl<T: Send> Send for MutCell<T> {}

impl<T: Future> Future for MutCell<T> {
    type Output = T::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().0).poll(cx) }
    }
}