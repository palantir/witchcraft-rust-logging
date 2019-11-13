use crate::{ExponentiallyDecayingReservoir, Meter, Reservoir, Snapshot};
use std::time::{Duration, Instant};

/// A metric tracking the duration and rate of events.
///
/// The timer's default reservoir implementation (used by its `Default` implementation) is the
/// `ExponentiallyDecayingReservoir`.
pub struct Timer {
    meter: Meter,
    reservoir: Box<dyn Reservoir>,
}

impl Default for Timer {
    #[inline]
    fn default() -> Timer {
        Timer::new(ExponentiallyDecayingReservoir::new())
    }
}

impl Timer {
    /// Creates a new timer.
    pub fn new<R>(reservoir: R) -> Timer
    where
        R: Reservoir,
    {
        Timer {
            meter: Meter::new(),
            reservoir: Box::new(reservoir),
        }
    }

    /// Adds a new timed event to the metric.
    #[inline]
    pub fn update(&self, duration: Duration) {
        self.meter.mark(1);
        let nanos = duration.as_nanos() as i64;
        self.reservoir.update(nanos);
    }

    /// Returns a guard type which reports the time elapsed since its creation when it drops.
    #[inline]
    pub fn time(&self) -> Time<'_> {
        Time {
            timer: self,
            start: Instant::now(),
        }
    }

    /// Returns the number of events reported to the metric.
    #[inline]
    pub fn count(&self) -> i64 {
        self.meter.count()
    }

    /// Returns the one minute rolling average rate of the occurrence of events measured in events per second.
    #[inline]
    pub fn one_minute_rate(&self) -> f64 {
        self.meter.one_minute_rate()
    }

    /// Returns the five minute rolling average rate of the occurrence of events measured in events per second.
    #[inline]
    pub fn five_minute_rate(&self) -> f64 {
        self.meter.five_minute_rate()
    }

    /// Returns the fifteen minute rolling average rate of the occurrence of events measured in events per second.
    #[inline]
    pub fn fifteen_minute_rate(&self) -> f64 {
        self.meter.fifteen_minute_rate()
    }

    /// Returns the mean rate of the occurrence of events since the creation of the timer measured in events per second.
    #[inline]
    pub fn mean_rate(&self) -> f64 {
        self.meter.mean_rate()
    }

    /// Returns a snapshot of the statistical distribution of durations of events, measured in nanoseconds.
    #[inline]
    pub fn snapshot(&self) -> Box<dyn Snapshot> {
        self.reservoir.snapshot()
    }
}

/// A guard type which reports the time elapsed since its creation to a timer when it drops.
pub struct Time<'a> {
    timer: &'a Timer,
    start: Instant,
}

impl Drop for Time<'_> {
    #[inline]
    fn drop(&mut self) {
        self.timer.update(self.start.elapsed());
    }
}
