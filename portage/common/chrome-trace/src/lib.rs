// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Implements reading/writing Chrome trace event data.
//!
//! See the following document for the official specification of the Chrome
//! trace event format.
//! https://docs.google.com/document/d/1CvAClvFfyA5R-PhYUmn5OOQtYMH4h6I0nSsKchNAySU/preview

use std::io::{BufReader, BufWriter, Error, ErrorKind, Read, Result, Write};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Holds a series of trace events.
///
/// Currently it only supports the JSON array format of Chrome trace events.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Trace {
    pub events: Vec<Event>,
}

impl Trace {
    /// Creates a new empty trace.
    pub fn new() -> Self {
        Default::default()
    }

    /// Loads trace events from [`Read`].
    pub fn load(r: impl Read) -> Result<Self> {
        let mut r = BufReader::new(r);

        // Read until the first `[`.
        // We can't simply decode the JSON array because the spec explicitly
        // says that "the ] at the end of the JSON Array Format is optional".
        consume_char(&mut r, '[')?;

        let mut events: Vec<Event> = Vec::new();
        loop {
            match serde_json::Deserializer::from_reader(&mut r)
                .into_iter()
                .next()
            {
                Some(Ok(event)) => {
                    events.push(event);
                }
                Some(Err(err)) if err.is_io() => {
                    // Propagate IO errors.
                    return Err(err.into());
                }
                Some(Err(_)) | None => {
                    // Ignore non-IO errors as it's possible that trace files
                    // are not flushed on program crashes.
                    // TODO: Let the caller know this error without making the
                    // load failure.
                    break;
                }
            }

            if let Err(err) = consume_char(&mut r, ',') {
                match err.kind() {
                    ErrorKind::UnexpectedEof | ErrorKind::InvalidData => {}
                    _ => {
                        return Err(err);
                    }
                }
            }
        }

        Ok(Trace { events })
    }

    /// Saves trace events to [`Write`].
    ///
    /// If you want to write trace events in a streamed way, use
    /// [`StreamWriter`] instead.
    pub fn save(&self, w: impl Write) -> Result<()> {
        let mut w = BufWriter::new(w);
        serde_json::to_writer(&mut w, &self.events)?;
        w.flush()
    }
}

/// Streaming writer of trace events.
pub struct StreamWriter<W>
where
    W: Write,
{
    writer: Option<BufWriter<W>>,
    first_event_was_written: bool,
    finished: bool,
}

impl<W> StreamWriter<W>
where
    W: Write,
{
    /// Creates a new [`StreamWriter`]. It returns an error if it fails to write
    /// the header part of a trace file.
    ///
    /// Remember to call [`StreamWriter::finish`] or
    /// [`StreamWriter::into_inner`] on finishing to write events.
    /// [`StreamWriter::finish`] is also called on drop, but errors will be
    /// ignored.
    pub fn new(writer: W) -> Result<Self> {
        let mut writer = BufWriter::new(writer);
        writer.write_all("[\n".as_bytes())?;
        Ok(Self {
            writer: Some(writer),
            first_event_was_written: false,
            finished: false,
        })
    }

    /// Writes a trace event.
    pub fn write_event(&mut self, event: &Event) -> Result<()> {
        if self.finished {
            return Err(Error::new(ErrorKind::Other, "Stream already finished"));
        }
        let writer = self.writer.as_mut().expect("Underlying writer missing");
        if self.first_event_was_written {
            writer.write_all(",\n".as_bytes())?;
        } else {
            self.first_event_was_written = true;
        }
        serde_json::to_writer(writer, event)?;
        Ok(())
    }

    /// Finishes writing events by writing a footer and flushing the buffer.
    ///
    /// Remember to call [`StreamWriter::finish`] or
    /// [`StreamWriter::into_inner`] on finishing to write events.
    /// This method is also called on drop, but errors will be ignored.
    ///
    /// It is safe to call this method multiple times.
    pub fn finish(&mut self) -> Result<()> {
        if self.finished {
            return Ok(());
        }
        let writer = self.writer.as_mut().expect("Underlying writer missing");
        writer.write_all("\n]\n".as_bytes())?;
        writer.flush()?;
        self.finished = true;
        Ok(())
    }

    /// Finishes writing events and returns the underlying [`Write`].
    ///
    /// It calls [`StreamWriter::finish`] automatically if it's not yet called,
    /// but calling it by yourself in advance guarantees that this method
    /// succeeds and thus always being able to get the underlying writer.
    pub fn into_inner(mut self) -> Result<W> {
        self.finish()?;
        let underlying_writer = self
            .writer
            .take()
            .expect("Underlying writer missing")
            .into_inner()?;
        Ok(underlying_writer)
    }
}

impl<W> Drop for StreamWriter<W>
where
    W: Write,
{
    fn drop(&mut self) {}
}

/// Reads the stream until reaching the first non-whitespace character and
/// ensures that it is the specified character.
fn consume_char(mut r: impl Read, c: char) -> Result<()> {
    let mut buf: [u8; 1] = [0];
    loop {
        let size = r.read(&mut buf)?;
        if size == 0 {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                format!("EOF reached while searching {}", c),
            ));
        }
        if (buf[0] as char).is_ascii_whitespace() {
            continue;
        }
        if buf[0] != c as u8 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("expected {}, got {}", c, buf[0] as char),
            ));
        }
        return Ok(());
    }
}

