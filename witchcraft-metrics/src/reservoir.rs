use exponential_decay_histogram::ExponentialDecayHistogram;
use parking_lot::Mutex;

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
#[derive(Default)]
pub struct ExponentiallyDecayingReservoir(Mutex<ExponentialDecayHistogram>);

impl ExponentiallyDecayingReservoir {
    /// Creates a new reservoir.
    pub fn new() -> ExponentiallyDecayingReservoir {
        ExponentiallyDecayingReservoir::default()
    }
}

impl Reservoir for ExponentiallyDecayingReservoir {
    fn update(&self, value: i64) {
        self.0.lock().update(value);
    }

    fn snapshot(&self) -> Box<dyn Snapshot> {
        Box::new(self.0.lock().snapshot())
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
