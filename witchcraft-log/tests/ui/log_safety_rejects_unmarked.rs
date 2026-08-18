fn main() {
    witchcraft_log::info!("message", safe: { value: "unmarked" });
    witchcraft_log::mdc::insert_safe("value", "unmarked");
}
