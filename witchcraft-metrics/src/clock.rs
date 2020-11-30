// Copyright 2020 Palantir Technologies, Inc.
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
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::time::Instant;

pub(crate) static SYSTEM_CLOCK: Lazy<Arc<SystemClock>> = Lazy::new(|| Arc::new(SystemClock));

/// A source of monotonic time.
pub trait Clock: 'static + Sync + Send {
    /// Returns the current time.
    fn now(&self) -> Instant;
}

/// A `Clock` implementation which uses the system clock.
pub struct SystemClock;

impl Clock for SystemClock {
    #[inline]
    fn now(&self) -> Instant {
        Instant::now()
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use parking_lot::Mutex;
    use std::time::Duration;

    pub struct TestClock {
        now: Mutex<Instant>,
    }

    impl TestClock {
        #[allow(clippy::new_without_default)]
        pub fn new() -> TestClock {
            TestClock {
                now: Mutex::new(Instant::now()),
            }
        }

        pub fn advance(&self, dur: Duration) {
            *self.now.lock() += dur;
        }
    }

    impl Clock for TestClock {
        fn now(&self) -> Instant {
            *self.now.lock()
        }
    }
}
