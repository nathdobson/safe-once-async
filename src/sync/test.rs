use std::future::Future;
use std::intrinsics::const_allocate;
use std::marker::Unsize;
use std::mem::{align_of, size_of};
use std::ops::{CoerceUnsized, Deref, DerefMut};
use std::panic::catch_unwind;
use std::pin::Pin;
use std::ptr::{NonNull, null_mut};
use std::sync::{Arc, Barrier};
use std::task::{Context, Poll};
use std::thread;
use std::time::Duration;
use parking_lot::{Mutex, RwLock};
use crate::{LockError, PoisonError};
use crate::async_lazy::AsyncLazy;
use crate::const_box::{ConstBoxFuture};
use crate::once::OnceEntry;
use crate::sync::async_once_lock::AsyncRawOnceLock;
use crate::sync::{AsyncLazyLock, AsyncLazyStatic, AsyncOnceLock, OnceLock};

#[test]
fn test_once() {
    let lock = OnceLock::<Box<usize>>::new();
    match lock.lock_checked().unwrap() {
        OnceEntry::Occupied(_) => unreachable!(),
        OnceEntry::Vacant(x) => { x.init(Box::new(1)); }
    }
    match lock.lock_checked().unwrap() {
        OnceEntry::Occupied(x) => assert_eq!(**x, 1),
        OnceEntry::Vacant(_) => unreachable!(),
    };
}

#[test]
fn test_direct() {
    assert!(OnceLock::<Box<isize>>::new().into_inner().is_none());
    assert_eq!(*OnceLock::from(Box::new(1)).into_inner().unwrap(), 1);
}

#[test]
fn test_relock() {
    let once = OnceLock::<Box<isize>>::new();
    match once.lock() {
        OnceEntry::Occupied(_) => unreachable!(),
        OnceEntry::Vacant(_) => {}
    }
    match once.lock() {
        OnceEntry::Occupied(_) => unreachable!(),
        OnceEntry::Vacant(x) => { x.init(Box::new(5)); }
    }
    assert_eq!(**once.try_get().unwrap(), 5);
}

#[test]
fn test_recurrent() {
    let once = OnceLock::<Box<isize>>::new();
    once.get_or_init(|| {
        assert_eq!(once.get_or_init_checked(|| unreachable!()).unwrap_err(), LockError::CycleError);
        Box::new(5)
    });
}

#[test]
fn test_panic() {
    let once = OnceLock::<Box<isize>>::new();
    assert!(catch_unwind(|| {
        once.get_or_init(|| {
            panic!();
        });
    }).is_err());
    assert_eq!(once.try_get_checked().unwrap_err(), PoisonError);
}

#[test]
fn test_get_blocking() {
    let once = Arc::new(OnceLock::<usize>::new());
    let barrier = Arc::new(Barrier::new(2));
    let t = thread::spawn({
        let once = once.clone();
        let barrier = barrier.clone();
        move || {
            once.get_or_init(|| {
                barrier.wait();
                thread::sleep(Duration::from_millis(100));
                42
            });
        }
    });
    barrier.wait();
    assert_eq!(once.get(), Some(&42));
}

#[test]
fn test_stress() {
    for threads in 1..=8 {
        let onces = Arc::new(vec![OnceLock::new(); 1000]);
        let barrier = Arc::new(Barrier::new(threads));
        let wins: usize = (0..threads).map(
            |_| {
                let barrier = barrier.clone();
                let onces = onces.clone();

                thread::spawn(move || {
                    let mut wins = 0;
                    for once in onces.iter() {
                        barrier.wait();
                        once.get_or_init(|| {
                            wins += 1;
                            ()
                        });
                    }
                    wins
                })
            }
        ).collect::<Vec<_>>().into_iter().map(|x| x.join()).sum::<thread::Result<usize>>().unwrap();
        assert_eq!(wins, onces.len());
    }
}

#[tokio::test]
async fn test_const_box() {
    static FOO: AsyncLazyStatic<usize> = AsyncLazyStatic::new_static(async {
        2
    });
    assert_eq!(2, *FOO.get().await);
}

#[tokio::test]
async fn test_async() {
    let x = AsyncLazyLock::<usize, _>::new(async {
        4
    });
    assert_eq!(4, *x.get().await);

    let x = AsyncOnceLock::<usize>::new();
    assert_eq!(8, *x.get_or_init_async(async { 8 }).await);
    assert_eq!(8, *x.get_or_init_async(async { 15 }).await);
}