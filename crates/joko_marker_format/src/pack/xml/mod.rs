//! When converting from XML packs, there's two kinds of failures:
//!     1. Errors: when we get an error, we just skip the whole "entity" that we are dealing with
//!         1. if png/xml/trl file has invalid contents. or if files / xml tags are not recognized etc..
//!         2. also, this might affect the deserialization of other elements. a missing trl (due to it being an invalid trl) can make lots of Trail tags be skipped that refer to this
//!     2. Warnings: when we can just ignore them, but are still not a good thing.
//!         1. it is still going to cause some data loss, but limited in scope.
//!         2. if markers / trails don't have a type attr, we don't know which category to place the markers into, so we skip
//!         3. Trail not referring to a valid trl (and thus no mapID) or markers not having a valid mapID
//!         4. we are skipping these because the trail or marker won't affect any other part of the pack, so its just a warning and not a hard error.
//!
//! NOTE: we refer to `trl` files as `tbin` (trail binary) files.
//!
//! conversion from XML to JSON pack:
//! 1. filesystem
//!     1. open the zip file and iterate through all entries. if filesystem, use walkdir
//!     2. collect all the files entries into a IndexMap with `RelativePathBuf` relative to pack root folder
//!         as keys and file contents `Vec<u8>` as values  
//!     3. we log / deal with any kind of filesystem errors before this step. we will probably stop and return if we get any errors at all.
//!     4. all file names must be valid Utf8
//! 2. separation
//!     1. get the extension of the file entry. skip the entries without xml, png or trl extensions.
//! 3. prepare:
//!     1. create an empty ZPack with a root category, the text "root" used for display name of the root category. also a bitvec to hold the status of a default_toggle (whether cat is enabled)
//!     2. we first check images, then tbin files to store them in the ZPack. but textures / tbins don't have a name in zpacks, just their indices.
//!         when we parse the xml files, the markers and categories will refer to trl files or textures by name. so, we keep some temporary structs
//!         which will map the names (paths) of textures, tbins, texts (strings), categories etc.. to their respective indices inside Zpack.
//!     3. these are the following temporary structs
//!         1. texture_indices
//!         2. tbin_indices: tbins also have mapIDs embeded. so, we also store mapID along with the index
//!         3. category_indices: in addition to the index, categories also have inheritable attributes (templates), so we store template along with the index
//!         4. text_indices
//! 4. texture
//!     1. iterate over the file entries with png extension
//!     2. load and verify that the texture is valid png. otherwise skip.
//!     3. push it into zpack.
//!     4. put the lowercased relative path of texture as key and index in the zpack as value into the indicies map.
//! 5. Trls
//!     1. make sure trl file is more than 8 bytes long (mapid and version bytes) and the rest is multiple of 12 bytes (vec3 position length) for it to be valid. otherwise skip.
//!     2. extract mapID, version, nodes (positions). otherwise skip.
//!     3. push into zpack
//!     4. insert into trl indices the newname AND mapID just like textures.
//! 6. Xmls:
//!     1. just extract the utf-8 string out of file bytes.
//!     2. filter the string with rapid_xml to remove some errors.
//!     3. parse into elements and store in entries with the file path (Arc<RelativePath>). skip if any errors.
//! 7. Categories:
//!     1. skip if root is not `OverlayData` tag.
//!     2. recursive parsing
//!     3. start with default values of root category as parent, empty "" as parent name, default template and children of OverlayData root tag.
//!     4. recurse
//!         1. iterate over children
//!         2. get name attr and get the full name using parent's name . skip if there's no name attr.
//!         3. get the category only elements like is_separator, default_toggle, display_name or use defaults.
//!         4. intern display_name. push the category into zpack with parent index, display_name index and is_separator.
//!         5. push the default_toggle bool into bitvec
//!         6. create a default template and inherit from this current category's xml node first by parsing (and recording any errors).
//!         7. then inherit attrs from parent template if its own attrs are not explicitly set from xml already.
//!         8. join this cat's name with parentname to get full name. use that as key and store template and cat index into the indices map.
//!         9. recurse for this category xml's children nodes using this category's new info like fullname, template, index for the children to use down the line.
//!     5. do this for ALL categories in ALL xml files before starting to parse POI or Trail tags.
//! 8. POI
//!     1. skip if not POI tag. maintain a index when enumerating for easy errors.
//!     2. get category from its type attr or skip. same for mapID. skip if you can't get them.
//!     3. get x/y/z pos or use defaults. same for
//!     4. clone category template from category indices map.
//!     5. inherit attributes from this marker's xml node.
//!     6. use those attributes to get texture index. if texture doesn't exist, skip.
//!     7. intern any of the string attributes and use those indices.
//!     8. insert marker into the relevant map data of zpack
//! 9. Trail
//!     1. just like POI, most of the steps are same.
//!     2. just get mapID / tbin index from the tbin indices map. skip if they don't exist.
//!     3. do the same stuff like templates, textures etc.. and insert into the relevant map
//!
//!
//! At present, we only care about a limited number of attributes:
//! 1. xpos,ypos,zpos of markers.
//! 2. traildatafile of trail
//! 3. texture or type (category) attrs of markers and trails
//! 4. name/displayname/issep/defaulttoggle of categories. and the above mentioned attributes as they might be inherited.
//!

