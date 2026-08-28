// Copyright 2026 Palantir Technologies, Inc.
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

//! Native-module load-base lookup for reconstructing stripped in-process backtraces.
//!
//! A logged backtrace records absolute frame instruction pointers. When the module is stripped
//! those no longer resolve in-process, and — unlike an OS tombstone — nothing normalizes them for a
//! logged trace. To recover symbols offline you subtract the module's load base
//! (`file_va = ip - base_address`) and resolve against a matching artifact. This module finds that
//! load base at runtime; the caller pairs it with a module name and version in a
//! [`witchcraft_log::TraceContext`] and installs it via `witchcraft_log::set_trace_context`.

/// Return the mapped load base of the native module this crate is linked into, or `None` if it
/// cannot be determined.
///
/// The module is identified by testing a marker address (a function in this crate) against each
/// loaded shared library's mapped range: because `witchcraft-log-util` is statically linked into the
/// consuming shared object, the marker necessarily lives inside the very module whose backtraces we
/// want to describe. Subtracting this base from an absolute frame IP yields the file-relative address
/// an offline symbolizer consumes.
#[cfg(not(target_family = "wasm"))]
pub fn find_address_offset() -> Option<usize> {
    use findshlibs::{IterationControl, SharedLibrary, TargetSharedLibrary};

    let marker = find_address_offset as *const () as usize;
    let mut found = None;
    TargetSharedLibrary::each(|shlib| {
        let base = shlib.actual_load_addr().0;
        if marker < base || marker >= base + shlib.len() {
            return IterationControl::Continue;
        }
        found = Some(base);
        IterationControl::Break
    });
    found
}

/// Stub for targets `findshlibs` does not support (e.g. wasm); always returns `None`.
#[cfg(target_family = "wasm")]
pub fn find_address_offset() -> Option<usize> {
    None
}
