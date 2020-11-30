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
use crate::Clock;
use exponential_decay_histogram::ExponentialDecayHistogram;
use parking_lot::Mutex;
use std::sync::Arc;

/// A statistically representative subset of a set of values.
pub trait Reservoir: 'static + Sync + Send {
    /// Adds a value to the reservoir.
    fn update(&self, value: i64);

    /// Returns a snapshot of statistics about the values in the reservoir.
    fn snapshot(&self) -> Box<dyn Snapshot>;
}

/// Statistics about a set of values.
pub trait Snapshot: 'static + Sync + Send {
    /// Returns the value at the specified quantile (from 0 to 1 inclusive), or 0 if empty.
    ///
    /// For example, `snapshot.value(0.5)` returns the median value.
    ///
    /// # Panics
    ///
    /// Panics if quantile is less than 0 or greater than 1.
    fn value(&self, quantile: f64) -> f64;

    /// Returns the maximum value in the snapshot, or 0 if empty.
    fn max(&self) -> i64;

    /// Returns the minimum value in the snapshot, or 0 if empty.
    fn min(&self) -> i64;

    /// Returns the average value in the snapshot, or 0 if empty.
    fn mean(&self) -> f64;

    /// Returns the standard deviation of the values in the snapshot.
    fn stddev(&self) -> f64;
}

/// A reservoir which exponentially weights in favor of recent values.
pub struct ExponentiallyDecayingReservoir {
    histogram: Mutex<ExponentialDecayHistogram>,
    clock: Arc<dyn Clock>,
}

impl Default for ExponentiallyDecayingReservoir {
    fn default() -> Self {
        Self::new()
    }
}

impl ExponentiallyDecayingReservoir {
    /// Creates a new reservoir with a [`SystemClock`](crate::SystemClock).
    pub fn new() -> Self {
        Self::new_with(crate::SYSTEM_CLOCK.clone())
    }

    /// Creates a new reservoir using the provided [`Clock`] as its time source.
    pub fn new_with(clock: Arc<dyn Clock>) -> Self {
        ExponentiallyDecayingReservoir {
            histogram: Mutex::new(ExponentialDecayHistogram::builder().at(clock.now()).build()),
            clock,
        }
    }
}

impl Reservoir for ExponentiallyDecayingReservoir {
    fn update(&self, value: i64) {
        self.histogram.lock().update_at(self.clock.now(), value);
    }

    fn snapshot(&self) -> Box<dyn Snapshot> {
        Box::new(self.histogram.lock().snapshot())
    }
}

impl Snapshot for exponential_decay_histogram::Snapshot {
    fn value(&self, quantile: f64) -> f64 {
        self.value(quantile) as f64
    }

    fn max(&self) -> i64 {
        self.max()
    }

    fn min(&self) -> i64 {
        self.min()
    }

    fn mean(&self) -> f64 {
        self.mean()
    }

    fn stddev(&self) -> f64 {
        self.stddev()
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod test {
    use crate::{ExponentiallyDecayingReservoir, Reservoir};

    #[test]
    fn exponential_basic() {
        let reservoir = ExponentiallyDecayingReservoir::new();

        for _ in 0..15 {
            reservoir.update(0);
        }

        for _ in 0..5 {
            reservoir.update(5);
        }

        let snapshot = reservoir.snapshot();

        assert_eq!(snapshot.value(0.5), 0.);
        assert_eq!(snapshot.value(0.8), 5.);
        assert_eq!(snapshot.max(), 5);
        assert_eq!(snapshot.min(), 0);
        assert_eq!(snapshot.mean(), 1.25);
        assert!((snapshot.stddev() - 2.165).abs() < 0.0001);
    }
}
