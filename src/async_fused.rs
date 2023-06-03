use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::sync::{PoisonError, TryLockError};
use std::thread::panicking;
use crate::raw::{AsyncRawFused, RawOnceState};

pub struct AsyncFused<R: AsyncRawFused, T> {
    raw: R,
    data: UnsafeCell<T>,
}

pub enum AsyncFusedEntry<'a, R: AsyncRawFused, T> {
    Write(AsyncFusedGuard<'a, R, T>),
    Read(&'a T),
}

pub struct AsyncFusedGuard<'a, R: AsyncRawFused, T> {
    fused: Option<&'a AsyncFused<R, T>>,
    marker: PhantomData<(&'a mut T, R::GuardMarker)>,
}

impl<'a, R: AsyncRawFused, T> AsyncFusedGuard<'a, R, T> {
    pub fn fuse(mut self) -> &'a T {
        unsafe {
            let once = self.fused.take().unwrap();
            once.raw.unlock_fuse();
            &*once.data.get()
        }
    }
}

impl<'a, R: AsyncRawFused, T> AsyncFusedEntry<'a, R, T> {
    pub fn or_fuse(self, modify: impl FnOnce(&mut T)) -> &'a T {
        match self {
            AsyncFusedEntry::Read(x) => x,
            AsyncFusedEntry::Write(x) => {
                let mut x = x;
                modify(&mut *x);
                x.fuse()
            }
        }
    }
}

impl<'a, R: AsyncRawFused, T> Deref for AsyncFusedGuard<'a, R, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.fused.unwrap().data.get() }
    }
}

impl<'a, R: AsyncRawFused, T> DerefMut for AsyncFusedGuard<'a, R, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.fused.unwrap().data.get() }
    }
}

impl<R: AsyncRawFused, T> AsyncFused<R, T> {
    pub const fn new(x: T) -> Self {
        AsyncFused {
            raw: R::UNLOCKED,
            data: UnsafeCell::new(x),
        }
    }
    pub const fn new_read(x: T) -> Self {
        AsyncFused {
            raw: R::READ,
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
    pub async fn write_checked(&self) -> Result<AsyncFusedEntry<R, T>, TryLockError<()>> {
        unsafe {
            Ok(self.make_entry(self.raw.write_checked().await?))
        }
    }
    pub async fn write(&self) -> AsyncFusedEntry<R, T> { self.write_checked().await.unwrap() }
    pub fn try_write_checked(&self) -> Result<Option<AsyncFusedEntry<R, T>>, TryLockError<()>> {
        unsafe {
            Ok(self.raw.try_write_checked()?.map(|e| self.make_entry(e)))
        }
    }
    pub fn try_write(&self) -> Option<AsyncFusedEntry<R, T>> {
        self.try_write_checked().unwrap()
    }
    pub async fn read_or_fuse(&self, init: impl FnOnce(&mut T)) -> &T {
        self.read_or_fuse_checked(init).await.unwrap()
    }
    pub async fn read_or_fuse_checked(&self, init: impl FnOnce(&mut T)) -> Result<&T, TryLockError<()>> {
        Ok(self.write_checked().await?.or_fuse(init))
    }
    pub fn try_read_checked(&self) -> Result<Option<&T>, PoisonError<()>> {
        unsafe {
            Ok(match self.raw.try_read_checked()? {
                RawOnceState::Vacant => None,
                RawOnceState::Occupied => Some(&*self.data.get())
            })
        }
    }
    pub async fn read_checked(&self) -> Result<Option<&T>, TryLockError<()>> {
        unsafe {
            Ok(match self.raw.read_checked().await? {
                RawOnceState::Vacant => None,
                RawOnceState::Occupied => Some(&*self.data.get())
            })
        }
    }
    pub fn try_read(&self) -> Option<&T> {
        self.try_read_checked().unwrap()
    }
    pub async fn read(&self) -> Option<&T> {
        self.read_checked().await.unwrap()
    }
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<R: AsyncRawFused, T: Default> Default for AsyncFused<R, T> {
    fn default() -> Self { AsyncFused::new(T::default()) }
}

unsafe impl<R: AsyncRawFused + Send, T: Send> Send for AsyncFused<R, T> {}

unsafe impl<R: AsyncRawFused + Send + Sync, T: Send + Sync> Sync for AsyncFused<R, T> {}

impl<R: AsyncRawFused + RefUnwindSafe + UnwindSafe, T: RefUnwindSafe + UnwindSafe> RefUnwindSafe for AsyncFused<R, T> {}

impl<R: AsyncRawFused + UnwindSafe, T: UnwindSafe> UnwindSafe for AsyncFused<R, T> {}

impl<'a, R: AsyncRawFused, T> Drop for AsyncFusedGuard<'a, R, T> {
    fn drop(&mut self) {
        unsafe {
            if let Some(once) = self.fused {
                if panicking() {
                    once.raw.unlock_poison();
                } else {
                    once.raw.unlock();
                }
            }
        }
    }
}
