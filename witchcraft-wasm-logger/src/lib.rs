// Copyright 2025 Palantir Technologies, Inc.
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
//! A simple Witchcraft logger that can be used in a WebAssembly to log messages to browser console.
//!
//! Witchcraft log levels are mapped to console functions as follows:
//!
//! | Witchcraft| Web Console       |
//! |------------|-------------------|
//! | `trace!()` | `console.debug()` |
//! | `debug!()` | `console.log()`   |
//! | `info!()`  | `console.info()`  |
//! | `warn!()`  | `console.warn()`  |
//! | `error!()` | `console.error()` |
//!
//! This mapping is similar to the [console_log crate](https://crates.io/crates/console_log).
//!
//! Logs are written to standard error in the standard Witchcraft `service.1` JSON format.
//!
#![warn(missing_docs)]

use chrono::SecondsFormat;
use serde::Serialize;
use web_sys::console;
use witchcraft_log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use witchcraft_log_util::{filter::Filter, service};
use witchcraft_logging_api::objects::ServiceLogV1;

struct Logger {
    filter: Filter,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.filter.enabled(metadata)
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let console_log = match record.level() {
            Level::Error | Level::Fatal => console::error_2,
            Level::Warn => console::warn_2,
            Level::Info => console::info_2,
            Level::Debug => console::log_2,
            Level::Trace => console::debug_2,
        };

        let service_log = service::from_record(record);
        let message = slslog(&service_log, record);
        let full_record = serde_wasm_bindgen::to_value(&service_log).unwrap();

        console_log(&message.into(), &full_record);

        // For errors and warns, at least get a stacktrace from where the log is coming from.
        if record.level() == Level::Error || record.level() == Level::Warn {
            console::trace_0(); // this logs the current stacktrace to console
        }
    }

    fn flush(&self) {}
}

/// Initializes the WASM environment with this logger and provided filter. Logger can only be
/// initialized once; else, an error is returned.
pub fn try_init(filter: Filter) -> Result<(), SetLoggerError> {
    let max_level = filter.max_level();
    witchcraft_log::set_boxed_logger(Box::new(Logger { filter }))?;
    witchcraft_log::set_max_level(max_level);

    Ok(())
}

/// Initializes the WASM environment with this logger and a global maximum log level. Logger can
/// only be initialized once; else, an error is returned.
pub fn try_init_with_level(level: LevelFilter) -> Result<(), SetLoggerError> {
    let filter = Filter::builder().level(level).build();
    try_init(filter)
}

// This mostly approximates what the slslog binary outputs
fn slslog(service_log: &ServiceLogV1, original_log: &Record<'_>) -> String {
    let mut msg = format!(
        "{:<5} [{}] {}: {} (file: {}, line: {})",
        service_log.level(),
        service_log
            .time()
            .to_rfc3339_opts(SecondsFormat::Millis, true),
        service_log.origin().unwrap_or(""),
        service_log.message(),
        original_log.file().unwrap_or(""),
        original_log.line().unwrap_or(0),
    );

    msg += get_params(service_log.params().iter(), true).as_str();
    msg += get_params(service_log.unsafe_params().iter(), true).as_str();

    msg
}

fn get_params<'a, T: Serialize + 'a, M: Iterator<Item = (&'a String, &'a T)>>(
    params: M,
    prefix_first: bool,
) -> String {
    let mut msg: String = String::new();
    let mut first = true;
    for (name, value) in params {
        if "file" == name || "line" == name {
            // Already printed in a specific format above
            continue;
        }
        if !first || prefix_first {
            msg += ", ";
        }
        first = false;
        msg += name.as_str();
        msg += ": ";

        let serialized_value = serde_json::to_value(value).unwrap();
        if serialized_value.is_object() {
            let obj = serialized_value.as_object().unwrap();
            msg += " map[";
            msg += get_params(obj.iter(), false).as_str();
            msg += "]";
        } else {
            msg += serde_json::to_string(&serialized_value).unwrap().as_str();
        }
    }
    msg
}