use base64::Engine;

use glam::Vec3A;
use relative_path::{RelativePath, RelativePathBuf};

use indexmap::IndexMap;

use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Read;

use uuid::Uuid;
use xot::{Element, NameId, Node, Xot};

use std::sync::Arc;
// use tracing::error;

use crate::pack::{Category, CommonAttributes, Trail};

use super::{Marker, Pack};

#[derive(Debug, Default)]
pub struct Failures {
    pub errors: Vec<FailureError>,
    pub warnings: Vec<FailureWarning>,
}

/// This first parses all the files in a zipfile into the memory and then it will try to parse a zpack out of all the files.
/// will return error if there's an issue with zipfile.
///
/// but any other errors like invalid attributes or missing markers etc.. will just be logged into the Failures struct that is returned.
/// the intention is "best effort" parsing and not "validating" xml marker packs.
/// we will ignore any issues like unknown attributes or xml tags. "unknown" attributes means Any attributes that jokolay doesn't parse into Zpack.
///
/// Generally speaking, if a pack works in `Taco` or `BlishHUD`, it should work here too.
pub fn get_pack_from_taco_zip(taco: &[u8]) -> Result<(Pack, Failures), ZipParseError> {
    Ok(pack_from_file_entries_in_zip(read_file_entries_from_zip(
        taco,
    )?))
}

