use std::path::Path;
use jokolay::log_initialize;

fn main() -> anyhow::Result<()>{
    let _guard = log_initialize(Path::new("./assets"))?;
    let window = jokolay::core::window::OverlayWindow::create([800, 600].into())?;
    unsafe { jokolay::core::renderer::init::Renderer::initialize_vulkan(&window, true); }
    Ok(())
}
