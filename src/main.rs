use color_eyre::eyre::WrapErr;
use jokolay::log_initialize;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use sysinfo::SystemExt;

use rfd::{MessageButtons, MessageLevel};

use jokolay::config::ConfigManager;

use jokolay::core::marker::MarkerManager;
use tracing::instrument;
use tracing_appender::non_blocking::WorkerGuard;

#[instrument]
async fn fake_main(
    guard: &mut Option<tracing_appender::non_blocking::WorkerGuard>,
) -> color_eyre::Result<()> {
    let BeforeLogData {
        mut cm,
        worker_guard,
        themes_dir,
        fonts_dir,
        data_dir,
    } = pre_logging_setup().map_err(|e| {
        rfd::MessageDialog::new()
            .set_title("Jokolay failed to start")
            .set_description(&format!("error: {:#?}", &e))
            .set_buttons(MessageButtons::Ok)
            .set_level(MessageLevel::Error)
            .show();
        e
    })?;
    *guard = Some(worker_guard);

    // use i18n_embed::{
    //     fluent::{fluent_language_loader, FluentLanguageLoader},
    //     LanguageLoader,
    // };
    //
    // use rust_embed::RustEmbed;
    // use i18n_embed::DesktopLanguageRequester;
    // #[derive(RustEmbed)]
    // #[folder = "i18n/"]
    // struct Localizations;
    //
    // let loader: FluentLanguageLoader = fluent_language_loader!();
    // loader
    //     .load_languages(&Localizations, &[loader.fallback_language()])
    //     .wrap_err("i18 load langauges error")?;
    // let requested_languages = DesktopLanguageRequester::requested_languages();
    // i18n_embed::select(
    //     &loader, &Localizations, &requested_languages).wrap_err("failed to select a language from requested_languages")?;
    //
    // dbg!(i18n_embed_fl::fl!(loader, "hello"));
    let mut window = jokolay::core::window::OverlayWindow::create(&cm.config)?;
    let wtx = jokolay::core::renderer::WgpuContextImpl::new(&window, &cm.config)
        .await
        .wrap_err("failed to create renderer")?;
    let mut wtx = Arc::new(RwLock::new(wtx));
    let mut etx =
        jokolay::core::gui::Etx::new(wtx.clone(), themes_dir, &cm.config.theme_name, fonts_dir)?;
    let window_id: u32 = (window.window.get_x11_window() as usize)
        .try_into()
        .wrap_err("failed to put x11 window id into u32")?;
    let mut mctx = jokolink::MumbleCtx::new(
        cm.config.mumble_config.clone(),
        window_id,
        window.glfw.get_time(),
    )?;
    // let mut mm = MarkerManager::new(data_dir.join("marker_packs"))
    //     .await
    //     .wrap_err("failed to create marker manager")?;
    let mut timer = std::time::Instant::now();
    let mut fps = 0u32;
    let mut sys = sysinfo::System::new();
    sys.refresh_all();
    while !window.window.should_close() {
        fps += 1;
        if timer.elapsed() > std::time::Duration::from_secs(1) {
            dbg!(fps);
            fps = 0;
            timer = std::time::Instant::now();
        }

        let input = window.tick()?;
        wtx.write()
            .init_framebuffer_view(window.window_state.read().framebuffer_size);
        if wtx.read().fb.is_none() {
            println!("no framebuffer view, so skipping");
            continue;
        }
        let (output, textures_delta, shapes) = etx
            .tick(input, &mut window, wtx.clone(), &mut cm, &mut mctx)
            .await?;
        let time = window.window_state.read().glfw_time;
        let mouse_state = window.window_state.read().mouse_state.clone();
        mctx.tick(time, &mut sys)?;
        if !output.copied_text.is_empty() {
            window.window.set_clipboard_string(&output.copied_text);
        }
        if etx.ctx.wants_pointer_input() || etx.ctx.wants_keyboard_input() {
            #[cfg(target_os = "linux")]
            if !window.window.is_mouse_passthrough() {
                // check if we have been clicked, while we are not passthrough but not focused either. it means mouse is being captured by gw2
                // and we will need to force focus to break that capture.
                if !window.window.is_mouse_passthrough()
                    && ((!mouse_state[0].button_pressed[1] && mouse_state[1].button_pressed[1])
                        || (!mouse_state[0].button_pressed[2] && mouse_state[1].button_pressed[2])
                        || (!mouse_state[0].button_pressed[3] && mouse_state[1].button_pressed[3]))
                    && !window.window.is_focused()
                {
                    window.window.focus();
                }
            }
            window.window.set_mouse_passthrough(false);
        } else {
            window.window.set_mouse_passthrough(true);
        }
        let framebuffer_size = window.window_state.read().framebuffer_size;
        let framebuffer_scale = window.window_state.read().scale;
        etx.draw_egui(wtx.clone(), textures_delta, shapes, framebuffer_scale)?;
        wtx.write().present_framebuffer_view();
    }

    Ok(())
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    {
        // the logger guard. as long as we have this alive, the logger will keep writing to file
        let mut guard = None;
        if let Err(e) = fake_main(&mut guard).await {
            tracing::error!("{:#?}", &e);
            return Err(e);
        }
        Ok(())
    }
}
struct BeforeLogData {
    cm: ConfigManager,
    worker_guard: WorkerGuard,
    themes_dir: PathBuf,
    fonts_dir: PathBuf,
    data_dir: PathBuf,
}
fn pre_logging_setup() -> color_eyre::Result<BeforeLogData> {
    let [config_dir, data_dir, _cache_dir, logs_dir, themes_dir, fonts_dir] =
        jokolay::get_config_data_cache_markers_dirs().wrap_err("failed to get current dir")?;

    let cm = ConfigManager::new(config_dir.join("joko_config.json"))
        .wrap_err("failed to create config manager")?;
    let worker_guard = log_initialize(&logs_dir.join("jokolay.log"), cm.config.log_level.clone())
        .wrap_err("log initialize failed")?;

    Ok(BeforeLogData {
        cm,
        worker_guard,
        themes_dir,
        fonts_dir,
        data_dir,
    })
}
