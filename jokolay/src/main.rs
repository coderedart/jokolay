use std::path::Path;
use jokolay::log_initialize;

fn main() -> anyhow::Result<()>{
    let _guard = log_initialize(Path::new("./assets"), tracing::level_filters::LevelFilter::INFO)?;
    let mut window = jokolay::core::window::OverlayWindow::create([800, 600].into())?;
    unsafe { jokolay::core::renderer::init::Renderer::initialize_vulkan(&window, true); }
while !window.window.should_close() {
    window.tick();
}
    Ok(())
}
