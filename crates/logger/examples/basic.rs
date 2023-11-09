use logger::{debug, error, fatal, info, trace, warn};

fn main() {
    trace!("I am a trace");
    logger::logger().disable_levels(logger::LogLevel::TRACE | logger::LogLevel::DEBUG);
    debug!("I am a debug");
    info!("I am an info");
    warn!("I am a warning");
    error!("I am an error");
    fatal!("I am a fatal");
}
