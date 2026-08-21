# curia

Structured logging with a sink pipeline and a tracing bridge.

Fields are `serde_json` values, so nested objects and arrays survive from the
call site to every sink with their types intact. The library knows nothing about
what a field *means* — routing, cardinality rules and destinations belong to the
consumer that implements a sink.

## Logging

Three forms. The message is always first.

```rust
curia::info!("stream started");

curia::warn!("capture stream ended", {
    device_host: "asio",
    frames: 0,
    live: true,
});

let payload = serde_json::json!({ "seq": 7, "transport": "quic" });
curia::debug!("packet dropped", payload);
```

Keys may be bare identifiers or string literals, and values may be anything that
implements `Serialize`:

```rust
curia::error!("decode failed", {
    "more data": serde_json::json!({ "ok": true }),
    codes: [1, 2, 3],
});
```

A value that fails to serialize is stored as `<unserializable: ...>` rather than
dropped. A logging call never fails and never returns an error.

### Format arguments are deliberately unsupported

There is no `warn!("failed {}", e)` form. An interpolated message is exactly what
destroys grouping downstream — every distinct interpolation becomes a distinct
issue in an error tracker. Put the varying part in a field:

```rust
curia::warn!("capture failed", { error: e.to_string() });
```

Where a formatted message is genuinely wanted, the message is an ordinary
expression, so the escape hatch already exists:

```rust
curia::warn!(format!("capture failed: {e:?}"), {});
```

It is an escape hatch, not a destination.

## Implementing sinks

Define your own enum and delegate by `match`. This keeps dynamic dispatch to the
single `Box<dyn Dispatch>` at the global boundary rather than a vtable lookup per
sink per event.

```rust
use curia::{Level, LogEvent, Sink};

pub enum AppSink {
    Stderr(StderrSink),
    JsonFile(JsonFileSink),
}

impl Sink for AppSink {
    fn level(&self) -> Level {
        match self {
            Self::Stderr(s) => s.level(),
            Self::JsonFile(s) => s.level(),
        }
    }

    fn emit(&self, event: &LogEvent) {
        match self {
            Self::Stderr(s) => s.emit(event),
            Self::JsonFile(s) => s.emit(event),
        }
    }
}
```

**`Sink::emit` must not block.** It runs on the caller's thread, which may be an
audio or network hot path. A sink that performs IO queues internally and returns
immediately. A sink that panics is caught and the remaining sinks still run, but
that is a backstop, not a design.

A sink's registered level is an admission filter, not its routing policy. Register
a sink at the most verbose level it needs to *see*, then decide what to do per
level inside it. Registering an error reporter at `Warn` because it only reports
warnings and errors will silently discard the `info` trail that makes a report
diagnosable.

## Installing

```rust
use curia::{Dispatcher, Level, Logger, TracingBridge};
use tracing_subscriber::prelude::*;

Logger::install(Box::new(Dispatcher::new(vec![
    AppSink::Stderr(StderrSink::new(Level::Info)),
    AppSink::JsonFile(JsonFileSink::new(path, Level::Debug)),
])))?;

tracing_subscriber::registry().with(TracingBridge::to_global()).init();
TracingBridge::install_log_capture()?;
```

`Logger::install` may be called once per process and returns
`InstallError::AlreadyInstalled` otherwise. Before it is called, `Logger::emit` is
a no-op — it never panics and never blocks.

`TracingBridge::to_global()` routes third-party `tracing::` events into the same
pipeline, and `install_log_capture()` adds dependencies still emitting through the
`log` crate.

**Do not register another reporting layer alongside the bridge.** If an error
reporter has its own `tracing` layer and also receives events through a sink, every
event is counted twice.

`TracingBridge::new(sink)` exists for tests: it emits into one sink directly,
without claiming the process-global dispatch.

## Development

Everything runs through mise.

| Task | Runs |
| --- | --- |
| `mise run fmt` | `cargo fmt --all` |
| `mise run fmt-check` | Fails on unformatted code. The direction CI runs |
| `mise run lint` | `cargo clippy --all-targets -- -D warnings` |
| `mise run test` | `cargo test` |
| `mise run check` | All three, in the order CI runs them |

Tests live in a `tests/` tree mirroring `src/`, behind one aggregator target
declared in `Cargo.toml`. `autotests = false` is required, or Cargo turns every
top-level file in `tests/` into its own binary.
