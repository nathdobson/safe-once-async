use std::future::Future;
use std::sync::PoisonError;
use std::sync::TryLockError;

pub enum RawOnceState { Occupied, Vacant }

pub unsafe trait AsyncRawOnce: 'static {
    type GuardMarker;
    const UNINIT: Self;
    const INIT: Self;
    const POISON: Self;
    fn try_lock_checked(&self) -> Result<Option<RawOnceState>, PoisonError<()>>;
    fn try_get_checked(&self) -> Result<RawOnceState, PoisonError<()>>;
    unsafe fn unlock_nopoison(&self);
    unsafe fn unlock_poison(&self);
    unsafe fn unlock_init(&self);

    type LockChecked<'a>: 'a + Send+Future<Output=Result<RawOnceState, TryLockError<() > > > where Self: 'a;
    fn lock_checked<'a>(&'a self) -> Self::LockChecked<'a>;

    type GetChecked<'a>: 'a + Send+Future<Output=Result<RawOnceState, TryLockError<() > > > where Self: 'a;
    fn get_checked<'a>(&'a self) -> Self::GetChecked<'a>;
}
