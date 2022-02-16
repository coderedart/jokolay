use anyhow::Context;
use egui::{CollapsingHeader, FontDefinitions, FontFamily, Style, Window};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct ThemeManager {
    pub list_of_themes: HashMap<String, Theme>,
    pub active_theme: String,
    pub theme_folder_path: PathBuf,
    pub font_definitions: FontDefinitions,
}

impl ThemeManager {
    pub fn new(
        theme_folder_path: PathBuf,
        fonts_dir: PathBuf,
        default_theme_name: &str,
    ) -> anyhow::Result<Self> {
        assert!(!default_theme_name.is_empty());
        let mut font_definitions = egui::FontDefinitions::default();

        for f in
            std::fs::read_dir(&fonts_dir).context("failed to read prop fonts directory entries")?
        {
            let font_path = f.context("failed to get dir entry of fonts dir")?.path();
            let mut font_bytes = vec![];
            std::fs::File::open(&font_path)
                .with_context(|| format!("failed to open font file: {}", font_path.display()))?
                .read_to_end(&mut font_bytes)
                .with_context(|| {
                    format!("failed to read font file into Vec: {}", font_path.display())
                })?;
            let font_data = egui::FontData::from_owned(font_bytes);
            let name = font_path
                .file_stem()
                .with_context(|| {
                    format!(
                        "failed to get file stem of font file: {}",
                        font_path.display()
                    )
                })?
                .to_string_lossy()
                .to_string();
            font_definitions
                .font_data
                .entry(name.clone())
                .or_insert(font_data);
            font_definitions
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, name.clone());
            font_definitions
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, name);
        }
        let mut list_of_themes: HashMap<String, Theme> = HashMap::new();
        if !theme_folder_path
            .join(format!("{default_theme_name}.json"))
            .exists()
        {
            let tf = std::fs::File::create(
                theme_folder_path.join(format!("{default_theme_name}.json")),
            )?;
            serde_json::to_writer_pretty(
                std::io::BufWriter::new(tf),
                &Theme::new(Style::default(), font_definitions.families.clone()),
            )?;
        }
        for f in std::fs::read_dir(&theme_folder_path)
            .context("failed to read entries of theme directory")?
        {
            let f = f.context("failed to get dir entry from theme dir")?;
            if f.file_type()
                .context("failed to get filetype of a theme dir entry")?
                .is_file()
            {
                let p = f.path();
                let theme: Theme =
                    serde_json::from_reader(std::io::BufReader::new(std::fs::File::open(&p)?))
                        .context("failed to deserialize theme")?;
                list_of_themes.insert(
                    p.file_stem()
                        .context("invalid file name")?
                        .to_string_lossy()
                        .to_string(),
                    theme,
                );
            }
        }
        font_definitions.families = list_of_themes
            .get(default_theme_name)
            .context("no default theme, impossible")?
            .fonts_priority
            .clone();
        Ok(Self {
            list_of_themes,
            theme_folder_path,
            active_theme: default_theme_name.to_string(),
            font_definitions,
        })
    }
    pub fn get_current_theme(&self) -> anyhow::Result<&Theme> {
        self.list_of_themes
            .get(&self.active_theme)
            .context("could not find active theme, impossible")
    }
    pub fn gui(&mut self, ctx: egui::Context) -> anyhow::Result<()> {
        let mut delete_themes = vec![];
        let mut fonts_changed = false;
        let mut save = false;
        Window::new("Theme Manager").show(&ctx, |ui| {
            ui.label("changes need to be saved manually by pressing save button.\n you can set default theme in config manager window");
            // show list of themes, and allow to delete the theme unless its the current active theme
            CollapsingHeader::new("Themes List")
                .show(ui, |ui| {
                for (name, theme) in self.list_of_themes.iter_mut() {
                    ui.horizontal(|ui| {
                        ui.label(name);
                        if ui.button("activate").clicked() {
                            self.font_definitions.families = theme.fonts_priority.clone();

                            ctx.set_fonts(self.font_definitions.clone());
                            ctx.set_style(theme.style.clone());
                            self.active_theme = name.clone();
                            self.font_definitions.families = theme.fonts_priority.clone();
                        }
                        if ui.button("delete").clicked() {
                            delete_themes.push(name.clone());
                        }
                    });
                }
            });
            CollapsingHeader::new("Active Theme Editor").show(ui, |ui| {
                CollapsingHeader::new("Fonts in families")
                    .show(ui, |ui| {
                        ui.label("Proportional family is used for text in general.\n Monospace is used for Number edit boxes and such");
                        ui.label("here you select which fonts are allowed to be used in which families");
                        for name in self.font_definitions.font_data.keys() {
                            ui.horizontal(|ui| {
                                ui.label(name);
                                for (family, font_order) in self.font_definitions.families.iter_mut() {
                                    let mut in_family = font_order.contains(name);
                                    if ui.checkbox(&mut in_family, format!("{family:?}")).changed() {
                                        if in_family && !font_order.contains(name) {
                                            font_order.insert(0, name.clone());
                                            fonts_changed = true;
                                        } else {
                                            fonts_changed = true;
                                            font_order.retain(|n| n != name);
                                        }
                                    }
                                }
                            });
                        }
                    });

                CollapsingHeader::new("fonts priority")
                    .show(ui ,|ui| {
                        ui.label("which font should be preferred first in a family");
                        ui.label("if that font doesn't provide a character, we try from the next font");
                        for (family, priority) in self.font_definitions.families.iter_mut() {
                            CollapsingHeader::new(format!("{family:?} priority")).show(ui, |ui| {
                                let mut increment_priority = None;
                                let mut decrement_priority = None;
                                for (index, font_name) in priority.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(font_name);
                                        if index > 0 && ui.button("up").clicked() {
                                            increment_priority = Some(index);
                                        }
                                        if index < priority.len() - 1 && ui.button("down").clicked() {
                                            decrement_priority = Some(index);
                                        }
                                    });
                                }
                                if let Some(index) = increment_priority {
                                    assert!(index > 0);
                                    priority.swap(index, index - 1);
                                    fonts_changed = true;
                                }
                                if let Some(index) = decrement_priority {
                                    assert!(index < priority.len() - 1);
                                    priority.swap(index, index + 1);
                                    fonts_changed = true;
                                }
                            });
                        }
                    });

                ctx.style_ui(ui);
            });
            ui.horizontal(|ui| {
                ui.label("Theme Name: ");
                ui.text_edit_singleline(&mut self.active_theme);
            });
            if ui.button("save active theme to file").clicked() {
                save = true;
            }
        });

        if fonts_changed {
            ctx.set_fonts(self.font_definitions.clone());
        }
        for name in delete_themes {
            if name == self.active_theme {
                continue;
            }
            self.list_of_themes.remove(&name);
            let theme_path = self.theme_folder_path.join(format!("{name}.json"));
            if theme_path.exists() {
                std::fs::remove_file(&theme_path)
                    .with_context(|| format!("failed to delete theme. {}", theme_path.display()))?;
            }
        }
        if save {
            let t = Theme {
                fonts_priority: self.font_definitions.families.clone(),
                style: ctx.style().as_ref().clone()
            };
            t.save(&self.active_theme, &self.theme_folder_path)?;
             self.list_of_themes.insert(self.active_theme.clone(), t);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub fonts_priority: BTreeMap<FontFamily, Vec<String>>,
    pub style: egui::Style,
}

impl Theme {
    pub fn save(&self, name: &str, theme_folder_path: &Path) -> anyhow::Result<()> {
        let tf = std::fs::File::create(theme_folder_path.join(format!("{}.json", name)))?;
        serde_json::to_writer_pretty(std::io::BufWriter::new(tf), self)?;
        Ok(())
    }
    pub fn new(style: Style, fonts_priority: BTreeMap<FontFamily, Vec<String>>) -> Self {
        Self {
            style,
            fonts_priority,
        }
    }
}
