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

impl LogLevel {
    fn to_coloured_str(&self) -> &str {
        match *self {
            Self::TRACE => "\x1b[34mTRACE\x1b[0m",
            Self::DEBUG => "\x1b[32mDEBUG\x1b[0m",
            Self::INFO => "\x1b[36mINFO\x1b[0m",
            Self::WARN => "\x1b[33mWARN\x1b[0m",
            Self::ERROR => "\x1b[31mERROR\x1b[0m",
            Self::FATAL => "\x1b[41mFATAL\x1b[0m",
            _ => unreachable!(),
        }
    }
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
            println!("[{}] {}", level.to_coloured_str(), msg);
        }
    }
}

pub fn logger() -> &'static mut Logger {
    unsafe { &mut LOGGER }
}

#[macro_export(local_inner_macros)]
macro_rules! trace {
    ($($arg:tt)*) => {
        internal_log!($crate::LogLevel::TRACE, $($arg)*)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! debug {
    ($($arg:tt)*) => {
        internal_log!($crate::LogLevel::DEBUG, $($arg)*)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! info {
    ($($arg:tt)*) => {
        internal_log!($crate::LogLevel::INFO, $($arg)*)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! warn {
    ($($arg:tt)*) => {
        internal_log!($crate::LogLevel::WARN, $($arg)*)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! error {
    ($($arg:tt)*) => {
        internal_log!($crate::LogLevel::ERROR, $($arg)*)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! fatal {
    ($($arg:tt)*) => {
        internal_log!($crate::LogLevel::FATAL, $($arg)*)
    };
}
#[macro_export]
macro_rules! internal_log {
    ($level:expr, $($arg:tt)*) => {
        $crate::logger().log($level, format!($($arg)*).as_str());
    };
}
