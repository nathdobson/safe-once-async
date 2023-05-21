use std::future::Future;
use std::intrinsics::const_allocate;
use std::marker::Unsize;
use std::mem::{align_of, size_of};
use std::ops::{CoerceUnsized, Deref, DerefMut};
use std::pin::Pin;
use std::ptr::NonNull;
use std::task::{Context, Poll};

pub struct ConstBox<T: ?Sized>(NonNull<T>);

unsafe impl<T: ?Sized + Sync> Sync for ConstBox<T> {}

unsafe impl<T: ?Sized + Send> Send for ConstBox<T> {}

impl<T: ?Sized> ConstBox<T> {
    pub const fn new(x: T) -> Self where T: Sized {
        unsafe {
            let ptr = const_allocate(size_of::<T>(), align_of::<T>()) as *mut T;
            ptr.write(x);
            ConstBox(NonNull::new(ptr).unwrap())
        }
    }
    pub const fn pin(x: T) -> Pin<Self> where T: Sized {
        unsafe { Pin::new_unchecked(Self::new(x)) }
    }
}

impl<T: ?Sized> Deref for ConstBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { unsafe { self.0.as_ref() } }
}

impl<T: ?Sized> DerefMut for ConstBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { unsafe { self.0.as_mut() } }
}

impl<T: ?Sized> Unpin for ConstBox<T> {}

impl<T: Unsize<U> + ?Sized, U: ?Sized> CoerceUnsized<ConstBox<U>> for ConstBox<T> {}

pub type ConstBoxFuture<T> = Pin<ConstBox<dyn Send + Future<Output=T>>>;

impl<T: ?Sized + Future> Future for ConstBox<T> {
    type Output = T::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> { todo!() }
}