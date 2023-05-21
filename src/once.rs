use std::cell::UnsafeCell;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::thread::panicking;
use crate::{LockError, PoisonError, RawOnce, RawOnceState};
use crate::raw::{AsyncRawOnce, BlockingRawOnce};

pub enum OnceEntry<'a, R: RawOnce, T> {
    Occupied(&'a T),
    Vacant(OnceGuard<'a, R, T>),
}

pub struct Once<R: RawOnce, T> {
    raw: R,
    data: UnsafeCell<MaybeUninit<T>>,
}

pub struct OnceGuard<'a, R: RawOnce, T> {
    once: Option<&'a Once<R, T>>,
    marker: PhantomData<(&'a mut T, R::GuardMarker)>,
}

impl<'a, R: RawOnce, T> OnceGuard<'a, R, T> {
    pub fn init(mut self, value: T) -> &'a T {
        unsafe {
            let once = self.once.take().unwrap();
            (*once.data.get()).write(value);
            once.raw.unlock_init();
            (*once.data.get()).assume_init_ref()
        }
    }
}

impl<'a, R: RawOnce, T> OnceEntry<'a, R, T> {
    pub fn or_init(self, value: impl FnOnce() -> T) -> &'a T {
        match self {
            OnceEntry::Occupied(x) => x,
            OnceEntry::Vacant(x) => x.init(value())
        }
    }
    pub async fn or_init_async(self, value: impl Future<Output=T>) -> &'a T {
        match self {
            OnceEntry::Occupied(x) => x,
            OnceEntry::Vacant(x) => x.init(value.await)
        }
    }
}

impl<R: RawOnce, T> Once<R, T> {
    pub const fn new() -> Self {
        Once {
            raw: R::UNINIT,
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
    pub const fn poisoned() -> Self {
        Once {
            raw: R::POISON,
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
    unsafe fn make_entry(&self, raw: RawOnceState) -> OnceEntry<R, T> {
        match raw {
            RawOnceState::Vacant => OnceEntry::Vacant(OnceGuard { once: Some(self), marker: PhantomData }),
            RawOnceState::Occupied => OnceEntry::Occupied((*self.data.get()).assume_init_ref()),
        }
    }
    pub fn lock_checked(&self) -> Result<OnceEntry<R, T>, LockError> where R: BlockingRawOnce {
        unsafe {
            Ok(self.make_entry(self.raw.lock_checked()?))
        }
    }
    pub async fn lock_checked_async(&self) -> Result<OnceEntry<R, T>, LockError> where R: AsyncRawOnce {
        unsafe {
            Ok(self.make_entry(self.raw.lock_checked().await?))
        }
    }
    pub fn lock(&self) -> OnceEntry<R, T> where R: BlockingRawOnce { self.lock_checked().unwrap() }
    pub fn try_lock_checked(&self) -> Result<Option<OnceEntry<R, T>>, LockError> {
        unsafe {
            Ok(self.raw.try_lock_checked()?.map(|e| self.make_entry(e)))
        }
    }
    pub fn try_lock(&self) -> Option<OnceEntry<R, T>> {
        self.try_lock_checked().unwrap()
    }
    pub fn get_or_init(&self, init: impl FnOnce() -> T) -> &T where R: BlockingRawOnce {
        self.get_or_init_checked(init).unwrap()
    }
    pub fn get_or_init_checked(&self, init: impl FnOnce() -> T) -> Result<&T, LockError> where R: BlockingRawOnce {
        Ok(self.lock_checked()?.or_init(init))
    }
    pub async fn get_or_init_async(&self, init: impl Future<Output=T>) -> &T where R: AsyncRawOnce {
        self.get_or_init_checked_async(init).await.unwrap()
    }
    pub async fn get_or_init_checked_async(&self, init: impl Future<Output=T>) -> Result<&T, LockError> where R: AsyncRawOnce {
        Ok(self.lock_checked_async().await?.or_init_async(init).await)
    }
    pub fn try_get_checked(&self) -> Result<Option<&T>, PoisonError> {
        unsafe {
            Ok(match self.raw.try_get_checked()? {
                RawOnceState::Vacant => None,
                RawOnceState::Occupied => Some((*self.data.get()).assume_init_ref())
            })
        }
    }
    pub fn get_checked(&self) -> Result<Option<&T>, LockError> where R: BlockingRawOnce {
        unsafe {
            Ok(match self.raw.get_checked()? {
                RawOnceState::Vacant => None,
                RawOnceState::Occupied => Some((*self.data.get()).assume_init_ref())
            })
        }
    }
    pub fn try_get(&self) -> Option<&T> {
        self.try_get_checked().unwrap()
    }
    pub fn get(&self) -> Option<&T> where R: BlockingRawOnce {
        self.get_checked().unwrap()
    }
    pub fn into_inner(mut self) -> Option<T> {
        unsafe {
            match self.raw.try_get_checked().unwrap() {
                RawOnceState::Occupied => {
                    self.raw = RawOnce::POISON;
                    Some((*self.data.get()).assume_init_read())
                }
                RawOnceState::Vacant => None
            }
        }
    }
}

impl<R: RawOnce, T> Drop for Once<R, T> {
    fn drop(&mut self) {
        unsafe {
            match self.raw.try_get_checked() {
                Ok(RawOnceState::Occupied) => {
                    self.raw = RawOnce::POISON;
                    (*self.data.get()).assume_init_drop();
                }
                _ => {}
            }
        }
    }
}

impl<'a, R: RawOnce, T> Drop for OnceGuard<'a, R, T> {
    fn drop(&mut self) {
        unsafe {
            if let Some(once) = self.once {
                if panicking() {
                    once.raw.unlock_poison();
                } else {
                    once.raw.unlock_nopoison();
                }
            }
        }
    }
}

impl<R: RawOnce, T> From<T> for Once<R, T> {
    fn from(value: T) -> Self {
        Once { raw: R::INIT, data: UnsafeCell::new(MaybeUninit::new(value)) }
    }
}

unsafe impl<R: RawOnce + Send, T: Send> Send for Once<R, T> {}

unsafe impl<R: RawOnce + Send + Sync, T: Send + Sync> Sync for Once<R, T> {}

impl<R: RawOnce + RefUnwindSafe + UnwindSafe, T: RefUnwindSafe + UnwindSafe> RefUnwindSafe for Once<R, T> {}

impl<R: RawOnce + UnwindSafe, T: UnwindSafe> UnwindSafe for Once<R, T> {}

impl<R: RawOnce + Debug, T: Debug> Debug for Once<R, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Once")
            .field("raw", &self.raw)
            .field("value", &self.try_get())
            .finish()
    }
}

impl<R: RawOnce, T> Default for Once<R, T> {
    fn default() -> Self { Once::new() }
}

impl<R: RawOnce, T: Clone> Clone for Once<R, T> {
    fn clone(&self) -> Self {
        match self.try_get_checked() {
            Ok(Some(x)) => Once::from(x.clone()),
            Ok(None) => Once::new(),
            Err(PoisonError) => Once::poisoned(),
        }
    }
}