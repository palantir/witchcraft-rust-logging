// Copyright 2019 Palantir Technologies, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
