// Copyright 2024 Palantir Technologies, Inc.
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

//! A reservoir which exponentially weights in favor of recent values.

use crate::{Clock, Exemplar, Reservoir, Snapshot};
use exponential_decay_histogram::ExponentialDecayHistogram;
use parking_lot::Mutex;
use std::sync::Arc;

/// A reservoir which exponentially weights in favor of recent values.
pub struct ExponentiallyDecayingReservoir {
    histogram: Mutex<ExponentialDecayHistogram<Option<Arc<dyn Exemplar>>>>,
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
        Self::builder().build()
    }

    /// Creates a new builder.
    pub fn builder() -> Builder {
        Builder {
            clock: crate::SYSTEM_CLOCK.clone(),
            exemplar_provider: Arc::new(|| None),
        }
    }

    /// Creates a new reservoir using the provided [`Clock`] as its time source.
    #[deprecated(note = "Use ExponentiallyDecayingReservoir::builder", since = "1.0.2")]
    pub fn new_with(clock: Arc<dyn Clock>) -> Self {
        Self::builder().clock(clock).build()
    }
}

/// A builder for `[ExponentiallyDecayingReservoir]`s.
pub struct Builder {
    clock: Arc<dyn Clock>,
    exemplar_provider: Arc<dyn Fn() -> Option<Arc<dyn Exemplar>> + Sync + Send>,
}

impl Builder {
    /// Sets the [`Clock`] used as the reservoir's time source.
    #[inline]
    pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Sets the provider used to associate [`Exemplar`]s with each measurement.
    #[inline]
    pub fn exemplar_provider(
        mut self,
        exemplar_provider: Arc<dyn Fn() -> Option<Arc<dyn Exemplar>> + Sync + Send>,
    ) -> Self {
        self.exemplar_provider = exemplar_provider;
        self
    }

    /// Creates the reservoir.
    #[inline]
    pub fn build(self) -> ExponentiallyDecayingReservoir {
        ExponentiallyDecayingReservoir {
            histogram: Mutex::new(
                ExponentialDecayHistogram::builder()
                    .at(self.clock.now())
                    .exemplar_provider(move || (self.exemplar_provider)())
                    .build(),
            ),
            clock: self.clock,
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

impl Snapshot for exponential_decay_histogram::Snapshot<Option<Arc<dyn Exemplar>>> {
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

    fn exemplars<'a>(&'a self) -> Box<dyn Iterator<Item = (i64, &'a Arc<dyn Exemplar>)> + 'a> {
        Box::new(
            self.exemplars()
                .filter_map(|(value, exemplar)| exemplar.as_ref().map(|e| (value, e))),
        )
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod test {
    use crate::exponentially_decaying::ExponentiallyDecayingReservoir;
    use crate::Reservoir;

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
