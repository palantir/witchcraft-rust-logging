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
//! Support for forwarding messages from the `log` crate to `witchcraft-log`.
//!
//! Even if your application uses this crate for logging, many of its dependencies probably use the `log` crate.
//! This module can be used to configure the `log` crate to forward its messages to `witchcraft-log`.
//!
//! # Examples
//!
//! ```
//! use witchcraft_log::bridge::BridgedLogger;
//! use witchcraft_log::LevelFilter;
//! # struct MyWitchcraftLogger;
//! # impl witchcraft_log::Log for MyWitchcraftLogger {
//! #    fn enabled(&self, _: &witchcraft_log::Metadata<'_>) -> bool { false }
//! #    fn log(&self, _: &witchcraft_log::Record<'_>) {}
//! #    fn flush(&self) {}
//! # }
//!
//! witchcraft_log::set_logger(&MyWitchcraftLogger);
//! witchcraft_log::set_max_level(LevelFilter::Warn);
//!
//! log::set_logger(&BridgedLogger);
//! // Don't forget to adjust the log crate's max level along with witchcraft_log's!
//! // That won't happen automatically.
//! witchcraft_log::bridge::set_max_level(LevelFilter::Warn);
//! ```

use crate::{Level, LevelFilter, Metadata, Record};
use log::Log;

/// A `log::Log` implementation that forwards records to the `witchcraft-log` logger.
pub struct BridgedLogger;

fn cvt_level(level: log::Level) -> Level {
    match level {
        log::Level::Error => Level::Error,
        log::Level::Warn => Level::Warn,
        log::Level::Info => Level::Info,
        log::Level::Debug => Level::Debug,
        log::Level::Trace => Level::Trace,
    }
}

impl Log for BridgedLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        crate::logger().enabled(
            &Metadata::builder()
                .level(cvt_level(metadata.level()))
                .target(metadata.target())
                .build(),
        )
    }

    fn log(&self, record: &log::Record<'_>) {
        let mut builder = Record::builder();
        builder
            .level(cvt_level(record.level()))
            .target(record.target())
            .file(record.file())
            .line(record.line());

        // If the log message is static, it is safe to log as the WC message. Otherwise, we have to conservatively
        // assume it contains unsafe data.
        let args = record.args();
        let unsafe_params = [("message", args as _)];
        match args.as_str() {
            Some(message) => {
                builder.message(message);
            }
            None => {
                builder.unsafe_params(&unsafe_params);
            }
        }
        crate::logger().log(&builder.build())
    }

    fn flush(&self) {
        crate::logger().flush();
    }
}

/// Sets the `log` crate's max log level.
///
/// This simply translates from a `witchcraft_log::LevelFilter` to a `log::LevelFilter` and calls `log::set_max_level`.
pub fn set_max_level(level: LevelFilter) {
    let level = match level {
        LevelFilter::Trace => log::LevelFilter::Trace,
        LevelFilter::Debug => log::LevelFilter::Debug,
        LevelFilter::Info => log::LevelFilter::Info,
        LevelFilter::Warn => log::LevelFilter::Warn,
        LevelFilter::Error => log::LevelFilter::Error,
        LevelFilter::Fatal | LevelFilter::Off => log::LevelFilter::Off,
    };
    log::set_max_level(level);
}
