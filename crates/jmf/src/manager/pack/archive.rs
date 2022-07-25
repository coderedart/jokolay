use crate::manager::pack::Pack;
use bytemuck::cast_slice;
use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use serde_json::to_writer_pretty;
use std::io::{Cursor, Write};
use zip::write::FileOptions;
use zip::ZipWriter;

impl Pack {
    pub fn create_zip(&self) -> Result<Vec<u8>> {
        let mut zip_writer = ZipWriter::new(Cursor::new(vec![]));
        zip_writer
            .start_file("cats.json", FileOptions::default())
            .wrap_err("failed to create cats.json file")?;
        to_writer_pretty(&mut zip_writer, &self.category_menu)
            .wrap_err("failed to write cats.json")?;

        zip_writer
            .add_directory("maps", FileOptions::default())
            .wrap_err("failed to create maps directory ")?;
        for (map_id, map_data) in self.maps.iter() {
            zip_writer
                .start_file(format!("maps/{map_id}.json"), FileOptions::default())
                .wrap_err_with(|| format!("failed to create mapdata file {map_id}.json"))?;
            to_writer_pretty(&mut zip_writer, map_data)
                .wrap_err_with(|| format!("failed to deserialize {map_id}.json"))?;
        }

        zip_writer
            .add_directory("textures", FileOptions::default())
            .wrap_err("failed to create textures directory ")?;
        for (texture_name, texture_data) in self.textures.iter() {
            zip_writer
                .start_file(
                    format!("textures/{texture_name}.png"),
                    FileOptions::default(),
                )
                .wrap_err_with(|| format!("failed to create png file {texture_name}.png"))?;
            zip_writer
                .write_all(texture_data)
                .wrap_err_with(|| format!("failed to write {texture_name}.png"))?;
        }

        zip_writer
            .add_directory("trls", FileOptions::default())
            .wrap_err("failed to create trls directory ")?;
        for (name, data) in self.trls.iter() {
            zip_writer
                .start_file(format!("trls/{name}.trl"), FileOptions::default())
                .wrap_err_with(|| format!("failed to create file {name}.trl"))?;
            zip_writer
                .write(data.map_id.to_ne_bytes().as_slice())
                .and(zip_writer.write(data.version.to_ne_bytes().as_slice()))
                .and(zip_writer.write(cast_slice(&data.nodes)))
                .wrap_err_with(|| format!("failed to write {name}.trl"))?;
        }
        Ok(zip_writer
            .finish()
            .wrap_err("failed to finalize json zip file")?
            .into_inner())
    }
}
