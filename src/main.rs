fn main() {
    install_tracing();
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .theme(color_eyre::config::Theme::new())
        .into_hooks();

    eyre_hook
        .install()
        .expect("there won't be any conflicting eyre hooks previously installed");

    std::panic::set_hook(Box::new(move |panic_info| {
        let panic_report = panic_hook.panic_report(panic_info);
        rfd::MessageDialog::new()
            .set_title("App Crash")
            .set_description(&format!("{}", &panic_report))
            .set_level(rfd::MessageLevel::Error)
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
    }));
    jokolay::start_jokolay();
}

fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}
