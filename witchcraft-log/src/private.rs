use crate::{Level, Metadata, Record};
use conjure_error::Error;
use erased_serde::Serialize;

// TODO might be worth adding special cased functions for e.g. no params/errors
pub fn log(
    // package all of the probably-constant bits together so they can just passed as one pointer into .rodata
    &(level, target, file, line, message): &(Level, &str, &str, u32, &'static str),
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

pub fn enabled(level: Level, target: &str) -> bool {
    crate::logger().enabled(&Metadata::builder().level(level).target(target).build())
}
