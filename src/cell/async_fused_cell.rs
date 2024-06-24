use crate::cell::condvar::Condvar;
use parking_lot::lock_api::GuardNoSend;
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::mem;
use std::ptr::{null, null_mut};
use std::sync::{PoisonError, TryLockError};
use std::task::Waker;

use crate::raw::{AsyncRawFused, RawOnceState};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum State {
    Unlocked,
    Write,
    Read,
    Poison,
}

#[derive(Debug)]
pub struct AsyncRawFusedCell {
    state: Cell<State>,
    writers: Condvar,
}

impl AsyncRawFusedCell {
    pub async fn write_checked_impl(&self) -> Result<RawOnceState, TryLockError<()>> {
        loop {
            match self.state.get() {
                State::Unlocked => {
                    self.state.set(State::Write);
                    return Ok(RawOnceState::Vacant);
                }
                State::Write => {
                    self.writers.wait().await.consume();
                }
                State::Read => {
                    return Ok(RawOnceState::Occupied);
                }
                State::Poison => {
                    return Err(TryLockError::Poisoned(PoisonError::new(())));
                }
            }
        }
    }
}

unsafe impl AsyncRawFused for AsyncRawFusedCell {
    type GuardMarker = GuardNoSend;
    const UNLOCKED: Self = AsyncRawFusedCell {
        state: Cell::new(State::Unlocked),
        writers: Condvar::new(),
    };
    const READ: Self = AsyncRawFusedCell {
        state: Cell::new(State::Read),
        writers: Condvar::new(),
    };
    const POISON: Self = AsyncRawFusedCell {
        state: Cell::new(State::Poison),
        writers: Condvar::new(),
    };

    fn try_write_checked(&self) -> Result<Option<RawOnceState>, PoisonError<()>> {
        match self.state.get() {
            State::Unlocked => {
                self.state.set(State::Write);
                Ok(Some(RawOnceState::Vacant))
            }
            State::Write => Ok(None),
            State::Read => Ok(Some(RawOnceState::Occupied)),
            State::Poison => Err(PoisonError::new(())),
        }
    }

    fn try_read_checked(&self) -> Result<RawOnceState, PoisonError<()>> {
        match self.state.get() {
            State::Unlocked => Ok(RawOnceState::Vacant),
            State::Write => Ok(RawOnceState::Vacant),
            State::Read => Ok(RawOnceState::Occupied),
            State::Poison => Err(PoisonError::new(())),
        }
    }
    unsafe fn unlock(&self) {
        assert_eq!(self.state.get(), State::Write);
        self.state.set(State::Unlocked);
        self.writers.notify();
    }
    unsafe fn unlock_poison(&self) {
        assert_eq!(self.state.get(), State::Write);
        self.state.set(State::Poison);
        self.writers.notify();
    }

    unsafe fn unlock_fuse(&self) {
        assert_eq!(self.state.get(), State::Write);
        self.state.set(State::Read);
        self.writers.notify();
    }
    type WriteChecked<'a> = impl 'a + Future<Output = Result<RawOnceState, TryLockError<()>>>;

    fn write_checked<'a>(&'a self) -> Self::WriteChecked<'a> {
        self.write_checked_impl()
    }

    // type ReadChecked<'a> = impl 'a + Future<Output=Result<RawOnceState, TryLockError<()>>>;
    //
    // fn read_checked<'a>(&'a self) -> Self::ReadChecked<'a> {
    //     async move {
    //         todo!()
    //     }
    // }
}
