use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::{Filter, InstallError, Level, LogEvent, Sink, Threshold};

pub struct Dispatcher<S: Sink> {
    sinks: Vec<S>,
    filter: Filter,
}

impl<S: Sink> Dispatcher<S> {
    pub fn new(sinks: Vec<S>) -> Self {
        Self {
            sinks,
            filter: Filter::default(),
        }
    }

    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }
}

pub trait Dispatch: Send + Sync {
    fn dispatch(&self, event: &LogEvent);

    // The most verbose level any event can still reach a sink at. `Logger` reads it once,
    // at install, and refuses every level past it before a macro builds the event.
    //
    // The default admits everything, so an implementation that does not answer keeps
    // receiving every event, as it did before this existed.
    fn max_threshold(&self) -> Threshold {
        Threshold::At(Level::Trace)
    }

    // Whether an event at `level` from `target` would reach any sink. Asked before the
    // event is built, so a refusal costs this call and nothing else.
    fn enabled(&self, _target: &str, _level: Level) -> bool {
        true
    }
}

impl<S: Sink> Dispatch for Dispatcher<S> {
    fn dispatch(&self, event: &LogEvent) {
        if !self.filter.admits(&event.target, event.level) {
            return;
        }

        for sink in &self.sinks {
            if event.level <= sink.level() {
                // A sink that panics must not take the others down with it, nor
                // unwind into the caller, which may be an audio thread.
                let _ = catch_unwind(AssertUnwindSafe(|| sink.emit(event)));
            }
        }
    }

    // Capped by the sinks as well as the filter: a filter admitting Trace in front of
    // sinks that stop at Info still delivers nothing past Info.
    fn max_threshold(&self) -> Threshold {
        let sinks = self.sinks.iter().fold(Threshold::Off, |widest, sink| {
            widest.widest(Threshold::At(sink.level()))
        });

        self.filter.widest().narrowest(sinks)
    }

    fn enabled(&self, target: &str, level: Level) -> bool {
        self.filter.admits(target, level) && self.sinks.iter().any(|sink| level <= sink.level())
    }
}

static DISPATCH: OnceLock<Box<dyn Dispatch>> = OnceLock::new();

// `Threshold::admitted` of the installed dispatcher. Zero until install, which refuses
// everything, matching `emit` being a no-op before install.
static ADMITTED: AtomicU8 = AtomicU8::new(0);

pub struct Logger;

impl Logger {
    pub fn install(dispatch: Box<dyn Dispatch>) -> Result<(), InstallError> {
        let admitted = dispatch.max_threshold().admitted();

        DISPATCH
            .set(dispatch)
            .map_err(|_| InstallError::AlreadyInstalled)?;

        // Stored after the dispatcher is in place, so a caller that sees the level
        // admitted always finds a dispatcher to deliver to.
        ADMITTED.store(admitted, Ordering::Release);
        Ok(())
    }

    // Whether an event at `level` from `target` would be delivered anywhere.
    //
    // The logging macros ask this before evaluating their message or fields, so a
    // refused level costs one atomic load. A level past every threshold never reaches
    // the dispatcher at all; one inside the widest threshold is then checked against
    // its own target.
    #[inline]
    pub fn enabled(level: Level, target: &str) -> bool {
        if level.rank() >= ADMITTED.load(Ordering::Acquire) {
            return false;
        }

        DISPATCH
            .get()
            .is_some_and(|dispatch| dispatch.enabled(target, level))
    }

    pub fn emit(event: LogEvent) {
        if let Some(dispatch) = DISPATCH.get() {
            dispatch.dispatch(&event);
        }
    }
}
