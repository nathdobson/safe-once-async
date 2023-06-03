use std::future::Future;
use crate::async_fused::{AsyncFused, AsyncFusedEntry, AsyncFusedGuard};
use crate::const_box::{ConstBox, ConstBoxFuture};
use crate::detached::detached;
use crate::raw::AsyncRawFused;
use crate::sync::AsyncStaticLock;
use crate::thunk::Thunk;

pub struct AsyncStatic<R: AsyncRawFused, T> {
    fused: AsyncFused<R, Thunk<T, ConstBoxFuture<T>>>,
}

impl<R: Send + Sync + AsyncRawFused, T: 'static + Sync + Send> AsyncStatic<R, T> where R::GuardMarker: Send {
    pub const fn new<Fu>(fu: Fu) -> Self where Fu: 'static + Send + Future<Output=T> {
        AsyncStatic {
            fused: AsyncFused::new(Thunk::new(ConstBox::pin(async move {
                detached(fu).await
            })))
        }
    }

    async fn lock(&self) -> AsyncFusedEntry<R, Thunk<T, ConstBoxFuture<T>>> {
        self.fused.write().await
    }

    pub async fn get(&self) -> &T {
        match self.lock().await {
            AsyncFusedEntry::Write(mut guard) => {
                guard.get_or_init().await;
                guard.fuse().get().unwrap()
            }
            AsyncFusedEntry::Read(x) => x.get().unwrap()
        }
    }
}

unsafe impl<R: Send + Sync + AsyncRawFused, T: Send + Sync> Sync for AsyncStatic<R, T> {}


#[tokio::test]
async fn test_static() {
    static FOO: AsyncStaticLock<usize> = AsyncStaticLock::new(async { return 2usize; });
    assert_eq!(FOO.get().await, &2usize);
}