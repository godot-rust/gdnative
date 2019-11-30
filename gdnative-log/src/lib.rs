//! # Adapter for the Rust log crate
//!
//! This crate writes an adapter for the Rust log crate such that logging statements
//! get output on the Godot console. Essentially it translates `error!()`, `info!()`
//! and so on into the appropriate `godot_error!()`, `godot_warn!()` and
//! `godot_print()` calls.

#[macro_use]
extern crate gdnative_core;

pub use log::Level;
use log::{self, Metadata, Record, SetLoggerError};

struct GodotLogger {
    level: Level,
}

impl log::Log for GodotLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                Level::Warn => godot_warn!("{} - {}", record.level(), record.args()),
                Level::Error => godot_error!("{} - {}", record.level(), record.args()),
                _ => godot_print!("{} - {}", record.level(), record.args()),
            }
        }
    }

    fn flush(&self) {}
}

/// Initialises the logger with the warning level set.
///
/// ```ignore
/// # #[macro_use] extern crate log;
/// #
/// # fn main() {
/// gdnative::init_logging_with_level(log::Level::Warn).unwrap();
///
/// warn!("This will be logged as a warning.");
/// info!("This message will not be logged.");
/// # }
/// ```
pub fn init_logging_with_level(level: Level) -> Result<(), SetLoggerError> {
    let logger = GodotLogger { level };
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(level.to_level_filter());
    Ok(())
}

/// Initialises the logger with warning level defaulted to `Level::Trace`
///
/// ```ignore
/// # #[macro_use] extern crate log;
/// #
/// # fn main() {
/// gdnative::init_logging().unwrap();
///
/// warn!("This will be logged as a warning.");
/// # }
/// ```
pub fn init_logging() -> Result<(), SetLoggerError> {
    init_logging_with_level(Level::Trace)
}
