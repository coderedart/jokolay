use std::sync::Arc;

use miette::*;
use relative_path::{RelativePath, RelativePathBuf};
use thiserror::Error;

#[derive(Debug, thiserror::Error)]
#[error("These are errors per file")]
pub struct PerFileErrors {
    /// represents the file to which the errors belong to
    /// Some errors like zip parsing don't belong to any file, so we use None for that
    pub file_path: Option<Arc<RelativePath>>,
    /// pack errors when dealing with this file
    pub errors: Vec<PackError>,
    /// warnings when dealing with this file. Only applies to "xml" files as most warnings occur when extracting markers/trails/categories etc..
    pub warnings: Vec<PackWarning>,
}

#[derive(Diagnostic, Debug, Error)]
#[error("Pack warnings when dealing with zip, png, trl and xml files")]
#[diagnostic()]
pub enum PackError {
    #[error("failed to parse bytes into a valid Zip Archive")]
    #[diagnostic(code(pack_error::zip_error))]
    InvalidZip(#[from] zip::result::ZipError),
    #[error("invalid path. mangled name: {0:?}")]
    #[diagnostic(code(pack_error::invalid_path))]
    InvalidPath(std::path::PathBuf),
    #[error("non-utf8 path. path: {0}")]
    #[diagnostic(code(pack_error::non_relative_path))]
    NonRelativePath(String),
    #[error("failed to read file from zip. file: ")]
    #[diagnostic(code(pack_error::read_file_error))]
    FailedToReadFile,
    #[error("Duplicate File inside zip file")]
    #[diagnostic(code(pack_error::dup_file))]
    DuplicateFile,
    #[error("texture decode error")]
    #[diagnostic(code(pack_error::png_error))]
    ImgError(#[from] image::ImageError),
    #[error("No Name for file")]
    #[diagnostic(code(pack_error::file_without_name))]
    NoNameFile,
    #[error("file doesn't have an extension")]
    #[diagnostic(code(pack_error::file_without_ext))]
    ExtensionLessFile,
    #[error("file extension not recognized")]
    #[diagnostic(code(pack_error::file_invalid_extension))]
    InvalidExtensionFile,
    #[error("xml file doesn't contain OverlayData tag")]
    #[diagnostic(code(pack_error::overlay_data_tag_missing))]
    NoOverlayData,
    #[error("TBin parsing error")]
    #[diagnostic(code(pack_error::tbin_error))]
    TBinInvalid,
    #[error("utf-8 error in xml file: {0}")]
    #[diagnostic(code(pack_error::xml_utf_8))]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("XML parsing error: {0}")]
    #[diagnostic(code(pack_error::invalid_xml))]
    XmlParseError(#[from] xot::Error),
}

#[derive(Debug, Diagnostic, thiserror::Error)]
#[error("Pack warnings when parsing info from xml")]
#[diagnostic()]
pub enum PackWarning {
    #[error("category doesn't have a name: {0}")]
    #[diagnostic(code(pack_error::missing_category))]
    CategoryDoesNotExist(String),
    #[error("missing category attribute for POI/Trail")]
    MissingCategoryAttribute,
    #[error("missing texture attribute for POI/Trail")]
    MissingTextureAttribute,
    #[error("texture not found")]
    TextureNotFound(RelativePathBuf),
    #[error("GUID not found")]
    GUIDNotFound,
    #[error("missing map_Id for Marker")]
    MissingMapID,
    #[error("missing_name_attr")]
    CategoryNameMissing,
}
