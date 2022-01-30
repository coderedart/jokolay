use jokolay::log_initialize;
use std::path::Path;

fn fake_main() -> anyhow::Result<()> {
    let _guard = log_initialize(
        Path::new("./assets"),
        tracing::level_filters::LevelFilter::INFO,
    )?;
    let mut window = jokolay::core::window::OverlayWindow::create([800, 600].into())?;
    let mut renderer = jokolay::core::renderer::Renderer::initialize_vulkan(&window, true)?;
    while !window.window.should_close() {
        window.tick()?;
        renderer.tick()?;
    }
    Ok(())
}
fn main() {
    let _ = dbg!(fake_main());
}
