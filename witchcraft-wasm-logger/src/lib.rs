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
//! Logs are written to standard error in the standard Witchcraft `service.1` JSON format.
//!
use conjure_serde::json;
use web_sys::console;
use witchcraft_log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use witchcraft_log_util::{filter::Filter, service};
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
            Level::Error | Level::Fatal => console::error_1,
            Level::Warn => console::warn_1,
            Level::Info => console::info_1,
            Level::Debug => console::debug_1,
            Level::Trace => console::trace_1,
        };

        let service_log = service::from_record(record);
        let buf = json::to_string(&service_log).unwrap();
        console_log(&buf.into());
    }

    fn flush(&self) {}
}

pub fn try_init(filter: Filter) -> Result<(), SetLoggerError> {
    let max_level = filter.max_level();
    witchcraft_log::set_boxed_logger(Box::new(Logger { filter }))?;
    witchcraft_log::set_max_level(max_level);

    Ok(())
}

pub fn try_init_with_level(level: LevelFilter) -> Result<(), SetLoggerError> {
    let filter = Filter::builder().level(level).build();
    try_init(filter)
}
