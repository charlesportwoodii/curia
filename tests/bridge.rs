use tracing::subscriber::DefaultGuard;
use tracing_subscriber::prelude::*;

use curia::{Level, LogEvent, TracingBridge};

use crate::support::TestSink;

// tests/macros.rs already claims the process-global Logger dispatch, so this
// suite asserts through a locally scoped subscriber instead.
fn install_bridge(sink: &TestSink) -> DefaultGuard {
    tracing::subscriber::set_default(
        tracing_subscriber::registry().with(TracingBridge::new(sink.clone())),
    )
}

fn captured(sink: &TestSink, message: &str) -> LogEvent {
    sink.captured
        .lock()
        .unwrap()
        .iter()
        .find(|e| e.message == message)
        .cloned()
        .unwrap_or_else(|| panic!("no event captured for {message}"))
}

#[test]
fn span_fields_are_inherited_by_events() {
    let sink = TestSink::new(Level::Trace);
    let _guard = install_bridge(&sink);

    let span = tracing::info_span!("connection", connection_id = "abc123");
    let _entered = span.enter();
    tracing::warn!("handshake failed");

    let event = captured(&sink, "handshake failed");
    assert_eq!(
        event.fields.get("connection_id").unwrap(),
        &serde_json::json!("abc123")
    );
}

#[test]
fn event_fields_win_over_span_fields() {
    let sink = TestSink::new(Level::Trace);
    let _guard = install_bridge(&sink);

    let span = tracing::info_span!("outer", transport = "quic");
    let _entered = span.enter();
    tracing::warn!(transport = "wss", "fell back");

    let event = captured(&sink, "fell back");
    assert_eq!(
        event.fields.get("transport").unwrap(),
        &serde_json::json!("wss")
    );
}

#[test]
fn inner_span_fields_win_over_outer_ones() {
    let sink = TestSink::new(Level::Trace);
    let _guard = install_bridge(&sink);

    let outer = tracing::info_span!("outer", transport = "quic");
    let _outer = outer.enter();
    let inner = tracing::info_span!("inner", transport = "wss");
    let _inner = inner.enter();
    tracing::warn!("nested spans");

    let event = captured(&sink, "nested spans");
    assert_eq!(
        event.fields.get("transport").unwrap(),
        &serde_json::json!("wss")
    );
}

#[test]
fn tracing_levels_map_onto_our_levels() {
    let sink = TestSink::new(Level::Trace);
    let _guard = install_bridge(&sink);

    tracing::error!("an error");

    assert_eq!(captured(&sink, "an error").level, Level::Error);
}

#[test]
fn a_sink_below_the_event_level_is_not_called() {
    let sink = TestSink::new(Level::Warn);
    let _guard = install_bridge(&sink);

    tracing::info!("too chatty");

    assert!(sink.messages().is_empty());
}

#[test]
fn an_enormous_debug_field_is_truncated_rather_than_flooding_the_log() {
    let sink = TestSink::new(Level::Trace);
    let _guard = install_bridge(&sink);

    // Stands in for a Tauri AppHandle: a Debug rendering that runs to thousands
    // of bytes and would otherwise land in every event inside the span.
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Huge(String);
    let huge = Huge("x".repeat(50_000));

    tracing::warn!(app_handle = ?huge, "inside an instrumented span");

    let event = captured(&sink, "inside an instrumented span");
    let value = event.fields.get("app_handle").unwrap().as_str().unwrap();

    assert!(value.len() < 1_000, "field was {} bytes", value.len());
    assert!(value.contains("truncated"));
}

// tracing-log rewrites a log::Record as an event targeted at the literal "log",
// with the real target in a log.target field. A consumer filtering on target
// sees "log" for every log-origin event unless the bridge restores it.
#[test]
fn a_log_origin_target_is_restored_from_its_field() {
    let sink = TestSink::new(Level::Trace);
    let _guard = install_bridge(&sink);

    tracing::event!(
        target: "log",
        tracing::Level::WARN,
        log.target = "bedrock_client::session",
        log.line = 42u64,
        "session opened"
    );

    let event = captured(&sink, "session opened");

    assert_eq!(event.target, "bedrock_client::session");
    assert_eq!(event.line, Some(42));
    assert!(
        event.fields.get("log.target").is_none(),
        "bridge bookkeeping must not reach sinks"
    );
}

#[test]
fn a_target_level_override_suppresses_a_verbose_crate() {
    let sink = TestSink::new(Level::Trace);
    let _guard = tracing::subscriber::set_default(
        tracing_subscriber::registry().with(
            TracingBridge::new(sink.clone())
                .with_max_level(Level::Debug)
                .with_target_level("h2", Level::Warn),
        ),
    );

    tracing::event!(target: "h2::codec::framed_read", tracing::Level::DEBUG, "received");
    tracing::event!(target: "bvc_client_lib::audio", tracing::Level::DEBUG, "capture started");

    assert_eq!(sink.messages(), vec!["capture started"]);
}

#[test]
fn the_max_level_floors_everything_without_an_override() {
    let sink = TestSink::new(Level::Trace);
    let _guard = tracing::subscriber::set_default(
        tracing_subscriber::registry()
            .with(TracingBridge::new(sink.clone()).with_max_level(Level::Info)),
    );

    tracing::event!(target: "anything", tracing::Level::DEBUG, "too quiet");
    tracing::event!(target: "anything", tracing::Level::INFO, "loud enough");

    assert_eq!(sink.messages(), vec!["loud enough"]);
}
