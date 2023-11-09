use util_macros::bitflags;

#[bitflags]
pub enum LogLevel {
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
    FATAL,
}

// Default levels
static mut LOGGER: Logger = Logger::new(LogLevel::from_bits(0b111111));

pub struct Logger {
    log_level: LogLevel,
}

impl Logger {
    const fn new(log_level: LogLevel) -> Self {
        Self { log_level }
    }

    pub fn enable_levels(&mut self, levels: LogLevel) {
        self.log_level.insert(levels);
    }

    pub fn disable_levels(&mut self, levels: LogLevel) {
        self.log_level.remove(levels);
    }

    pub fn set_levels(&mut self, levels: LogLevel) {
        self.log_level = levels;
    }

    pub fn log(&self, level: LogLevel, msg: &str) {
        if self.log_level.intersects(level) {
            println!("[{}] {}", level, msg); // TODO add colours cause pretty
        }
    }
}

pub fn logger() -> &'static mut Logger {
    unsafe { &mut LOGGER }
}

#[macro_export(local_inner_macros)]
macro_rules! trace {
    ($msg:expr) => {
        internal_log!($crate::LogLevel::TRACE, $msg)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! debug {
    ($msg:expr) => {
        internal_log!($crate::LogLevel::DEBUG, $msg)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! info {
    ($msg:expr) => {
        internal_log!($crate::LogLevel::INFO, $msg)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! warn {
    ($msg:expr) => {
        internal_log!($crate::LogLevel::WARN, $msg)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! error {
    ($msg:expr) => {
        internal_log!($crate::LogLevel::ERROR, $msg)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! fatal {
    ($msg:expr) => {
        internal_log!($crate::LogLevel::FATAL, $msg)
    };
}
#[macro_export]
macro_rules! internal_log {
    ($level:expr, $msg:expr) => {
        $crate::logger().log($level, $msg);
    };
}
