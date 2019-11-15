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
use parking_lot::Mutex;
use std::convert::TryFrom;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::Instant;

const INTERVAL_SECS: u64 = 5;
const SECONDS_PER_MINUTE: f64 = 60.;

struct State {
    count: i64,
    rate_1m: Ewma,
    rate_5m: Ewma,
    rate_15m: Ewma,
}

/// A metric tracking the rate of occurrence of an event.
///
/// The meter tracks rolling average rates in the same manner as the Linux kernel's load factor measurement.
pub struct Meter {
    uncounted: AtomicI64,
    last_tick: AtomicU64,
    start_time: Instant,
    state: Mutex<State>,
}

impl Default for Meter {
    fn default() -> Meter {
        Meter::new()
    }
}

impl Meter {
    /// Creates a new meter.
    pub fn new() -> Meter {
        Meter::new_at(Instant::now())
    }

    fn new_at(start_time: Instant) -> Meter {
        Meter {
            uncounted: AtomicI64::new(0),
            last_tick: AtomicU64::new(0),
            start_time,
            state: Mutex::new(State {
                count: 0,
                rate_1m: Ewma::new(1.),
                rate_5m: Ewma::new(5.),
                rate_15m: Ewma::new(15.),
            }),
        }
    }

    /// Mark the occurrence of `n` event(s).
    pub fn mark(&self, n: i64) {
        self.mark_at(Instant::now(), n);
    }

    fn mark_at(&self, time: Instant, n: i64) {
        self.tick_if_necessary(time);
        self.uncounted.fetch_add(n, Ordering::SeqCst);
    }

    /// Returns the number of events registered by the meter.
    pub fn count(&self) -> i64 {
        self.state.lock().count + self.uncounted.load(Ordering::SeqCst)
    }

    /// Returns the one minute rolling average rate of the occurrence of events measured in events per second.
    pub fn one_minute_rate(&self) -> f64 {
        self.one_minute_rate_at(Instant::now())
    }

    fn one_minute_rate_at(&self, now: Instant) -> f64 {
        self.tick_if_necessary(now);
        self.state.lock().rate_1m.get()
    }

    /// Returns the five minute rolling average rate of the occurrence of events measured in events per second.
    pub fn five_minute_rate(&self) -> f64 {
        self.five_minute_rate_at(Instant::now())
    }

    fn five_minute_rate_at(&self, now: Instant) -> f64 {
        self.tick_if_necessary(now);
        self.state.lock().rate_5m.get()
    }

    /// Returns the fifteen minute rolling average rate of the occurrence of events measured in events per second.
    pub fn fifteen_minute_rate(&self) -> f64 {
        self.fifteen_minute_rate_at(Instant::now())
    }

    fn fifteen_minute_rate_at(&self, now: Instant) -> f64 {
        self.tick_if_necessary(now);
        self.state.lock().rate_15m.get()
    }

    /// Returns the mean rate of the occurrence of events since the creation of the meter measured in events per second.
    pub fn mean_rate(&self) -> f64 {
        self.mean_rate_at(Instant::now())
    }

    fn mean_rate_at(&self, now: Instant) -> f64 {
        let count = self.count() as f64;
        if count == 0. {
            0.
        } else {
            let time = (now - self.start_time).as_secs_f64();
            count / time
        }
    }

    fn tick_if_necessary(&self, time: Instant) {
        let old_tick = self.last_tick.load(Ordering::SeqCst);
        let new_tick = (time - self.start_time).as_secs();
        let age = new_tick - old_tick;

        if age < INTERVAL_SECS {
            return;
        }

        let new_interval_start_tick = new_tick - age % INTERVAL_SECS;
        if self
            .last_tick
            .compare_exchange(
                old_tick,
                new_interval_start_tick,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_err()
        {
            // another thread has already ticked for us
            return;
        }

        let required_ticks = age / INTERVAL_SECS;
        let mut state = self.state.lock();

        let uncounted = self.uncounted.swap(0, Ordering::SeqCst);
        state.count += uncounted;

        state.rate_1m.tick(uncounted);
        state.rate_1m.decay(required_ticks - 1);

        state.rate_5m.tick(uncounted);
        state.rate_5m.decay(required_ticks - 1);

        state.rate_15m.tick(uncounted);
        state.rate_15m.decay(required_ticks - 1);
    }
}

// Modeled after Java metrics-core's EWMA.java
struct Ewma {
    rate: f64,
    alpha: f64,
    initialized: bool,
}

impl Ewma {
    fn new(minutes: f64) -> Ewma {
        Ewma {
            rate: 0.,
            alpha: 1. - (-(INTERVAL_SECS as f64) / SECONDS_PER_MINUTE / minutes).exp(),
            initialized: false,
        }
    }

    fn tick(&mut self, count: i64) {
        let instant_rate = count as f64 / INTERVAL_SECS as f64;
        if self.initialized {
            self.rate += self.alpha * (instant_rate - self.rate);
        } else {
            self.rate = instant_rate;
            self.initialized = true;
        }
    }

    /// Equivalent to calling ewma.tick(0) `ticks` times, but isn't linear in `ticks`.
    ///
    /// x1 = x0 + alpha * (0 - x0)
    /// x1 = x0 - alpha * x0
    /// x1 = x0 * (1 - alpha)
    ///
    /// x2 = x1 * (1 - alpha)
    /// x2 = x0 * (1 - alpha) * (1 - alpha)
    fn decay(&mut self, ticks: u64) {
        match i32::try_from(ticks) {
            Ok(ticks) => self.rate *= (1. - self.alpha).powi(ticks),
            Err(_) => self.rate = 0.,
        }
    }

    fn get(&self) -> f64 {
        self.rate
    }
}

#[cfg(test)]
mod test {
    use crate::Meter;
    use assert_approx_eq::assert_approx_eq;
    use std::time::{Duration, Instant};

    #[test]
    #[allow(clippy::float_cmp)]
    fn starts_out_with_no_rates_or_count() {
        let time = Instant::now();
        let meter = Meter::new_at(time);

        assert_eq!(meter.count(), 0);
        assert_eq!(meter.one_minute_rate_at(time), 0.);
        assert_eq!(meter.five_minute_rate_at(time), 0.);
        assert_eq!(meter.fifteen_minute_rate_at(time), 0.);
        assert_eq!(meter.mean_rate_at(time), 0.)
    }

    #[test]
    fn marks_events_and_updates_rate_and_count() {
        let time = Instant::now();
        let meter = Meter::new_at(time);

        meter.mark_at(time, 1);

        let time = time + Duration::from_secs(10);
        meter.mark_at(time, 2);

        assert_approx_eq!(meter.mean_rate_at(time), 0.3, 0.001);
        assert_approx_eq!(meter.one_minute_rate_at(time), 0.1840, 0.001);
        assert_approx_eq!(meter.five_minute_rate_at(time), 0.1966, 0.001);
        assert_approx_eq!(meter.fifteen_minute_rate_at(time), 0.1988, 0.001);
    }
}
