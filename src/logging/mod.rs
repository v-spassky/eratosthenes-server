use crate::cli::Args;
use crate::logging::quickwit::QuickwitLoggingLayerBuilder;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub mod consts;
pub mod quickwit;

pub fn init(args: &Args) {
    let quickwit_logging_layer = QuickwitLoggingLayerBuilder::new(args.quickwit_url.clone())
        .marker_field("task")
        .map_marker_to_index("http_request", "http_requests")
        .map_marker_to_index("client_sent_ws_message", "client_sent_ws_messages")
        .map_marker_to_index("sockets_count", "sockets_counts")
        .with_batch_size(100)
        .build();
    let env_filter = EnvFilter::default()
        .add_directive("eratosthenes_server=info".parse().unwrap());
    tracing_subscriber::registry()
        .with(quickwit_logging_layer)
        .with(env_filter)
        .with(fmt::layer())
        .init();
}
