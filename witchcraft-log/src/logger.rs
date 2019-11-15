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
use crate::{LevelFilter, Metadata, Record};
use lazycell::AtomicLazyCell;
use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fmt, mem};

/// A trait encapsulating the operations required of a logger.
pub trait Log: Sync + Send {
    /// Determines ifa log message with the specified metadata would be logged.
    ///
    /// This is used by the `enabled!` macro to allow callers to avoid expensive computation of log message parameters
    /// if the message would be discarded anyway.
    fn enabled(&self, metadata: &Metadata<'_>) -> bool;

    /// Logs a `Record`.
    ///
    /// Note that `enabled` is *not* necessarily called before this method. Implementations of `log` should perform all
    /// necessary filtering internally.
    fn log(&self, record: &Record<'_>);

    /// Flushes any buffered records.
    fn flush(&self);
}

struct NopLogger;

impl Log for NopLogger {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        false
    }

    fn log(&self, _: &Record<'_>) {}

    fn flush(&self) {}
}

static LOGGER: AtomicLazyCell<&'static dyn Log> = AtomicLazyCell::NONE;

/// Sets the global logger.
///
/// The global logger can only be set once. Further calls will return an error.
pub fn set_logger(logger: &'static dyn Log) -> Result<(), SetLoggerError> {
    LOGGER.fill(logger).map_err(|_| SetLoggerError(()))
}

/// Returns the global logger.
///
/// If one has not been set, a no-op implementation will be returned.
pub fn logger() -> &'static dyn Log {
    LOGGER.get().unwrap_or(&NopLogger)
}

/// An error trying to set the logger when one is already installed.
#[derive(Debug)]
pub struct SetLoggerError(());

impl fmt::Display for SetLoggerError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("a logger is already installed")
    }
}

impl Error for SetLoggerError {}

static MAX_LOG_LEVEL_FILTER: AtomicUsize = AtomicUsize::new(0);

/// Sets the global maximum log level.
///
/// Generally, this should only be called by the logging implementation.
pub fn set_max_level(level: LevelFilter) {
    MAX_LOG_LEVEL_FILTER.store(level as usize, Ordering::Relaxed);
}

/// Returns the current maximum log level.
///
/// The log macros check this value and discard any message logged at a higher level as an optimization. The maximum
/// level is set by the `set_max_level` function.
#[inline(always)]
pub fn max_level() -> LevelFilter {
    unsafe { mem::transmute(MAX_LOG_LEVEL_FILTER.load(Ordering::Relaxed)) }
}