/// parses the given `Vec<u8>` as a zipfile and reads all the files into a Map with file paths as keys and file contents as Vec<u8>
/// will return error if there's any issues with the zip file or file names etc..
/// File paths will be converted to lowercase
fn read_file_entries_from_zip(
    taco: &[u8],
) -> std::result::Result<IndexMap<Arc<RelativePath>, Vec<u8>>, ZipParseError> {
    // parse zip file
    let mut zip_file = zip::ZipArchive::new(std::io::Cursor::new(taco))?;
    let mut entries = IndexMap::default();
    // for each entry in zip file
    for index in 0..zip_file.len() {
        // get the entry from zip file. return if we can't find it
        let mut file = zip_file.by_index(index)?;
        // ignore if directory. skip to next entry
        if file.is_dir() {
            continue;
        }
        let file_path = {
            // if it has invalid pathbuf, return
            let file_path = file
                .enclosed_name()
                .ok_or_else(|| ZipParseError::InvalidName(file.mangled_name()))?
                .to_str()
                .ok_or_else(|| ZipParseError::InvalidName(file.mangled_name()))?
                .to_lowercase();
            let file_path = RelativePathBuf::try_from(file_path.clone())
                .map_err(|_| ZipParseError::NonRelativePath(file_path))?;
            Arc::from(file_path)
        };

        let mut file_content = vec![];
        // read the contents. return with error
        file.read_to_end(&mut file_content)
            .map_err(|_| ZipParseError::FailedToReadFile(Arc::clone(&file_path)))?;
        // check that the path is unique and we didn't insert one previously. if it isn't unique, return error
        if entries
            .insert(Arc::clone(&file_path), file_content)
            .is_some()
        {
            return Err(ZipParseError::DuplicateEntry(Arc::clone(&file_path)));
        };
    }
    Ok(entries)
}
/// This is the main funtion that converts all the files in xml pack into a zpack
/// refer to the module docs for a rough step by step explanation of the process.
/// All the paths in the argument are assumed to be lowercase
fn pack_from_file_entries_in_zip(
    entries: IndexMap<Arc<RelativePath>, Vec<u8>>,
) -> (Pack, Failures) {
    // record of all the errors.
    // libraries should avoid panicking as much as possible.
    // some invalid marker pack should not bring down the whole library
    let mut failures = Failures::default();

    // all the contents of ZPack
    let mut pack = Pack::default();
    // we try to gather the xml entries in this map, and only deal with images/tbins first.
    let mut xml_entries: IndexMap<Arc<RelativePath>, String> = Default::default();
    // while parsing tbins, we store the map ids. Later, when parsing xml files,
    // we can sort the trails into their respective maps by using this map to look up the mapid from respective tbin
    let mut tbin_map_ids: HashMap<Arc<RelativePath>, u16> = HashMap::new();
    // delaying xml file parsing also allows us to detect if we are missing any images/tbins.
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
                let xml = crate::rapid_filter_rust(xml);
                // We will delay xml parsing for later. Because when sorting trails into different maps, we need to know the map id from the tbin.
                if xml_entries.insert(entry_path.clone(), xml).is_some() {
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
                let version = u32::from_ne_bytes(version_bytes);
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
                let nodes: Vec<Vec3A> = entry_contents[8..]
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

                        Vec3A::from_array(arr)
                    })
                    .collect();
                pack.tbins.insert(
                    entry_path.to_relative_path_buf(),
                    super::TBin {
                        map_id,
                        version,
                        nodes,
                    },
                );
                if tbin_map_ids.insert(entry_path.clone(), map_id).is_some() {
                    failures
                        .errors
                        .push(FailureError::DuplicateFile(entry_path.clone()));
                }
            }
            Some("png") => {
                match image::load_from_memory_with_format(&entry_contents, image::ImageFormat::Png)
                {
                    Ok(_) => {
                        pack.textures
                            .insert(entry_path.to_relative_path_buf(), entry_contents);
                    }
                    Err(e) => failures
                        .errors
                        .push(FailureError::ImgError(entry_path.clone(), e)),
                }
            }
            Some(_) => {
                failures
                    .warnings
                    .push(FailureWarning::InvalidExtensionFile(entry_path));
                continue;
            }
        }
    }
    const BASE64_ENGINE: base64::engine::GeneralPurpose = base64::engine::GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        base64::engine::GeneralPurposeConfig::new(),
    );
    for (entry_path, xml_entry) in xml_entries {
        let mut tree = Xot::new();
        let root_node = match tree.parse(&xml_entry) {
            Ok(root) => root,
            Err(e) => {
                failures
                    .errors
                    .push(FailureError::XmlParseError(entry_path.clone(), e));

                continue;
            }
        };
        let names = XotAttributeNameIDs::register_with_xot(&mut tree);
        // check if root tag is OverlayData
        if !tree
            .element(root_node)
            .map(|ele| (ele.name() == names.overlay_data))
            .unwrap_or_default()
        {
            failures
                .errors
                .push(FailureError::NoOverlayData(entry_path.clone()));
            continue;
        };

        // parse_categories
        recursive_marker_category_parser(
            &tree,
            tree.children(root_node),
            &mut failures,
            &entry_path,
            &mut pack.categories,
            &names,
        );

        if let Some(pois) = tree.children(root_node).find(|node| {
            tree.element(*node)
                .map(|ele| ele.name() == names.pois)
                .unwrap_or_default()
        }) {
            for (poi_index, child_node) in tree
                .children(pois)
                .filter(|node| tree.is_element(*node))
                .enumerate()
            {
                let child = tree.element(child_node).unwrap();
                let category = child
                    .get_attribute(names.category)
                    .unwrap_or_default()
                    .to_lowercase();
                if !pack.categories.contains_key(&category) {
                    failures.warnings.push(FailureWarning::CategoryNameMissing(
                        entry_path.clone(),
                        category.to_string(),
                    ));
                    continue;
                }
                let guid = child
                    .get_attribute(names.guid)
                    .and_then(|guid| {
                        let mut buffer = [0u8; 20];
                        BASE64_ENGINE
                            .decode_slice(guid, &mut buffer)
                            .ok()
                            .and_then(|_| Uuid::from_slice(&buffer[..16]).ok())
                    })
                    .unwrap_or_else(|| Uuid::new_v4());
                if child.name() == names.poi {
                    if let Some(map_id) = child
                        .get_attribute(names.map_id)
                        .and_then(|map_id| map_id.parse::<u16>().ok())
                    {
                        let xpos = child
                            .get_attribute(names.xpos)
                            .unwrap_or_default()
                            .parse::<f32>()
                            .unwrap_or_default();
                        let ypos = child
                            .get_attribute(names.ypos)
                            .unwrap_or_default()
                            .parse::<f32>()
                            .unwrap_or_default();
                        let zpos = child
                            .get_attribute(names.zpos)
                            .unwrap_or_default()
                            .parse::<f32>()
                            .unwrap_or_default();
                        let mut common_attributes = CommonAttributes::default();
                        update_from_element(&mut common_attributes, &child, &names);
                        if let Some(t) = common_attributes.icon_file.as_ref() {
                            if !pack.textures.contains_key(t) {
                                failures.warnings.push(FailureWarning::POITrailWarnings(
                                    entry_path.clone(),
                                    poi_index,
                                    POITrailWarning::TextureNotFound,
                                ));
                            }
                        }
                        let marker = Marker {
                            position: [xpos, ypos, zpos].into(),
                            map_id,
                            category,
                            props: common_attributes.into(),
                            guid,
                        };

                        pack.maps.entry(map_id).or_default().markers.push(marker);
                    } else {
                        failures.warnings.push(FailureWarning::MarkerWarnings(
                            entry_path.clone(),
                            poi_index,
                            MarkerWarning::MissingMapID,
                        ));
                    }
                } else if child.name() == names.trail {
                    if let Some(map_id) =
                        child
                            .get_attribute(names.trail_data)
                            .and_then(|trail_data| {
                                RelativePathBuf::try_from(trail_data.to_lowercase())
                                    .ok()
                                    .map(|t| pack.tbins.get(&t).map(|tb| tb.map_id))
                                    .flatten()
                            })
                    {
                        let mut common_attributes = CommonAttributes::default();
                        update_from_element(&mut common_attributes, &child, &names);

                        if let Some(tex) = common_attributes.texture.as_ref() {
                            if !pack.textures.contains_key(tex) {
                                failures.warnings.push(FailureWarning::POITrailWarnings(
                                    entry_path.clone(),
                                    poi_index,
                                    POITrailWarning::TextureNotFound,
                                ));
                            }
                        }

                        let trail = Trail {
                            map_id,
                            category,
                            props: common_attributes.into(),
                            guid,
                        };
                        pack.maps.entry(map_id).or_default().trails.push(trail);
                    } else {
                        failures.warnings.push(FailureWarning::TrailWarnings(
                            entry_path.clone(),
                            poi_index,
                            TrailWarning::MissingMapID,
                        ));
                    }
                }
            }
        };
    }

    (pack, failures)
}

