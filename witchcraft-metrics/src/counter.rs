use std::sync::atomic::{AtomicI64, Ordering};

/// A metric which counts a value.
#[derive(Debug, Default)]
pub struct Counter(AtomicI64);

impl Counter {
    /// Creates a new counter initialized to 0.
    #[inline]
    pub fn new() -> Counter {
        Counter::default()
    }

    /// Resets the counter to 0.
    #[inline]
    pub fn clear(&self) {
        self.0.store(0, Ordering::Relaxed);
    }

    /// Adds 1 to the counter.
    #[inline]
    pub fn inc(&self) {
        self.add(1);
    }

    /// Subtracts 1 from the counter.
    #[inline]
    pub fn dec(&self) {
        self.sub(1);
    }

    /// Adds a number to the counter.
    #[inline]
    pub fn add(&self, n: i64) {
        self.0.fetch_add(n, Ordering::Relaxed);
    }

    /// Subtracts a number from the counter.
    #[inline]
    pub fn sub(&self, n: i64) {
        self.0.fetch_sub(n, Ordering::Relaxed);
    }

    /// Returns the current value of the counter.
    #[inline]
    pub fn count(&self) -> i64 {
        self.0.load(Ordering::Relaxed)
    }
}
