use std::{collections::BTreeMap, io::Read};

use cap_std::fs_utf8::Dir;
use egui::Style;
use miette::{Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use tracing::error;
pub struct ThemeManager {
    dir: Dir,
    themes_dir: Dir,
    fonts_dir: Dir,
    themes: BTreeMap<String, Theme>,
    fonts: BTreeMap<String, Vec<u8>>,
    config: ThemeManagerConfig,
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
        let dir = jdir
            .open_dir(Self::THEME_MANAGER_DIR_NAME)
            .into_diagnostic()
            .wrap_err("failed to open theme_manager dir")?;
        dir.create_dir_all(Self::THEMES_DIR_NAME)
            .into_diagnostic()
            .wrap_err("failed to create themes dir")?;
        let themes_dir = dir
            .open_dir(Self::THEMES_DIR_NAME)
            .into_diagnostic()
            .wrap_err("failed to open themes dir")?;

        dir.create_dir_all(Self::FONTS_DIR_NAME)
            .into_diagnostic()
            .wrap_err("failed to create themes dir")?;
        let fonts_dir = dir
            .open_dir(Self::FONTS_DIR_NAME)
            .into_diagnostic()
            .wrap_err("failed to open themes dir")?;
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
                &serde_json::to_vec_pretty(&ThemeManagerConfig::default())
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
    pub fn gui(&mut self, ui: &mut egui::Ui) {}
}
