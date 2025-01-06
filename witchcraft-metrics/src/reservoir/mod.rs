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
#[doc(inline)]
pub use crate::reservoir::exponentially_decaying::ExponentiallyDecayingReservoir;
use crate::Exemplar;
use std::iter;
use std::sync::Arc;

pub mod exponentially_decaying;

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

    /// Returns an iterator over the values in the snapshot with associated exemplars.
    ///
    /// The default implementation returns an empty iterator.
    fn exemplars<'a>(&'a self) -> Box<dyn Iterator<Item = (i64, &'a Arc<dyn Exemplar>)> + 'a> {
        Box::new(iter::empty())
    }
}
