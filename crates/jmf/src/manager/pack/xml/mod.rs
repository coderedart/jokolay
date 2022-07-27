//! conversion from XML to JSON pack:
//! 1. filesyste&m
//!     1. open the zip file and iterate through all entries. if filesystem, use walkdir
//!     2. collect all the files entries into a hashmap with `PathBuf` relative to pack root folder
//!         as keys and file contents `Vec<u8>` as values  
//!     3. we log / deal with any kind of filesystem errors before this step. we will probably return if we get any errors
//! 2. Parsing
//!     1. get the extension of the file entry. skip the entries without xml, png or trl extensions.
//!     2. convert paths to unique names. we store relative Paths of textures/trls and their new names
//!         in a `HashMap<String, String>`. key is lowercase relative path, and value is the new name
//!         in json pack img/trl entries.
//!         1. extract the file_stem (file name without extension) portion from `Path`
//!             skip if file_stem doesn't exist.
//!         2. convert String lossy (utf-8) and lowercase it.
//!         3. if name already taken, add a number. repeat until we get a new name. insert it into
//!             the pack img/trl entries and names map.
//!     2. if png extension, try to deserialize `Vec<u8>` using `texture` crate just to check that its valid texture.
//!         if error skip the texture entry and log error.
//!     3. if trl extension, keep a `HashMap<String, (u16, [f32;3], String)>`, along with new name,
//!         insert mapid and the first node position. translate all nodes into model space and insert
//!         into jsonpack. if error, skip the trl entry and log error.
//!     4. if xml extension, filter with `rapid_filter`. then, parse into a elementtree Element.
//!         if error skip the xml entry, and log error.
//!     5. finally, you should have the following
//!         1. `HashMap<String, String>` for texture relative paths to new names.
//!         2. `HashMap<String, (u16, [f32; 3], String)>` for trl relative paths to (mapid, position, new name)
//!         3. json pack's texture and trl entries.
//!         4. Element doms of xml files.
//! 3. Deserialization
//!     1. iterate through all MarkerCategory Tags in all Elements. keep a
//!         HashMap<String, (u16, MarkerTemplate)>. recurse (with MarkerCateogry tree and jsonpack cattree)
//!         and build up the full name of the MC.
//!         if path doesn't exist in template, create one.
//!         extract `display_name`, `is_separator`, `default_toggle` and set them in `CategoryMenu`.
//!         Deserialize the `MarkerTemplate`  and
//!         recurse the children of `MarkerCategory` with the json category's children.
//!     2. iterate through all POIs. if you find a Marker, extract the template, xyz pos, mapID, type.
//!         use type to get cat id from previous Map in step 3.1. inherit all props. convert all attributes properly
//!         and insert it into the json pack in the  appropriate mapid.json. use the Map from 2.2 to get
//!         texture name to use in template.
//!     3. repeat the same for Trail. but also use trail_file path to get MapID and trail position
//!         from the trl file in step 2.3.
//!     4. all xml related semantic errors like invalid values or such must be logged by this point
//!     5. most of the errrors in this pack can be logged and safely ignored. best effort basis is enough.

mod template;

use super::trail::Trail;
use super::Pack;

use crate::manager::pack::marker::Marker;
use crate::manager::pack::xml::template::MarkerTemplate;
use crate::manager::pack::Trl;
use crate::rapid_filter_rust;

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::{eyre, Context, ContextCompat};
use color_eyre::Result;
use elementtree::{Children, Element};
use std::collections::HashMap;

use std::io::Read;

use std::sync::Arc;
use tracing::error;

/// The function takes a zipfile, and tries to parse a Marker Pack out of it.
/// Arguments:
///     * taco: must be a valid zip file. any errors while parsing the zipfile will result in error
///
/// any other errors like invalid attributes or missing markers etc.. will just be logged and ignored.
/// the intention is "best effort" parsing and not "validating" xml marker packs.
/// although, if it works in `Taco` or `Blish`, it should work here too.   
pub fn get_pack_from_taco(taco: &Vec<u8>) -> Result<(Pack, Failures)> {
    let entries = read_files_from_zip(taco).wrap_err("failed to read files from zip file")?;
    let pack = parse_entries(entries);

    Ok(pack)
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("zip file had an error: {0}")]
    ZipError(color_eyre::Report),
}

