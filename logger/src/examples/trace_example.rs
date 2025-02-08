use log::{debug, info, log_enabled, warn, Level};

fn main() {
    // Initialize the logger
    env_logger::init();

    info!("Rust logging n = {}", 42);
    warn!("This is too much fun!");
    debug!("Maybe we can make this code work");

    let error_is_enabled = log_enabled!(Level::Error);
    let warn_is_enabled = log_enabled!(Level::Warn);
    let info_is_enabled = log_enabled!(Level::Info);
    let debug_is_enabled = log_enabled!(Level::Debug);
    let trace_is_enabled = log_enabled!(Level::Trace);
    println!(
        "is_enabled?  error: {:5?}, warn: {:5?}, info: {:5?}, debug: {:5?}, trace: {:5?}",
        error_is_enabled, warn_is_enabled, info_is_enabled, debug_is_enabled, trace_is_enabled,
    );
}