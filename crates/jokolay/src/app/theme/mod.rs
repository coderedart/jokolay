use std::{collections::BTreeMap, io::Read, sync::Arc};

use cap_std::fs_utf8::Dir;
use egui::Style;
use miette::{Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
pub struct ThemeManager {
    dir: Arc<Dir>,
    themes_dir: Arc<Dir>,
    fonts_dir: Arc<Dir>,
    themes: BTreeMap<String, Theme>,
    fonts: BTreeMap<String, Vec<u8>>,
    config: ThemeManagerConfig,
    ui_data: ThemeUIData,
}

#[derive(Debug, Default)]
struct ThemeUIData {
    tab: ThemeUITab,
    theme_name: String,
    current_theme_name: String,
}
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum ThemeUITab {
    #[default]
    LiveEditor,
    Config,
}
/// This holds all the theme settings for jokolay
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct Theme {
    pub style: Style,
}
#[derive(Debug, Serialize, Deserialize)]
struct ThemeManagerConfig {
    default_theme: String,
}
impl Default for ThemeManagerConfig {
    fn default() -> Self {
        Self {
            default_theme: ThemeManager::DEFAULT_THEME_NAME.to_owned(),
        }
    }
}

impl ThemeManager {
    const THEME_MANAGER_DIR_NAME: &str = "theme_manager";
    const THEMES_DIR_NAME: &str = "themes";
    const FONTS_DIR_NAME: &str = "fonts";
    const DEFAULT_FONT_NAME: &str = "default";
    const DEFAULT_THEME_NAME: &str = "default";
    const THEME_MANAGER_CONFIG_NAME: &str = "theme_manager_config";
    pub fn new(jdir: &Dir) -> Result<Self> {
        jdir.create_dir_all(Self::THEME_MANAGER_DIR_NAME)
            .into_diagnostic()
            .wrap_err("failed to create theme manager dir")?;
        let dir: Arc<Dir> = jdir
            .open_dir(Self::THEME_MANAGER_DIR_NAME)
            .into_diagnostic()
            .wrap_err("failed to open theme_manager dir")?
            .into();
        dir.create_dir_all(Self::THEMES_DIR_NAME)
            .into_diagnostic()
            .wrap_err("failed to create themes dir")?;
        let themes_dir: Arc<Dir> = dir
            .open_dir(Self::THEMES_DIR_NAME)
            .into_diagnostic()
            .wrap_err("failed to open themes dir")?
            .into();

        dir.create_dir_all(Self::FONTS_DIR_NAME)
            .into_diagnostic()
            .wrap_err("failed to create themes dir")?;
        let fonts_dir: Arc<Dir> = dir
            .open_dir(Self::FONTS_DIR_NAME)
            .into_diagnostic()
            .wrap_err("failed to open themes dir")?
            .into();
        if !fonts_dir.exists(&format!("{}.ttf", Self::DEFAULT_FONT_NAME)) {
            fonts_dir
                .write(
                    format!("{}.ttf", Self::DEFAULT_FONT_NAME),
                    include_bytes!("roboto.ttf"),
                )
                .into_diagnostic()
                .wrap_err("failed to write roboto/default font file to fonts dir")?;
        }
        if !themes_dir.exists(&format!("{}.json", Self::DEFAULT_THEME_NAME)) {
            themes_dir
                .write(
                    format!("{}.json", Self::DEFAULT_THEME_NAME),
                    serde_json::to_string_pretty(&Theme::default())
                        .into_diagnostic()
                        .wrap_err("failed to serialize egui style")?
                        .as_bytes(),
                )
                .into_diagnostic()
                .wrap_err("failed to write default theme file to themes dir")?;
        }
        let mut themes = BTreeMap::default();
        for entry in themes_dir
            .entries()
            .into_diagnostic()
            .wrap_err("failed to read themes dir entries")?
        {
            let entry = entry
                .into_diagnostic()
                .wrap_err("failed to read theme entry in themes dir")?;
            if entry
                .file_type()
                .into_diagnostic()
                .wrap_err("failed to get file tyep of theme dir entry")?
                .is_file()
            {
                let theme_name = entry
                    .file_name()
                    .into_diagnostic()?
                    .trim_end_matches(".json")
                    .to_string();
                let mut theme_json = String::new();
                entry
                    .open()
                    .into_diagnostic()
                    .wrap_err("failed to open theme file")?
                    .read_to_string(&mut theme_json)
                    .into_diagnostic()
                    .wrap_err("failed to read json from theme file")?;
                let theme: Theme = serde_json::from_str(&theme_json)
                    .into_diagnostic()
                    .wrap_err_with(|| format!("failed to deserialize theme: {theme_name}"))?;
                themes.insert(theme_name, theme);
            }
        }
        let mut fonts = BTreeMap::default();
        for entry in fonts_dir
            .entries()
            .into_diagnostic()
            .wrap_err("failed to read themes dir entries")?
        {
            let entry = entry
                .into_diagnostic()
                .wrap_err("failed to read theme entry in themes dir")?;
            if entry
                .file_type()
                .into_diagnostic()
                .wrap_err("failed to get file tyep of theme dir entry")?
                .is_file()
            {
                let theme_name = entry
                    .file_name()
                    .into_diagnostic()?
                    .trim_end_matches(".ttf")
                    .trim_end_matches(".otf")
                    .to_string();

                let mut font_bytes = Vec::new();
                entry
                    .open()
                    .into_diagnostic()
                    .wrap_err("failed to open theme file")?
                    .read_to_end(&mut font_bytes)
                    .into_diagnostic()
                    .wrap_err("failed to read json from theme file")?;

                fonts.insert(theme_name, font_bytes);
            }
        }
        if !dir.exists(format!("{}.json", Self::THEME_MANAGER_CONFIG_NAME)) {
            dir.write(
                format!("{}.json", Self::THEME_MANAGER_CONFIG_NAME),
                serde_json::to_vec_pretty(&ThemeManagerConfig::default())
                    .into_diagnostic()
                    .wrap_err("failed to serialize theme manager config")?,
            )
            .into_diagnostic()
            .wrap_err("failed to write theme manager config to the theme manager dir")?;
        }
        let config = serde_json::from_str(
            &dir.read_to_string(format!("{}.json", Self::THEME_MANAGER_CONFIG_NAME))
                .into_diagnostic()
                .wrap_err("failed to read theme manager config file")?,
        )
        .into_diagnostic()
        .wrap_err("failed to deserialize theme manager config file")?;
        Ok(Self {
            dir,
            themes_dir,
            fonts_dir,
            themes,
            fonts,
            config,
            ui_data: Default::default(),
        })
    }
    pub fn init_egui(&mut self, etx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        for (name, font_data) in self.fonts.iter() {
            fonts.font_data.insert(
                name.to_owned(),
                egui::FontData::from_owned(font_data.to_owned()),
            );
        }
        etx.set_fonts(fonts);
        if let Some(theme) = self.themes.get(&self.config.default_theme) {
            etx.set_style(theme.style.clone());
        } else {
            error!(%self.config.default_theme, "failed to find the default theme in the loaded themes :(");
        }
    }
    pub fn gui(&mut self, etx: &egui::Context, open: &mut bool) {
        egui::Window::new("Theme Manager")
            .open(open)
            .scroll2([false, true])
            .show(etx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.ui_data.tab,
                        ThemeUITab::LiveEditor,
                        "Live Editor",
                    );
                    ui.selectable_value(&mut self.ui_data.tab, ThemeUITab::Config, "Configuration");
                });
                match self.ui_data.tab {
                    ThemeUITab::LiveEditor => {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("current theme name: ");
                                ui.text_edit_singleline(&mut self.ui_data.theme_name);
                            });
                            if ui
                                .button("save")
                                .on_hover_text("save this theme with the above name")
                                .clicked()
                            {
                                let style = etx.style().as_ref().clone();
                                let theme = Theme { style };
                                let theme_name = self.ui_data.theme_name.clone();
                                match serde_json::to_string_pretty(&theme) {
                                    Ok(theme_json) => {
                                        match self.themes_dir.try_clone() {
                                            Ok(themes_dir) => {
                                                let theme_name = theme_name.clone();
                                                rayon::spawn(move || {
                                                    match themes_dir.write(
                                                        format!("{theme_name}.json"),
                                                        theme_json.as_bytes(),
                                                    ) {
                                                        Ok(_) => {
                                                            tracing::info!(
                                                                notify = 3.0f64,
                                                                "saved theme {theme_name} to themes directory"
                                                            );
                                                        }
                                                        Err(e) => {
                                                            error!(?e, "failed to save theme to directory:(");
                                                        }
                                                    }
                                                });
                                            },
                                            Err(e) => {
                                                error!(?e, "failed to clone themes dir to save theme");
                                            },
                                        }
                                    }
                                    Err(e) => {
                                        error!(?e, "failed to serialize theme to json :(");
                                    }
                                }
                                self.themes.insert(theme_name, theme);
                            }
                            etx.style_ui(ui);
                        });
                    }
                    ThemeUITab::Config => {
                        ui.group(|ui| {
                            ui.heading("Theme Manger Config");
                            egui::Grid::new("theme manager config")
                                .num_columns(2)
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.label("default theme: ");
                                    egui::ComboBox::new("default theme", "default theme")
                                        .selected_text(&self.config.default_theme)
                                        .show_ui(ui, |ui| {
                                            for theme_name in self.themes.keys() {
                                                let checked =
                                                    theme_name == &self.config.default_theme;
                                                if ui
                                                    .selectable_label(checked, theme_name)
                                                    .clicked()
                                                    && !checked
                                                {
                                                    self.config.default_theme = theme_name.clone();
                                                }
                                            }
                                        });
                                    ui.end_row();
                                    ui.label("current theme: ");
                                    egui::ComboBox::new("default theme", "default theme")
                                        .selected_text(&self.ui_data.current_theme_name)
                                        .show_ui(ui, |ui| {
                                            for (theme_name, theme) in self.themes.iter() {
                                                let checked =
                                                    theme_name == &self.config.default_theme;
                                                if ui
                                                    .selectable_label(checked, theme_name)
                                                    .clicked()
                                                    && !checked
                                                {
                                                    etx.set_style(theme.style.clone());
                                                }
                                            }
                                        });
                                    ui.end_row();
                                });
                            if ui.button("save config").clicked() {
                                match serde_json::to_string_pretty(&self.config) {
                                    Ok(config_json) => {
                                        match self.dir.try_clone() {
                                            Ok(theme_manager_dir) => {
                                                rayon::spawn(move || {
                                                    match
                                                    theme_manager_dir.write(Self::THEME_MANAGER_CONFIG_NAME, config_json.as_bytes()) {
                                                        Ok(_) => {
                                                            info!(notify = 5.0f64, "saved theme manager configuration");
                                                        },
                                                        Err(e) => {
                                                            error!(?e, "failed to save theme manager config");
                                                        },
                                                    }
                                                });
                                            },
                                            Err(e) => {
                                                error!(?e, "failed to clone theme manager directory to save config");
                                            },
                                        }
                                    },
                                    Err(e) => {
                                        error!(?e, "failed to serialize theme config");
                                    },
                                }
                            }
                            if ui.button("import font").clicked() {
                                match self.fonts_dir.try_clone() {
                                    Ok(fonts_dir) => {
                                        rayon::spawn(move || {
                                            if let Some(font_path) =  rfd::FileDialog::default().add_filter("fonts", &["ttf"])
                                            .pick_file()
                                            {
                                                match std::fs::read(&font_path) {
                                                    Ok(font_data) => {
                                                        match font_path.file_name().and_then(std::ffi::OsStr::to_str) {
                                                            Some(name) => {
                                                                if name.ends_with("ttf") {

                                                                    match fonts_dir.write(name, font_data) {
                                                                        Ok(_) => {
                                                                            info!(notify = 5.0f64, name, "saved font");
                                                                        },
                                                                        Err(e) => {
                                                                            error!(?e, name, "failed to save font");
                                                                        },
                                                                    }
                                                                } else {
                                                                    error!(name, "only ttf font files are supported");
                                                                }
                                                            },
                                                            None => {
                                                                error!(?font_path, "invalid file name");
                                                            },
                                                        }
                                                    },
                                                    Err(e) => {
                                                        error!(?e, ?font_path, "failed to read font");
                                                    },
                                                }
                                            }
                                        });
                                    },
                                    Err(e) => {
                                        error!(?e, "failed to clone fonts directory to import font");
                                    },
                                }
                            }
                        });
                    }
                }
            });
    }
}
