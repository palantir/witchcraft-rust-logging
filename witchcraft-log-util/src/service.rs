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
use witchcraft_logging_api::{
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
        .type_("service.")
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
            writeln!(stacktrace, "{:?}", trace).unwrap();
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