#[derive(Debug, thiserror::Error)]
pub enum FailureError {
    #[error("error trying to parse the zip file: {0}")]
    ZipParseError(ZipParseError),
    #[error("Duplicate File: {0}\n")]
    DuplicateFile(Arc<RelativePath>),
    #[error("texture File Error:\nfile: {0}\nerror: {1}")]
    ImgError(Arc<RelativePath>, image::ImageError),
    #[error("No Name for file: {0}\n")]
    NoNameFile(Arc<RelativePath>),
    #[error("new name limit reached Error: {0}")]
    NewNameLimitReached(Arc<RelativePath>),
    #[error("xml file doesn't contain OverlayData tag: {0}")]
    NoOverlayData(Arc<RelativePath>),
    #[error("Trl File Error:\nfile: {0}\nerror: {1}")]
    TrlError(Arc<RelativePath>, TrlError),
    #[error("utf-8 error:\n file: {0}\n error: {1}")]
    Utf8Error(Arc<RelativePath>, std::string::FromUtf8Error),
    #[error("invalid xml:\n file: {0}\n error: {1}")]
    XmlParseError(Arc<RelativePath>, xot::Error),
}
#[derive(Debug, thiserror::Error)]
pub enum FailureWarning {
    #[error("category doesn't have a name: {0}")]
    CategoryNameMissing(Arc<RelativePath>, String),
    #[error("file doesn't have an extension: {0}")]
    ExtensionLessFile(Arc<RelativePath>),
    #[error("file extension must be xml / png / trl : {0}")]
    InvalidExtensionFile(Arc<RelativePath>),
    #[error("category number {1} in file {0}. warning: {2}")]
    CategoryWarnings(Arc<RelativePath>, usize, CategoryWarning),
    #[error("Marker or Trail number {1} in file {0}. warning: {2}")]
    POITrailWarnings(Arc<RelativePath>, usize, POITrailWarning),

