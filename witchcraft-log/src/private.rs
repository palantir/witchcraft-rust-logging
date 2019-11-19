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
use crate::{Level, Metadata, Record};
use conjure_error::Error;
use erased_serde::Serialize;

pub fn log(
    level: Level,
    // package all of the probably-constant bits together so they can just passed as one pointer into .rodata
    &(target, file, line, message): &(&str, &str, u32, &'static str),
    safe_params: &[(&'static str, &dyn Serialize)],
    unsafe_params: &[(&'static str, &dyn Serialize)],
    error: Option<&Error>,
) {
    crate::logger().log(
        &Record::builder()
            .level(level)
            .target(target)
            .file(Some(file))
            .line(Some(line))
            .message(message)
            .safe_params(safe_params)
            .unsafe_params(unsafe_params)
            .error(error)
            .build(),
    )
}

pub fn log_minimal(level: Level, &(target, file, line, message): &(&str, &str, u32, &'static str)) {
    crate::logger().log(
        &Record::builder()
            .level(level)
            .target(target)
            .file(Some(file))
            .line(Some(line))
            .message(message)
            .build(),
    )
}

pub fn enabled(level: Level, target: &str) -> bool {
    crate::logger().enabled(&Metadata::builder().level(level).target(target).build())
}
