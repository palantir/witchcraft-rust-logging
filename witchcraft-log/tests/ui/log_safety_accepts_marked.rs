use serde::Serialize;
use witchcraft_log::log_safety::derive::LogSafe;

#[derive(LogSafe, Serialize)]
struct SafeValue;

fn main() {
    witchcraft_log::info!("message", safe: { value: SafeValue });
    witchcraft_log::mdc::insert_safe("value", SafeValue);
}
