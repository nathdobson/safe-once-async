use std::cell::{Cell, RefCell};
use std::future::Future;
use std::ptr::null_mut;
use std::sync::{PoisonError, TryLockError};
use std::task::Waker;
use parking_lot::lock_api::GuardNoSend;

use crate::raw::{AsyncRawFused, RawOnceState};

#[derive(Copy, Clone, Debug)]
enum State {
    Uninit,
    Initializing,
    Initialized,
    Poison,
}

#[derive(Debug)]
struct Waiter {
    next: *mut Waiter,
    prev: *mut Waiter,
    waker: Option<Waker>,
}

#[derive(Debug)]
struct Inner {
    state: State,
    lockers: *mut Waiter,
    getters: *mut Waiter,
}

#[derive(Debug)]
pub struct AsyncRawFusedCell(RefCell<Inner>);

unsafe impl AsyncRawFused for AsyncRawFusedCell {
    type GuardMarker = GuardNoSend;
    const UNLOCKED: Self = AsyncRawFusedCell(RefCell::new(Inner {
        state: State::Uninit,
        lockers: null_mut(),
        getters: null_mut(),
    }));
    const READ: Self = AsyncRawFusedCell(RefCell::new(Inner {
        state: State::Initialized,
        lockers: null_mut(),
        getters: null_mut(),
    }));
    const POISON: Self = AsyncRawFusedCell(RefCell::new(Inner {
        state: State::Poison,
        lockers: null_mut(),
        getters: null_mut(),
    }));

    fn try_write_checked(&self) -> Result<Option<RawOnceState>, PoisonError<()>> {
        todo!()
        // match self.0.get() {
        //     State::Uninit => {
        //         self.0.set(State::Initializing);
        //         Ok(Some(RawOnceState::Vacant))
        //     }
        //     State::Initializing =>
        //         Ok(None),
        //     State::Initialized =>
        //         Ok(Some(RawOnceState::Occupied)),
        //     State::Poison =>
        //         Err(PoisonError),
        // }
    }

    fn try_read_checked(&self) -> Result<RawOnceState, PoisonError<()>> {
        todo!()
        // match self.0.get() {
        //     State::Uninit => Ok(RawOnceState::Vacant),
        //     State::Initializing => Ok(RawOnceState::Vacant),
        //     State::Initialized => Ok(RawOnceState::Occupied),
        //     State::Poison => Err(PoisonError),
        // }
    }
    unsafe fn unlock(&self) {
        todo!()
        // match self.0.get() {
        //     State::Initializing => self.0.set(State::Uninit),
        //     _ => panic!("Not already initializing"),
        // }
    }
    unsafe fn unlock_fuse(&self) {
        todo!()
        // match self.0.get() {
        //     State::Initializing => self.0.set(State::Initialized),
        //     _ => panic!("Not already initializing"),
        // }
    }

    unsafe fn unlock_poison(&self) {
        // match self.0.get() {
        //     State::Initializing => self.0.set(State::Poison),
        //     _ => panic!("Not already initializing"),
        // }
        todo!()
    }
    type LockChecked<'a> = impl 'a + Future<Output=Result<RawOnceState, TryLockError<()>>>;

    fn write_checked<'a>(&'a self) -> Self::LockChecked<'a> {
        async move {
            todo!()
        }
    }

    type GetChecked<'a> = impl 'a + Future<Output=Result<RawOnceState, TryLockError<()>>>;

    fn read_checked<'a>(&'a self) -> Self::GetChecked<'a> {
        async move {
            todo!()
        }
    }
}