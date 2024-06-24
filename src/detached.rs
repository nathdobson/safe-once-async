use std::future::Future;
use std::panic::resume_unwind;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::task::JoinHandle;

pub struct Detached<T>(JoinHandle<T>);

impl<T> Unpin for Detached<T> {}

pub fn detached<T: 'static + Send>(x: impl 'static + Send + Future<Output = T>) -> Detached<T> {
    Detached(tokio::spawn(x))
}

impl<T> Drop for Detached<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

impl<T> Future for Detached<T> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.get_mut().0).poll(cx) {
            Poll::Ready(Ok(x)) => Poll::Ready(x),
            Poll::Ready(Err(e)) => resume_unwind(e.into_panic()),
            Poll::Pending => Poll::Pending,
        }
    }
}
//
// pub type DetachedLazy<T: 'static + Send, F: 'static + Send + Future<Output=T>> = impl 'static + Send + Future<Output=T>;
//
// const fn detached_lazy<T: 'static + Send, F: 'static + Send + Future<Output=T>>(x: F) -> DetachedLazy<T, F> {
//     async move {
//         let x: F::Output = detached(x).await;
//         x
//     }
// }

pub enum DetachedLazy<T, Fu> {
    Future(Option<Fu>),
    Detached(Detached<T>),
}

pub const fn detached_lazy<T: 'static + Send, F: 'static + Send + Future<Output = T>>(
    x: F,
) -> DetachedLazy<T, F> {
    DetachedLazy::Future(Some(x))
}

impl<T, Fu> Unpin for DetachedLazy<T, Fu> {}

impl<Fu: 'static + Send + Unpin + Future> Future for DetachedLazy<Fu::Output, Fu>
where
    Fu::Output: Send,
{
    type Output = Fu::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this: &mut Self = self.get_mut();
        loop {
            match this {
                DetachedLazy::Future(x) => {
                    let x = x.take();
                    *this = DetachedLazy::Detached(detached(x.unwrap()));
                }
                DetachedLazy::Detached(x) => {
                    return Pin::new(x).poll(cx);
                }
            }
        }
    }
}
