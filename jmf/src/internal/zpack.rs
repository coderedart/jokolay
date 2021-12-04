use std::{io::Read, path::Path};
use tempfile::TempDir;
use tokio::io::AsyncWriteExt;
use zip::result::ZipError;

#[derive(Debug, thiserror::Error)]
pub enum ZipTempError {
    #[error("failed to create temporary directory")]
    TempDirCreationFailed(#[from] std::io::Error),
    #[error("failed to extract zip file")]
    ZipFileExtractionFailed(#[from] zip::result::ZipError),
}
/// though its async, we do some blocking calls because zip crate is sync only :(
pub async fn extract_zip_to_temp(zfile: &Path) -> Result<TempDir, ZipTempError> {
    let pack_dir = tempfile::tempdir()?;
    let zip_bytes = tokio::fs::read(zfile).await?;
    let cursor = std::io::Cursor::new(zip_bytes);
    let reader = std::io::BufReader::new(cursor);
    let mut archive = zip::read::ZipArchive::new(reader).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let filepath = file
            .enclosed_name()
            .ok_or(ZipError::InvalidArchive("Invalid file path"))?;

        let outpath = pack_dir.as_ref().join(filepath);

        if file.name().ends_with('/') {
            tokio::fs::create_dir_all(&outpath).await?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    tokio::fs::create_dir_all(&p).await?;
                }
            }
            let mut out_buffer = vec![];
            file.read_to_end(&mut out_buffer)?;
            let mut outfile = tokio::fs::File::create(&outpath).await?;
            outfile.write_all(&out_buffer).await?;
        }
    }
    Ok(pack_dir)
}
