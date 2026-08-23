mod console;
mod file;

pub use console::ConsoleSink;
pub use file::FileSink;

use crate::{Level, LogEvent};

pub trait Sink: Send + Sync {
    fn level(&self) -> Level;

    // Must not block. A sink that performs network IO queues internally and
    // returns immediately.
    fn emit(&self, event: &LogEvent);
}
