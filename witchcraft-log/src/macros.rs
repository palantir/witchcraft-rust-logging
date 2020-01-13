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

/// Logs a message at the specified level.
#[macro_export]
macro_rules! log {
    ($lvl:expr, $msg:expr) => {{
        let level = $lvl;
        if level <= $crate::max_level() {
            $crate::private::log_minimal(
                level,
                &(module_path!(), file!(), line!(), $msg),
            );
        }
    }};
    (
        $lvl:expr,
        $msg:expr
        $(, safe: { $($safe_key:ident: $safe_value:expr),* $(,)? })?
        $(, unsafe: { $($unsafe_key:ident: $unsafe_value:expr),* $(,)? })?
        $(, error: $error:expr)?
        $(,)?
    ) => {{
        let level = $lvl;
        if level <= $crate::max_level() {
            $crate::private::log(
                level,
                &(module_path!(), file!(), line!(), $msg),
                &[$($((stringify!($safe_key), &$safe_value)),*)*],
                &[$($((stringify!($unsafe_key), &$unsafe_value)),*)*],
                None $(.or(Some(&$error)))?,
            );
        }
    }};
}

/// Logs a message at the "fatal" level.
#[macro_export]
macro_rules! fatal {
    ($($v:tt)*) => {
        $crate::log!($crate::Level::Fatal, $($v)*)
    }
}

/// Logs a message at the "error" level.
#[macro_export]
macro_rules! error {
    ($($v:tt)*) => {
        $crate::log!($crate::Level::Error, $($v)*)
    }
}

/// Logs a message at the "warn" level.
#[macro_export]
macro_rules! warn {
    ($($v:tt)*) => {
        $crate::log!($crate::Level::Warn, $($v)*)
    }
}

/// Logs a message at the "info" level.
#[macro_export]
macro_rules! info {
    ($($v:tt)*) => {
        $crate::log!($crate::Level::Info, $($v)*)
    }
}

/// Logs a message at the "debug" level.
#[macro_export]
macro_rules! debug {
    ($($v:tt)*) => {
        $crate::log!($crate::Level::Debug, $($v)*)
    }
}

/// Logs a message at the "trace" level.
#[macro_export]
macro_rules! trace {
    ($($v:tt)*) => {
        $crate::log!($crate::Level::Trace, $($v)*)
    }
}

/// Determines if a message logged at the specified level in the same module would be logged or not.
#[macro_export]
macro_rules! enabled {
    ($lvl:expr) => {{
        let level = $lvl;
        level <= $crate::max_level() && $crate::private::enabled(level, module_path!())
    }};
}
