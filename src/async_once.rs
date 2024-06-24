use crate::async_fused::{AsyncFused, AsyncFusedEntry, AsyncFusedGuard};
use crate::detached::{detached, Detached};
use std::cell::UnsafeCell;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::marker::PhantomData;
use std::mem;
use std::mem::MaybeUninit;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::sync::{PoisonError, TryLockError};
use std::thread::panicking;
use tokio::task::JoinHandle;
// use safe_once::cell::OnceCell;
use crate::raw::{AsyncRawFused, RawOnceState};
use crate::sync::AsyncOnceLock;
use crate::thunk::OptionThunk;

pub struct AsyncOnce<R: AsyncRawFused, T> {
    fused: AsyncFused<R, OptionThunk<T, Detached<T>>>,
}

pub enum AsyncOnceEntry<'a, R: AsyncRawFused, T: 'static> {
    Vacant(AsyncOnceVacant<'a, R, T>),
    Occupied(AsyncOnceOccupied<'a, R, T>),
}

pub struct AsyncOnceVacant<'a, R: AsyncRawFused, T> {
    guard: AsyncFusedGuard<'a, R, OptionThunk<T, Detached<T>>>,
}

pub type AsyncOnceOccupied<'a, R, T> = <() as AsyncOnceOccupiedTrait>::Fut<'a, R, T>;

pub trait AsyncOnceOccupiedTrait {
    type Fut<'a, R: AsyncRawFused, T: 'static>: 'a + Future<Output = &'a T>
    where
        Self: 'a;
    fn async_once_occupied<'a, R: AsyncRawFused, T>(
        entry: AsyncFusedEntry<'a, R, OptionThunk<T, Detached<T>>>,
    ) -> AsyncOnceOccupied<'a, R, T>;
}

impl AsyncOnceOccupiedTrait for () {
    type Fut<'a, R: AsyncRawFused, T: 'static> = impl 'a + Future<Output = &'a T>;
    fn async_once_occupied<'a, R: AsyncRawFused, T>(
        entry: AsyncFusedEntry<'a, R, OptionThunk<T, Detached<T>>>,
    ) -> AsyncOnceOccupied<'a, R, T> {
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

impl<'a, R: AsyncRawFused, T: 'static + Send> AsyncOnceVacant<'a, R, T> {
    pub fn start<Fu: 'static + Send + Future<Output = T>>(
        mut self,
        fu: Fu,
    ) -> AsyncOnceOccupied<'a, R, T> {
        self.guard.start(detached(fu));
        <()>::async_once_occupied(AsyncFusedEntry::Write(self.guard))
    }
    pub fn start_detached(mut self, fu: Detached<T>) -> AsyncOnceOccupied<'a, R, T> {
        self.guard.start(fu);
        <()>::async_once_occupied(AsyncFusedEntry::Write(self.guard))
    }
}

impl<R: AsyncRawFused, T: 'static + Send> AsyncOnce<R, T> {
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
    pub async fn lock(&self) -> AsyncOnceEntry<R, T> {
        match self.fused.write().await {
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
    pub async fn get_or_init_fn<Fu>(&self, f: impl FnOnce() -> Fu) -> &Fu::Output
    where
        Fu: 'static + Send + Future<Output = T>,
    {
        let occupied = match self.lock().await {
            AsyncOnceEntry::Vacant(x) => x.start(f()),
            AsyncOnceEntry::Occupied(x) => x,
        };
        occupied.await
    }
    pub async fn get_or_init<'a, Fu>(&'a self, f: Fu) -> &'a Fu::Output
    where
        Fu: 'static + Send + Future<Output = T>,
    {
        self.get_or_init_fn(|| f).await
    }
    pub async fn get_or_init_detached(&self, f: impl FnOnce() -> Detached<T>) -> &T {
        let occupied = match self.lock().await {
            AsyncOnceEntry::Vacant(x) => x.start_detached(f()),
            AsyncOnceEntry::Occupied(x) => x,
        };
        occupied.await
    }
}

impl<R: AsyncRawFused, T> From<T> for AsyncOnce<R, T> {
    fn from(value: T) -> Self {
        AsyncOnce {
            fused: AsyncFused::new_read(OptionThunk::Value(value)),
        }
    }
}

impl<R: AsyncRawFused, T: 'static + Send> Default for AsyncOnce<R, T> {
    fn default() -> Self {
        AsyncOnce::new()
    }
}

#[tokio::test]
async fn test_async_once() {
    let foo = AsyncOnceLock::<usize>::new();
    assert_eq!(*foo.get_or_init(async { 2 }).await, 2);
    assert_eq!(*foo.get_or_init(async { 3 }).await, 2);
}
