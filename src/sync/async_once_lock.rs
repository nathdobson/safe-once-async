use std::future::Future;
use std::intrinsics::unreachable;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Acquire, Release};
use parking_lot::lock_api::GuardSend;
use tokio::sync::{AcquireError, Semaphore, SemaphorePermit, TryAcquireError};
use crate::raw::AsyncRawOnce;
use crate::{LockError, PoisonError, RawOnce, RawOnceState};

const STATE_UNINIT: usize = 0;
const STATE_INIT: usize = 1;
const STATE_POISON: usize = 2;

pub struct AsyncRawOnceLock {
    state: AtomicUsize,
    semaphore: Semaphore,
}

impl AsyncRawOnceLock {}

unsafe impl RawOnce for AsyncRawOnceLock {
    type GuardMarker = GuardSend;
    const UNINIT: Self = AsyncRawOnceLock { state: AtomicUsize::new(STATE_UNINIT), semaphore: Semaphore::const_new(1) };
    const INIT: Self = AsyncRawOnceLock { state: AtomicUsize::new(STATE_INIT), semaphore: Semaphore::const_new(1) };
    const POISON: Self = AsyncRawOnceLock { state: AtomicUsize::new(STATE_POISON), semaphore: Semaphore::const_new(1) };

    fn try_lock_checked(&self) -> Result<Option<RawOnceState>, PoisonError> {
        match self.try_get_checked()? {
            RawOnceState::Occupied => { return Ok(Some(RawOnceState::Occupied)); }
            _ => {}
        }
        match self.semaphore.try_acquire() {
            Ok(lock) => {
                match self.try_get_checked()? {
                    RawOnceState::Occupied => unreachable!(),
                    RawOnceState::Vacant => {
                        lock.forget();
                        Ok(Some(RawOnceState::Vacant))
                    }
                }
            }
            Err(TryAcquireError::Closed) => {
                match self.try_get_checked()? {
                    RawOnceState::Occupied => { return Ok(Some(RawOnceState::Occupied)); }
                    RawOnceState::Vacant => unreachable!()
                }
            }
            Err(TryAcquireError::NoPermits) => Ok(None)
        }
    }

    fn try_get_checked(&self) -> Result<RawOnceState, PoisonError> {
        match self.state.load(Acquire) {
            STATE_UNINIT => Ok(RawOnceState::Vacant),
            STATE_INIT => Ok(RawOnceState::Occupied),
            STATE_POISON => Err(PoisonError),
            _ => unreachable!()
        }
    }

    unsafe fn unlock_nopoison(&self) {
        self.semaphore.add_permits(1);
    }

    unsafe fn unlock_poison(&self) {
        self.state.store(STATE_POISON, Release);
        self.semaphore.close();
    }

    unsafe fn unlock_init(&self) {
        self.state.store(STATE_INIT, Release);
        self.semaphore.close();
    }
}

unsafe impl AsyncRawOnce for AsyncRawOnceLock {
    type LockChecked<'a> = impl 'a + Future<Output=Result<RawOnceState, LockError>>;
    fn lock_checked<'a>(&'a self) -> Self::LockChecked<'a> {
        async move {
            match self.try_get_checked()? {
                RawOnceState::Occupied => { return Ok(RawOnceState::Occupied); }
                _ => {}
            }
            match self.semaphore.acquire().await {
                Ok(lock) => match self.try_get_checked()? {
                    RawOnceState::Occupied => unreachable!(),
                    RawOnceState::Vacant => {
                        lock.forget();
                        Ok(RawOnceState::Vacant)
                    }
                }
                Err(_) => match self.try_get_checked()? {
                    RawOnceState::Occupied => Ok(RawOnceState::Occupied),
                    RawOnceState::Vacant => unreachable!()
                }
            }
        }
    }

    type GetChecked<'a> = impl 'a + Future<Output=Result<RawOnceState, LockError>>;

    fn get_checked<'a>(&'a self) -> Self::GetChecked<'a> {
        async move {
            match self.try_get_checked()? {
                RawOnceState::Occupied => { return Ok(RawOnceState::Occupied); }
                _ => {}
            }
            match self.semaphore.acquire().await {
                Ok(lock) => match self.try_get_checked()? {
                    RawOnceState::Occupied => unreachable!(),
                    RawOnceState::Vacant => Ok(RawOnceState::Vacant),
                }
                Err(_) => match self.try_get_checked()? {
                    RawOnceState::Occupied => Ok(RawOnceState::Occupied),
                    RawOnceState::Vacant => unreachable!()
                }
            }
        }
    }
}