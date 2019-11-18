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
use crate::{Counter, Gauge, Histogram, Meter, MetricId, Timer};
use parking_lot::Mutex;
use std::collections::hash_map::Entry;
use std::collections::{hash_map, HashMap};
use std::sync::Arc;

/// An enum of all metric types.
#[derive(Clone)]
pub enum Metric {
    /// A counter metric.
    Counter(Arc<Counter>),
    /// A meter metric.
    Meter(Arc<Meter>),
    /// A gauge metric.
    Gauge(Arc<dyn Gauge>),
    /// A histogram metric.
    Histogram(Arc<Histogram>),
    /// A timer metric.
    Timer(Arc<Timer>),
}

/// A collection of metrics.
///
/// Many of the registry's methods take a `T: Into<MetricId>` rather than just a `MetricId`. This allows you to pass
/// either a full `MetricId` or just a `&str` for more convenient use:
///
/// ```
/// use witchcraft_metrics::{MetricRegistry, MetricId};
///
/// let registry = MetricRegistry::new();
///
/// let requests_meter = registry.meter("server.requests");
/// let yak_shavings = registry.counter(MetricId::new("shavings").with_tag("animal", "yak"));
/// ```
#[derive(Default)]
pub struct MetricRegistry {
    metrics: Mutex<Arc<HashMap<Arc<MetricId>, Metric>>>,
}

impl MetricRegistry {
    /// Creates a new, empty registry.
    #[inline]
    pub fn new() -> MetricRegistry {
        MetricRegistry::default()
    }

    /// Returns the counter with the specified ID, using make_counter to create it if absent.
    ///
    /// # Panics
    ///
    /// Panics if a metric is registered with the ID that is not a counter.
    pub fn counter_with<T, F>(&self, id: T, make_counter: F) -> Arc<Counter>
    where
        T: Into<MetricId>,
        F: FnOnce() -> Counter,
    {
        match Arc::make_mut(&mut self.metrics.lock()).entry(Arc::new(id.into())) {
            Entry::Occupied(e) => match e.get() {
                Metric::Counter(c) => c.clone(),
                _ => panic!("metric already registered as a non-counter: {:?}", e.key()),
            },
            Entry::Vacant(e) => {
                let counter = Arc::new(make_counter());
                e.insert(Metric::Counter(counter.clone()));
                counter
            }
        }
    }

    /// Returns the counter with the specified ID, creating a default instance if absent.
    ///
    /// # Panics
    ///
    /// Panics if a metric is registered with the ID that is not a counter.
    pub fn counter<T>(&self, id: T) -> Arc<Counter>
    where
        T: Into<MetricId>,
    {
        self.counter_with(id, Counter::default)
    }

    /// Returns the meter with the specified ID, using make_meter to create it if absent.
    ///
    /// # Panics
    ///
    /// Panics if a metric is registered with the ID that is not a meter.
    pub fn meter_with<T, F>(&self, id: T, make_meter: F) -> Arc<Meter>
    where
        T: Into<MetricId>,
        F: FnOnce() -> Meter,
    {
        match Arc::make_mut(&mut self.metrics.lock()).entry(Arc::new(id.into())) {
            Entry::Occupied(e) => match e.get() {
                Metric::Meter(m) => m.clone(),
                _ => panic!("metric already registered as a non-meter: {:?}", e.key()),
            },
            Entry::Vacant(e) => {
                let meter = Arc::new(make_meter());
                e.insert(Metric::Meter(meter.clone()));
                meter
            }
        }
    }

    /// Returns the meter with the specified ID, creating a default instance if absent.
    ///
    /// # Panics
    ///
    /// Panics if a metric is registered with the ID that is not a meter.
    pub fn meter<T>(&self, id: T) -> Arc<Meter>
    where
        T: Into<MetricId>,
    {
        self.meter_with(id, Meter::default)
    }

    /// Returns the gauge with the specified ID, registering a new one if absent.
    ///
    /// # Panics
    ///
    /// Panics if a metric is registered with the ID that is not a gauge.
    pub fn gauge<T, G>(&self, id: T, gauge: G) -> Arc<dyn Gauge>
    where
        T: Into<MetricId>,
        G: Gauge,
    {
        match Arc::make_mut(&mut self.metrics.lock()).entry(Arc::new(id.into())) {
            Entry::Occupied(e) => match e.get() {
                Metric::Gauge(m) => m.clone(),
                _ => panic!("metric already registered as a non-gauge: {:?}", e.key()),
            },
            Entry::Vacant(e) => {
                let gauge = Arc::new(gauge);
                e.insert(Metric::Gauge(gauge.clone()));
                gauge
            }
        }
    }

