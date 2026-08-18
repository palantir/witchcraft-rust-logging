use conjure_object::Uuid;
use witchcraft_log::{Level, debug, error, info};

fn main() {
    witchcraft_env_logger::init();

    debug!("this is a debug message");
    error!("this is printed by default");

    if witchcraft_log::enabled!(Level::Info) {
        let request_id = Uuid::nil(); // expensive computation
        info!("figured out the request ID", safe: { request_id: request_id });
    }
}
