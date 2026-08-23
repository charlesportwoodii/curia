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
fn a_preformatted_message_needs_no_trailing_block() {
    sink();
    let cause = std::io::Error::other("device gone");
    curia::warn!(format!("capture failed: {cause:?}"));

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

#[test]
fn a_format_string_with_one_argument_becomes_the_message() {
    sink();
    let dsn = "mysql://user@host/db";
    curia::info!("Database: {}", dsn);

    let event = captured("Database: mysql://user@host/db");
    assert_eq!(event.level, Level::Info);
    assert!(event.fields.is_empty());
}

#[test]
fn a_format_string_with_several_arguments_becomes_the_message() {
    sink();
    curia::warn!("token exchange failed ({}): {}", 403, "denied");

    let event = captured("token exchange failed (403): denied");
    assert_eq!(event.level, Level::Warn);
    assert!(event.fields.is_empty());
}

#[test]
fn a_brace_block_still_reaches_the_fields_arm() {
    sink();
    curia::warn!("format arm must not steal the block", { device_host: "asio" });

    let event = captured("format arm must not steal the block");
    assert_eq!(
        event.fields.get("device_host").unwrap(),
        &serde_json::json!("asio")
    );
}

#[test]
fn a_format_call_can_still_carry_fields_through_the_block_arm() {
    sink();
    let cause = "device gone";
    curia::warn!(format!("capture failed: {cause}"), { retry: true });

    let event = captured("capture failed: device gone");
    assert_eq!(event.fields.get("retry").unwrap(), &serde_json::json!(true));
}