/// Represents the type of a trace event, aka phase.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Phase {
    #[serde(rename = "B")]
    Begin,
    #[serde(rename = "E")]
    End,
    #[serde(rename = "i", alias = "I")]
    Instant,
    #[serde(rename = "M")]
    Metadata,
}

/// Represents a trace event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    #[serde(rename = "name")]
    pub name: String,
    // TODO: Consider changing this to Vec<String>. The spec defines this as a
    // comma-separated array of category strings.
    #[serde(rename = "cat")]
    pub category: String,
    #[serde(rename = "ph")]
    pub phase: Phase,
    #[serde(rename = "ts")]
    pub timestamp: f64,
    #[serde(rename = "pid")]
    pub process_id: i64,
    #[serde(rename = "tid")]
    pub thread_id: i64,
    #[serde(rename = "args", skip_serializing_if = "Option::is_none")]
    pub args: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use lazy_static::lazy_static;

    // Sample data from the spec.
    // https://docs.google.com/document/d/1CvAClvFfyA5R-PhYUmn5OOQtYMH4h6I0nSsKchNAySU/preview
    const SAMPLE_DATA: &str = r#"
    [ {"name": "Asub", "cat": "PERF", "ph": "B", "pid": 22630, "tid": 22630, "ts": 829},
      {"name": "Asub", "cat": "PERF", "ph": "E", "pid": 22630, "tid": 22630, "ts": 833} ]
    "#;

    lazy_static! {
        static ref SAMPLE_EVENTS: Vec<Event> = vec![
            Event {
                name: "Asub".to_owned(),
                category: "PERF".to_owned(),
                phase: Phase::Begin,
                process_id: 22630,
                thread_id: 22630,
                timestamp: 829.0,
                args: None,
            },
            Event {
                name: "Asub".to_owned(),
                category: "PERF".to_owned(),
                phase: Phase::End,
                process_id: 22630,
                thread_id: 22630,
                timestamp: 833.0,
                args: None,
            },
        ];
    }

    #[test]
    fn test_trace_load() -> Result<()> {
        assert_eq!(
            Trace::load(SAMPLE_DATA.as_bytes())?,
            Trace {
                events: vec![
                    Event {
                        name: "Asub".to_owned(),
                        category: "PERF".to_owned(),
                        phase: Phase::Begin,
                        process_id: 22630,
                        thread_id: 22630,
                        timestamp: 829.0,
                        args: None,
                    },
                    Event {
                        name: "Asub".to_owned(),
                        category: "PERF".to_owned(),
                        phase: Phase::End,
                        process_id: 22630,
                        thread_id: 22630,
                        timestamp: 833.0,
                        args: None,
                    },
                ],
            },
        );
        Ok(())
    }

    #[test]
    fn test_trace_load_empty() -> Result<()> {
        assert_eq!(Trace::load("[]".as_bytes())?, Trace { events: vec![] });
        assert!(Trace::load("".as_bytes()).is_err());
        Ok(())
    }

    #[test]
    fn test_trace_load_robust() -> Result<()> {
        assert_eq!(Trace::load("[".as_bytes())?, Trace { events: vec![] });
        assert_eq!(Trace::load("[{".as_bytes())?, Trace { events: vec![] });
        assert_eq!(Trace::load("[{\"nam".as_bytes())?, Trace { events: vec![] });
        Ok(())
    }

    #[test]
    fn test_trace_save_load() -> Result<()> {
        let original_trace = Trace {
            events: SAMPLE_EVENTS.clone(),
        };

        let loaded_trace = {
            let mut buf: Vec<u8> = Vec::new();
            original_trace.save(&mut buf)?;
            Trace::load(buf.as_slice())?
        };

        assert_eq!(original_trace, loaded_trace);

        Ok(())
    }

    #[test]
    fn test_stream_writer() -> Result<()> {
        let mut buf: Vec<u8> = Vec::new();

        let mut stream = StreamWriter::new(&mut buf)?;
        for event in SAMPLE_EVENTS.iter() {
            stream.write_event(event)?;
        }
        drop(stream);

        let trace = Trace::load(buf.as_slice())?;
        assert_eq!(trace.events, *SAMPLE_EVENTS);

        Ok(())
    }

    #[test]
    fn test_stream_writer_empty() -> Result<()> {
        let mut buf: Vec<u8> = Vec::new();

        StreamWriter::new(&mut buf)?.finish()?;

        assert_eq!(buf.as_slice(), "[\n\n]\n".as_bytes());

        Ok(())
    }

    #[test]
    fn test_stream_writer_finish() -> Result<()> {
        let mut buf: Vec<u8> = Vec::new();

        let mut stream = StreamWriter::new(&mut buf)?;
        for event in SAMPLE_EVENTS.iter() {
            stream.write_event(event)?;
        }

        // Call finish. Subsequent write_event() calls should fail.
        stream.finish()?;

        for event in SAMPLE_EVENTS.iter() {
            assert!(
                stream.write_event(event).is_err(),
                "StreamWriter::write_event unexpectedly succeeded after finish"
            );
        }

        // Second finish is permitted.
        stream.finish()?;

        drop(stream);

        let trace = Trace::load(buf.as_slice())?;
        assert_eq!(trace.events, *SAMPLE_EVENTS);

        Ok(())
    }
}