#[derive(Debug, Default)]
pub struct Failures {
    pub errors: Vec<FailureError>,
    pub warnings: Vec<FailureWarning>,
}

#[derive(Debug, thiserror::Error)]
pub enum FailureError {
    #[error("Duplicate File: {0}\n")]
    DuplicateFile(Arc<Utf8Path>),
    #[error("texture File Error:\nfile: {0}\nerror: {1}")]
    ImgError(Arc<Utf8Path>, image::ImageError),
    #[error("No Name for file: {0}\n")]
    NoNameFile(Arc<Utf8Path>),
    #[error("new name limit reached Error: {0}")]
    NewNameLimitReached(Arc<Utf8Path>),
    #[error("xml file doesn't contain OverlayData tag: {0}")]
    NoOverlayData(Arc<Utf8Path>),
    #[error("Trl File Error:\nfile: {0}\nerror: {1}")]
    TrlError(Arc<Utf8Path>, TrlError),
    #[error("utf-8 error:\n file: {0}\n error: {1}")]
    Utf8Error(Arc<Utf8Path>, std::string::FromUtf8Error),
    #[error("invalid xml:\n file: {0}\n error: {1}")]
    XmlParseError(Arc<Utf8Path>, elementtree::Error),
}
#[derive(Debug, thiserror::Error)]
pub enum FailureWarning {
    #[error("category doesn't have a name: {0}")]
    CategoryNameMissing(Arc<Utf8Path>, String),
    #[error("file doesn't have an extension: {0}")]
    ExtensionLessFile(Arc<Utf8Path>),
    #[error("file extension must be xml / png / trl : {0}")]
    InvalidExtensionFile(Arc<Utf8Path>),
    #[error("category number {2} with parent '{1}' in file {0}. warning: {3}")]
    CategoryWarnings(Arc<Utf8Path>, Arc<str>, usize, CategoryWarning),
    #[error("category number {2} with parent '{1}' in file {0}. warning: {3}")]
    POITrailWarnings(Arc<Utf8Path>, String, usize, POITrailWarning),

    #[error("category number {2} with parent '{1}' in file {0}. warning: {3}")]
    MarkerWarnings(Arc<Utf8Path>, String, usize, MarkerWarning),

    #[error("category number {2} with parent '{1}' in file {0}. warning: {3}")]
    TrailWarnings(Arc<Utf8Path>, String, usize, TrailWarning),
}
#[derive(Debug, thiserror::Error)]
pub enum MarkerWarning {
    #[error("missing map_Id for Marker")]
    MissingMapID,
}
#[derive(Debug, thiserror::Error)]
pub enum TrailWarning {
    #[error("missing map_Id for Trail")]
    MissingMapID,
}
#[derive(Debug, thiserror::Error)]
pub enum POITrailWarning {
    #[error("missing category for POI/Trail")]
    MissingCategory,
}
#[derive(Debug, thiserror::Error)]
pub enum CategoryWarning {
    #[error("missing_name_attr")]
    CategoryNameMissing,
}
#[derive(Debug, thiserror::Error)]
pub enum TrlError {
    #[error("trl file has invalid map_id: {0}")]
    InvalidMapID(u32),
    #[error("trl invalid size: {0}")]
    InvalidLength(usize),
}
/// parses the given `Vec<u8>` as a zipfile and reads all the files into Vec<u8>.
/// returns a map with file paths as keys and contents as values.
fn read_files_from_zip(taco: &Vec<u8>) -> Result<HashMap<Arc<Utf8Path>, Vec<u8>>> {
    // get zip file
    let mut zip_file =
        zip::ZipArchive::new(std::io::Cursor::new(taco)).wrap_err("invalid zip file")?;
    let mut entries = HashMap::default();
    // for each entry in zip filea
    for index in 0..zip_file.len() {
        // get the entry from zip file. return if we can't find it
        let mut file = zip_file
            .by_index(index)
            .wrap_err("failed to find index inside zip file")?;
        // ignore if directory. skip to next entry
        if file.is_dir() {
            continue;
        }
        // if it has invalid pathbuf, return
        let file_path = file
            .enclosed_name()
            .wrap_err("taco has a file without enclosed name")?
            .to_path_buf();
        let file_path = Utf8PathBuf::from_path_buf(file_path).map_err(|e| {
            eyre!("failed to create Arc<Utf8Path> from PathBuf. non-utf8 path encountered: {e:?}")
        })?;
        let file_path = Arc::from(file_path);
        let mut file_content = vec![];
        // read the contents. return with error
        file.read_to_end(&mut file_content)
            .wrap_err("failed to read file contents inside zip")?;
        // check that the path is unique and we didn't insert one previously. if it isn't unique, return error
        if entries.insert(file_path, file_content).is_some() {
            return Err(eyre!("duplicate entries in zip file"));
        };
    }
    Ok(entries)
}

