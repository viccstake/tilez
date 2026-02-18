use std::fmt;

/// Log verbosity level — ordered from least to most detailed.
///
/// | Level   | Flag needed |
/// |---------|-------------|
/// | Warn    | always      |
/// | Info    | always      |
/// | Verbose | `-v`        |
/// | Debug   | `-vv`       |
/// | Trace   | `-vvv`      |
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Warn,
    Info,
    Verbose,
    Debug,
    Trace,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tag = match self {
            Level::Warn    => "WARN",
            Level::Info    => "INFO",
            Level::Verbose => "VERB",
            Level::Debug   => "DEBG",
            Level::Trace   => "TRCE",
        };
        write!(f, "{tag}")
    }
}

/// Lightweight, verbosity-gated logger.
///
/// Every log method accepts any value that implements [`fmt::Display`],
/// so callers can pass plain strings, `format_args!(…)` expressions,
/// or custom event types that derive their own `Display` implementation.
///
/// ```text
/// logger.info(ServerEvent::GameStarted { id: 1 });
/// logger.debug(format_args!("raw bytes: {:?}", buf));
/// logger.verbose("player connected");
/// ```
pub struct Logger {
    verbosity: u8,
}

impl Logger {
    pub fn new(verbosity: u8) -> Self {
        Self { verbosity }
    }

    fn emit(&self, level: Level, msg: &dyn fmt::Display) {
        let min_v: u8 = match level {
            Level::Warn    => 0,
            Level::Info    => 0,
            Level::Verbose => 1,
            Level::Debug   => 2,
            Level::Trace   => 3,
        };
        if self.verbosity >= min_v {
            eprintln!("[{level}] {msg}");
        }
    }

    pub fn warn   (&self, msg: impl fmt::Display) { self.emit(Level::Warn,    &msg); }
    pub fn info   (&self, msg: impl fmt::Display) { self.emit(Level::Info,    &msg); }
    pub fn verbose(&self, msg: impl fmt::Display) { self.emit(Level::Verbose, &msg); }
    pub fn debug  (&self, msg: impl fmt::Display) { self.emit(Level::Debug,   &msg); }
    pub fn trace  (&self, msg: impl fmt::Display) { self.emit(Level::Trace,   &msg); }
}
