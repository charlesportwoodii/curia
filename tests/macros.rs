use std::sync::OnceLock;

use curia::{Dispatcher, Level, LogEvent, Logger};

use crate::support::{TestSink, TestSinkType};

// Logger::install claims a process-global OnceLock, so every test in this file
// shares one collector and disambiguates by message.
static SINK: OnceLock<TestSink> = OnceLock::new();

fn sink() -> &'static TestSink {
    SINK.get_or_init(|| {
        let sink = TestSink::new(Level::Trace);
        Logger::install(Box::new(Dispatcher::new(vec![TestSinkType::Capture(
            sink.clone(),
        )])))
        .expect("install");
        sink
    })
}

fn captured(message: &str) -> LogEvent {
    sink()
        .captured
        .lock()
        .unwrap()
        .iter()
        .find(|e| e.message == message)
        .cloned()
        .unwrap_or_else(|| panic!("no event captured for {message}"))
}

#[test]
fn message_only_form_carries_no_fields() {
    sink();
    curia::info!("plain message");

    let event = captured("plain message");
    assert_eq!(event.level, Level::Info);
    assert!(event.fields.is_empty());
}

#[test]
fn inline_block_form_captures_typed_fields() {
    sink();
    curia::warn!("inline fields", { device_host: "asio", frames: 0, live: true });

    let event = captured("inline fields");
    assert_eq!(event.level, Level::Warn);
    assert_eq!(
        event.fields.get("device_host").unwrap(),
        &serde_json::json!("asio")
    );
    assert_eq!(event.fields.get("frames").unwrap(), &serde_json::json!(0));
    assert_eq!(event.fields.get("live").unwrap(), &serde_json::json!(true));
}

#[test]
fn inline_block_form_accepts_string_keys_and_nested_values() {
    sink();
    curia::error!("nested fields", { "more data": serde_json::json!({ "ok": true }) });

    let event = captured("nested fields");
    assert_eq!(
        event.fields.get("more data").unwrap(),
        &serde_json::json!({ "ok": true })
    );
}

#[test]
fn expression_form_flattens_a_serializable_value() {
    sink();
    let payload = serde_json::json!({ "seq": 7, "transport": "quic" });
    curia::debug!("expression fields", payload);

    let event = captured("expression fields");
    assert_eq!(event.fields.get("seq").unwrap(), &serde_json::json!(7));
    assert_eq!(
        event.fields.get("transport").unwrap(),
        &serde_json::json!("quic")
    );
}

#[test]
fn empty_block_form_is_the_format_escape_hatch() {
    sink();
    let cause = std::io::Error::other("device gone");
    curia::warn!(format!("capture failed: {cause:?}"), {});

    let event = captured(&format!("capture failed: {cause:?}"));
    assert_eq!(event.level, Level::Warn);
    assert!(event.fields.is_empty());
}

#[test]
fn call_site_metadata_is_recorded() {
    sink();
    curia::trace!("call site");

    let event = captured("call site");
    // The aggregator target is named `lib`, so module_path! here is `lib::macros`
    assert_eq!(event.target, "lib::macros");
    assert!(event.file.unwrap().ends_with("macros.rs"));
    assert!(event.line.unwrap() > 0);
}