#[derive(Default)]
struct ParsedEntries {
    texture_entries: HashMap<String, String>,
    trl_entries: HashMap<String, (u16, String)>,
    elements: HashMap<Arc<Utf8Path>, Element>,
}
fn parse_entries(entries: HashMap<Arc<Utf8Path>, Vec<u8>>) -> (Pack, Failures) {
    let mut parsed_entries: ParsedEntries = Default::default();
    let mut pack = Pack::default();
    let mut failures = Failures::default();
    for (entry_path, entry_contents) in entries {
        match entry_path.extension() {
            None => {
                failures
                    .warnings
                    .push(FailureWarning::ExtensionLessFile(entry_path));
                continue;
            }
            Some("xml") => {
                let xml = match String::from_utf8(entry_contents) {
                    Ok(s) => s,
                    Err(e) => {
                        failures.errors.push(FailureError::Utf8Error(entry_path, e));
                        continue;
                    }
                };
                let xml = rapid_filter_rust(xml);
                let element = match Element::from_reader(xml.as_bytes()) {
                    Ok(e) => e,
                    Err(e) => {
                        failures
                            .errors
                            .push(FailureError::XmlParseError(entry_path, e));
                        continue;
                    }
                };
                if parsed_entries
                    .elements
                    .insert(entry_path.clone(), element)
                    .is_some()
                {
                    failures
                        .errors
                        .push(FailureError::DuplicateFile(entry_path))
                }
            }
            Some("trl") => {
                let content_length = entry_contents.len();
                // content_length must be atleast 8 to contain version + map_id
                // and the remaining length must be a multiple of 12 bytes (size of vec3) to be valid series of position nodes
                if content_length < 8 || ((content_length - 8) % 12) != 0 {
                    failures.errors.push(FailureError::TrlError(
                        entry_path,
                        TrlError::InvalidLength(content_length),
                    ));
                    continue;
                }

                let mut version_bytes = [0_u8; 4];
                version_bytes.copy_from_slice(&entry_contents[4..8]);
                let _version = u32::from_ne_bytes(version_bytes);
                let mut map_id_bytes = [0_u8; 4];
                map_id_bytes.copy_from_slice(&entry_contents[4..8]);
                let map_id = u32::from_ne_bytes(map_id_bytes);
                let map_id = match map_id.try_into() {
                    Ok(map_id) => map_id,
                    Err(_e) => {
                        failures.errors.push(FailureError::TrlError(
                            entry_path,
                            TrlError::InvalidMapID(map_id),
                        ));
                        continue;
                    }
                };
                // because we already checked before that the len of the slice is divisible by 12
                // this will either be empty vec or series of vec3s.
                let nodes: Vec<[f32; 3]> = entry_contents[8..]
                    .chunks_exact(12)
                    .map(|float_bytes| {
                        // make [f32 ;3] out of those 12 bytes
                        let arr = [
                            f32::from_le_bytes([
                                // first float
                                float_bytes[0],
                                float_bytes[1],
                                float_bytes[2],
                                float_bytes[3],
                            ]),
                            f32::from_le_bytes([
                                // second float
                                float_bytes[4],
                                float_bytes[5],
                                float_bytes[6],
                                float_bytes[7],
                            ]),
                            f32::from_le_bytes([
                                // third float
                                float_bytes[8],
                                float_bytes[9],
                                float_bytes[10],
                                float_bytes[11],
                            ]),
                        ];
                        arr
                    })
                    .collect();
                let name = match entry_path.file_stem() {
                    Some(s) => s.to_lowercase(),
                    None => {
                        failures.errors.push(FailureError::NoNameFile(entry_path));
                        continue;
                    }
                };

                let name = if pack.trls.contains_key(&name) {
                    let mut new_name = name.clone();
                    let mut count = 0;
                    for number in 0..=u16::MAX {
                        new_name = format!("{name}{number}");

                        if !pack.trls.contains_key(&new_name) {
                            break;
                        }
                        count = number;
                    }
                    if count == u16::MAX {
                        failures
                            .errors
                            .push(FailureError::NewNameLimitReached(entry_path));
                        continue;
                    }
                    new_name
                } else {
                    name
                };

                let lower_case_path = entry_path.as_str().to_lowercase();

                if pack
                    .trls
                    .insert(name.clone(), Trl::new(map_id, 2, nodes))
                    .is_some()
                {
                    failures
                        .errors
                        .push(FailureError::DuplicateFile(entry_path));
                    panic!("should be unreachable");
                }
                assert!(parsed_entries
                    .trl_entries
                    .insert(lower_case_path, (map_id, name.clone()))
                    .is_none());
            }
            Some("png") => {
                let name = match entry_path.file_stem() {
                    Some(s) => s.to_lowercase(),
                    None => {
                        failures.errors.push(FailureError::NoNameFile(entry_path));
                        continue;
                    }
                };
                match image::load_from_memory(&entry_contents) {
                    Ok(_) => {}
                    Err(e) => {
                        failures.errors.push(FailureError::ImgError(entry_path, e));
                        continue;
                    }
                }
                let name = if pack.textures.contains_key(&name) {
                    let mut new_name = name.clone();
                    let mut count = 0;
                    for number in 0..=u16::MAX {
                        new_name = format!("{name}{number}");

                        if !pack.textures.contains_key(&new_name) {
                            break;
                        }
                        count = number;
                    }
                    if count == u16::MAX {
                        failures
                            .errors
                            .push(FailureError::NewNameLimitReached(entry_path));
                        continue;
                    }
                    new_name
                } else {
                    name
                };
                let lower_case_path = entry_path.as_str().to_lowercase();

                if pack.textures.insert(name.clone(), entry_contents).is_some() {
                    failures
                        .errors
                        .push(FailureError::DuplicateFile(entry_path));
                    panic!("should be unreachable");
                }
                assert!(parsed_entries
                    .texture_entries
                    .insert(lower_case_path, name.clone())
                    .is_none());
            }
            Some(_) => {
                failures
                    .warnings
                    .push(FailureWarning::InvalidExtensionFile(entry_path));
                continue;
            }
        }
    }
    let mut templates: HashMap<String, MarkerTemplate> = HashMap::new();
    for (path, ele) in parsed_entries.elements.iter() {
        if "OverlayData" == ele.tag().name() {
            struct State<'a> {
                children: Children<'a>,
                parent_name: Arc<str>,
                template: Arc<MarkerTemplate>,
                index: usize,
            }

            let mut stack = vec![State {
                children: ele.children(),
                parent_name: Arc::from(""),
                template: Arc::new(MarkerTemplate::default()),
                index: 0,
            }];
            let mut children_to_push = None;
            loop {
                if let Some(children_to_push) = children_to_push.take() {
                    stack.push(children_to_push);
                }
                match stack.last_mut() {
                    Some(top_state) => match top_state.children.next() {
                        Some(category_element) => {
                            if category_element.tag().name() != "MarkerCategory" {
                                continue;
                            }
                            let name = category_element.get_attr("name").unwrap_or_default();
                            if name.is_empty() {
                                failures.warnings.push(FailureWarning::CategoryWarnings(
                                    path.clone(),
                                    top_state.parent_name.clone(),
                                    top_state.index,
                                    CategoryWarning::CategoryNameMissing,
                                ));
                                continue;
                            }
                            let full_name = if top_state.parent_name.is_empty() {
                                name.to_string()
                            } else {
                                format!("{}.{}", top_state.parent_name, name)
                            };
                            let full_name = full_name.to_lowercase();

                            let cat_path = Utf8PathBuf::from_iter(full_name.split("."));
                            let template = templates.entry(full_name.clone()).or_default();
                            template.update_from_element(category_element);
                            template.inherit_from_template(&top_state.template);
                            let display_name =
                                category_element.get_attr("displayName").unwrap_or_default();

                            let is_separator = category_element
                                .get_attr("isSeparator")
                                .unwrap_or_default()
                                .parse()
                                .unwrap_or_default();
                            let default_toggle = category_element
                                .get_attr("defaulttoggle")
                                .unwrap_or_default()
                                .parse()
                                .unwrap_or(true);
                            pack.category_menu.create_category(&cat_path);
                            pack.category_menu.set_is_separator(&cat_path, is_separator);
                            pack.category_menu
                                .set_default_toggle(&cat_path, default_toggle);
                            pack.category_menu
                                .set_display_name(&cat_path, display_name.to_string());
                            let parent_name = Arc::from(full_name.as_str());
                            children_to_push = Some(State {
                                children: category_element.children(),
                                template: Arc::new(template.clone()),
                                parent_name,
                                index: 0,
                            });
                            top_state.index += 1;
                        }
                        None => {
                            stack.pop();
                        }
                    },
                    None => break,
                }
            }
        } else {
            failures
                .errors
                .push(FailureError::NoOverlayData(path.clone()));
        }
    }

    for (path, ele) in parsed_entries.elements.iter() {
        if "OverlayData" == ele.tag().name() {
            if let Some(pois) = ele.children().find(|e| e.tag().name() == "POIs") {
                for (poi_index, child) in pois.children().enumerate() {
                    // if type attribute exists, get the category id and the template. otherwise, skip this element.
                    let (cat_path, mut template) = if let Some(x) =
                        child.get_attr("type").and_then(|category_name| {
                            templates
                                .get(&category_name.to_lowercase())
                                .map(|template| {
                                    (
                                        Utf8PathBuf::from_iter(
                                            category_name.to_lowercase().split("."),
                                        ),
                                        template.clone(),
                                    )
                                })
                        }) {
                        x
                    } else {
                        failures.warnings.push(FailureWarning::POITrailWarnings(
                            path.clone(),
                            child.to_string().unwrap_or_default(),
                            poi_index,
                            POITrailWarning::MissingCategory,
                        ));
                        continue;
                    };

                    match child.tag().name() {
                        "POI" => {
                            if let Some(map_id) = child
                                .get_attr("mapID")
                                .and_then(|map_id| map_id.parse::<u16>().ok())
                            {
                                let xpos = child
                                    .get_attr("xpos")
                                    .unwrap_or_default()
                                    .parse()
                                    .unwrap_or_default();
                                let ypos = child
                                    .get_attr("ypos")
                                    .unwrap_or_default()
                                    .parse()
                                    .unwrap_or_default();
                                let zpos = child
                                    .get_attr("zpos")
                                    .unwrap_or_default()
                                    .parse()
                                    .unwrap_or_default();
                                template.update_from_element(child);

                                let mut marker = Marker {
                                    cat: cat_path,
                                    position: [xpos, ypos, zpos],
                                    ..Default::default()
                                };
                                marker.color = template.color;
                                if let Some(alpha) = template.alpha {
                                    marker.alpha = Some((alpha * 255.0) as u8);
                                }
                                marker.rotation = template.rotate;
                                if let Some(rotate_x) = template.rotate_x {
                                    let rotation = marker.rotation.get_or_insert([0.0f32; 3]);
                                    rotation[0] = rotate_x;
                                }
                                if let Some(rotate_y) = template.rotate_y {
                                    let rotation = marker.rotation.get_or_insert([0.0f32; 3]);
                                    rotation[1] = rotate_y;
                                }
                                if let Some(rotate_z) = template.rotate_z {
                                    let rotation = marker.rotation.get_or_insert([0.0f32; 3]);
                                    rotation[2] = rotate_z;
                                }

                                marker.scale = template.icon_size.map(|scale| [scale; 3]);

                                if let Some(texture) = template
                                    .icon_file
                                    .as_ref()
                                    .and_then(|texture| parsed_entries.texture_entries.get(texture))
                                {
                                    marker.texture = texture.clone();
                                }

                                marker.alpha = template.alpha.map(|a| (255.0 * a) as u8);
                                marker.position[1] += template.height_offset.unwrap_or_default();
                                pack.maps.entry(map_id).or_default().markers.push(marker);
                            } else {
                                failures.warnings.push(FailureWarning::MarkerWarnings(
                                    path.clone(),
                                    child.to_string().unwrap_or_default(),
                                    poi_index,
                                    MarkerWarning::MissingMapID,
                                ));
                            }
                        }
                        "Trail" => {
                            if let Some((map_id, trl_name)) = child
                                .get_attr("trailData")
                                .and_then(|trail_data| {
                                    parsed_entries.trl_entries.get(&trail_data.to_lowercase())
                                })
                                .cloned()
                            {
                                let mut trail = Trail {
                                    cat: cat_path,
                                    trl: trl_name,
                                    ..Default::default()
                                };
                                template.update_from_element(child);
                                trail.color = template.color;
                                if let Some(alpha) = template.alpha {
                                    trail.alpha = Some((alpha * 255.0) as u8);
                                }

                                if let Some(texture) = template
                                    .texture
                                    .as_ref()
                                    .and_then(|texture| parsed_entries.texture_entries.get(texture))
                                {
                                    trail.texture = texture.clone();
                                }
                                pack.maps.entry(map_id).or_default().trails.push(trail);
                            } else {
                                failures.warnings.push(FailureWarning::TrailWarnings(
                                    path.clone(),
                                    child.to_string().unwrap_or_default(),
                                    poi_index,
                                    TrailWarning::MissingMapID,
                                ));
                            }
                        }
                        _rest => {
                            continue;
                        }
                    }
                }
            }
        }
    }
    (pack, failures)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::manager::pack::category::CategoryMenu;
    use crate::manager::pack::{MARKER_PNG, TRAIL_PNG};
    use camino::Utf8Path;
    use test_log::test;

    use rstest::*;

    use similar_asserts::{assert_eq, assert_str_eq};
    use std::io::Write;

    use zip::write::FileOptions;
    use zip::ZipWriter;

    const TEST_XML: &str = include_str!("test.xml");

    #[fixture]
    #[once]
    fn make_taco() -> Vec<u8> {
        let mut writer = ZipWriter::new(std::io::Cursor::new(vec![]));
        writer
            .start_file("category.xml", FileOptions::default())
            .expect("failed to create category.xml");
        writer
            .write_all(TEST_XML.as_bytes())
            .expect("failed to write category.xml");
        writer
            .start_file("marker.png", FileOptions::default())
            .expect("failed to create marker.png");
        writer
            .write_all(MARKER_PNG)
            .expect("failed to write marker.png");
        writer
            .start_file("trail.png", FileOptions::default())
            .expect("failed to create trail.png");
        writer
            .write_all(TRAIL_PNG)
            .expect("failed to write trail.png");
        writer
            .start_file("basic.trl", FileOptions::default())
            .expect("failed to create basic trail");
        writer
            .write_all(&0u32.to_ne_bytes())
            .expect("failed to write version");
        writer
            .write_all(&15u32.to_ne_bytes())
            .expect("failed to write mapid ");
        writer
            .write_all(bytemuck::cast_slice(&[0f32; 3]))
            .expect("failed to write first node");
        writer
            .finish()
            .expect("failed to finalize zip")
            .into_inner()
    }

    #[rstest]
    fn test_read_entries_from_zip(make_taco: &Vec<u8>) {
        let file_entries = read_files_from_zip(make_taco).expect("failed to deserialize");
        assert_eq!(file_entries.len(), 4);
        let test_xml = std::str::from_utf8(
            file_entries
                .get(Utf8Path::new("category.xml"))
                .expect("failed to get category.xml"),
        )
        .expect("failed to get str from category.xml contents");
        assert_str_eq!(test_xml, TEST_XML);
        let test_marker_png = file_entries
            .get(Utf8Path::new("marker.png"))
            .expect("failed to get marker.png");
        assert_eq!(test_marker_png, MARKER_PNG);

        let test_trail_png = file_entries
            .get(Utf8Path::new("trail.png"))
            .expect("failed to get trail.png");
        assert_eq!(test_trail_png, TRAIL_PNG);
    }

    #[rstest]
    fn test_parse_entries(make_taco: &Vec<u8>) {
        let entries =
            read_files_from_zip(make_taco).expect("failed to read entries from make_taco");
        let (pack, _failures) = parse_entries(entries);
        // assert_eq!(parsed_entries.elements.len(), 1);
        // assert_eq!(parsed_entries.trl_entries.len(), 1);
        assert_eq!(pack.textures.len(), 2);
        // assert_eq!(parsed_entries.texture_entries.len(), pack.textures.len());

        assert_eq!(
            pack.textures
                .get("marker")
                .expect("failed to get marker.png from textures"),
            MARKER_PNG
        );
        assert_eq!(
            pack.textures
                .get("trail")
                .expect("failed to get trail.png from textures"),
            TRAIL_PNG
        );
        // let (map_id, first, _name) = parsed_entries
        //     .trl_entries
        //     .get("basic.trl")
        //     .expect("failed to get basic trail")
        //     .clone();
        // assert_eq!(map_id, 15);
        // assert_eq!(first, [0.0f32; 3]);
    }

    #[rstest]
    fn test_category_element(make_taco: &Vec<u8>) {
        let entries = read_files_from_zip(make_taco).expect("failed to get file entries from taco");
        let (pack, failures) = parse_entries(entries);
        assert!(failures.errors.is_empty());
        assert!(failures.warnings.is_empty());
        let mut test_category_menu = CategoryMenu::default();
        let parent_path = Utf8Path::new("parent");
        let child1_path = Utf8Path::new("parent/child1");
        let subchild_path = Utf8Path::new("parent/child1/subchild");
        let child2_path = Utf8Path::new("parent/child2");
        test_category_menu.create_category(subchild_path);
        test_category_menu.create_category(child2_path);
        test_category_menu.set_display_name(parent_path, "Parent".to_string());
        test_category_menu.set_display_name(child1_path, "Child 1".to_string());
        test_category_menu.set_display_name(subchild_path, "Sub Child".to_string());
        test_category_menu.set_display_name(child2_path, "Child 2".to_string());

        assert_eq!(test_category_menu, pack.category_menu)
    }
    #[rstest]
    fn test_deserialize_xml(make_taco: &Vec<u8>) {
        let entries = read_files_from_zip(make_taco).expect("failed to get entries from taco");
        let (pack, _failures) = parse_entries(entries);
        // let pack = deserialize_xml(pack, parsed_entries);
        // assert_str_eq!(
        //     pack.category_menu
        //         .get_category(0)
        //         .expect("failed to get category")
        //         .display_name,
        //     "Parent"
        // );
        // assert_str_eq!(
        //     pack.category_menu
        //         .get_category(1)
        //         .expect("failed to get category")
        //         .display_name,
        //     "Child 1"
        // );
        assert!(!pack.maps.is_empty());
    }
    #[rstest]
    fn test_get_pack_from_taco(make_taco: &Vec<u8>) {
        let (pack, _failures) =
            get_pack_from_taco(make_taco).expect("failed to get pack from taco");
        let qd = pack
            .maps
            .get(&15)
            .expect("failed to get queensdale mapdata");
        assert_eq!(
            qd.markers[0],
            Marker {
                cat: Utf8PathBuf::from("parent"),
                texture: "marker".to_string(),
                position: [1.0f32; 3],
                alpha: Some(127),
                ..Default::default()
            }
        );
    }
    #[rstest]
    fn check_alpha(make_taco: &Vec<u8>) {
        let (pack, _failures) =
            get_pack_from_taco(make_taco).expect("failed to get pack from taco");
        let qd = pack
            .maps
            .get(&15)
            .expect("failed to get queensdale mapdata");
        for marker in &qd.markers {
            assert_eq!(marker.alpha, Some(127));
        }
    }
}
