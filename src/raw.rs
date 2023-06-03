use std::future::Future;
use std::sync::PoisonError;
use std::sync::TryLockError;

pub enum RawOnceState { Occupied, Vacant }

pub unsafe trait AsyncRawFused: 'static {
    type GuardMarker;
    const UNLOCKED: Self;
    const READ: Self;
    const POISON: Self;
    fn try_write_checked(&self) -> Result<Option<RawOnceState>, PoisonError<()>>;
    fn try_read_checked(&self) -> Result<RawOnceState, PoisonError<()>>;
    unsafe fn unlock(&self);
    unsafe fn unlock_poison(&self);
    unsafe fn unlock_fuse(&self);

    type LockChecked<'a>: 'a + Send+Future<Output=Result<RawOnceState, TryLockError<() > > > where Self: 'a;
    fn write_checked<'a>(&'a self) -> Self::LockChecked<'a>;

    type GetChecked<'a>: 'a + Send+Future<Output=Result<RawOnceState, TryLockError<() > > > where Self: 'a;
    fn read_checked<'a>(&'a self) -> Self::GetChecked<'a>;
}
