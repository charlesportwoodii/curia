use curia::{Dispatch, Dispatcher, Fields, Filter, Level, Logger, Threshold};

use crate::support::{PanicSink, TestSink, TestSinkType, Unserializable, event};

#[test]
fn sink_below_event_level_is_not_called() {
    let quiet = TestSink::new(Level::Warn);
    let chatty = TestSink::new(Level::Trace);
    let dispatcher = Dispatcher::new(vec![
        TestSinkType::Capture(quiet.clone()),
        TestSinkType::Capture(chatty.clone()),
    ]);

    dispatcher.dispatch(&event(Level::Info, "info line"));

    assert!(quiet.messages().is_empty());
    assert_eq!(chatty.messages(), vec!["info line"]);
}

#[test]
fn severe_events_reach_a_less_verbose_sink() {
    let quiet = TestSink::new(Level::Warn);
    let dispatcher = Dispatcher::new(vec![TestSinkType::Capture(quiet.clone())]);

    dispatcher.dispatch(&event(Level::Error, "error line"));

    assert_eq!(quiet.messages(), vec!["error line"]);
}

#[test]
fn a_panicking_sink_does_not_stop_the_others() {
    let survivor = TestSink::new(Level::Trace);
    let dispatcher = Dispatcher::new(vec![
        TestSinkType::Panic(PanicSink),
        TestSinkType::Capture(survivor.clone()),
    ]);

    dispatcher.dispatch(&event(Level::Error, "still delivered"));

    assert_eq!(survivor.messages(), vec!["still delivered"]);
}

#[test]
fn emit_before_install_is_a_no_op() {
    Logger::emit(event(Level::Error, "nobody is listening"));
}

#[test]
fn the_max_threshold_is_capped_by_the_most_verbose_sink() {
    let dispatcher = Dispatcher::new(vec![
        TestSinkType::Capture(TestSink::new(Level::Warn)),
        TestSinkType::Capture(TestSink::new(Level::Info)),
    ]);

    assert_eq!(dispatcher.max_threshold(), Threshold::At(Level::Info));
}

#[test]
fn the_max_threshold_is_capped_by_the_filter_when_the_sinks_are_more_verbose() {
    let dispatcher = Dispatcher::new(vec![TestSinkType::Capture(TestSink::new(Level::Trace))])
        .with_filter(Filter::from_directives("warn"));

    assert_eq!(dispatcher.max_threshold(), Threshold::At(Level::Warn));
}

#[test]
fn a_dispatcher_without_sinks_admits_nothing() {
    let dispatcher: Dispatcher<TestSinkType> = Dispatcher::new(Vec::new());

    assert_eq!(dispatcher.max_threshold(), Threshold::Off);
    assert!(!dispatcher.enabled("anything", Level::Error));
}

#[test]
fn enabled_honours_a_per_target_override() {
    let dispatcher = Dispatcher::new(vec![TestSinkType::Capture(TestSink::new(Level::Trace))])
        .with_filter(Filter::from_directives("info,bvc::route=debug"));

    assert!(dispatcher.enabled("bvc::route::fanout", Level::Debug));
    assert!(!dispatcher.enabled("bvc::http", Level::Debug));
}

#[test]
fn enabled_refuses_a_level_no_sink_accepts() {
    let dispatcher = Dispatcher::new(vec![TestSinkType::Capture(TestSink::new(Level::Info))]);

    assert!(dispatcher.enabled("anything", Level::Info));
    assert!(!dispatcher.enabled("anything", Level::Debug));
}

#[test]
fn unserializable_values_are_recorded_rather_than_dropped() {
    let mut fields = Fields::new();
    fields.insert("good", 42);
    fields.insert("bad", Unserializable);

    assert_eq!(fields.get("good").unwrap(), &serde_json::json!(42));
    assert!(
        fields
            .get("bad")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("unserializable")
    );
}
