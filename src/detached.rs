// use std::future::Future;
// use std::panic::resume_unwind;
// use std::pin::Pin;
// use std::task::{Context, Poll};
// use tokio::task::JoinHandle;
//
// pub struct Detached<T>(JoinHandle<T>);
//
// impl<T> Unpin for Detached<T> {}
//
// pub fn detached<T: 'static + Send>(x: impl 'static + Send + Future<Output = T>) -> Detached<T> {
//     Detached(tokio::spawn(x))
// }
//
// impl<T> Drop for Detached<T> {
//     fn drop(&mut self) {
//         self.0.abort();
//     }
// }
//
// impl<T> Future for Detached<T> {
//     type Output = T;
//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         match Pin::new(&mut self.get_mut().0).poll(cx) {
//             Poll::Ready(Ok(x)) => Poll::Ready(x),
//             Poll::Ready(Err(e)) => resume_unwind(e.into_panic()),
//             Poll::Pending => Poll::Pending,
//         }
//     }
// }
// //
// // pub type DetachedLazy<T: 'static + Send, F: 'static + Send + Future<Output=T>> = impl 'static + Send + Future<Output=T>;
// //
// // const fn detached_lazy<T: 'static + Send, F: 'static + Send + Future<Output=T>>(x: F) -> DetachedLazy<T, F> {
// //     async move {
// //         let x: F::Output = detached(x).await;
// //         x
// //     }
// // }
//
// pub enum DetachedLazy<T, Fu> {
//     Future(Option<Fu>),
//     Detached(Detached<T>),
// }
//
// pub const fn detached_lazy<T: 'static + Send, F: 'static + Send + Future<Output = T>>(
//     x: F,
// ) -> DetachedLazy<T, F> {
//     DetachedLazy::Future(Some(x))
// }
//
// impl<T, Fu> Unpin for DetachedLazy<T, Fu> {}
//
// impl<Fu: 'static + Send + Unpin + Future> Future for DetachedLazy<Fu::Output, Fu>
// where
//     Fu::Output: Send,
// {
//     type Output = Fu::Output;
//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         let this: &mut Self = self.get_mut();
//         loop {
//             match this {
//                 DetachedLazy::Future(x) => {
//                     let x = x.take();
//                     *this = DetachedLazy::Detached(detached(x.unwrap()));
//                 }
//                 DetachedLazy::Detached(x) => {
//                     return Pin::new(x).poll(cx);
//                 }
//             }
//         }
//     }
// }

use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

pub trait DetachedFuture: Future {}

impl<F: ?Sized + DetachedFuture> DetachedFuture for Pin<Box<F>> {}

#[cfg(feature = "tokio-rt")]
impl<T> DetachedFuture for tokio::task::JoinHandle<T> {}

impl<Fu: Future, F: FnOnce(Fu::Output) -> T, T> DetachedFuture for futures::future::Map<Fu, F> {}

#[cfg(feature = "tokio-rt")]
pub struct JoinTransparent<T> {
    inner: tokio::task::JoinHandle<T>,
}

#[cfg(feature = "tokio-rt")]
impl<T> Unpin for JoinTransparent<T> {}

#[cfg(feature = "tokio-rt")]
impl<T> Future for JoinTransparent<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.as_mut().inner)
            .poll(cx)
            .map(|x| x.unwrap())
    }
}

#[cfg(feature = "tokio-rt")]
impl<T> DetachedFuture for JoinTransparent<T> {}

#[cfg(feature = "tokio-rt")]
impl<T> Drop for JoinTransparent<T> {
    fn drop(&mut self) {
        self.inner.abort();
    }
}

#[cfg(feature = "tokio-rt")]
pub fn spawn_transparent<Fu: 'static + Send + Future>(f: Fu) -> JoinTransparent<Fu::Output>
where
    Fu::Output: Send,
{
    JoinTransparent {
        inner: tokio::spawn(f),
    }
}

#[cfg(feature = "tokio-rt")]
pub fn spawn_local_transparent<Fu: 'static + Future>(f: Fu) -> JoinTransparent<Fu::Output> {
    JoinTransparent {
        inner: tokio::task::spawn_local(f),
    }
}
