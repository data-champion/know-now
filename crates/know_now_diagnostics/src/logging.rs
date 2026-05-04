use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    Default,
    Verbose,
    Debug,
}

pub struct LogConfig {
    pub verbosity: Verbosity,
    pub format: OutputFormat,
    pub color: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            verbosity: Verbosity::Default,
            format: OutputFormat::Text,
            color: atty_stderr(),
        }
    }
}

pub fn init_subscriber(config: &LogConfig) {
    let filter = match config.verbosity {
        Verbosity::Default => EnvFilter::new("warn"),
        Verbosity::Verbose => EnvFilter::new("info"),
        Verbosity::Debug => EnvFilter::new("debug"),
    };

    match config.format {
        OutputFormat::Text => {
            let layer = fmt::layer()
                .with_target(false)
                .with_ansi(config.color)
                .with_span_events(FmtSpan::CLOSE)
                .with_writer(std::io::stderr);

            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .init();
        }
        OutputFormat::Json => {
            let layer = fmt::layer()
                .json()
                .with_span_events(FmtSpan::CLOSE)
                .with_writer(std::io::stderr);

            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .init();
        }
    }
}

fn atty_stderr() -> bool {
    use std::io::IsTerminal;
    std::io::stderr().is_terminal()
}
