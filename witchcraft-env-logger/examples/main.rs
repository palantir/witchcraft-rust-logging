use witchcraft_log::{Level, debug, error, info};

fn main() {
    witchcraft_env_logger::init();

    debug!("this is a debug message");
    error!("this is printed by default");

    if witchcraft_log::enabled!(Level::Info) {
        let x = 3 * 4; // expensive computation
        info!("figured out the answer", safe: { answer: x });
    }
}
