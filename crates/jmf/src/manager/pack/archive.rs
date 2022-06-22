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
            .add_directory("images", FileOptions::default())
            .wrap_err("failed to create images directory ")?;
        for (image_name, image_data) in self.images.iter() {
            zip_writer
                .start_file(format!("images/{image_name}.png"), FileOptions::default())
                .wrap_err_with(|| format!("failed to create png file {image_name}.png"))?;
            zip_writer
                .write_all(image_data)
                .wrap_err_with(|| format!("failed to write {image_name}.png"))?;
        }

        zip_writer
            .add_directory("tbins", FileOptions::default())
            .wrap_err("failed to create tbins directory ")?;
        for (name, data) in self.tbins.iter() {
            zip_writer
                .start_file(format!("tbins/{name}.tbin"), FileOptions::default())
                .wrap_err_with(|| format!("failed to create file {name}.tbin"))?;
            zip_writer
                .write_all(cast_slice(data))
                .wrap_err_with(|| format!("failed to write {name}.tbin"))?;
        }
        Ok(zip_writer
            .finish()
            .wrap_err("failed to finalize json zip file")?
            .into_inner())
    }
}
