// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    fs::File,
    io::Result,
    path::Path,
    sync::{Arc, Mutex},
    time::{Instant, SystemTime},
};

use chrome_trace::{Event, Phase, StreamWriter};
use serde_json::json;
use tracing::{
    field::{Field, Visit},
    span::Attributes,
    Id, Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

/// A [`tracing_subscriber::Layer`] that writes instrumentation data to a local
/// JSON file in the Chrome trace event data format.
pub struct ChromeTraceLayer {
    writer: Arc<Mutex<Option<StreamWriter<File>>>>,
    start_time: Instant,
}

impl ChromeTraceLayer {
    /// Creates a [`ChromeTraceLayer`] that writes instrumentation data to a
    /// local JSON file in the specified path.
    ///
    /// The function also returns [`FlushGuard`]. Drop it when you're done with
    /// [`ChromeTraceLayer`] to flush remaining events in the buffer to the
    /// disk.
    pub fn new(path: impl AsRef<Path>) -> Result<(ChromeTraceLayer, FlushGuard)> {
        let file = File::create(path)?;
        let writer = Arc::new(Mutex::new(Some(StreamWriter::new(file)?)));

        let layer = ChromeTraceLayer {
            writer: writer.clone(),
            start_time: Instant::now(),
        };
        let guard = FlushGuard::new(writer);

        layer.emit_initial_events();

        Ok((layer, guard))
    }

    fn emit_initial_events(&self) {
        // Emit a metadata event to record the start time.
        let start_clock = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System time is before UNIX epoch")
            .as_secs_f64();

        self.write_event(|| Event {
            name: "clock_sync".to_owned(),
            category: "".to_owned(),
            phase: Phase::Metadata,
            timestamp: self.get_current_timestamp(),
            process_id: nix::unistd::getpid().as_raw().into(),
            thread_id: nix::unistd::gettid().as_raw().into(),
            args: Some(json!({ "system_time": start_clock })),
        });

        // Emit a metadata event to record the process name.
        self.write_event(|| Event {
            name: "process_name".to_owned(),
            category: "".to_owned(),
            phase: Phase::Metadata,
            timestamp: self.get_current_timestamp(),
            process_id: nix::unistd::getpid().as_raw().into(),
            thread_id: nix::unistd::gettid().as_raw().into(),
            args: Some(json!({ "name": get_current_process_name() })),
        });
    }

    fn get_current_timestamp(&self) -> f64 {
        // Chrome trace event format represents timestamps in microseconds.
        self.start_time.elapsed().as_secs_f64() * 1_000_000.0
    }

    fn write_event<F>(&self, f: F)
    where
        F: FnOnce() -> Event,
    {
        // Check if the writer has been already closed.
        let mut guard = self.writer.lock().expect("Failed to lock StreamWriter");
        let writer = if let Some(writer) = &mut *guard {
            writer
        } else {
            return;
        };

        let event = f();

        // TODO: Offload writes to a separate thread.
        writer.write_event(&event).ok(); // ignore errors
    }
}

impl<S> Layer<S> for ChromeTraceLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let mut args = Args::new();
        attrs.record(&mut ArgsVisitor::new(&mut args));

        // Keep fields in Extensions so we can read them later in `on_enter`.
        let span = ctx.span(id).expect("BUG: span not found");
        span.extensions_mut().insert(ArgsExtension(args));
    }

    fn on_enter(&self, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("BUG: span not found");
        let metadata = span.metadata();

        let args = span
            .extensions_mut()
            .remove::<ArgsExtension>()
            .expect("BUG: ArgsExtension not found")
            .0;
        let args = if args.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(args))
        };

        self.write_event(|| Event {
            name: metadata.name().to_owned(),
            category: metadata.target().to_owned(),
            phase: Phase::Begin,
            timestamp: self.get_current_timestamp(),
            process_id: nix::unistd::getpid().as_raw().into(),
            thread_id: nix::unistd::gettid().as_raw().into(),
            args,
        });
    }

    fn on_exit(&self, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("BUG: span not found");
        let metadata = span.metadata();

        self.write_event(|| Event {
            name: metadata.name().to_owned(),
            category: metadata.target().to_owned(),
            phase: Phase::End,
            timestamp: self.get_current_timestamp(),
            process_id: nix::unistd::getpid().as_raw().into(),
            thread_id: nix::unistd::gettid().as_raw().into(),
            args: None,
        });
    }

    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut args = Args::new();
        event.record(&mut ArgsVisitor::new(&mut args));

        // The event message is in the "message" field.
        let message = match args.remove("message") {
            Some(serde_json::Value::String(message)) => message,
            _ => "".to_owned(),
        };

        let args = if args.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(args))
        };

        let metadata = event.metadata();

        self.write_event(|| Event {
            name: message,
            category: metadata.target().to_owned(),
            phase: Phase::Instant,
            timestamp: self.get_current_timestamp(),
            process_id: nix::unistd::getpid().as_raw().into(),
            thread_id: nix::unistd::gettid().as_raw().into(),
            args,
        });
    }
}

