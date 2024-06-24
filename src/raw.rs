use std::future::Future;
use std::marker::PhantomData;
use std::sync::PoisonError;
use std::sync::TryLockError;

pub enum RawOnceState {
    Occupied,
    Vacant,
}

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

    type WriteChecked<'a>: 'a + Future<Output = Result<RawOnceState, TryLockError<()>>>
    where
        Self: 'a;
    fn write_checked<'a>(&'a self) -> Self::WriteChecked<'a>;

    // type ReadChecked<'a>: 'a + Future<Output=Result<RawOnceState, TryLockError<() > > > where Self: 'a;
    // fn read_checked<'a>(&'a self) -> Self::ReadChecked<'a>;
}

pub trait AsyncRawFusedSync = AsyncRawFused + Sync + Send
where
    <Self as AsyncRawFused>::GuardMarker: Send,
    for<'a> <Self as AsyncRawFused>::WriteChecked<'a>: Send;
