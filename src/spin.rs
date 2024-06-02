use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};
use volatile::Volatile;

pub struct Spinlock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> Spinlock<T> {
    pub const fn new(data: T) -> Self {
        Spinlock {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinlockGuard<T> {
        while self.lock.compare_and_swap(false, true, Ordering::Acquire) {
            core::hint::spin_loop();
        }
        SpinlockGuard { lock: self }
    }

    pub fn try_lock(&self) -> Option<SpinlockGuard<T>> {
        if self.lock.compare_and_swap(false, true, Ordering::Acquire) {
            None
        } else {
            Some(SpinlockGuard { lock: self })
        }
    }
}

pub struct SpinlockGuard<'a, T> {
    lock: &'a Spinlock<T>,
}

impl<'a, T> Drop for SpinlockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.lock.store(false, Ordering::Release);
    }
}

impl<'a, T> core::ops::Deref for SpinlockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> core::ops::DerefMut for SpinlockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}
