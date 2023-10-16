// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use anyhow::{bail, Context, Result};
use itertools::Itertools;
use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};
use tracing_chrome_trace::ChromeTraceLayer;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::prelude::*;
use tracing_subscriber::Layer;

/// Name of the environment variable containing the trace directory and file respectively.
/// If both are provided, an error is thrown.
/// If neither is provided, no tracing is performed.
pub const TRACE_DIR_ENV: &str = "CROS_BAZEL_TRACE_DIR";
pub const TRACE_FILE_ENV: &str = "CROS_BAZEL_TRACE_FILE";

/// Name of the environment variable containing the log directory and file respectively.
/// If both are provided, an error is thrown.
/// If neither is provided, no logging to files is performed.
pub const LOG_DIR_ENV: &str = "CROS_BAZEL_LOG_DIR";
pub const LOG_FILE_ENV: &str = "CROS_BAZEL_LOG_FILE";

/// An environment variable choosing whether to log to the console.
/// If "0", don't log to the console.
/// Otherwise, do log to the console.
pub const CONSOLE_LOG_ENV: &str = "CROS_BAZEL_LOG_CONSOLE";

/// A guard object to perform cleanups with RAII.
pub struct LogGuard {
    _span_guard: tracing::span::EnteredSpan,
    _flush_guard: Option<tracing_chrome_trace::FlushGuard>,
}

/// The configuration for the logger.
/// Technically this also supports tracing, but most users are more familiar with logging than
/// tracing, and thus I suspect most users will use log::info rather than trace::info.
pub struct LoggingConfig {
    /// The path to dump the trace json file to.
    pub trace_file: Option<PathBuf>,
    /// The path to dump the logs to, and a filter for which logs should be dumped there.
    /// If None, logs will not be written to a file.
    pub log_file: Option<(PathBuf, EnvFilter)>,
    /// A filter for which logs should be written to the console.
    /// If None, logs will not be written to the console.
    pub console_logger: Option<EnvFilter>,
}

impl LoggingConfig {
    pub fn from_env() -> Result<Self> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let get_file = |dir_env, file_env, ext| {
            Ok(
                match (std::env::var_os(file_env), std::env::var_os(dir_env)) {
                    (Some(_), Some(_)) => bail!("You can't have both {file_env} and {dir_env} set"),
                    (Some(file), None) => {
                        // Subprocesses shouldn't try and share a log file - that'd get confusing.
                        // If you start subprocesses that do logging, CROS_BAZEL_LOG_DIR is
                        // probably more appropriate.
                        std::env::remove_var(file_env);
                        Some(PathBuf::from(file))
                    }
                    (None, Some(dir)) => {
                        let name =
                            format!("{}.{timestamp}.{ext}", crate::get_current_process_name(),);
                        Some(Path::new(&dir).join(name))
                    }
                    (None, None) => None,
                },
            )
        };

        let trace_file = get_file(TRACE_DIR_ENV, TRACE_FILE_ENV, "json")?;
        let log_file = get_file(LOG_DIR_ENV, LOG_FILE_ENV, "log")?;

        let console_logger = match std::env::var(CONSOLE_LOG_ENV).ok().as_deref() {
            Some("0") => None,
            _ => Some(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env()?,
            ),
        };

        let log_file = match log_file {
            Some(log_file) => Some((
                log_file,
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env()?,
            )),
            None => None,
        };

        Ok(Self {
            trace_file,
            log_file,
            console_logger,
        })
    }

    /// Sets up the standard tracing subscriber in accordance with the config, and starts a span
    /// named "main".
    pub fn setup(self) -> Result<LogGuard> {
        // It's impossible to do tracing and logging independently because both require a
        // subscription to a global logger, of which there can only be one.
        // Thus, we use the tracing crate to do logging as well as tracing.
        let mut layers = Vec::new();

        let flush_guard = if let Some(trace_file) = &self.trace_file {
            let (chrome_layer, flush_guard) = ChromeTraceLayer::new(trace_file)
                .with_context(|| format!("Failed to set up tracing to {trace_file:?}"))?;
            layers.push(chrome_layer.boxed());
            Some(flush_guard)
        } else {
            None
        };

        if let Some(filter) = self.console_logger {
            layers.push(
                tracing_subscriber::fmt::layer()
                    .with_ansi(true)
                    .with_writer(std::io::stderr)
                    .with_filter(filter)
                    .boxed(),
            );
        }

        if let Some((log_file, filter)) = self.log_file {
            let f = std::fs::File::create(&log_file)
                .with_context(|| format!("Failed to open log file {log_file:?}"))?;
            layers.push(
                tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .with_writer(f)
                    .with_filter(filter)
                    .boxed(),
            );
        }

        tracing_subscriber::registry()
            .with(layers)
            .try_init()
            .context(
                "Failed to start tracing. You probably already have either a trace or log
                subscriber running.",
            )?;

        let args = std::env::args()
            .map(|s| shell_escape::escape(s.into()))
            .join(" ");
        let env = std::env::vars()
            .map(|(key, value)| format!("{}={}", key, value))
            .join("\n");
        // Make this trace level to avoid getting printed every invocation, since env can be *very*
        // long.
        let span_guard = tracing::trace_span!("main", args = args, env = env).entered();

        Ok(LogGuard {
            _span_guard: span_guard,
            _flush_guard: flush_guard,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fileutil::SafeTempDir;

    #[test]
    fn setup_logging_works() -> Result<()> {
        const INFO_MESSAGE: &str = "log at level info";
        const WARN_MESSAGE: &str = "log at level warn";
        const DEBUG_MESSAGE: &str = "log at level debug";

        let dir = SafeTempDir::new()?;
        std::env::set_var("RUST_LOG", "INFO");
        let trace_file = dir.path().join("trace.json");
        let log_file = dir.path().join("out.log");
        std::env::set_var(TRACE_FILE_ENV, &trace_file);
        std::env::set_var(LOG_FILE_ENV, &log_file);
        // We can't really verify this very easily and it'll just pollute stderr.
        std::env::set_var(CONSOLE_LOG_ENV, "0");

        {
            // Unfortunately we can't run multiple tests, because the tracing library attempts to
            // subscribe to a global logger, and only one thing can subscribe to a a global logger.
            let _guard = LoggingConfig::from_env()?.setup()?;

            tracing::warn!("{}", WARN_MESSAGE);
            tracing::info!("{}", INFO_MESSAGE);
            tracing::debug!("{}", DEBUG_MESSAGE);
        }

        let trace_content = std::fs::read_to_string(trace_file)?;
        // The trace is json-formatted.
        assert!(
            trace_content.contains(&format!("\"{}\"", DEBUG_MESSAGE)),
            "Unable to find debug message in {}",
            trace_content
        );
        assert!(
            trace_content.contains(&format!("\"{}\"", INFO_MESSAGE)),
            "Unable to find info message in {}",
            trace_content
        );
        assert!(
            trace_content.contains(&format!("\"{}\"", WARN_MESSAGE)),
            "Unable to find warn message in {}",
            trace_content
        );
        let log_content = std::fs::read_to_string(log_file)?;
        assert!(
            !log_content.contains(DEBUG_MESSAGE),
            "Found unexpected debug message in {}",
            log_content
        );
        assert!(
            log_content.contains(INFO_MESSAGE),
            "Unable to find info message in {}",
            log_content
        );
        assert!(
            log_content.contains(WARN_MESSAGE),
            "Unable to find warn message in {}",
            log_content
        );

        Ok(())
    }
}
