mod core;
mod services;
mod ui;
mod utils;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "compare_observer=info".into()),
        )
        .init();

    dioxus::launch(ui::app::App);
}
