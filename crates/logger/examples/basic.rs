use logger::{debug, error, fatal, info, trace, warn};

fn main() {
    trace!("I am a trace");
    debug!("I am a debug");
    info!("I am an info");
    warn!("I am a warning");
    error!("I am an error");
    fatal!("I am a fatal");
}
