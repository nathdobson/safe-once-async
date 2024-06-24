use crate::async_fused::{AsyncFused, AsyncFusedEntry, AsyncFusedGuard};
// use crate::const_box::{ConstBox, ConstBoxFuture};
use crate::detached::{detached, detached_lazy, DetachedLazy};
use futures::future::BoxFuture;
use futures::FutureExt;
use std::cell::{Cell, UnsafeCell};
use std::future::{poll_fn, Future};
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

pub struct AsyncLazy<R: AsyncRawFused, T> {
    fused: AsyncFused<R, Thunk<T, BoxFuture<'static, T>>>,
}

impl<R: AsyncRawFused, T: 'static + Send> AsyncLazy<R, T> {
    pub fn new<Fu>(fu: Fu) -> Self
    where
        Fu: 'static + Send + Future<Output = T>,
    {
        AsyncLazy {
            fused: AsyncFused::new(Thunk::new(async move { detached(fu).await }.boxed())),
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

    fn get_is_send(&self) -> impl Send + Future<Output = &T>
    where
        R: AsyncRawFusedSync,
        T: Send + Sync,
    {
        self.get()
    }
}

impl<R: AsyncRawFused, T> From<T> for AsyncLazy<R, T> {
    fn from(value: T) -> Self {
        AsyncLazy {
            fused: AsyncFused::new_read(Thunk::new_value(value)),
        }
    }
}