    /// Returns the histogram with the specified ID, using make_histogram to create it if absent.
    ///
    /// # Panics
    ///
    /// Panics if a metric is registered with the ID that is not a histogram.
    pub fn histogram_with<T, F>(&self, id: T, make_histogram: F) -> Arc<Histogram>
    where
        T: Into<MetricId>,
        F: FnOnce() -> Histogram,
    {
        match Arc::make_mut(&mut self.metrics.lock()).entry(Arc::new(id.into())) {
            Entry::Occupied(e) => match e.get() {
                Metric::Histogram(m) => m.clone(),
                _ => panic!(
                    "metric already registered as a non-histogram: {:?}",
                    e.key()
                ),
            },
            Entry::Vacant(e) => {
                let histogram = Arc::new(make_histogram());
                e.insert(Metric::Histogram(histogram.clone()));
                histogram
            }
        }
    }

    /// Returns the histogram with the specified ID, creating a default instance if absent.
    ///
    /// # Panics
    ///
    /// Panics if a metric is registered with the ID that is not a histogram.
    pub fn histogram<T>(&self, id: T) -> Arc<Histogram>
    where
        T: Into<MetricId>,
    {
        self.histogram_with(id, Histogram::default)
    }

    /// Returns the timer with the specified ID, using make_timer to create it if absent.
    ///
    /// # Panics
    ///
    /// Panics if a metric is registered with the ID that is not a timer.
    pub fn timer_with<T, F>(&self, id: T, make_timer: F) -> Arc<Timer>
    where
        T: Into<MetricId>,
        F: FnOnce() -> Timer,
    {
        match Arc::make_mut(&mut self.metrics.lock()).entry(Arc::new(id.into())) {
            Entry::Occupied(e) => match e.get() {
                Metric::Timer(m) => m.clone(),
                _ => panic!("metric already registered as a non-timer: {:?}", e.key()),
            },
            Entry::Vacant(e) => {
                let timer = Arc::new(make_timer());
                e.insert(Metric::Timer(timer.clone()));
                timer
            }
        }
    }

    /// Returns the timer with the specified ID, creating a default instance if absent.
    ///
    /// # Panics
    ///
    /// Panics if a metric is registered with the ID that is not a timer.
    pub fn timer<T>(&self, id: T) -> Arc<Timer>
    where
        T: Into<MetricId>,
    {
        self.timer_with(id, Timer::default)
    }

    /// Removes a metric from the registry, returning it if present.
    pub fn remove<T>(&self, id: T) -> Option<Metric>
    where
        T: Into<MetricId>,
    {
        Arc::make_mut(&mut self.metrics.lock()).remove(&id.into())
    }

    /// Returns a snapshot of the metrics in the registry.
    ///
    /// Modifications to the registry after this method is called will not affect the state of the returned `Metrics`.
    pub fn metrics(&self) -> Metrics {
        Metrics(self.metrics.lock().clone())
    }
}

/// A snapshot of the metrics in a registry.
pub struct Metrics(Arc<HashMap<Arc<MetricId>, Metric>>);

impl Metrics {
    /// Returns an iterator over the metrics.
    pub fn iter(&self) -> MetricsIter<'_> {
        MetricsIter(self.0.iter())
    }
}

impl<'a> IntoIterator for &'a Metrics {
    type Item = (&'a MetricId, &'a Metric);
    type IntoIter = MetricsIter<'a>;

    fn into_iter(self) -> MetricsIter<'a> {
        self.iter()
    }
}

/// An iterator over metrics and their IDs.
pub struct MetricsIter<'a>(hash_map::Iter<'a, Arc<MetricId>, Metric>);

impl<'a> Iterator for MetricsIter<'a> {
    type Item = (&'a MetricId, &'a Metric);

    #[inline]
    fn next(&mut self) -> Option<(&'a MetricId, &'a Metric)> {
        self.0.next().map(|(k, v)| (&**k, v))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> ExactSizeIterator for MetricsIter<'a> {}

#[cfg(test)]
mod test {
    use crate::{MetricId, MetricRegistry};
    use serde_value::Value;
    use std::time::Duration;

    #[test]
    fn first_metric_wins() {
        let registry = MetricRegistry::new();

        let a = registry.counter("counter");
        let b = registry.counter("counter");
        a.add(1);
        assert_eq!(b.count(), 1);

        registry.gauge("gauge", || 1);
        let b = registry.gauge("gauge", || 2);
        assert_eq!(b.value(), Value::I32(1));

        let a = registry.histogram("histogram");
        let b = registry.histogram("histogram");
        a.update(0);
        assert_eq!(b.count(), 1);

        let a = registry.meter("meter");
        let b = registry.meter("meter");
        a.mark(1);
        assert_eq!(b.count(), 1);

        let a = registry.timer("timer");
        let b = registry.timer("timer");
        a.update(Duration::from_secs(0));
        assert_eq!(b.count(), 1);
    }

    #[test]
    fn metrics_returns_snapshot() {
        let registry = MetricRegistry::new();

        registry.counter("counter");

        let metrics = registry.metrics();

        registry.timer("timer");

        let metrics = metrics.iter().collect::<Vec<_>>();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].0, &MetricId::new("counter"));
    }

    #[test]
    fn tagged_distinct_from_untagged() {
        let registry = MetricRegistry::new();

        let a = registry.counter("counter");
        let b = registry.counter(MetricId::new("counter").with_tag("foo", "bar"));
        a.inc();
        assert_eq!(b.count(), 0);
    }
}
