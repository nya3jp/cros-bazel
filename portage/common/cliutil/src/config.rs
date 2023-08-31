// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::stdio_redirector::{RedirectorConfig, StdioRedirector};
use crate::LoggingConfig;
use anyhow::Result;

/// Similar to Option::unwrap_or_else, but handles the Result type.
fn unwrap_or_else<T, F>(value: Option<T>, default: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    match value {
        Some(value) => Ok(value),
        None => default(),
    }
}

/// The configuration for the current process.
/// This should rarely be used, as most users will just prefer Default::default().
#[derive(Default)]
pub struct ConfigBuilder {
    logging: Option<LoggingConfig>,

    log_command_line: bool,

    redirect_stdio: Option<RedirectorConfig>,
}

// Manually mark as inline, because rust doesn't like inlining across crate boundaries.
impl ConfigBuilder {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            logging: None,
            log_command_line: true,
            redirect_stdio: None,
        }
    }

    #[inline(always)]
    /// Overrides the logging config. If this isn't called, it defaults to
    /// `LoggingConfig::from_env()`.
    pub fn logging(mut self, cfg: LoggingConfig) -> Self {
        self.logging = Some(cfg);
        self
    }

    #[inline(always)]
    /// `enable` controls whether to log the command-line of the current process.
    pub fn log_command_line(mut self, enable: bool) -> Self {
        self.log_command_line = enable;
        self
    }

    #[inline(always)]
    /// `cfg` controls redirection. Defaults to `RedirectorConfig::from_env()`.
    pub fn redirect_stdio(mut self, cfg: RedirectorConfig) -> Self {
        self.redirect_stdio = Some(cfg);
        self
    }

    #[inline(always)]
    /// Builds a Config suitable for use with cli_main.
    pub fn build(self) -> Result<Config> {
        Ok(Config {
            logging: unwrap_or_else(self.logging, LoggingConfig::from_env)?,
            log_command_line: self.log_command_line,
            stdio_redirector: self
                .redirect_stdio
                .unwrap_or_else(RedirectorConfig::from_env)
                .create()?,
        })
    }
}

/// A POD struct containing the configs, after applying any defaults for unset values.
/// Build it with `cliutil::ConfigBuilder::new().set_<field>().build()`.
pub struct Config {
    pub(crate) logging: LoggingConfig,
    pub(crate) log_command_line: bool,
    pub(crate) stdio_redirector: Option<StdioRedirector>,
}

impl Default for Config {
    fn default() -> Self {
        ConfigBuilder::new().build().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_config() {
        let config = ConfigBuilder::new()
            .log_command_line(false)
            .build()
            .unwrap();
        assert_eq!(config.log_command_line, false);
    }
}
