use std::sync::{Arc, Mutex};

use curia::{Level, LogEvent, Sink};

#[derive(Clone)]
pub struct TestSink {
    level: Level,
    pub captured: Arc<Mutex<Vec<LogEvent>>>,
}

impl TestSink {
    pub fn new(level: Level) -> Self {
        Self {
            level,
            captured: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn messages(&self) -> Vec<String> {
        self.captured
            .lock()
            .unwrap()
            .iter()
            .map(|e| e.message.clone())
            .collect()
    }
}

impl Sink for TestSink {
    fn level(&self) -> Level {
        self.level
    }

    fn emit(&self, event: &LogEvent) {
        self.captured.lock().unwrap().push(event.clone());
    }
}

pub struct PanicSink;

impl Sink for PanicSink {
    fn level(&self) -> Level {
        Level::Trace
    }

    fn emit(&self, _event: &LogEvent) {
        panic!("this sink always panics");
    }
}

pub enum TestSinkType {
    Capture(TestSink),
    Panic(PanicSink),
}

impl Sink for TestSinkType {
    fn level(&self) -> Level {
        match self {
            Self::Capture(s) => s.level(),
            Self::Panic(s) => s.level(),
        }
    }

    fn emit(&self, event: &LogEvent) {
        match self {
            Self::Capture(s) => s.emit(event),
            Self::Panic(s) => s.emit(event),
        }
    }
}

// serde_json maps f64::NAN to Value::Null rather than erroring, so a value that
// genuinely fails to serialize has to say so itself.
pub struct Unserializable;

impl serde::Serialize for Unserializable {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom("cannot serialize"))
    }
}

pub fn event(level: Level, message: &str) -> LogEvent {
    LogEvent {
        level,
        target: "test".to_string(),
        message: message.to_string(),
        fields: curia::Fields::new(),
        timestamp: chrono::Utc::now(),
        file: None,
        line: None,
    }
}
