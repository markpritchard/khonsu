use crate::config;
use serde::export::Formatter;

mod detect;
mod service;

pub type RefreshResult<T> = std::result::Result<T, RefreshErrorKind>;

/// An error returned when attempting to execute a refresh
#[derive(Debug)]
pub enum RefreshErrorKind {
    ConfigError(&'static str),
    DetectError(Box<dyn std::error::Error>),
    RuntimeError(String),
    ServiceError(Box<dyn std::error::Error>),
}

impl std::error::Error for RefreshErrorKind {}

impl std::fmt::Display for RefreshErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self)
    }
}

impl From<reqwest::Error> for RefreshErrorKind {
    fn from(err: reqwest::Error) -> Self {
        RefreshErrorKind::DetectError(Box::new(err))
    }
}

pub(crate) fn run(config: &config::Config) -> RefreshResult<()> {
    // Detect
    let address = if let Some(cmd_cfg) = &config.detect.cmd {
        detect::cmd(cmd_cfg)?
    } else if let Some(http_cfg) = &config.detect.http {
        detect::http(http_cfg)?
    } else {
        return Err(RefreshErrorKind::ConfigError("No detector configuration provided"));
    };

    // Update
    if let Some(cf) = &config.service.cloudflare {
        service::cloudflare(cf, address)?
    }

    Ok(())
}
