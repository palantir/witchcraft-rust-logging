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
//! A simple Witchcraft logger that can be configured with an environment variable.
//!
//! This is similar to the [env_logger](https://docs.rs/env_logger) crate, but using the [`witchcraft_log`] crate
//! instead of the `log` crate. Configuration of logging levels is the same as `env_logger` except for the additional
//! `fatal` log level. Regex filters are not supported.
//!
//! Logs are written to standard error in the standard Witchcraft `service.1` JSON format.
//!
//! # Example
//!
//! ```
//! use witchcraft_log::{debug, error, info, Level};
//!
//! witchcraft_env_logger::init();
//!
//! debug!("this is a debug message");
//! error!("this is printed by default");
//!
//! if witchcraft_log::enabled!(Level::Info) {
//!     let x = 3 * 4; // expensive computation
//!     info!("figured out the answer", safe: { answer: x });
//! }
//! ```
//!
//! ```not_rust
//! $ RUST_LOG=error ./main
//! {"type":"service.1","level":"ERROR","time":"2025-05-26T16:45:59.204677531Z","origin":"main","thread":"main","message":"this is printed by default","safe":true,"params":{"file":"witchcraft-env-logger/examples/main.rs","line":7}}
//! ```
//!
//! ```not_rust
//! $ RUST_LOG=info ./main
//! {"type":"service.1","level":"ERROR","time":"2025-05-26T16:46:31.043928664Z","origin":"main","thread":"main","message":"this is printed by default","safe":true,"params":{"file":"witchcraft-env-logger/examples/main.rs","line":7}}
//! {"type":"service.1","level":"INFO","time":"2025-05-26T16:46:31.043976765Z","origin":"main","thread":"main","message":"figured out the answer","safe":true,"params":{"answer":12,"file":"witchcraft-env-logger/examples/main.rs","line":11}}
//! ```
//!
//! ```not_rust
//! $ RUST_LOG=main=debug ./main
//! {"type":"service.1","level":"DEBUG","time":"2025-05-26T16:47:04.831643644Z","origin":"main","thread":"main","message":"this is a debug message","safe":true,"params":{"file":"witchcraft-env-logger/examples/main.rs","line":6}}
//! {"type":"service.1","level":"ERROR","time":"2025-05-26T16:47:04.831691314Z","origin":"main","thread":"main","message":"this is printed by default","safe":true,"params":{"file":"witchcraft-env-logger/examples/main.rs","line":7}}
//! {"type":"service.1","level":"INFO","time":"2025-05-26T16:47:04.831720469Z","origin":"main","thread":"main","message":"figured out the answer","safe":true,"params":{"answer":12,"file":"witchcraft-env-logger/examples/main.rs","line":11}}
//! ```
#![warn(missing_docs)]

use std::{
    env,
    io::{self, Write},
};

use conjure_serde::json;
use witchcraft_log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
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

        let service_log = service::from_record(record);
        let mut buf = json::to_string(&service_log).unwrap();
        buf.push('\n');
        // Using the macro so output is intercepted in tests properly
        eprint!("{buf}");
    }

    fn flush(&self) {
        let _ = io::stderr().flush();
    }
}

/// Initializes the global logger, reading configuration from the `RUST_LOG` environment variable.
///
/// Returns an error if the logger is already initialized.
pub fn try_init() -> Result<(), SetLoggerError> {
    let mut builder = Filter::builder();

    if let Ok(rust_log) = env::var("RUST_LOG") {
        for directive in rust_log.split(",") {
            let mut it = directive.splitn(2, "=");
            let first = it.next().unwrap();
            let second = it.next();

            match second {
                Some(level) => {
                    if let Ok(level) = level.parse::<LevelFilter>() {
                        builder = builder.target_level(first, level);
                    };
                }
                None => match first.parse::<LevelFilter>() {
                    Ok(level) => builder = builder.level(level),
                    Err(_) => builder = builder.target_level(first, LevelFilter::Trace),
                },
            }
        }
    }

    let filter = builder.build();
    let max_level = filter.max_level();

    witchcraft_log::set_boxed_logger(Box::new(Logger { filter }))?;
    witchcraft_log::set_max_level(max_level);

    Ok(())
}

/// Like [`try_init()`], but panics if the logger is already initialized.
pub fn init() {
    try_init().unwrap();
}