    #[error("marker number {1} in file {0}. warning: {2}")]
    MarkerWarnings(Arc<RelativePath>, usize, MarkerWarning),

    #[error("trail number {1}  in file {0}. warning: {2}")]
    TrailWarnings(Arc<RelativePath>, usize, TrailWarning),
}
#[derive(Debug, thiserror::Error)]
pub enum ZipParseError {
    #[error("failed to parse bytes into a valid Zip Archive")]
    InvalidZip(#[from] zip::result::ZipError),
    #[error("The name is weird and we cannot get a proper enclosed name *within* the zip file. mangled name: {0}")]
    InvalidName(std::path::PathBuf),
    #[error("non-utf8 path. path: {0}")]
    NonRelativePath(String),
    #[error("failed to read file from zip. file: {0}")]
    FailedToReadFile(Arc<RelativePath>),
    #[error("we have duplicate entries in zip: {0}")]
    DuplicateEntry(Arc<RelativePath>),
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
    #[error("missing category attribute for POI/Trail")]
    MissingCategoryAttribute,
    #[error("category not found")]
    CategoryNotFound,
    #[error("missing texture attribute for POI/Trail")]
    MissingTextureAttribute,
    #[error("texture not found")]
    TextureNotFound,
    #[error("GUID not found")]
    GUIDNotFound,
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

// Welcome to the messiest part of the code
#[allow(clippy::too_many_arguments)]
// a temporary recursive function to parse the marker category tree.
fn recursive_marker_category_parser(
    tree: &Xot,
    tags: impl Iterator<Item = Node>,
    failures: &mut Failures,
    entry_path: &Arc<RelativePath>,
    cats: &mut IndexMap<String, Category>,
    names: &XotAttributeNameIDs,
) {
    for (tag_index, tag) in tags.filter(|node| tree.is_element(*node)).enumerate() {
        let element = tree.element(tag).unwrap();
        if element.name() != names.marker_category {
            continue;
        }

        let name = element.get_attribute(names.name).unwrap_or_default();
        if name.is_empty() {
            failures.warnings.push(FailureWarning::CategoryWarnings(
                entry_path.clone(),
                tag_index,
                CategoryWarning::CategoryNameMissing,
            ));
            continue;
        }
        let mut common_attributes = CommonAttributes::default();
        update_from_element(&mut common_attributes, element, names);

        let display_name = element
            .get_attribute(names.display_name)
            .unwrap_or_default();

        let separator = element
            .get_attribute(names.separator)
            .unwrap_or_default()
            .parse()
            .unwrap_or_default();

        let default_enabled = element
            .get_attribute(names.default_enabled)
            .unwrap_or_default()
            .parse()
            .unwrap_or(true);
        recursive_marker_category_parser(
            tree,
            tree.children(tag),
            failures,
            entry_path,
            &mut cats
                .entry(name.to_string())
                .or_insert_with(|| Category {
                    display_name: display_name.to_string(),
                    separator,
                    default_enabled,
                    props: common_attributes.into(),
                    children: Default::default(),
                })
                .children,
            names,
        );
    }
}
fn update_from_element(
    commont_attributes: &mut CommonAttributes,
    ele: &Element,
    names: &XotAttributeNameIDs,
) {
    if let Some(path) = ele.get_attribute(names.icon_file) {
        commont_attributes.icon_file = RelativePathBuf::try_from(path.to_lowercase()).ok();
    }
    if let Some(path) = ele.get_attribute(names.texture) {
        commont_attributes.texture = RelativePathBuf::try_from(path.to_lowercase()).ok();
    }
    if let Some(path) = ele.get_attribute(names.trail_data) {
        commont_attributes.trail_data_file = RelativePathBuf::try_from(path.to_lowercase()).ok();
    }
}
struct XotAttributeNameIDs {
    overlay_data: NameId,
    marker_category: NameId,
    pois: NameId,
    poi: NameId,
    trail: NameId,
    category: NameId,
    xpos: NameId,
    ypos: NameId,
    zpos: NameId,
    icon_file: NameId,
    texture: NameId,
    trail_data: NameId,
    separator: NameId,
    display_name: NameId,
    default_enabled: NameId,
    name: NameId,
    map_id: NameId,
    guid: NameId,
}
impl XotAttributeNameIDs {
    fn register_with_xot(tree: &mut Xot) -> Self {
        Self {
            overlay_data: tree.add_name("OverlayData"),
            marker_category: tree.add_name("MarkerCategory"),
            pois: tree.add_name("POIs"),
            poi: tree.add_name("POI"),
            trail: tree.add_name("Trail"),
            category: tree.add_name("type"),
            xpos: tree.add_name("xpos"),
            ypos: tree.add_name("ypos"),
            zpos: tree.add_name("zpos"),
            icon_file: tree.add_name("iconfile"),
            texture: tree.add_name("texture"),
            trail_data: tree.add_name("trailfile"),
            separator: tree.add_name("IsSeparator"),
            name: tree.add_name("name"),
            default_enabled: tree.add_name("defaulttoggle"),
            display_name: tree.add_name("DisplayName"),
            map_id: tree.add_name("MapID"),
            guid: tree.add_name("GUID"),
        }
    }
}
// #[cfg(test)]
// mod test {

//     use indexmap::IndexMap;
//     use rstest::*;

//     use semver::Version;
//     use similar_asserts::assert_eq;
//     use std::io::Write;
//     use std::sync::Arc;

//     use zip::write::FileOptions;
//     use zip::ZipWriter;

//     use crate::{
//         pack::{xml::zpack_from_xml_entries, Pack, MARKER_PNG},
//         INCHES_PER_METER,
//     };

//     const TEST_XML: &str = include_str!("test.xml");
//     const TEST_MARKER_PNG_NAME: &str = "marker.png";
//     const TEST_TRL_NAME: &str = "basic.trl";

//     #[fixture]
//     #[once]
//     fn test_zip() -> Vec<u8> {
//         let mut writer = ZipWriter::new(std::io::Cursor::new(vec![]));
//         // category.xml
//         writer
//             .start_file("category.xml", FileOptions::default())
//             .expect("failed to create category.xml");
//         writer
//             .write_all(TEST_XML.as_bytes())
//             .expect("failed to write category.xml");
//         // marker.png
//         writer
//             .start_file(TEST_MARKER_PNG_NAME, FileOptions::default())
//             .expect("failed to create marker.png");
//         writer
//             .write_all(MARKER_PNG)
//             .expect("failed to write marker.png");
//         // basic.trl
//         writer
//             .start_file(TEST_TRL_NAME, FileOptions::default())
//             .expect("failed to create basic trail");
//         writer
//             .write_all(&0u32.to_ne_bytes())
//             .expect("failed to write version");
//         writer
//             .write_all(&15u32.to_ne_bytes())
//             .expect("failed to write mapid ");
//         writer
//             .write_all(bytemuck::cast_slice(&[0f32; 3]))
//             .expect("failed to write first node");
//         // done
//         writer
//             .finish()
//             .expect("failed to finalize zip")
//             .into_inner()
//     }

//     #[fixture]
//     fn test_file_entries(test_zip: &[u8]) -> IndexMap<Arc<RelativePath>, Vec<u8>> {
//         let file_entries = super::read_files_from_zip(test_zip).expect("failed to deserialize");
//         assert_eq!(file_entries.len(), 3);
//         let test_xml = std::str::from_utf8(
//             file_entries
//                 .get(RelativePath::new("category.xml"))
//                 .expect("failed to get category.xml"),
//         )
//         .expect("failed to get str from category.xml contents");
//         assert_eq!(test_xml, TEST_XML);
//         let test_marker_png = file_entries
//             .get(RelativePath::new("marker.png"))
//             .expect("failed to get marker.png");
//         assert_eq!(test_marker_png, MARKER_PNG);
//         file_entries
//     }
//     #[fixture]
//     #[once]
//     fn test_pack(test_file_entries: IndexMap<Arc<RelativePath>, Vec<u8>>) -> Pack {
//         let (pack, failures) = zpack_from_xml_entries(test_file_entries, Version::new(0, 0, 0));
//         assert!(failures.errors.is_empty() && failures.warnings.is_empty());
//         assert_eq!(pack.tbins.len(), 1);
//         assert_eq!(pack.textures.len(), 1);
//         assert_eq!(
//             pack.textures
//                 .get(RelativePath::new(TEST_MARKER_PNG_NAME))
//                 .expect("failed to get marker.png from textures"),
//             MARKER_PNG
//         );

//         let tbin = pack
//             .tbins
//             .get(RelativePath::new(TEST_TRL_NAME))
//             .expect("failed to get basic trail")
//             .clone();

//         assert_eq!(tbin.nodes[0], [0.0f32; 3].into());
//         pack
//     }

//     // #[rstest]
//     // fn test_tag(test_pack: &Pack) {
//     //     let mut test_category_menu = CategoryMenu::default();
//     //     let parent_path = RelativePath::new("parent");
//     //     let child1_path = RelativePath::new("parent/child1");
//     //     let subchild_path = RelativePath::new("parent/child1/subchild");
//     //     let child2_path = RelativePath::new("parent/child2");
//     //     test_category_menu.create_category(subchild_path);
//     //     test_category_menu.create_category(child2_path);
//     //     test_category_menu.set_display_name(parent_path, "Parent".to_string());
//     //     test_category_menu.set_display_name(child1_path, "Child 1".to_string());
//     //     test_category_menu.set_display_name(subchild_path, "Sub Child".to_string());
//     //     test_category_menu.set_display_name(child2_path, "Child 2".to_string());

//     //     assert_eq!(test_category_menu, test_pack.category_menu)
//     // }

//     #[rstest]
//     fn test_markers(test_pack: &Pack) {
//         let marker = test_pack
//             .markers
//             .values()
//             .next()
//             .expect("failed to get queensdale mapdata");
//         assert_eq!(
//             marker.props.texture.as_ref().unwrap(),
//             RelativePath::new(TEST_MARKER_PNG_NAME)
//         );
//         assert_eq!(marker.position, [INCHES_PER_METER; 3].into());
//     }
//     #[rstest]
//     fn test_trails(test_pack: &Pack) {
//         let trail = test_pack
//             .trails
//             .values()
//             .next()
//             .expect("failed to get queensdale mapdata");
//         assert_eq!(
//             trail.props.tbin.as_ref().unwrap(),
//             RelativePath::new(TEST_TRL_NAME)
//         );
//         assert_eq!(
//             trail.props.trail_texture.as_ref().unwrap(),
//             RelativePath::new(TEST_MARKER_PNG_NAME)
//         );
//     }
// }
