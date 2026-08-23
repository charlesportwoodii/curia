#![doc = "Structured logging with a sink pipeline and a tracing bridge."]

#[macro_use]
mod macros;

mod bridge;
mod dispatcher;
mod error;
mod event;
mod fields;
mod filter;
mod format;
mod level;
mod rotation;
mod sink;
mod strategy;

pub use bridge::TracingBridge;
pub use dispatcher::{Dispatch, Dispatcher, Logger};
pub use error::{InstallError, SinkError};
pub use event::LogEvent;
pub use fields::Fields;
pub use filter::{Filter, Threshold};
pub use format::LineFormatter;
pub use level::Level;
pub use rotation::RotatingFile;
pub use sink::{ConsoleSink, FileSink, Sink};
pub use strategy::{FileOpenStrategy, RotationStrategy, TimezoneStrategy};

pub const DEFAULT_MAX_FILE_SIZE: u64 = 40_000;
pub const DEFAULT_ROTATION_STRATEGY: RotationStrategy = RotationStrategy::KeepOne;
pub const DEFAULT_TIMEZONE_STRATEGY: TimezoneStrategy = TimezoneStrategy::UseUtc;
pub const DEFAULT_FILE_OPEN_STRATEGY: FileOpenStrategy = FileOpenStrategy::Append;

// Re-exported so the macros can name chrono without every consumer being forced
// to depend on it directly.
#[doc(hidden)]
pub mod __private {
    pub use chrono;
}
