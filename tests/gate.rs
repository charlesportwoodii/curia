// A separate test binary because it needs its own process-global logger. The `lib`
// target installs one at Trace, and a refused level can only be observed where the
// installed dispatcher refuses it.

// Shared with the `lib` target, which uses the parts this binary does not.
#[allow(dead_code)]
mod support;

use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use curia::{Dispatcher, Filter, Level, Logger};

use support::{TestSink, TestSinkType};

static SINK: OnceLock<TestSink> = OnceLock::new();

// Info everywhere, Debug for `gate::verbose` only, so the installed dispatcher refuses
// Trace outright and refuses Debug for every other target.
fn sink() -> &'static TestSink {
    SINK.get_or_init(|| {
        let sink = TestSink::new(Level::Trace);
        let dispatcher = Dispatcher::new(vec![TestSinkType::Capture(sink.clone())])
            .with_filter(Filter::from_directives("info,gate::verbose=debug"));
        Logger::install(Box::new(dispatcher)).expect("install");
        sink
    })
}

fn delivered(message: &str) -> bool {
    sink().messages().iter().any(|m| m == message)
}

fn counted(counter: &AtomicUsize, message: &str) -> String {
    counter.fetch_add(1, Ordering::SeqCst);
    message.to_string()
}

#[test]
fn a_level_past_every_threshold_never_evaluates_its_arguments() {
    sink();
    let evaluated = AtomicUsize::new(0);

    curia::trace!("trace {}", counted(&evaluated, "a"));
    curia::trace!(counted(&evaluated, "b"));
    curia::trace!("trace fields", { value: counted(&evaluated, "c") });

    assert_eq!(evaluated.load(Ordering::SeqCst), 0);
}

#[test]
fn a_level_refused_for_this_target_never_evaluates_its_arguments() {
    sink();
    let evaluated = AtomicUsize::new(0);

    curia::debug!("debug {}", counted(&evaluated, "a"));
    curia::debug!("debug fields", { value: counted(&evaluated, "b") });

    assert_eq!(evaluated.load(Ordering::SeqCst), 0);
    assert!(!Logger::enabled(Level::Debug, "gate"));
}

#[test]
fn an_admitted_level_is_evaluated_once_and_delivered() {
    sink();
    let evaluated = AtomicUsize::new(0);

    curia::info!("gate info {}", counted(&evaluated, "a"));

    assert_eq!(evaluated.load(Ordering::SeqCst), 1);
    assert!(delivered("gate info a"));
}

mod verbose {
    use super::*;

    #[test]
    fn a_target_override_admits_its_own_module() {
        sink();

        curia::debug!("verbose debug");

        assert!(delivered("verbose debug"));
    }
}
