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
use crate::{ExponentiallyDecayingReservoir, Reservoir, Snapshot};
use std::sync::atomic::{AtomicU64, Ordering};

/// A metric tracking a statistical distribution of values.
///
/// The histogram's default reservoir implementation (used by its `Default` implementation) is the
/// `ExponentiallyDecayingReservoir`.
pub struct Histogram {
    count: AtomicU64,
    reservoir: Box<dyn Reservoir>,
}

impl Default for Histogram {
    #[inline]
    fn default() -> Histogram {
        Histogram::new(ExponentiallyDecayingReservoir::new())
    }
}

impl Histogram {
    /// Creates a new histogram using the provided reservoir.
    pub fn new<R>(reservoir: R) -> Histogram
    where
        R: Reservoir,
    {
        Histogram {
            count: AtomicU64::new(0),
            reservoir: Box::new(reservoir),
        }
    }

    /// Adds a value to the histogram.
    #[inline]
    pub fn update(&self, value: i64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.reservoir.update(value);
    }

    /// Returns the number of values added to the histogram.
    #[inline]
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Returns a snapshot of the statistical distribution of values.
    #[inline]
    pub fn snapshot(&self) -> Box<dyn Snapshot> {
        self.reservoir.snapshot()
    }
}
