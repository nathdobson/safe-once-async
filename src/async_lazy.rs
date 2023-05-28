use std::cell::{Cell, UnsafeCell};
use std::default::default;
use std::future::{Future, poll_fn};
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::thread::panicking;
use futures::future::BoxFuture;
use futures::FutureExt;
use tokio::task::JoinHandle;
use crate::async_fused::{AsyncFused, AsyncFusedEntry, AsyncFusedGuard};
use crate::const_box::{ConstBox, ConstBoxFuture};
use crate::detached::{detached, detached_lazy, DetachedLazy};
// use crate::pure_future::PureFuture;
// use crate::async_once::{AsyncOnce, AsyncOnceEntry};
use crate::raw::{AsyncRawOnce, RawOnceState};
// use crate::spawned_future::SpawnedFuture;
use crate::thunk::{OptionThunk, Thunk};

pub struct AsyncLazy<R: AsyncRawOnce, T> {
    fused: AsyncFused<R, Thunk<T, BoxFuture<'static, T>>>,
}

impl<R: AsyncRawOnce, T: 'static + Send> AsyncLazy<R, T> {
    pub fn new<Fu>(fu: Fu) -> Self where Fu: 'static + Send + Future<Output=T> {
        AsyncLazy {
            fused: AsyncFused::new(Thunk::new(async move {
                detached(fu).await
            }.boxed()))
        }
    }

    pub async fn get(&self) -> &T {
        match self.fused.lock().await {
            AsyncFusedEntry::Write(mut guard) => {
                guard.get_or_init().await;
                guard.init().get().unwrap()
            }
            AsyncFusedEntry::Read(x) => x.get().unwrap()
        }
    }
}

impl<R: AsyncRawOnce, T> From<T> for AsyncLazy<R, T> {
    fn from(value: T) -> Self {
        AsyncLazy { fused: AsyncFused::inited(Thunk::new_value(value)) }
    }
}