use std::sync::{Arc, Mutex};

use serde_json::Value;
use tracing::subscriber::DefaultGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub struct CapturedEvents {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl CapturedEvents {
    /// # Panics
    /// Panics if the internal mutex is poisoned.
    pub fn events(&self) -> Vec<Value> {
        let text = {
            let buf = self.buffer.lock().expect("lock poisoned");
            String::from_utf8_lossy(&buf).into_owned()
        };
        text.lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }

    pub fn stage_names(&self) -> Vec<String> {
        self.events()
            .iter()
            .filter_map(|e| {
                e.get("span")
                    .and_then(|s| s.get("name"))
                    .and_then(Value::as_str)
            })
            .map(String::from)
            .collect()
    }
}

#[derive(Clone)]
struct SharedWriter(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("lock poisoned").extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for SharedWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

pub fn install_test_subscriber() -> (DefaultGuard, CapturedEvents) {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let writer = SharedWriter(Arc::clone(&buffer));

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::new("trace"))
        .with(
            fmt::layer()
                .json()
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
                .with_writer(writer),
        );

    let guard = tracing::subscriber::set_default(subscriber);
    let captured = CapturedEvents { buffer };
    (guard, captured)
}

#[macro_export]
macro_rules! assert_no_secrets {
    ($events:expr) => {
        for event in &$events {
            let text = event.to_string();
            assert!(
                !::know_now_audit::redaction::contains_secret(&text),
                "secret detected in log event: {}",
                text
            );
        }
    };
}

#[macro_export]
macro_rules! assert_stage_sequence {
    ($events:expr, $expected:expr) => {{
        let stage_names: Vec<String> = $events
            .iter()
            .filter_map(|e| {
                e.get("span")
                    .and_then(|s| s.get("name"))
                    .and_then(::serde_json::Value::as_str)
            })
            .map(String::from)
            .collect();
        let expected: Vec<&str> = $expected.iter().map(|s| s.as_str()).collect();
        assert_eq!(stage_names, expected, "stage sequence mismatch");
    }};
}
