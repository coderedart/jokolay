#[tokio::main]
async fn main() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("warn"))
        .unwrap();
    let writer = std::io::BufWriter::new(std::fs::File::create("./jokolay.log").unwrap());
    let (nb, guard) = tracing_appender::non_blocking(writer);
    std::mem::forget(guard);
    let fmt_layer = fmt::layer().with_target(false).with_writer(nb);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
    miette::set_hook(Box::new(|diagnostic| {
        let handler = Box::new(miette::NarratableReportHandler::new());
        let mut panic_report = String::new();
        if let Err(e) = handler.render_report(&mut panic_report, diagnostic) {
            tracing::error!("failed to render report: {e}");
        }
        tracing::error!("crashing: {}", &panic_report);
        tokio::task::spawn(
            rfd::AsyncMessageDialog::new()
                .set_title("App Crash")
                .set_description(&format!("{}", &panic_report))
                .set_level(rfd::MessageLevel::Error)
                .set_buttons(rfd::MessageButtons::Ok)
                .show(),
        );
        handler
    }))
    .expect("failed to install miette hoook");
    jokolay::start_jokolay();
}
