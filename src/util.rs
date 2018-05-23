use log::{LogLevel, LogLevelFilter, SetLoggerError, ShutdownLoggerError,
          LogMetadata, self};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _: &LogMetadata) -> bool { true }
    fn log(&self, record: &log::LogRecord) {
        println!("{} - {}", record.level(), record.args());
    }
}

impl SimpleLogger {
    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn log_init() -> Result<(), SetLoggerError> {
    unsafe {
        log::set_logger_raw(|max_log_level| {
            static LOGGER: SimpleLogger = SimpleLogger;
            max_log_level.set(LogLevelFilter::Info);
            &SimpleLogger
        })
    }
}