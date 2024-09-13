use crate::logging::consts::{DEFAULT_LOGGING_BUFFER_SIZE, QUICKWIT_LOGGING_CHANNEL_CAPACITY};
use reqwest::Client;
use serde::Serialize;
use std::collections::HashMap;
use std::io;
use tokio::sync::mpsc;
use tokio::task;
use tracing::field::{Field, Visit};
use tracing_core::Event;
use tracing_core::Subscriber;
use tracing_subscriber::layer::Context as TracingContext;
use tracing_subscriber::Layer;
use url::Url;
pub struct QuickwitLoggingLayerBuilder {
    quickwit_url: Url,
    target_field: String,
    field_to_index: HashMap<String, String>,
    batch_size: usize,
}

impl QuickwitLoggingLayerBuilder {
    // TODO: make `new()` accept something that can be made into `Url` (`&str`, `String` or `Url`)
    // TODO: check that `quickwit_url` is reachable
    pub fn new(quickwit_url: Url) -> Self {
        Self {
            quickwit_url,
            target_field: String::new(),
            field_to_index: HashMap::new(),
            batch_size: DEFAULT_LOGGING_BUFFER_SIZE,
        }
    }

    pub fn marker_field(mut self, field: &str) -> Self {
        self.target_field = field.to_string();
        self
    }

    pub fn map_marker_to_index(mut self, field_value: &str, index_id: &str) -> Self {
        self.field_to_index
            .insert(field_value.to_string(), index_id.to_string());
        self
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    pub fn build(self) -> QuickwitLoggingLayer {
        QuickwitLoggingLayer::new(
            self.target_field,
            self.field_to_index,
            self.quickwit_url,
            self.batch_size,
        )
    }
}

pub struct QuickwitLoggingLayer {
    sender: mpsc::Sender<QuickwitLogMessage>,
    target_field: String,
    // TODO: consider `&' static` instead of `String`
    field_to_index: HashMap<String, String>,
}

impl QuickwitLoggingLayer {
    fn new(
        target_field: String,
        field_to_index: HashMap<String, String>,
        quickwit_url: Url,
        batch_size: usize,
    ) -> Self {
        // TODO: capacity should be configurable
        let (sender, receiver) = mpsc::channel(QUICKWIT_LOGGING_CHANNEL_CAPACITY);
        let index_names = field_to_index.values().cloned().collect::<Vec<_>>();
        QuickwitLoggingTask::new(quickwit_url, receiver, batch_size, index_names);
        Self {
            sender,
            target_field,
            field_to_index,
        }
    }

    fn should_log_event(&self, event: &Event<'_>) -> Option<String> {
        let mut visitor = TargetFieldVisitor::new(&self.target_field);
        event.record(&mut visitor);
        visitor
            .target_value
            .and_then(|value| self.field_to_index.get(&value).cloned())
    }
}

impl<S: Subscriber> Layer<S> for QuickwitLoggingLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: TracingContext<'_, S>) {
        if let Some(index_id) = self.should_log_event(event) {
            let mut visitor = LogVisitor::new();
            event.record(&mut visitor);
            let log_message = QuickwitLogMessage {
                index_id,
                log: visitor.log,
            };
            // TODO: let the client configure sending strategy (blocking or non-blocking, timeout,
            // `on_error` callback etc.)
            let _ = self.sender.try_send(log_message);
        }
    }
}

pub struct QuickwitLoggingTask {}

impl QuickwitLoggingTask {
    fn new(
        quickwit_url: Url,
        mut receiver: mpsc::Receiver<QuickwitLogMessage>,
        batch_size: usize,
        index_names: Vec<String>,
    ) -> Self {
        let http_client = Client::new();
        task::spawn(async move {
            let mut buffers = HashMap::new();
            for index_name in index_names {
                buffers.insert(index_name, Vec::with_capacity(batch_size));
            }
            while let Some(QuickwitLogMessage { index_id, log }) = receiver.recv().await {
                // TODO: this `.unwrap()` is bad (can break the whole task)
                let buffer = buffers.get_mut(&index_id).unwrap();
                (*buffer).push(log);
                if buffer.len() >= batch_size {
                    // TODO: reuse `ndjson_body`
                    let mut ndjson_body = Vec::new();
                    for log in buffer.iter() {
                        serialize_to_ndjson(&mut ndjson_body, log).unwrap();
                    }
                    // TODO: provide a user-specified `on_error()` callback here
                    let _response = http_client
                        .post(&format!("{}api/v1/{}/ingest", quickwit_url, index_id))
                        .body(ndjson_body)
                        .send()
                        .await;
                    buffer.clear();
                }
            }
            for (index_id, buffer) in buffers.iter_mut() {
                let mut ndjson_body = Vec::new();
                for log in buffer.iter() {
                    serialize_to_ndjson(&mut ndjson_body, log).unwrap();
                }
                let _response = http_client
                    .post(&format!("{}api/v1/{}/ingest", quickwit_url, index_id))
                    .body(ndjson_body)
                    .send()
                    .await;
                buffer.clear();
            }
        });
        Self {}
    }
}

struct TargetFieldVisitor {
    target_field: String,
    target_value: Option<String>,
}

impl TargetFieldVisitor {
    pub fn new(target_field: &str) -> Self {
        Self {
            target_field: target_field.to_string(),
            target_value: None,
        }
    }
}

impl Visit for TargetFieldVisitor {
    fn record_debug(&mut self, _field: &Field, _value: &dyn std::fmt::Debug) {}

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == self.target_field {
            self.target_value = Some(value.to_string());
        }
    }
}

struct LogVisitor {
    log: serde_json::Map<String, serde_json::Value>,
}

impl LogVisitor {
    pub fn new() -> Self {
        Self {
            log: serde_json::Map::new(),
        }
    }
}

impl Visit for LogVisitor {
    // TODO: add methods for all types

    fn record_str(&mut self, field: &Field, value: &str) {
        self.log.insert(field.name().to_string(), value.into());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.log.insert(field.name().to_string(), value.into());
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        // TODO: this isn't ideal and must be documented (Quickwit uses u64)
        self.log
            .insert(field.name().to_string(), (value as u64).into());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.log
            .insert(field.name().to_string(), format!("{:?}", value).into());
    }
}

#[derive(Debug, Serialize)]
struct QuickwitLogMessage {
    index_id: String,
    log: serde_json::Map<String, serde_json::Value>,
}

fn serialize_to_ndjson<W, V>(mut writer: W, value: &V) -> io::Result<()>
where
    W: io::Write,
    V: ?Sized + Serialize,
{
    serde_json::to_writer(&mut writer, value)?;
    writer.write_all(&[b'\n'])
}
