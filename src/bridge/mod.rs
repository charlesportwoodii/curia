mod visitor;

use tracing_core::{Event, Level as TracingLevel, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

use crate::{Fields, Filter, InstallError, Level, LogEvent, Logger, Sink, Threshold};

use visitor::FieldVisitor;

enum BridgeTarget {
    Global,
    Sink(Box<dyn Sink>),
}

pub struct TracingBridge {
    target: BridgeTarget,
    // Global floor, and per-prefix overrides for crates that are verbose by
    // design. Replaces what tauri-plugin-log's `.level()` / `.level_for()` did
    // before this pipeline existed; without it every dependency's debug output
    // reaches every sink.
    filter: Filter,
}

impl TracingBridge {
    // The production pairing. Events reach whatever dispatcher was installed.
    pub fn to_global() -> Self {
        Self {
            target: BridgeTarget::Global,
            filter: Filter::default(),
        }
    }

    pub fn with_max_level(mut self, level: Level) -> Self {
        self.filter.set_global(Threshold::At(level));
        self
    }

    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }

    // Prefix match, longest first at lookup, so a specific module can override
    // a broad crate rule.
    pub fn with_target_level(mut self, prefix: impl Into<String>, level: Level) -> Self {
        self.filter.add_target(prefix, Threshold::At(level));
        self
    }

    fn admits(&self, target: &str, level: Level) -> bool {
        self.filter.admits(target, level)
    }

    // Emits into one sink directly, bypassing the process-global dispatch so a
    // test can assert without claiming the OnceLock.
    pub fn new(sink: impl Sink + 'static) -> Self {
        Self {
            target: BridgeTarget::Sink(Box::new(sink)),
            filter: Filter::default(),
        }
    }

    // Captures dependencies still emitting through the `log` crate.
    pub fn install_log_capture() -> Result<(), InstallError> {
        tracing_log::LogTracer::init().map_err(|_| InstallError::AlreadyInstalled)
    }

    fn level_from(level: &TracingLevel) -> Level {
        match *level {
            TracingLevel::ERROR => Level::Error,
            TracingLevel::WARN => Level::Warn,
            TracingLevel::INFO => Level::Info,
            TracingLevel::DEBUG => Level::Debug,
            TracingLevel::TRACE => Level::Trace,
        }
    }

    fn deliver(&self, event: LogEvent) {
        match &self.target {
            BridgeTarget::Global => Logger::emit(event),
            BridgeTarget::Sink(sink) => {
                if event.level <= sink.level() {
                    sink.emit(&event);
                }
            }
        }
    }
}

impl<S> Layer<S> for TracingBridge
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &tracing_core::span::Attributes<'_>,
        id: &tracing_core::span::Id,
        ctx: Context<'_, S>,
    ) {
        let mut visitor = FieldVisitor::default();
        attrs.record(&mut visitor);

        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(visitor.fields);
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut fields = Fields::new();

        // Outermost span first, so an inner scope overwrites an outer one
        if let Some(scope) = ctx.event_scope(event) {
            for span in scope.from_root() {
                if let Some(recorded) = span.extensions().get::<Fields>() {
                    for (key, value) in recorded.as_map() {
                        fields.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);

        // Event fields last, so they win over any span field of the same name
        for (key, value) in visitor.fields.as_map() {
            fields.insert(key.clone(), value.clone());
        }

        let meta = event.metadata();

        // tracing-log rewrites a log::Record as an event whose target is the
        // literal "log", stashing the real one in a log.target field. A consumer
        // filtering on target would reject every log-origin event, so the real
        // target is restored here and the bridge's own bookkeeping fields are
        // dropped rather than shipped to every sink.
        let target = fields
            .get("log.target")
            .and_then(|v| v.as_str())
            .map(|t| t.to_string())
            .unwrap_or_else(|| meta.target().to_string());

        let file = fields
            .get("log.file")
            .and_then(|v| v.as_str())
            .map(|f| f.to_string())
            .or_else(|| meta.file().map(|f| f.to_string()));

        let line = fields
            .get("log.line")
            .and_then(|v| v.as_u64())
            .map(|l| l as u32)
            .or_else(|| meta.line());

        fields.remove_prefixed("log.");

        let level = Self::level_from(meta.level());
        if !self.admits(&target, level) {
            return;
        }

        self.deliver(LogEvent {
            level,
            target,
            message: visitor.message.unwrap_or_default(),
            fields,
            timestamp: chrono::Utc::now(),
            file,
            line,
        });
    }
}
