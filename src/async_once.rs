use crate::async_fused::{AsyncFused, AsyncFusedEntry, AsyncFusedGuard};
// use crate::detached::{detached, Detached};
use std::cell::UnsafeCell;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::marker::{PhantomData, Unsize};
use std::mem;
use std::mem::MaybeUninit;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::pin::Pin;
use std::sync::{PoisonError, TryLockError};
use std::thread::panicking;
// use safe_once::cell::OnceCell;
use crate::detached::DetachedFuture;
use crate::raw::{AsyncRawFused, RawOnceState};
use crate::sync::AsyncOnceLock;
use crate::thunk::OptionThunk;

pub struct AsyncOnce<R: AsyncRawFused, F: DetachedFuture> {
    fused: AsyncFused<R, OptionThunk<F::Output, F>>,
}

pub enum AsyncOnceEntry<'a, R: AsyncRawFused, F: Unpin + DetachedFuture<Output = T>, T: 'static> {
    Vacant(AsyncOnceVacant<'a, R, F, T>),
    Occupied(AsyncOnceOccupied<'a, R, F, T>),
}

pub struct AsyncOnceVacant<'a, R: AsyncRawFused, F: Unpin + DetachedFuture<Output = T>, T> {
    guard: AsyncFusedGuard<'a, R, OptionThunk<T, F>>,
}

pub type AsyncOnceOccupied<'a, R, F, T> = <() as AsyncOnceOccupiedTrait>::Fut<'a, R, F, T>;

pub trait AsyncOnceOccupiedTrait {
    type Fut<'a, R: AsyncRawFused, F: 'a + Unpin + DetachedFuture<Output = T>, T: 'static>: 'a
        + Future<Output = &'a T>
    where
        Self: 'a,
        F: 'a;
    fn async_once_occupied<'a, R: AsyncRawFused, F: Unpin + DetachedFuture<Output = T>, T>(
        entry: AsyncFusedEntry<'a, R, OptionThunk<T, F>>,
    ) -> AsyncOnceOccupied<'a, R, F, T>;
}

impl AsyncOnceOccupiedTrait for () {
    type Fut<'a, R: AsyncRawFused, F: 'a + Unpin + DetachedFuture<Output = T>, T: 'static> =
        impl 'a + Future<Output = &'a T>;
    fn async_once_occupied<'a, R: AsyncRawFused, F: Unpin + DetachedFuture<Output = T>, T>(
        entry: AsyncFusedEntry<'a, R, OptionThunk<T, F>>,
    ) -> AsyncOnceOccupied<'a, R, F, T> {
        async move {
            match entry {
                AsyncFusedEntry::Write(mut w) => {
                    w.force().await;
                    let w = w.fuse();
                    w.get().unwrap()
                }
                AsyncFusedEntry::Read(r) => r.get().unwrap(),
            }
        }
    }
}

impl<'a, R: AsyncRawFused, F: Unpin + DetachedFuture<Output = T>, T: 'static>
    AsyncOnceVacant<'a, R, F, T>
{
    pub fn start(mut self, f: F) -> AsyncOnceOccupied<'a, R, F, T> {
        self.guard.start(f);
        <()>::async_once_occupied(AsyncFusedEntry::Write(self.guard))
    }
    pub fn start_detached(mut self, f: F) -> AsyncOnceOccupied<'a, R, F, T> {
        self.guard.start(f);
        <()>::async_once_occupied(AsyncFusedEntry::Write(self.guard))
    }
}

impl<R: AsyncRawFused, F: Unpin + DetachedFuture<Output = T>, T: 'static> AsyncOnce<R, F> {
    pub const fn new() -> Self {
        AsyncOnce {
            fused: AsyncFused::new(OptionThunk::new()),
        }
    }
    pub const fn poisoned() -> Self {
        AsyncOnce {
            fused: AsyncFused::poisoned(OptionThunk::new()),
        }
    }
    pub fn try_lock(&self) -> Option<AsyncOnceEntry<R, F, T>> {
        Some(self.raw_lock(self.fused.try_write()?))
    }
    fn raw_lock<'a>(
        &'a self,
        raw: AsyncFusedEntry<'a, R, OptionThunk<T, F>>,
    ) -> AsyncOnceEntry<'a, R, F, T> {
        match raw {
            AsyncFusedEntry::Write(w) => {
                if w.started() {
                    AsyncOnceEntry::Occupied(<()>::async_once_occupied(AsyncFusedEntry::Write(w)))
                } else {
                    AsyncOnceEntry::Vacant(AsyncOnceVacant { guard: w })
                }
            }
            AsyncFusedEntry::Read(r) => {
                AsyncOnceEntry::Occupied(<()>::async_once_occupied(AsyncFusedEntry::Read(r)))
            }
        }
    }
    pub async fn lock(&self) -> AsyncOnceEntry<R, F, T> {
        self.raw_lock(self.fused.write().await)
    }
    pub async fn get_or_init_fn(&self, f: impl FnOnce() -> F) -> &T {
        let occupied = match self.lock().await {
            AsyncOnceEntry::Vacant(x) => x.start(f()),
            AsyncOnceEntry::Occupied(x) => x,
        };
        occupied.await
    }
    pub async fn get_or_init<'a>(&'a self, f: F) -> &'a T {
        self.get_or_init_fn(|| f).await
    }
    pub async fn get_or_init_detached(&self, f: impl FnOnce() -> F) -> &T {
        let occupied = match self.lock().await {
            AsyncOnceEntry::Vacant(x) => x.start_detached(f()),
            AsyncOnceEntry::Occupied(x) => x,
        };
        occupied.await
    }
}

impl<R: AsyncRawFused, F: Unpin + DetachedFuture<Output = T>, T: 'static> Default
    for AsyncOnce<R, F>
{
    fn default() -> Self {
        AsyncOnce::new()
    }
}

#[cfg(test)]
mod test {
    use crate::detached::{spawn_transparent, JoinTransparent};
    use crate::sync::AsyncOnceLock;

    #[tokio::test]
    async fn test_async_once() {
        let foo = AsyncOnceLock::<JoinTransparent<usize>>::new();
        assert_eq!(*foo.get_or_init(spawn_transparent(async { 2 })).await, 2);
        assert_eq!(*foo.get_or_init(spawn_transparent(async { 3 })).await, 2);
    }
}

impl<R: AsyncRawFused, F: Unpin + DetachedFuture> Debug for AsyncOnce<R, F>
where
    F::Output: 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.try_lock() {
            None => write!(f, "locked"),
            Some(AsyncOnceEntry::Vacant(v)) => write!(f, "vacant"),
            Some(AsyncOnceEntry::Occupied(v)) => write!(f, "occupied"),
        }
    }
}
