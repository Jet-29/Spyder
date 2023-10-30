pub struct LoggerPlugin {}

#[macro_export(local_inner_macros)]
macro_rules! trace {
    ($msg:expr) => {
        internal_log!("TRACE", $msg)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! debug {
    ($msg:expr) => {
        internal_log!("DEBUG", $msg)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! info {
    ($msg:expr) => {
        internal_log!("INFO", $msg)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! warn {
    ($msg:expr) => {
        internal_log!("WARN", $msg)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! error {
    ($msg:expr) => {
        internal_log!("ERROR", $msg)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! fatal {
    ($msg:expr) => {
        internal_log!("FATAL", $msg)
    };
}
#[macro_export]
macro_rules! internal_log {
    ($level:expr, $msg:expr) => {
        println!(
            "{}",
            format!("[{}] {}", $level, format!($msg).as_str()).as_str()
        ); // TODO add colours cause pretty
    };
}
