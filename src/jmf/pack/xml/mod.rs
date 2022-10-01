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
//!     2. collect all the files entries into a IndexMap with `Utf8PathBuf` relative to pack root folder
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
//!     3. parse into elements and store in entries with the file path (Arc<Utf8Path>). skip if any errors.
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

pub mod template;
mod zcopy;

use bitvec::vec::BitVec;
use camino::{Utf8Path, Utf8PathBuf};
use glam::Vec3;

use indexmap::IndexMap;
use roxmltree::{Children, Document, Node, TextPos};
use semver::Version;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::io::Read;

use std::sync::Arc;
// use tracing::error;

use self::template::MarkerTemplate;

use super::{ZCat, ZMarker, ZPack, ZTex, ZTrail};

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
pub fn get_zpack_from_taco(
    taco: &[u8],
    version: Version,
) -> Result<(ZPack, BitVec, Failures), ZipParseError> {
    Ok(zpack_from_xml_entries(read_files_from_zip(taco)?, version))
}

/// parses the given `Vec<u8>` as a zipfile and reads all the files into a Map with file paths as keys and file contents as Vec<u8>
/// will return error if there's any issues with the zip file or file names etc..
fn read_files_from_zip(
    taco: &[u8],
) -> std::result::Result<IndexMap<Arc<Utf8Path>, Vec<u8>>, ZipParseError> {
    // get zip file
    let mut zip_file = zip::ZipArchive::new(std::io::Cursor::new(taco))?;
    let mut entries = IndexMap::default();
    // for each entry in zip filea
    for index in 0..zip_file.len() {
        // get the entry from zip file. return if we can't find it
        let mut file = zip_file.by_index(index)?;
        // ignore if directory. skip to next entry
        if file.is_dir() {
            continue;
        }
        // if it has invalid pathbuf, return
        let file_path = file
            .enclosed_name()
            .ok_or_else(||ZipParseError::InvalidName(file.mangled_name()))?
            .to_path_buf();
        let file_path =
            Utf8PathBuf::from_path_buf(file_path).map_err(ZipParseError::NonUtf8Path)?;
        let file_path = Arc::from(file_path);
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
fn zpack_from_xml_entries(
    entries: IndexMap<Arc<Utf8Path>, Vec<u8>>,
    version: Version,
) -> (ZPack, BitVec, Failures) {
    // record of all the errors.
    // libraries should avoid panicking as much as possible.
    // some invalid marker pack should not bring down the whole library
    let mut failures = Failures::default();

    // all the contents of ZPack
    let mut zpack = ZPack {
        textures: Vec::new(),
        tbins: Vec::new(),
        // display name of the root category. although we don't need one, this will keep it consistent with the rest of the categories.
        text: vec!["root".to_string()],
        // default root category. this can be used as the parent_id for all the top level categories
        cats: vec![ZCat {
            display_name: 0,
            is_separator: false,
            parent_id: 0,
        }],
        maps: BTreeMap::new(),
        version: version.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("failed to get timestamp duration since unix epoch")
            .as_secs_f64(),
    };
    // all the categories and their default toggles
    let mut default_toggles: BitVec = Default::default();
    default_toggles.push(true); // root category enabled and so that other categories go into their proper matching indices
                                // all the containers to hold temporary data while we are converting a marker pack
                                // these will store the indicies of all the entries in zpack. while parsing xml files, we will resolve the property names to indices using these.
                                // tbin_indices also contains mapID (first) along with the index (second) as Trail tags don't have a map id
    let mut tbin_indices: IndexMap<String, (u16, u16)> = Default::default();
    let mut texture_indices: IndexMap<String, u16> = Default::default();
    let mut cat_indices: IndexMap<String, (u16, MarkerTemplate)> = Default::default();

    // string interner which holds the index of strings that were already added.
    let mut text_indices: IndexMap<String, u16> = Default::default();
    text_indices.insert("root".to_string(), 0); // add the root category display name to the interned strings

    let mut xml_entries: IndexMap<Arc<Utf8Path>, String> = Default::default();

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
                let xml = crate::jmf::rapid_filter_rust(xml);

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
                let nodes: Vec<Vec3> = entry_contents[8..]
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
                        
                        Vec3::from_array(arr)
                    })
                    .collect();
                zpack.tbins.push(nodes);
                if tbin_indices
                    .insert(
                        entry_path.to_string().to_lowercase(),
                        (map_id, (zpack.tbins.len() - 1).try_into().unwrap()),
                    )
                    .is_some()
                {
                    failures
                        .errors
                        .push(FailureError::DuplicateFile(entry_path));
                    panic!("should be unreachable");
                }
            }
            Some("png") => {
                match image::load_from_memory_with_format(&entry_contents, image::ImageFormat::Png)
                {
                    Ok(img) => {
                        zpack.textures.push(ZTex {
                            width: img.width().try_into().unwrap(),
                            height: img.height().try_into().unwrap(),
                            bytes: entry_contents,
                        });
                        assert!(texture_indices
                            .insert(
                                entry_path.to_string().to_lowercase(),
                                (zpack.textures.len() - 1).try_into().unwrap()
                            )
                            .is_none());
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
    let doc_entries: Vec<(&Arc<Utf8Path>, Document)> = xml_entries
        .iter()
        .filter_map(|(entry_path, xml)| {
            Document::parse(xml)
                .map_err(|e| {
                    failures
                        .errors
                        .push(FailureError::XmlParseError(entry_path.clone(), e));
                    MarkerWarning::MissingMapID
                })
                .ok()
                .map(|doc| (entry_path, doc))
        })
        .collect();

    for (entry_path, doc) in doc_entries.iter() {
        let root_node = doc.root_element();

        let entry_path = Arc::clone(entry_path);
        if "OverlayData" == root_node.tag_name().name() {
            // Welcome to the messiest part of the code
            #[allow(clippy::too_many_arguments)]
            // a temporary recursive function to parse the marker category tree.
            fn recursive_marker_category_parser<'node, 'input>(
                doc: &Document<'input>,
                tags: Children<'node, 'input>,
                parent_name: &str,
                parent_template: &MarkerTemplate,
                failures: &mut Failures,
                entry_path: &Arc<Utf8Path>,
                cat_indices: &mut IndexMap<String, (u16, MarkerTemplate)>,
                text_indices: &mut IndexMap<String, u16>,
                zpack: &mut ZPack,
                default_toggles: &mut BitVec,
            ) {
                for (tag_index, tag) in tags.filter(Node::is_element).enumerate() {
                    if tag.tag_name().name() != "MarkerCategory" {
                        continue;
                    }

                    let name = tag
                        .attribute("name")
                        .unwrap_or_else(|| tag.attribute("Name").unwrap_or_default());
                    if name.is_empty() {
                        failures.warnings.push(FailureWarning::CategoryWarnings(
                            entry_path.clone(),
                            doc.text_pos_at(tag.range().start),
                            tag_index,
                            CategoryWarning::CategoryNameMissing,
                        ));
                        continue;
                    }
                    let full_name = if parent_name.is_empty() {
                        name.to_string()
                    } else {
                        format!("{}.{}", parent_name, name)
                    };
                    let mut template = MarkerTemplate::default();
                    template.update_from_element(&tag);
                    template.inherit_from_template(parent_template);

                    let display_name = tag.attribute("DisplayName").unwrap_or_default();

                    let is_separator = tag
                        .attribute("IsSeparator")
                        .unwrap_or_default()
                        .parse()
                        .unwrap_or_default();

                    let default_toggle = tag
                        .attribute("defaulttoggle")
                        .unwrap_or_default()
                        .parse()
                        .unwrap_or(true);

                    if !cat_indices.contains_key(&full_name) {
                        let display_name_index = *text_indices
                            .entry(display_name.to_owned())
                            .or_insert_with(|| {
                                zpack.text.push(display_name.to_string());
                                (zpack.text.len() - 1).try_into().unwrap()
                            });

                        let parent_index = if let Some(end) = full_name.rfind('.') {
                            let parent_id = &full_name[..end];
                            cat_indices.get(parent_id).expect("must exist").0
                        } else {
                            0
                        };

                        zpack.cats.push(ZCat {
                            display_name: display_name_index,
                            is_separator,
                            parent_id: parent_index,
                        });
                        default_toggles.push(default_toggle);
                        cat_indices.insert(
                            full_name.clone(),
                            ((zpack.cats.len() - 1).try_into().unwrap(), template.clone()),
                        );
                    };
                    recursive_marker_category_parser(
                        doc,
                        tag.children(),
                        &full_name,
                        &template,
                        failures,
                        entry_path,
                        cat_indices,
                        text_indices,
                        zpack,
                        default_toggles,
                    )
                }
            }
            recursive_marker_category_parser(
                doc,
                root_node.children(),
                "",
                &MarkerTemplate::default(),
                &mut failures,
                &entry_path,
                &mut cat_indices,
                &mut text_indices,
                &mut zpack,
                &mut default_toggles,
            );
        } else {
            failures
                .errors
                .push(FailureError::NoOverlayData(entry_path.clone()));
        }
    }
    for (entry_path, doc) in doc_entries.iter() {
        let root_node = doc.root_element();
        let entry_path = Arc::clone(entry_path);
        if "OverlayData" == root_node.tag_name().name() {
            if let Some(pois) = root_node
                .children()
                .filter(Node::is_element)
                .find(|e| e.tag_name().name() == "POIs")
            {
                for (poi_index, child) in pois.children().filter(Node::is_element).enumerate() {
                    // if type attribute exists, get the category id and the template. otherwise, skip this element.
                    let (cat_index, mut template) =
                        if let Some(cat_full_name) = child.attribute("type") {
                            match cat_indices.get(cat_full_name) {
                                Some(cat_index_template) => cat_index_template.clone(),
                                None => {
                                    failures.warnings.push(FailureWarning::POITrailWarnings(
                                        entry_path.clone(),
                                        doc.text_pos_at(child.range().start),
                                        poi_index,
                                        POITrailWarning::CategoryNotFound,
                                    ));
                                    continue;
                                }
                            }
                        } else {
                            failures.warnings.push(FailureWarning::POITrailWarnings(
                                entry_path.clone(),
                                doc.text_pos_at(child.range().start),
                                poi_index,
                                POITrailWarning::MissingCategoryAttribute,
                            ));
                            continue;
                        };

                    match child.tag_name().name() {
                        "POI" => {
                            if let Some(map_id) = child
                                .attribute("MapID")
                                .and_then(|map_id| map_id.parse::<u16>().ok())
                            {
                                let xpos = child
                                    .attribute("xpos")
                                    .unwrap_or_default()
                                    .parse()
                                    .unwrap_or_default();
                                let ypos = child
                                    .attribute("ypos")
                                    .unwrap_or_default()
                                    .parse()
                                    .unwrap_or_default();
                                let zpos = child
                                    .attribute("zpos")
                                    .unwrap_or_default()
                                    .parse()
                                    .unwrap_or_default();
                                template.update_from_element(&child);
                                let tex_index = match template.icon_file.as_ref() {
                                    Some(texture_path) => match texture_indices.get(texture_path) {
                                        Some(tex_index) => *tex_index,
                                        None => {
                                            failures.warnings.push(
                                                FailureWarning::POITrailWarnings(
                                                    entry_path.clone(),
                                                    doc.text_pos_at(child.range().start),
                                                    poi_index,
                                                    POITrailWarning::TextureNotFound,
                                                ),
                                            );
                                            continue;
                                        }
                                    },
                                    None => {
                                        failures.warnings.push(FailureWarning::POITrailWarnings(
                                            entry_path.clone(),
                                            doc.text_pos_at(child.range().start),
                                            poi_index,
                                            POITrailWarning::MissingTextureAttribute,
                                        ));
                                        continue;
                                    }
                                };
                                let marker = ZMarker {
                                    position: [xpos, ypos, zpos].into(),
                                    cat: cat_index,
                                    texture: tex_index,
                                };

                                zpack.maps.entry(map_id).or_default().markers.push(marker);
                            } else {
                                failures.warnings.push(FailureWarning::MarkerWarnings(
                                    entry_path.clone(),
                                    doc.text_pos_at(child.range().start),
                                    poi_index,
                                    MarkerWarning::MissingMapID,
                                ));
                            }
                        }
                        "Trail" => {
                            if let Some((map_id, tbin_index)) =
                                child.attribute("trailData").and_then(|trail_data| {
                                    tbin_indices.get(&trail_data.to_lowercase()).copied()
                                })
                            {
                                template.update_from_element(&child);

                                let tex_index = match template.texture.as_ref() {
                                    Some(texture_path) => match texture_indices.get(texture_path) {
                                        Some(tex_index) => *tex_index,
                                        None => {
                                            failures.warnings.push(
                                                FailureWarning::POITrailWarnings(
                                                    entry_path.clone(),
                                                    doc.text_pos_at(child.range().start),
                                                    poi_index,
                                                    POITrailWarning::TextureNotFound,
                                                ),
                                            );
                                            continue;
                                        }
                                    },
                                    None => {
                                        failures.warnings.push(FailureWarning::POITrailWarnings(
                                            entry_path.clone(),
                                            doc.text_pos_at(child.range().start),
                                            poi_index,
                                            POITrailWarning::MissingTextureAttribute,
                                        ));
                                        continue;
                                    }
                                };

                                let trail = ZTrail {
                                    cat: cat_index,
                                    texture: tex_index,
                                    tbin: tbin_index,
                                };
                                zpack.maps.entry(map_id).or_default().trails.push(trail);
                            } else {
                                failures.warnings.push(FailureWarning::TrailWarnings(
                                    entry_path.clone(),
                                    doc.text_pos_at(child.range().start),
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
    (zpack, default_toggles, failures)
}

#[cfg(test)]
mod test {
    use camino::Utf8Path;

    use indexmap::IndexMap;
    use rstest::*;

    use semver::Version;
    use similar_asserts::assert_eq;
    use std::io::Write;
    use std::sync::Arc;

    use zip::write::FileOptions;
    use zip::ZipWriter;

    use crate::jmf::pack::{xml::zpack_from_xml_entries, ZPack, MARKER_PNG};

    const TEST_XML: &str = include_str!("test.xml");

    #[fixture]
    #[once]
    fn test_zip() -> Vec<u8> {
        let mut writer = ZipWriter::new(std::io::Cursor::new(vec![]));
        // category.xml
        writer
            .start_file("category.xml", FileOptions::default())
            .expect("failed to create category.xml");
        writer
            .write_all(TEST_XML.as_bytes())
            .expect("failed to write category.xml");
        // marker.png
        writer
            .start_file("marker.png", FileOptions::default())
            .expect("failed to create marker.png");
        writer
            .write_all(MARKER_PNG)
            .expect("failed to write marker.png");
        // basic.trl
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
        // done
        writer
            .finish()
            .expect("failed to finalize zip")
            .into_inner()
    }

    #[fixture]
    fn test_file_entries(test_zip: &[u8]) -> IndexMap<Arc<Utf8Path>, Vec<u8>> {
        let file_entries = super::read_files_from_zip(test_zip).expect("failed to deserialize");
        assert_eq!(file_entries.len(), 3);
        let test_xml = std::str::from_utf8(
            file_entries
                .get(Utf8Path::new("category.xml"))
                .expect("failed to get category.xml"),
        )
        .expect("failed to get str from category.xml contents");
        assert_eq!(test_xml, TEST_XML);
        let test_marker_png = file_entries
            .get(Utf8Path::new("marker.png"))
            .expect("failed to get marker.png");
        assert_eq!(test_marker_png, MARKER_PNG);
        file_entries
    }
    #[fixture]
    #[once]
    fn test_pack(test_file_entries: IndexMap<Arc<Utf8Path>, Vec<u8>>) -> ZPack {
        let (pack, cats_enabled_status, failures) =
            zpack_from_xml_entries(test_file_entries, Version::new(0, 0, 0));
        assert_eq!(pack.cats.len(), cats_enabled_status.len());
        assert!(failures.errors.is_empty() && failures.warnings.is_empty());
        assert_eq!(pack.tbins.len(), 1);
        assert_eq!(pack.textures.len(), 1);
        assert_eq!(
            pack.textures
                .get(0)
                .as_ref()
                .expect("failed to get marker.png from textures")
                .bytes,
            MARKER_PNG
        );

        let trl = pack
            .tbins
            .get(0)
            .expect("failed to get basic trail")
            .clone();

        assert_eq!(trl[0], [0.0f32; 3].into());
        pack
    }

    // #[rstest]
    // fn test_tag(test_pack: &Pack) {
    //     let mut test_category_menu = CategoryMenu::default();
    //     let parent_path = Utf8Path::new("parent");
    //     let child1_path = Utf8Path::new("parent/child1");
    //     let subchild_path = Utf8Path::new("parent/child1/subchild");
    //     let child2_path = Utf8Path::new("parent/child2");
    //     test_category_menu.create_category(subchild_path);
    //     test_category_menu.create_category(child2_path);
    //     test_category_menu.set_display_name(parent_path, "Parent".to_string());
    //     test_category_menu.set_display_name(child1_path, "Child 1".to_string());
    //     test_category_menu.set_display_name(subchild_path, "Sub Child".to_string());
    //     test_category_menu.set_display_name(child2_path, "Child 2".to_string());

    //     assert_eq!(test_category_menu, test_pack.category_menu)
    // }

    #[rstest]
    fn test_markers(test_pack: &ZPack) {
        let pack = test_pack;
        let qd = pack
            .maps
            .get(&15)
            .expect("failed to get queensdale mapdata");
        let marker = &qd.markers[0];
        assert_eq!(marker.texture, 0);
        assert_eq!(marker.position, [1.0f32; 3].into());
    }
    #[rstest]
    fn test_trails(test_pack: &ZPack) {
        let pack = test_pack;
        let qd = pack
            .maps
            .get(&15)
            .expect("failed to get queensdale mapdata");
        let trail = &qd.trails[0];
        assert_eq!(trail.tbin, 0);
        assert_eq!(trail.texture, 0);
    }
}
#[derive(Debug, thiserror::Error)]
pub enum FailureError {
    #[error("error trying to parse the zip file: {0}")]
    ZipParseError(ZipParseError),
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
    XmlParseError(Arc<Utf8Path>, roxmltree::Error),
}
#[derive(Debug, thiserror::Error)]
pub enum FailureWarning {
    #[error("category doesn't have a name: {0}")]
    CategoryNameMissing(Arc<Utf8Path>, String),
    #[error("file doesn't have an extension: {0}")]
    ExtensionLessFile(Arc<Utf8Path>),
    #[error("file extension must be xml / png / trl : {0}")]
    InvalidExtensionFile(Arc<Utf8Path>),
    #[error("category number {2} at '{1}' in file {0}. warning: {3}")]
    CategoryWarnings(Arc<Utf8Path>, TextPos, usize, CategoryWarning),
    #[error("Marker or Trail number {2} at '{1}' in file {0}. warning: {3}")]
    POITrailWarnings(Arc<Utf8Path>, TextPos, usize, POITrailWarning),

    #[error("marker number {2} at '{1}' in file {0}. warning: {3}")]
    MarkerWarnings(Arc<Utf8Path>, TextPos, usize, MarkerWarning),

    #[error("trail number {2} at '{1}' in file {0}. warning: {3}")]
    TrailWarnings(Arc<Utf8Path>, TextPos, usize, TrailWarning),
}
#[derive(Debug, thiserror::Error)]
pub enum ZipParseError {
    #[error("failed to parse bytes into a valid Zip Archive")]
    InvalidZip(#[from] zip::result::ZipError),
    #[error("The name is weird and we cannot get a proper enclosed name *within* the zip file. mangled name: {0}")]
    InvalidName(std::path::PathBuf),
    #[error("non-utf8 path. path: {0}")]
    NonUtf8Path(std::path::PathBuf),
    #[error("failed to read file from zip. file: {0}")]
    FailedToReadFile(Arc<Utf8Path>),
    #[error("we have duplicate entries in zip: {0}")]
    DuplicateEntry(Arc<Utf8Path>),
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
