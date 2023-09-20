use std::io::{BufRead, BufReader};

use indicatif::ProgressStyle;
use tracing::Span;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub fn default_init() {
    let indicatif_layer = IndicatifLayer::new();

    if let Ok(directives) = std::env::var("ANYTREE_LOG") {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(indicatif_layer.get_stderr_writer())
                    .event_format(
                        tracing_subscriber::fmt::format()
                            .with_ansi(true)
                            .with_thread_ids(true)
                            .with_source_location(false),
                    ),
            )
            .with(EnvFilter::new(directives))
            .with(indicatif_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(indicatif_layer.get_stderr_writer())
                    .event_format(
                        tracing_subscriber::fmt::format()
                            .with_ansi(true)
                            .with_thread_ids(true)
                            .with_source_location(false),
                    ),
            )
            .with(EnvFilter::new("info"))
            .with(indicatif_layer)
            .init();
    }
}

pub fn wrap_cmd_with_tracing(child: &mut std::process::Child) {
    let stdout = child.stdout.take().expect("failed to capture stdout");
    let stderr = child.stderr.take().expect("failed to capture stderr");
    std::thread::scope(|s| {
        s.spawn(|| {
            tracing::trace!("follow stdout");
            for line in BufReader::new(stdout).lines() {
                println!("out | {}", line.unwrap());
            }
        });
        s.spawn(|| {
            tracing::trace!("follow stderr");
            for line in BufReader::new(stderr).lines() {
                println!("err | {}", line.unwrap());
            }
        });
    });
}

pub fn start_progress(length: u64) -> Span {
    let header_span = tracing::info_span!("progress");
    header_span.pb_set_style(&ProgressStyle::default_bar());
    header_span.pb_set_length(length);
    header_span
}

pub fn increase_progress() {
    Span::current().pb_inc(1);
}
