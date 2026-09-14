#[cfg(feature = "log-safety")]
#[test]
fn log_safe_values_are_accepted() {
    let value = conjure_object::Uuid::nil();
    witchcraft_log::info!("message", safe: { value: value });
    witchcraft_log::mdc::insert_safe("value", value);
}
