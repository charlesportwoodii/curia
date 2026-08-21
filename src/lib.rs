#![doc = "Structured logging with a sink pipeline and a tracing bridge."]

#[macro_use]
mod macros;

mod bridge;
mod dispatcher;
mod error;
mod event;
mod fields;
mod level;
mod sink;

pub use bridge::TracingBridge;
pub use dispatcher::{Dispatch, Dispatcher, Logger};
pub use error::InstallError;
pub use event::LogEvent;
pub use fields::Fields;
pub use level::Level;
pub use sink::Sink;

// Re-exported so the macros can name chrono without every consumer being forced
// to depend on it directly.
#[doc(hidden)]
pub mod __private {
    pub use chrono;
}
