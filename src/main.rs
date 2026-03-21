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

    let window = dioxus::desktop::WindowBuilder::new()
        .with_title("Compare Observer")
        .with_always_on_top(false);

    // Pass an empty Menu so the native "Window / Edit / Help" bar is removed.
    let empty_menu = muda::Menu::new();

    // LastWindowHides: the X button hides the window instead of immediately
    // exiting the process, giving the app a chance to show a confirmation
    // dialog when file-watching is active.
    use dioxus::desktop::WindowCloseBehaviour;

    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::default()
                .with_window(window)
                .with_menu(Some(empty_menu))
                .with_close_behaviour(WindowCloseBehaviour::LastWindowHides),
        )
        .launch(ui::app::App);
}
