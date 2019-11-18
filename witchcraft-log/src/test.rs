use crate::bridge::{self, BridgedLogger};
use crate::{Level, LevelFilter, Log, Metadata, Record};
use conjure_error::Error;
use serde_value::Value;
use std::cell::RefCell;

thread_local! {
    static RECORDS: RefCell<Vec<TestRecord>> = RefCell::new(vec![]);
}

struct TestLogger;

impl Log for TestLogger {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        let record = TestRecord {
            level: record.level(),
            target: record.target().to_string(),
            file: record.file().map(|s| s.to_string()),
            line: record.line(),
            message: record.message(),
            safe_params: record
                .safe_params()
                .iter()
                .map(|(k, v)| (*k, serde_value::to_value(v).unwrap()))
                .collect(),
            unsafe_params: record
                .unsafe_params()
                .iter()
                .map(|(k, v)| (*k, serde_value::to_value(v).unwrap()))
                .collect(),
            error: record.error().map(|e| e.cause().to_string()),
        };
        RECORDS.with(|r| r.borrow_mut().push(record));
    }

    fn flush(&self) {}
}

struct TestRecord {
    level: Level,
    target: String,
    file: Option<String>,
    line: Option<u32>,
    message: &'static str,
    safe_params: Vec<(&'static str, Value)>,
    unsafe_params: Vec<(&'static str, Value)>,
    error: Option<String>,
}

fn init() {
    let _ = crate::set_logger(&TestLogger);
    crate::set_max_level(LevelFilter::Trace);
    RECORDS.with(|r| r.borrow_mut().clear());
}

fn get_records() -> Vec<TestRecord> {
    RECORDS.with(|r| r.replace(vec![]))
}

#[test]
fn minimal() {
    init();

    info!("message");
    let records = get_records();
    assert_eq!(records.len(), 1);

    assert_eq!(records[0].level, Level::Info);
    assert_eq!(records[0].target, module_path!());
    assert_eq!(records[0].file.as_ref().unwrap(), file!());
    assert!(records[0].line.is_some());
    assert_eq!(records[0].message, "message");
    assert_eq!(records[0].safe_params, &[]);
    assert_eq!(records[0].unsafe_params, &[]);
    assert_eq!(records[0].error, None);
}

#[test]
fn params() {
    init();

    warn!("message", safe: { safe_param: "foobar" });
    warn!("message", unsafe: { unsafe_param: 15 });
    warn!("message", safe: { safe_param: "foobar" }, unsafe: { unsafe_param: 15 });
    let records = get_records();
    assert_eq!(records.len(), 3);

    assert_eq!(
        records[0].safe_params,
        &[("safe_param", Value::String("foobar".to_string()))],
    );
    assert_eq!(records[0].unsafe_params, &[]);

    assert_eq!(records[1].safe_params, &[]);
    assert_eq!(
        records[1].unsafe_params,
        &[("unsafe_param", Value::I32(15))],
    );

    assert_eq!(
        records[2].safe_params,
        &[("safe_param", Value::String("foobar".to_string()))],
    );
    assert_eq!(
        records[2].unsafe_params,
        &[("unsafe_param", Value::I32(15))],
    );
}

#[test]
fn errors() {
    init();

    warn!("message", error: Error::internal_safe("error message"));
    let records = get_records();
    assert_eq!(records.len(), 1);

    assert_eq!(records[0].error.as_ref().unwrap(), "error message");
}

#[test]
fn bridge() {
    init();

    log::set_logger(&BridgedLogger);
    bridge::set_max_level(LevelFilter::Trace);

    log::info!("foobar {}", 123);
    let records = get_records();
    assert_eq!(records.len(), 1);

    assert_eq!(records[0].level, Level::Info);
    assert_eq!(records[0].target, module_path!());
    assert_eq!(records[0].file.as_ref().unwrap(), file!());
    assert!(records[0].line.is_some());
    assert_eq!(records[0].message, "");
    assert_eq!(records[0].safe_params, &[]);
    assert_eq!(
        records[0].unsafe_params,
        &[("message", Value::String("foobar 123".to_string()))],
    );
    assert_eq!(records[0].error, None);
}
