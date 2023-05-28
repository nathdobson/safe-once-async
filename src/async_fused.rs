use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::sync::{PoisonError, TryLockError};
use std::thread::panicking;
use crate::raw::{AsyncRawOnce, RawOnceState};

pub struct AsyncFused<R: AsyncRawOnce, T> {
    raw: R,
    data: UnsafeCell<T>,
}

pub enum AsyncFusedEntry<'a, R: AsyncRawOnce, T> {
    Write(AsyncFusedGuard<'a, R, T>),
    Read(&'a T),
}

pub struct AsyncFusedGuard<'a, R: AsyncRawOnce, T> {
    fused: Option<&'a AsyncFused<R, T>>,
    marker: PhantomData<(&'a mut T, R::GuardMarker)>,
}

impl<'a, R: AsyncRawOnce, T> AsyncFusedGuard<'a, R, T> {
    pub fn init(mut self) -> &'a T {
        unsafe {
            let once = self.fused.take().unwrap();
            once.raw.unlock_init();
            &*once.data.get()
        }
    }
}

impl<'a, R: AsyncRawOnce, T> AsyncFusedEntry<'a, R, T> {
    pub fn or_init(self, modify: impl FnOnce(&mut T)) -> &'a T {
        match self {
            AsyncFusedEntry::Read(x) => x,
            AsyncFusedEntry::Write(x) => {
                let mut x = x;
                modify(&mut *x);
                x.init()
            }
        }
    }
}

impl<'a, R: AsyncRawOnce, T> Deref for AsyncFusedGuard<'a, R, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.fused.unwrap().data.get() }
    }
}

impl<'a, R: AsyncRawOnce, T> DerefMut for AsyncFusedGuard<'a, R, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.fused.unwrap().data.get() }
    }
}

impl<R: AsyncRawOnce, T> AsyncFused<R, T> {
    pub const fn new(x: T) -> Self {
        AsyncFused {
            raw: R::UNINIT,
            data: UnsafeCell::new(x),
        }
    }
    pub const fn inited(x: T) -> Self {
        AsyncFused {
            raw: R::INIT,
            data: UnsafeCell::new(x),
        }
    }
    pub const fn poisoned(x: T) -> Self {
        AsyncFused {
            raw: R::POISON,
            data: UnsafeCell::new(x),
        }
    }
    unsafe fn make_entry(&self, raw: RawOnceState) -> AsyncFusedEntry<R, T> {
        match raw {
            RawOnceState::Vacant => AsyncFusedEntry::Write(AsyncFusedGuard { fused: Some(self), marker: PhantomData }),
            RawOnceState::Occupied => AsyncFusedEntry::Read(&*self.data.get()),
        }
    }
    pub async fn lock_checked(&self) -> Result<AsyncFusedEntry<R, T>, TryLockError<()>> {
        unsafe {
            Ok(self.make_entry(self.raw.lock_checked().await?))
        }
    }
    pub async fn lock(&self) -> AsyncFusedEntry<R, T> { self.lock_checked().await.unwrap() }
    pub fn try_lock_checked(&self) -> Result<Option<AsyncFusedEntry<R, T>>, TryLockError<()>> {
        unsafe {
            Ok(self.raw.try_lock_checked()?.map(|e| self.make_entry(e)))
        }
    }
    pub fn try_lock(&self) -> Option<AsyncFusedEntry<R, T>> {
        self.try_lock_checked().unwrap()
    }
    pub async fn get_or_init(&self, init: impl FnOnce(&mut T)) -> &T {
        self.get_or_init_checked(init).await.unwrap()
    }
    pub async fn get_or_init_checked(&self, init: impl FnOnce(&mut T)) -> Result<&T, TryLockError<()>> {
        Ok(self.lock_checked().await?.or_init(init))
    }
    pub fn try_get_checked(&self) -> Result<Option<&T>, PoisonError<()>> {
        unsafe {
            Ok(match self.raw.try_get_checked()? {
                RawOnceState::Vacant => None,
                RawOnceState::Occupied => Some(&*self.data.get())
            })
        }
    }
    pub async fn get_checked(&self) -> Result<Option<&T>, TryLockError<()>> {
        unsafe {
            Ok(match self.raw.get_checked().await? {
                RawOnceState::Vacant => None,
                RawOnceState::Occupied => Some(&*self.data.get())
            })
        }
    }
    pub fn try_get(&self) -> Option<&T> {
        self.try_get_checked().unwrap()
    }
    pub async fn get(&self) -> Option<&T> {
        self.get_checked().await.unwrap()
    }
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<R: AsyncRawOnce, T: Default> Default for AsyncFused<R, T> {
    fn default() -> Self { AsyncFused::new(T::default()) }
}

unsafe impl<R: AsyncRawOnce + Send, T: Send> Send for AsyncFused<R, T> {}

unsafe impl<R: AsyncRawOnce + Send + Sync, T: Send + Sync> Sync for AsyncFused<R, T> {}

impl<R: AsyncRawOnce + RefUnwindSafe + UnwindSafe, T: RefUnwindSafe + UnwindSafe> RefUnwindSafe for AsyncFused<R, T> {}

impl<R: AsyncRawOnce + UnwindSafe, T: UnwindSafe> UnwindSafe for AsyncFused<R, T> {}

impl<'a, R: AsyncRawOnce, T> Drop for AsyncFusedGuard<'a, R, T> {
    fn drop(&mut self) {
        unsafe {
            if let Some(once) = self.fused {
                if panicking() {
                    once.raw.unlock_poison();
                } else {
                    once.raw.unlock_nopoison();
                }
            }
        }
    }
}
