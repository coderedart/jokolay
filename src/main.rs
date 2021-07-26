use jokolay::{gui::eapp::EguiApp, JokolayApp};

fn main() -> anyhow::Result<()> {
    jokolay::log_init(
        log::LevelFilter::Error,
        log::LevelFilter::Trace,
        "./joko.log".into(),
    )?;
    let app = JokolayApp::new()?;
    app.run()?;
    Ok(())
}
