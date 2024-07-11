use crate::async_fused::{AsyncFused, AsyncFusedEntry, AsyncFusedGuard};
// use crate::const_box::{ConstBox, ConstBoxFuture};
// use crate::detached::{detached, detached_lazy, DetachedLazy};
use crate::detached::DetachedFuture;
use futures::future::BoxFuture;
use futures::FutureExt;
use std::cell::{Cell, UnsafeCell};
use std::future::{poll_fn, Future};
use std::marker::Unsize;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::thread::panicking;
use tokio::task::JoinHandle;
// use crate::pure_future::PureFuture;
// use crate::async_once::{AsyncOnce, AsyncOnceEntry};
use crate::raw::{AsyncRawFused, RawOnceState};
// use crate::spawned_future::SpawnedFuture;
use crate::raw::AsyncRawFusedSync;
use crate::thunk::{OptionThunk, Thunk};

pub struct AsyncLazy<R: AsyncRawFused, F: DetachedFuture> {
    fused: AsyncFused<R, Thunk<F::Output, F>>,
}

impl<R: AsyncRawFused, F: Unpin + DetachedFuture<Output = T>, T: 'static + Send>
    AsyncLazy<R, F>
{
    pub fn new(f: F) -> Self {
        AsyncLazy {
            fused: AsyncFused::new(Thunk::new(f)),
        }
    }

    pub async fn get(&self) -> &T {
        match self.fused.write().await {
            AsyncFusedEntry::Write(mut guard) => {
                guard.get_or_init().await;
                guard.fuse().get().unwrap()
            }
            AsyncFusedEntry::Read(x) => x.get().unwrap(),
        }
    }
}

impl<R: AsyncRawFused, F, T: 'static + Send> AsyncLazy<R, F>
where
    F: Send + Unpin + DetachedFuture<Output = T>,
{
    fn get_is_send(&self) -> impl Send + Future<Output = &T>
    where
        R: AsyncRawFusedSync,
        T: Send + Sync,
    {
        self.get()
    }
}