/// RAII object to flush remaining events in the buffer to the disk on drop.
///
/// This object is returned by [`ChromeTraceLayer::new`] to allow you to flush
/// events when you're done with [`ChromeTraceLayer`].
pub struct FlushGuard {
    writer: Arc<Mutex<Option<StreamWriter<File>>>>,
}

impl FlushGuard {
    pub(crate) fn new(writer: Arc<Mutex<Option<StreamWriter<File>>>>) -> Self {
        Self { writer }
    }
}

impl Drop for FlushGuard {
    fn drop(&mut self) {
        let mut guard = self.writer.lock().expect("Failed to lock StreamWriter");
        if let Some(writer) = guard.take() {
            writer.into_inner().ok(); // ignore errors
        }
    }
}

type Args = serde_json::Map<String, serde_json::Value>;

struct ArgsVisitor<'a> {
    args: &'a mut Args,
}

impl<'a> ArgsVisitor<'a> {
    pub fn new(args: &'a mut Args) -> Self {
        Self { args }
    }
}

impl Visit for ArgsVisitor<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.args
            .insert(field.name().to_owned(), format!("{:?}", value).into());
    }
}

struct ArgsExtension(Args);

fn get_current_process_name() -> String {
    let current_exe = std::env::current_exe().unwrap_or_default();
    current_exe
        .file_name()
        .unwrap_or(std::ffi::OsStr::new("__unknown__"))
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use chrome_trace::Trace;
    use tempfile::NamedTempFile;
    use tracing_subscriber::prelude::*;

    use super::*;

    #[test]
    fn test_simple() -> Result<()> {
        let trace_path = NamedTempFile::new()?;
        let trace_path = trace_path.path();

        {
            let (layer, _guard) = ChromeTraceLayer::new(trace_path)?;
            let subscriber = tracing_subscriber::registry().with(layer);
            tracing::subscriber::with_default(subscriber, || {
                let _span = tracing::info_span!("this_is_span").entered();
                tracing::info!("this_is_event");
            });
        }

        let mut trace = Trace::load(File::open(trace_path)?)?;

        // Overwrite timestamps to make the test deterministic.
        for (i, event) in trace.events.iter_mut().enumerate() {
            event.timestamp = i as f64;
        }

        // The first event must be clock_sync which contains system_time arg.
        // Overwrite it with a constant to make the test deterministic.
        const FAKE_SYSTEM_TIME: f64 = 28.0;
        *trace.events[0]
            .args
            .as_mut()
            .expect("First event does not have args")
            .as_object_mut()
            .expect("First event does not have object args")
            .get_mut("system_time")
            .expect("First event does not have system_time arg") = json!(FAKE_SYSTEM_TIME);

        let process_id = nix::unistd::getpid().as_raw().into();
        let thread_id = nix::unistd::gettid().as_raw().into();

        assert_eq!(
            trace,
            Trace {
                events: vec![
                    Event {
                        name: "clock_sync".to_owned(),
                        category: "".to_owned(),
                        phase: Phase::Metadata,
                        timestamp: 0.0,
                        process_id,
                        thread_id,
                        args: Some(json!({ "system_time": FAKE_SYSTEM_TIME })),
                    },
                    Event {
                        name: "process_name".to_owned(),
                        category: "".to_owned(),
                        phase: Phase::Metadata,
                        timestamp: 1.0,
                        process_id,
                        thread_id,
                        args: Some(json!({ "name": get_current_process_name() })),
                    },
                    Event {
                        name: "this_is_span".to_owned(),
                        category: "tracing_chrome_trace::tests".to_owned(),
                        phase: Phase::Begin,
                        timestamp: 2.0,
                        process_id,
                        thread_id,
                        args: None,
                    },
                    Event {
                        name: "this_is_event".to_owned(),
                        category: "tracing_chrome_trace::tests".to_owned(),
                        phase: Phase::Instant,
                        timestamp: 3.0,
                        process_id,
                        thread_id,
                        args: None,
                    },
                    Event {
                        name: "this_is_span".to_owned(),
                        category: "tracing_chrome_trace::tests".to_owned(),
                        phase: Phase::End,
                        timestamp: 4.0,
                        process_id,
                        thread_id,
                        args: None,
                    },
                ]
            }
        );

        Ok(())
    }
}
