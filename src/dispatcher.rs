use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::OnceLock;

use crate::{InstallError, LogEvent, Sink};

pub struct Dispatcher<S: Sink> {
    sinks: Vec<S>,
}

impl<S: Sink> Dispatcher<S> {
    pub fn new(sinks: Vec<S>) -> Self {
        Self { sinks }
    }
}

pub trait Dispatch: Send + Sync {
    fn dispatch(&self, event: &LogEvent);
}

impl<S: Sink> Dispatch for Dispatcher<S> {
    fn dispatch(&self, event: &LogEvent) {
        for sink in &self.sinks {
            if event.level <= sink.level() {
                // A sink that panics must not take the others down with it, nor
                // unwind into the caller, which may be an audio thread.
                let _ = catch_unwind(AssertUnwindSafe(|| sink.emit(event)));
            }
        }
    }
}

static DISPATCH: OnceLock<Box<dyn Dispatch>> = OnceLock::new();

pub struct Logger;

impl Logger {
    pub fn install(dispatch: Box<dyn Dispatch>) -> Result<(), InstallError> {
        DISPATCH
            .set(dispatch)
            .map_err(|_| InstallError::AlreadyInstalled)
    }

    pub fn emit(event: LogEvent) {
        if let Some(dispatch) = DISPATCH.get() {
            dispatch.dispatch(&event);
        }
    }
}
