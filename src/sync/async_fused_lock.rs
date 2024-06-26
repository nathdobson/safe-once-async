use crate::raw::{AsyncRawFused, RawOnceState};
use parking_lot::lock_api::GuardSend;
use std::future::Future;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::sync::{PoisonError, TryLockError};
use tokio::sync::{Semaphore, TryAcquireError};

const STATE_UNINIT: usize = 0;
const STATE_INIT: usize = 1;
const STATE_POISON: usize = 2;

pub struct AsyncRawFusedLock {
    state: AtomicUsize,
    semaphore: Semaphore,
}

impl AsyncRawFusedLock {}

unsafe impl AsyncRawFused for AsyncRawFusedLock {
    type GuardMarker = GuardSend;
    const UNLOCKED: Self = AsyncRawFusedLock {
        state: AtomicUsize::new(STATE_UNINIT),
        semaphore: Semaphore::const_new(1),
    };
    const READ: Self = AsyncRawFusedLock {
        state: AtomicUsize::new(STATE_INIT),
        semaphore: Semaphore::const_new(1),
    };
    const POISON: Self = AsyncRawFusedLock {
        state: AtomicUsize::new(STATE_POISON),
        semaphore: Semaphore::const_new(1),
    };

    fn try_write_checked(&self) -> Result<Option<RawOnceState>, PoisonError<()>> {
        match self.try_read_checked()? {
            RawOnceState::Occupied => {
                return Ok(Some(RawOnceState::Occupied));
            }
            _ => {}
        }
        match self.semaphore.try_acquire() {
            Ok(lock) => match self.try_read_checked()? {
                RawOnceState::Occupied => unreachable!(),
                RawOnceState::Vacant => {
                    lock.forget();
                    Ok(Some(RawOnceState::Vacant))
                }
            },
            Err(TryAcquireError::Closed) => match self.try_read_checked()? {
                RawOnceState::Occupied => {
                    return Ok(Some(RawOnceState::Occupied));
                }
                RawOnceState::Vacant => unreachable!(),
            },
            Err(TryAcquireError::NoPermits) => Ok(None),
        }
    }

    fn try_read_checked(&self) -> Result<RawOnceState, PoisonError<()>> {
        match self.state.load(Acquire) {
            STATE_UNINIT => Ok(RawOnceState::Vacant),
            STATE_INIT => Ok(RawOnceState::Occupied),
            STATE_POISON => Err(PoisonError::new(())),
            _ => unreachable!(),
        }
    }

    unsafe fn unlock(&self) {
        self.semaphore.add_permits(1);
    }

    unsafe fn unlock_poison(&self) {
        self.state.store(STATE_POISON, Release);
        self.semaphore.close();
    }

    unsafe fn unlock_fuse(&self) {
        self.state.store(STATE_INIT, Release);
        self.semaphore.close();
    }

    type WriteChecked<'a> =
        impl 'a + Send + Future<Output = Result<RawOnceState, TryLockError<()>>>;
    fn write_checked<'a>(&'a self) -> Self::WriteChecked<'a> {
        async move {
            match self.try_read_checked()? {
                RawOnceState::Occupied => {
                    return Ok(RawOnceState::Occupied);
                }
                _ => {}
            }
            match self.semaphore.acquire().await {
                Ok(lock) => match self.try_read_checked()? {
                    RawOnceState::Occupied => unreachable!(),
                    RawOnceState::Vacant => {
                        lock.forget();
                        Ok(RawOnceState::Vacant)
                    }
                },
                Err(_) => match self.try_read_checked()? {
                    RawOnceState::Occupied => Ok(RawOnceState::Occupied),
                    RawOnceState::Vacant => unreachable!(),
                },
            }
        }
    }

    // type ReadChecked<'a> = impl 'a + Send + Future<Output=Result<RawOnceState, TryLockError<()>>>;
    // fn read_checked<'a>(&'a self) -> Self::ReadChecked<'a> {
    //     async move {
    //         match self.try_read_checked()? {
    //             RawOnceState::Occupied => { return Ok(RawOnceState::Occupied); }
    //             _ => {}
    //         }
    //         match self.semaphore.acquire().await {
    //             Ok(lock) => match self.try_read_checked()? {
    //                 RawOnceState::Occupied => unreachable!(),
    //                 RawOnceState::Vacant => Ok(RawOnceState::Vacant),
    //             }
    //             Err(_) => match self.try_read_checked()? {
    //                 RawOnceState::Occupied => Ok(RawOnceState::Occupied),
    //                 RawOnceState::Vacant => unreachable!()
    //             }
    //         }
    //     }
    // }
}

unsafe impl Send for AsyncRawFusedLock {}

unsafe impl Sync for AsyncRawFusedLock {}
