use std::future::Future;
use crate::{LockError, PoisonError};

pub enum RawOnceState { Occupied, Vacant }

pub unsafe trait RawOnce: 'static {
    type GuardMarker;
    const UNINIT: Self;
    const INIT: Self;
    const POISON: Self;
    fn try_lock_checked(&self) -> Result<Option<RawOnceState>, PoisonError>;
    fn try_get_checked(&self) -> Result<RawOnceState, PoisonError>;
    unsafe fn unlock_nopoison(&self);
    unsafe fn unlock_poison(&self);
    unsafe fn unlock_init(&self);
}

pub unsafe trait BlockingRawOnce: RawOnce {
    fn lock_checked(&self) -> Result<RawOnceState, LockError>;
    fn get_checked(&self) -> Result<RawOnceState, LockError>;
}

pub unsafe trait AsyncRawOnce: RawOnce {
    type LockChecked<'a>: 'a + Future<Output=Result<RawOnceState, LockError> >;
    fn lock_checked<'a>(&'a self) -> Self::LockChecked<'a>;

    type GetChecked<'a>: 'a + Future<Output=Result<RawOnceState, LockError>>;
    fn get_checked<'a>(&'a self) -> Self::GetChecked<'a>;
}
