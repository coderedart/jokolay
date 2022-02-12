
use std::path::{Path, PathBuf};
use anyhow::Context;
use egui::Style;
use serde::{Serialize, Deserialize};

pub struct ThemeManager {
    pub list_of_themes: Vec<Theme>,
    pub theme_folder_path: PathBuf,
}
impl ThemeManager {
    pub fn new(theme_folder_path: PathBuf, default_theme_name: &str) -> anyhow::Result<Self> {
        let mut list_of_themes: Vec<Theme> = vec![];
        if !theme_folder_path.join(format!("{default_theme_name}.json")).exists() {
            let tf = std::fs::File::create(theme_folder_path.join(format!("{default_theme_name}.json")))?;
            serde_json::to_writer_pretty(std::io::BufWriter::new(tf), &Style::default())?;
        }
        for f in std::fs::read_dir(&theme_folder_path).context("failed to read entries of theme directory")? {
            let f = f.context("failed to get dir entry from theme dir")?;
            if f.file_type().context("failed to get filetype of a theme dir entry")?.is_file() {
                let p = f.path();
                let style: Style = serde_json::from_reader( std::io::BufReader::new(std::fs::File::open(&p)?)).context("failed to deserialize theme")?;
                list_of_themes.push(
                    Theme {
                        name: p.file_stem().context("invalid file name")?.to_string_lossy().to_string(),
                        style
                    }
                )
            }
        }
        Ok(Self {
            list_of_themes,
            theme_folder_path
        })
    }

}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub style: egui::Style,
}

impl Theme {
    pub fn save(&self, theme_folder_path: &Path) -> anyhow::Result<()> {
        let tf = std::fs::File::create(theme_folder_path.join(format!("{}.json", &self.name)))?;
        serde_json::to_writer_pretty(std::io::BufWriter::new(tf), &Style::default())?;
        Ok(())
    }
}