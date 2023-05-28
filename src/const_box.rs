use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::intrinsics::const_allocate;
use std::marker::Unsize;
use std::mem::{align_of, size_of};
use std::ops::{CoerceUnsized, Deref, DerefMut};
use std::pin::Pin;
use std::process::abort;
use std::ptr::{NonNull, null, null_mut};
use std::slice::SliceIndex;
use std::task::{Context, Poll};

/// It is a compilation error to initialize a ConstBox dynamically:
/// ```compile_fail
/// use safe_once_async::const_box::ConstBox;
/// ConstBox::new(1);
/// ```
pub struct ConstBox<T: ?Sized>(NonNull<T>);

unsafe impl<T: ?Sized + Sync> Sync for ConstBox<T> {}

unsafe impl<T: ?Sized + Send> Send for ConstBox<T> {}

#[allow(unconditional_recursion)]
const fn cannot_dynamically_initialize_const_box<T>(x: T) {
    return cannot_dynamically_initialize_const_box::<Option<T>>(Some(x));
}

impl<T: ?Sized> ConstBox<T> {
    pub const fn new(x: T) -> Self where T: Sized {
        unsafe {
            let ptr = const_allocate(size_of::<T>(), align_of::<T>()) as *mut T;
            if ptr.is_null() {
                cannot_dynamically_initialize_const_box(1);
            }
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

impl<T: ?Sized + Debug> Debug for ConstBox<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&**self, f)
    }
}

#[test]
fn test_const_box() {
    static X: ConstBox<usize> = ConstBox::new(10);
    println!("{:?}", X);
}