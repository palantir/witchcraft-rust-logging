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
//! Utilities for Witchcraft service logs.
use std::fmt::Write;
use std::{error, thread};

use conjure_error::ErrorKind;
use conjure_object::Utc;
use witchcraft_log::{Level, Record, mdc};
use witchcraft_logging_api::objects::{
    LogLevel, OrganizationId, ServiceLogV1, SessionId, TokenId, TraceId, UserId,
};

/// Serialize a `witchcraft-log` record into a standard `ServiceLogV1` object.
pub fn from_record(record: &Record<'_>) -> ServiceLogV1 {
    let level = match record.level() {
        Level::Fatal => LogLevel::Fatal,
        Level::Error => LogLevel::Error,
        Level::Warn => LogLevel::Warn,
        Level::Info => LogLevel::Info,
        Level::Debug => LogLevel::Debug,
        Level::Trace => LogLevel::Trace,
    };

    let mut message = ServiceLogV1::builder()
        .type_("service.1")
        .level(level)
        .time(Utc::now())
        .message(record.message())
        .safe(true)
        .origin(record.target().to_string())
        .thread(thread::current().name().map(ToString::to_string));

    let mdc = mdc::snapshot();
    for (key, value) in mdc.safe().iter() {
        match key {
            crate::mdc::UID_KEY => {
                if let Ok(uid) = value.clone().deserialize_into::<UserId>() {
                    message = message.uid(uid);
                }
            }
            crate::mdc::SID_KEY => {
                if let Ok(sid) = value.clone().deserialize_into::<SessionId>() {
                    message = message.sid(sid);
                }
            }
            crate::mdc::TOKEN_ID_KEY => {
                if let Ok(token_id) = value.clone().deserialize_into::<TokenId>() {
                    message = message.token_id(token_id);
                }
            }
            crate::mdc::ORG_ID_KEY => {
                if let Ok(org_id) = value.clone().deserialize_into::<OrganizationId>() {
                    message = message.org_id(org_id);
                }
            }
            crate::mdc::TRACE_ID_KEY => {
                if let Ok(trace_id) = value.clone().deserialize_into::<TraceId>() {
                    message = message.trace_id(trace_id);
                }
            }
            key => message = message.insert_params(key, value),
        }
    }
    message = message.extend_unsafe_params(
        mdc.unsafe_()
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone())),
    );

    if let Some(file) = record.file() {
        message = message.insert_params("file", file);
    }
    if let Some(line) = record.line() {
        message = message.insert_params("line", line);
    }
    if let Some(error) = record.error() {
        if let ErrorKind::Service(s) = error.kind() {
            message = message
                .insert_params("errorInstanceId", s.error_instance_id())
                .insert_params("errorCode", s.error_code())
                .insert_params("errorName", s.error_name());
        }

        let mut stacktrace = String::new();
        for trace in error.backtraces() {
            // Render each backtrace twice and merge per frame:
            //   * plain `{:?}`  -> PrintFmt::Short -> `<idx>: <name>` (no address)
            //   * alt   `{:#?}` -> PrintFmt::Full  -> `<idx>: 0x<ip> - <name>` (+ `at file:line`)
            // For frames the in-process symbolizer *can* name, keep the concise short line. For
            // frames it cannot (a stripped library resolves to `<unknown>`), keep the full,
            // address-bearing line so the frame can still be reconstructed offline via
            // `ip - baseAddress` against a build-id-matched debug companion. This keeps named
            // frames readable while preserving addresses only where there's nothing else to go on.
            writeln!(stacktrace, "{}", merge_backtrace(&format!("{trace:?}"), &format!("{trace:#?}")))
                .unwrap();
        }
        message = message.stacktrace(stacktrace);

        let mut causes = vec![];
        let mut cause = Some(error.cause() as &dyn error::Error);
        while let Some(e) = cause {
            causes.push(e.to_string());
            cause = e.source();
        }
        if error.cause_safe() {
            message = message.insert_params("errorCause", causes);
        } else {
            message = message.insert_unsafe_params("errorCause", causes);
        }
        for (key, value) in &error.safe_params() {
            message = message.insert_params(key, value);
        }
        for (key, value) in &error.unsafe_params() {
            message = message.insert_unsafe_params(key, value);
        }
    }
    for (key, value) in record.safe_params() {
        message = message.insert_params(*key, value);
    }
    for (key, value) in record.unsafe_params() {
        message = message.insert_unsafe_params(*key, value);
    }

    message.build()
}

/// Merge the Short (`{:?}`) and Full (`{:#?}`) renderings of one backtrace, preferring the concise
/// short line for every frame the in-process symbolizer could name and falling back to the full,
/// address-bearing line only where the short line is `<unknown>`.
///
/// Both renderings list the same frames in the same order. Short is one line per frame
/// (`  <idx>: <name>`); Full is a header line per frame (`  <idx>:  0x<ip> - <name>`) optionally
/// followed by `at <file>:<line>` continuation lines. We walk the Full rendering frame by frame:
/// for a named frame we emit the Short line (dropping the address and any continuation lines), for
/// an unnamed frame we emit the Full block verbatim so its IP survives for offline reconstruction.
fn merge_backtrace(short: &str, full: &str) -> String {
    let short_lines: std::collections::HashMap<usize, &str> = short
        .lines()
        .filter_map(|line| Some((frame_index(line)?, line)))
        .collect();

    let mut out = String::new();
    let mut keep_continuations = true;
    for line in full.lines() {
        match frame_index(line) {
            Some(idx) => {
                let short_line = short_lines.get(&idx).copied();
                if short_line.is_some_and(|l| !frame_is_unknown(l)) {
                    // Named frame: concise short line, no address, drop its continuations.
                    out.push_str(short_line.unwrap());
                    out.push('\n');
                    keep_continuations = false;
                } else {
                    // Unnamed (or unmatched) frame: keep the full address-bearing block.
                    out.push_str(line);
                    out.push('\n');
                    keep_continuations = true;
                }
            }
            // Continuation (`at file:line`) / blank line: keep only under a full-form frame.
            None if keep_continuations => {
                out.push_str(line);
                out.push('\n');
            }
            None => {}
        }
    }
    // Drop the trailing newline; the caller re-adds one via writeln!.
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

/// The frame index of a backtrace line like `  12: ...`, or `None` for continuation/blank lines.
/// An `at /path/file.rs:585` continuation splits into a non-numeric left side and is rejected.
fn frame_index(line: &str) -> Option<usize> {
    let (num, rest) = line.trim_start().split_once(':')?;
    if rest.is_empty() || rest.starts_with(' ') {
        num.parse().ok()
    } else {
        None
    }
}

/// Whether a backtrace line names its frame `<unknown>` (e.g. the short form `  12: <unknown>`).
fn frame_is_unknown(line: &str) -> bool {
    line.trim_start()
        .split_once(':')
        .is_some_and(|(_, name)| name.trim() == "<unknown>")
}
