use tracing_subscriber::EnvFilter;

pub fn default_init() {
    if let Ok(directives) = std::env::var("ANYTREE_LOG") {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(EnvFilter::new(directives))
            .event_format(
                tracing_subscriber::fmt::format()
                    .with_ansi(true)
                    .with_thread_ids(true)
                    .with_source_location(false),
            )
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(EnvFilter::new("info"))
            .event_format(
                tracing_subscriber::fmt::format()
                    .without_time()
                    .with_ansi(true)
                    .with_target(false)
                    .with_source_location(false),
            )
            .init();
    };
}
